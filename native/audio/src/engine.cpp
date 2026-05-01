/**
 * engine.cpp — PhraseDJ audio engine: implements the C ABI from pdj_engine.h.
 *
 * This file ties together the Decoder, Deck, Mixer, and PortAudioBackend.
 * It owns all the C++ objects and exposes only the C ABI to Rust.
 *
 * Thread model (see specs/02-audio-engine.md):
 *   - Audio callback: RT, managed by PortAudio.
 *   - Prefetch threads: one per deck, managed by Deck.
 *   - Caller thread: anything — all public C-ABI setters are atomic.
 */

#include "../include/pdj_engine.h"

#include "bpm.hpp"
#include "deck.hpp"
#include "mixer.hpp"
#include "portaudio_backend.hpp"

#include <array>
#include <atomic>
#include <cstring>
#include <memory>
#include <mutex>
#include <string>

// ---------------------------------------------------------------------------
// PdjEngine struct — the opaque handle returned to callers
// ---------------------------------------------------------------------------

struct PdjEngine {
    PdjEngineConfig config;

    // Two decks.
    std::array<std::unique_ptr<pdj::Deck>, 2> decks;

    // Mixer that sums the decks.
    std::unique_ptr<pdj::Mixer> mixer;

    // Audio output backend.
    std::unique_ptr<pdj::PortAudioBackend> backend;

    // BPM result per deck (set after analysis; read by the UI).
    std::array<pdj::BeatgridResult, 2> beatgrids;

    // Mutex protects beatgrids and non-RT control operations.
    std::mutex control_mutex;

    // Per-deck crossfader position (exposed via setter).
    std::atomic<float> crossfader{0.5f};

