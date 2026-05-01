/**
 * engine_stub_test.cpp — Smoke tests for the engine C ABI.
 *
 * These tests verify the lifecycle and control plane of the engine.
 * They do NOT require a working audio device — the engine still creates
 * decks and accepts control calls when the backend can't open a device
 * (e.g. CI, container, or headless macOS runner).
 */

#include "pdj_engine.h"

#include <gtest/gtest.h>

namespace {

/// Create an engine with standard config for testing.
PdjEngine* make_engine() {
    PdjEngineConfig cfg{44100, 128, 2};
    return pdj_engine_create(&cfg);
}

} // namespace

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

TEST(EngineLifecycle, CreateAndDestroy) {
    PdjEngine* e = make_engine();
    ASSERT_NE(e, nullptr);
    pdj_engine_destroy(e);
}

TEST(EngineLifecycle, CreateWithNullConfigReturnsNull) {
    EXPECT_EQ(pdj_engine_create(nullptr), nullptr);
}

// ---------------------------------------------------------------------------
// Deck control
// ---------------------------------------------------------------------------

TEST(DeckControl, NoFileLoadedInitially) {
    PdjEngine* e = make_engine();
    EXPECT_EQ(pdj_engine_is_loaded(e, 0), 0);
    EXPECT_EQ(pdj_engine_is_loaded(e, 1), 0);
    pdj_engine_destroy(e);
}

TEST(DeckControl, PlayPauseOnEmptyDeckIsNoOp) {
    // No file loaded — playing toggles the state but produces no audio.
    PdjEngine* e = make_engine();
    EXPECT_EQ(pdj_engine_play(e, 0), PdjResult_Ok);
    EXPECT_EQ(pdj_engine_pause(e, 0), PdjResult_Ok);
    pdj_engine_destroy(e);
}

TEST(DeckControl, InvalidDeckIndexReturnsError) {
    PdjEngine* e = make_engine();
    EXPECT_EQ(pdj_engine_play(e, 99), PdjResult_InvalidArg);
    EXPECT_EQ(pdj_engine_pause(e, 99), PdjResult_InvalidArg);
    EXPECT_EQ(pdj_engine_load(e, 99, "/tmp/x.wav"), PdjResult_InvalidArg);
    pdj_engine_destroy(e);
}

TEST(DeckControl, LoadNonexistentFileReturnsIo) {
    PdjEngine* e = make_engine();
    EXPECT_EQ(pdj_engine_load(e, 0, "/this/file/does/not/exist.wav"),
              PdjResult_Io);
    pdj_engine_destroy(e);
}

TEST(DeckControl, LoadWithNullPathReturnsInvalid) {
    PdjEngine* e = make_engine();
    EXPECT_EQ(pdj_engine_load(e, 0, nullptr), PdjResult_InvalidArg);
    pdj_engine_destroy(e);
}

// ---------------------------------------------------------------------------
// Mixer
// ---------------------------------------------------------------------------

TEST(Mixer, FaderAndCrossfaderAreNoOp) {
    PdjEngine* e = make_engine();
    pdj_engine_set_fader(e, 0, 0.7f);
    pdj_engine_set_fader(e, 1, 0.3f);
    pdj_engine_set_crossfader(e, 0.4f);
    pdj_engine_set_master_gain(e, 0.85f);
    pdj_engine_destroy(e);
}

TEST(Mixer, SetStemGainAcceptsAllStems) {
    PdjEngine* e = make_engine();
    for (uint32_t s = 0; s < 4; ++s)
        pdj_engine_set_stem_gain(e, 0, s, 0.5f);
    pdj_engine_set_stem_gain(e, 0, 99, 0.5f);  // out-of-range, no crash
    pdj_engine_destroy(e);
}

// ---------------------------------------------------------------------------
// BPM
// ---------------------------------------------------------------------------

TEST(Bpm, AnalyseRequiresValidArgs) {
    PdjEngine* e = make_engine();
    EXPECT_EQ(pdj_engine_analyse_bpm(e, 0, nullptr, 0), PdjResult_InvalidArg);
    EXPECT_FLOAT_EQ(pdj_engine_get_bpm(e, 0), 120.0f);  // default
    pdj_engine_destroy(e);
}

// ---------------------------------------------------------------------------
// Null-safety
// ---------------------------------------------------------------------------

TEST(NullSafety, AllFunctionsHandleNullEngine) {
    pdj_engine_destroy(nullptr);
    pdj_engine_play(nullptr, 0);
    pdj_engine_pause(nullptr, 0);
    pdj_engine_seek(nullptr, 0, 0);
    EXPECT_EQ(pdj_engine_position(nullptr, 0), 0u);
    pdj_engine_set_fader(nullptr, 0, 1.0f);
    pdj_engine_set_crossfader(nullptr, 0.5f);
    pdj_engine_set_master_gain(nullptr, 1.0f);
    pdj_engine_set_stem_gain(nullptr, 0, 0, 1.0f);
    EXPECT_EQ(pdj_engine_is_playing(nullptr, 0), 0);
    EXPECT_EQ(pdj_engine_is_loaded(nullptr, 0), 0);
    EXPECT_EQ(pdj_engine_is_running(nullptr), 0);
    EXPECT_FLOAT_EQ(pdj_engine_get_bpm(nullptr, 0), 0.0f);
}
