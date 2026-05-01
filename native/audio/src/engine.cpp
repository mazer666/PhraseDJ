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
#include "decoder.hpp"
#include "mixer.hpp"
#include "portaudio_backend.hpp"

#include <algorithm>
#include <array>
#include <atomic>
#include <cmath>
#include <cstring>
#include <memory>
#include <mutex>
#include <string>
#include <vector>

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

    // File path last loaded per deck — used for waveform peak computation.
    std::array<std::string, 2> deck_paths;

    // Stem paths last loaded per deck. Empty if stems not loaded.
    std::array<std::array<std::string, 4>, 2> deck_stem_paths;

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

    const std::string path(file_path);
    const bool ok = engine->decks[deck_index]->load(path, engine->config.sample_rate);
    if (ok) engine->deck_paths[deck_index] = path;
    return ok ? PdjResult_Ok : PdjResult_Io;
}

PdjResult pdj_engine_load_stems(PdjEngine* engine,
                                uint32_t   deck_index,
                                const char* path_main,
                                const char* path_v,
                                const char* path_d,
                                const char* path_b,
                                const char* path_o) {
    if (!path_main || !path_v || !path_d || !path_b || !path_o) return PdjResult_InvalidArg;
    if (!valid_deck(engine, deck_index)) return PdjResult_InvalidArg;

    const std::string pm(path_main);
    const bool ok = engine->decks[deck_index]->load_stems(
        pm, std::string(path_v), std::string(path_d),
        std::string(path_b), std::string(path_o),
        engine->config.sample_rate);
    
    if (ok) {
        engine->deck_paths[deck_index] = pm;
        engine->deck_stem_paths[deck_index] = {
            std::string(path_v), std::string(path_d),
            std::string(path_b), std::string(path_o)
        };
    }
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
    return engine->decks[deck_index]->get_pitch();
}

// ---------------------------------------------------------------------------
// Waveform analysis
// ---------------------------------------------------------------------------

PdjResult pdj_engine_compute_waveform(PdjEngine* engine,
                                       uint32_t   deck_index,
                                       uint32_t   num_bins,
                                       float*     out_min,
                                       float*     out_max) {
    if (!valid_deck(engine, deck_index)) return PdjResult_InvalidArg;
    if (!out_min || !out_max || num_bins == 0) return PdjResult_InvalidArg;

    const std::string& path = engine->deck_paths[deck_index];
    if (path.empty()) return PdjResult_NotReady;

    auto dec = pdj::Decoder::open(path, engine->config.sample_rate);
    if (!dec) return PdjResult_Io;

    const uint64_t total_frames = dec->info().total_frames;
    if (total_frames == 0) return PdjResult_Internal;

    // Pre-fill output with silence so partially-decoded bins are valid.
    for (uint32_t i = 0; i < num_bins; ++i) {
        out_min[i] = 0.0f;
        out_max[i] = 0.0f;
    }

    const double frames_per_bin =
        static_cast<double>(total_frames) / static_cast<double>(num_bins);

    constexpr uint32_t CHUNK = 4096;
    std::vector<float> buf(CHUNK * 2);
    uint64_t frames_done = 0;

    while (true) {
        uint32_t got = 0;
        const auto res = dec->read_frames(buf.data(), CHUNK, got);
        if (got == 0) break;

        for (uint32_t f = 0; f < got; ++f) {
            const uint64_t fi  = frames_done + f;
            const uint32_t bin = static_cast<uint32_t>(
                static_cast<double>(fi) / frames_per_bin);
            if (bin >= num_bins) break;

            // Mono RMS-ish peak: average absolute value of L and R.
            const float l = std::abs(buf[f * 2]);
            const float r = std::abs(buf[f * 2 + 1]);
            const float peak = (l + r) * 0.5f;

            if (peak > out_max[bin]) out_max[bin] = peak;
            out_min[bin] = -out_max[bin];  // symmetric display
        }

        frames_done += got;
        if (res == pdj::DecodeResult::EndOfFile) break;
        if (res != pdj::DecodeResult::Ok) break;
    }

    return PdjResult_Ok;
}

PdjResult pdj_engine_compute_stem_waveforms(PdjEngine* engine,
                                            uint32_t   deck_index,
                                            uint32_t   num_bins,
                                            float*     out_v,
                                            float*     out_d,
                                            float*     out_b,
                                            float*     out_o) {
    if (!valid_deck(engine, deck_index)) return PdjResult_InvalidArg;
    if (!out_v || !out_d || !out_b || !out_o || num_bins == 0) return PdjResult_InvalidArg;

    const auto& paths = engine->deck_stem_paths[deck_index];
    if (paths[0].empty()) return PdjResult_NotReady;

    float* outs[4] = { out_v, out_d, out_b, out_o };

    for (int s = 0; s < 4; ++s) {
        auto dec = pdj::Decoder::open(paths[s], engine->config.sample_rate);
        if (!dec) return PdjResult_Io;

        const uint64_t total_frames = dec->info().total_frames;
        if (total_frames == 0) return PdjResult_Internal;

        for (uint32_t i = 0; i < num_bins; ++i) outs[s][i] = 0.0f;

        const double frames_per_bin =
            static_cast<double>(total_frames) / static_cast<double>(num_bins);

        constexpr uint32_t CHUNK = 4096;
        std::vector<float> buf(CHUNK * 2);
        uint64_t frames_done = 0;

        while (true) {
            uint32_t got = 0;
            const auto res = dec->read_frames(buf.data(), CHUNK, got);
            if (got == 0) break;

            for (uint32_t f = 0; f < got; ++f) {
                const uint64_t fi  = frames_done + f;
                const uint32_t bin = static_cast<uint32_t>(
                    static_cast<double>(fi) / frames_per_bin);
                if (bin >= num_bins) break;

                const float l = std::abs(buf[f * 2]);
                const float r = std::abs(buf[f * 2 + 1]);
                const float peak = (l + r) * 0.5f;

                if (peak > outs[s][bin]) outs[s][bin] = peak;
            }

            frames_done += got;
            if (res == pdj::DecodeResult::EndOfFile) break;
            if (res != pdj::DecodeResult::Ok) break;
        }
    }

    return PdjResult_Ok;
}

uint64_t pdj_engine_total_frames(PdjEngine* engine, uint32_t deck_index) {
    if (!valid_deck(engine, deck_index)) return 0;
    const auto* info = engine->decks[deck_index]->audio_info();
    return info ? info->total_frames : 0;
}