    explicit PdjEngine(const PdjEngineConfig& cfg) : config(cfg) {}
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

static bool valid_deck(const PdjEngine* e, uint32_t idx) {
    return e != nullptr && idx < 2;
}

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

PdjEngine* pdj_engine_create(const PdjEngineConfig* config) {
    if (!config) return nullptr;

    auto* e = new PdjEngine(*config);

    // Create two decks.
    for (int i = 0; i < 2; ++i) {
        e->decks[i] = std::make_unique<pdj::Deck>();
    }

    // Create the mixer with raw deck pointers.
    std::array<pdj::Deck*, 2> deck_ptrs = {
        e->decks[0].get(), e->decks[1].get()
    };
    e->mixer = std::make_unique<pdj::Mixer>(deck_ptrs);

    // Open PortAudio output.
    e->backend = std::make_unique<pdj::PortAudioBackend>();
    const auto result = e->backend->open(
        e->mixer.get(),
        config->sample_rate,
        config->buffer_size,
        "");  // empty = system default device

    if (result != pdj::BackendResult::Ok) {
        // Audio device unavailable (headless environment, CI, etc.).
        // Engine still works for deck control — just no sound.
        // Callers can check pdj_engine_is_running().
        delete e->backend.release();
    }

    return e;
}

void pdj_engine_destroy(PdjEngine* engine) {
    delete engine;
}

// ---------------------------------------------------------------------------
// Track loading
// ---------------------------------------------------------------------------

PdjResult pdj_engine_load(PdjEngine*  engine,
                           uint32_t    deck_index,
                           const char* file_path) {
    if (!file_path) return PdjResult_InvalidArg;
    if (!valid_deck(engine, deck_index)) return PdjResult_InvalidArg;

    const bool ok = engine->decks[deck_index]->load(
        std::string(file_path), engine->config.sample_rate);
    return ok ? PdjResult_Ok : PdjResult_Io;
}

// ---------------------------------------------------------------------------
// Transport
// ---------------------------------------------------------------------------

PdjResult pdj_engine_play(PdjEngine* engine, uint32_t deck_index) {
    if (!valid_deck(engine, deck_index)) return PdjResult_InvalidArg;
    engine->decks[deck_index]->play();
    return PdjResult_Ok;
}

PdjResult pdj_engine_pause(PdjEngine* engine, uint32_t deck_index) {
    if (!valid_deck(engine, deck_index)) return PdjResult_InvalidArg;
    engine->decks[deck_index]->pause();
    return PdjResult_Ok;
}

PdjResult pdj_engine_seek(PdjEngine* engine,
                           uint32_t   deck_index,
                           uint64_t   position_frames) {
    if (!valid_deck(engine, deck_index)) return PdjResult_InvalidArg;
    engine->decks[deck_index]->seek(position_frames);
    return PdjResult_Ok;
}

uint64_t pdj_engine_position(PdjEngine* engine, uint32_t deck_index) {
    if (!valid_deck(engine, deck_index)) return 0;
    return engine->decks[deck_index]->position();
}

// ---------------------------------------------------------------------------
// Mixer controls
// ---------------------------------------------------------------------------

void pdj_engine_set_fader(PdjEngine* engine, uint32_t deck_index, float value) {
    if (!valid_deck(engine, deck_index)) return;
    engine->decks[deck_index]->set_gain(value);
}

void pdj_engine_set_crossfader(PdjEngine* engine, float value) {
    if (!engine) return;
    engine->crossfader.store(value);
    if (engine->mixer) engine->mixer->set_crossfader(value);
}

void pdj_engine_set_stem_gain(PdjEngine* engine,
                               uint32_t   deck_index,
                               uint32_t   stem_index,
                               float      value) {
    if (!valid_deck(engine, deck_index)) return;
    if (stem_index >= 4) return;
    engine->decks[deck_index]->set_stem_gain(
        static_cast<int>(stem_index), value);
}

void pdj_engine_set_master_gain(PdjEngine* engine, float value) {
    if (!engine || !engine->mixer) return;
    engine->mixer->set_master_gain(value);
}

// ---------------------------------------------------------------------------
// Status queries
// ---------------------------------------------------------------------------

int pdj_engine_is_playing(PdjEngine* engine, uint32_t deck_index) {
    if (!valid_deck(engine, deck_index)) return 0;
    return engine->decks[deck_index]->is_playing() ? 1 : 0;
}

int pdj_engine_is_loaded(PdjEngine* engine, uint32_t deck_index) {
    if (!valid_deck(engine, deck_index)) return 0;
    return engine->decks[deck_index]->is_loaded() ? 1 : 0;
}

int pdj_engine_is_running(PdjEngine* engine) {
    if (!engine || !engine->backend) return 0;
    return engine->backend->is_running() ? 1 : 0;
}

// ---------------------------------------------------------------------------
// BPM analysis
// ---------------------------------------------------------------------------

PdjResult pdj_engine_analyse_bpm(PdjEngine*  engine,
                                   uint32_t    deck_index,
                                   const float* samples,
                                   uint64_t     frame_count) {
    if (!valid_deck(engine, deck_index)) return PdjResult_InvalidArg;
    if (!samples || frame_count == 0) return PdjResult_InvalidArg;

    auto result = pdj::detect_bpm(samples, frame_count,
                                    engine->config.sample_rate);
    {
        std::lock_guard<std::mutex> lk(engine->control_mutex);
        engine->beatgrids[deck_index] = std::move(result);
    }
    return PdjResult_Ok;
}

float pdj_engine_get_bpm(PdjEngine* engine, uint32_t deck_index) {
    if (!valid_deck(engine, deck_index)) return 0.0f;
    std::lock_guard<std::mutex> lk(engine->control_mutex);
    return engine->beatgrids[deck_index].bpm;
}

// ---------------------------------------------------------------------------
// Tempo control
// ---------------------------------------------------------------------------

void pdj_engine_set_tempo_ratio(PdjEngine* engine,
                                 uint32_t   deck_index,
                                 float      ratio) {
    if (!valid_deck(engine, deck_index)) return;
    // Clamp to a sensible DJ range: ±50 % from original speed.
    const float clamped = std::max(0.5f, std::min(2.0f, ratio));
    engine->decks[deck_index]->set_pitch(clamped);
}

float pdj_engine_get_tempo_ratio(PdjEngine* engine, uint32_t deck_index) {
    if (!valid_deck(engine, deck_index)) return 1.0f;
    // state_.pitch is an atomic float; access it via the public setter's path.
    // We expose a dedicated getter to avoid reaching into private members.
    return engine->decks[deck_index]->get_pitch();
}
