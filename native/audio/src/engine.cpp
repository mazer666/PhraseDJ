/**
 * engine.cpp — Phase 0 stub implementation of the PhraseDJ audio engine.
 *
 * This file implements the C ABI declared in pdj_engine.h.  In Phase 0 all
 * functions exist but don't produce audio — that work begins in Phase 1 when
 * the CoreAudio / PortAudio backend is wired in.
 *
 * The goal of Phase 0 is:
 *   1. The C ABI compiles cleanly with strict warnings.
 *   2. GoogleTest tests run and pass.
 *   3. Rust's pdj-engine-bridge can link against the shared library.
 *
 * Realtime-audio rules (enforced from Phase 1 onwards):
 *   - No allocations inside the audio callback.
 *   - No mutexes or blocking calls.
 *   - No I/O (file, log, network).
 * Violations here in Phase 0 are acceptable because no audio callback
 * runs yet.
 */

#include "pdj_engine.h"

#include <array>
#include <atomic>
#include <cstdlib>
#include <cstring>

namespace {

// Maximum number of decks supported.
constexpr uint32_t MAX_DECKS = 2;

// Per-deck state held inside the engine.
struct DeckState {
    std::atomic<bool>     playing{false};
    std::atomic<uint64_t> position{0};   // frames
    std::atomic<float>    fader{1.0f};
    // Per-stem gains: 0=vocals 1=drums 2=bass 3=other.
    std::array<std::atomic<float>, 4> stem_gain;

    DeckState() {
        for (auto& g : stem_gain) { g.store(1.0f); }
    }
};

// The engine's internal state.
struct PdjEngine {
    PdjEngineConfig config;
    std::array<DeckState, MAX_DECKS> decks;
    std::atomic<float> crossfader{0.5f};

    explicit PdjEngine(const PdjEngineConfig& cfg) : config(cfg) {}
};

// Validate deck_index and return PdjResult_InvalidArg if out of range.
PdjResult check_deck(uint32_t deck_index) {
    return (deck_index < MAX_DECKS) ? PdjResult_Ok : PdjResult_InvalidArg;
}

} // anonymous namespace

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

PdjEngine* pdj_engine_create(const PdjEngineConfig* config) {
    if (!config) { return nullptr; }
    // Phase 0: no real audio device opened yet.
    return new PdjEngine(*config);
}

void pdj_engine_destroy(PdjEngine* engine) {
    delete engine;
}

// ---------------------------------------------------------------------------
// Deck control
// ---------------------------------------------------------------------------

PdjResult pdj_engine_load(PdjEngine*  engine,
                           uint32_t    deck_index,
                           const char* file_path) {
    if (!engine || !file_path) { return PdjResult_InvalidArg; }
    if (auto r = check_deck(deck_index); r != PdjResult_Ok) { return r; }
    // Phase 0: no decoder; file path accepted but not used.
    engine->decks[deck_index].position.store(0);
    engine->decks[deck_index].playing.store(false);
    return PdjResult_Ok;
}

PdjResult pdj_engine_play(PdjEngine* engine, uint32_t deck_index) {
    if (!engine) { return PdjResult_InvalidArg; }
    if (auto r = check_deck(deck_index); r != PdjResult_Ok) { return r; }
    engine->decks[deck_index].playing.store(true);
    return PdjResult_Ok;
}

PdjResult pdj_engine_pause(PdjEngine* engine, uint32_t deck_index) {
    if (!engine) { return PdjResult_InvalidArg; }
    if (auto r = check_deck(deck_index); r != PdjResult_Ok) { return r; }
    engine->decks[deck_index].playing.store(false);
    return PdjResult_Ok;
}

PdjResult pdj_engine_seek(PdjEngine* engine,
                           uint32_t   deck_index,
                           uint64_t   position_frames) {
    if (!engine) { return PdjResult_InvalidArg; }
    if (auto r = check_deck(deck_index); r != PdjResult_Ok) { return r; }
    engine->decks[deck_index].position.store(position_frames);
    return PdjResult_Ok;
}

uint64_t pdj_engine_position(PdjEngine* engine, uint32_t deck_index) {
    if (!engine || deck_index >= MAX_DECKS) { return 0; }
    return engine->decks[deck_index].position.load();
}

// ---------------------------------------------------------------------------
// Mixer controls
// ---------------------------------------------------------------------------

void pdj_engine_set_fader(PdjEngine* engine, uint32_t deck_index, float value) {
    if (!engine || deck_index >= MAX_DECKS) { return; }
    engine->decks[deck_index].fader.store(value);
}

void pdj_engine_set_crossfader(PdjEngine* engine, float value) {
    if (!engine) { return; }
    engine->crossfader.store(value);
}

void pdj_engine_set_stem_gain(PdjEngine* engine,
                               uint32_t   deck_index,
                               uint32_t   stem_index,
                               float      value) {
    if (!engine || deck_index >= MAX_DECKS) { return; }
    if (stem_index >= 4) { return; }
    engine->decks[deck_index].stem_gain[stem_index].store(value);
}

// ---------------------------------------------------------------------------
// Status
// ---------------------------------------------------------------------------

int pdj_engine_is_playing(PdjEngine* engine, uint32_t deck_index) {
    if (!engine || deck_index >= MAX_DECKS) { return 0; }
    return engine->decks[deck_index].playing.load() ? 1 : 0;
}
