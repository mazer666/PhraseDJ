/**
 * engine_stub_test.cpp — Unit tests for the Phase 0 audio engine stub.
 *
 * These tests verify that the C ABI functions behave correctly even before
 * real audio output is implemented.  They run in CI via ctest.
 */

#include "pdj_engine.h"

#include <gtest/gtest.h>

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create an engine with standard defaults for testing.
PdjEngine* make_engine() {
    PdjEngineConfig cfg{44100, 128, 2};
    return pdj_engine_create(&cfg);
}

// ---------------------------------------------------------------------------
// Lifecycle tests
// ---------------------------------------------------------------------------

TEST(EngineLifecycle, CreateAndDestroy) {
    PdjEngine* e = make_engine();
    ASSERT_NE(e, nullptr);
    pdj_engine_destroy(e);
}

TEST(EngineLifecycle, CreateWithNullConfigReturnsNull) {
    PdjEngine* e = pdj_engine_create(nullptr);
    EXPECT_EQ(e, nullptr);
}

// ---------------------------------------------------------------------------
// Deck control tests
// ---------------------------------------------------------------------------

TEST(DeckControl, PlayAndPause) {
    PdjEngine* e = make_engine();

    // Initially not playing.
    EXPECT_EQ(pdj_engine_is_playing(e, 0), 0);

    // After play, is playing.
    EXPECT_EQ(pdj_engine_play(e, 0), PdjResult_Ok);
    EXPECT_EQ(pdj_engine_is_playing(e, 0), 1);

    // After pause, not playing.
    EXPECT_EQ(pdj_engine_pause(e, 0), PdjResult_Ok);
    EXPECT_EQ(pdj_engine_is_playing(e, 0), 0);

    pdj_engine_destroy(e);
}

TEST(DeckControl, SeekSetsPosition) {
    PdjEngine* e = make_engine();
    EXPECT_EQ(pdj_engine_seek(e, 0, 44100), PdjResult_Ok);
    EXPECT_EQ(pdj_engine_position(e, 0), 44100U);
    pdj_engine_destroy(e);
}

TEST(DeckControl, InvalidDeckIndexReturnsError) {
    PdjEngine* e = make_engine();
    EXPECT_EQ(pdj_engine_play(e, 99), PdjResult_InvalidArg);
    EXPECT_EQ(pdj_engine_pause(e, 99), PdjResult_InvalidArg);
    pdj_engine_destroy(e);
}

TEST(DeckControl, LoadAcceptsValidPath) {
    PdjEngine* e = make_engine();
    EXPECT_EQ(pdj_engine_load(e, 0, "/tmp/test.flac"), PdjResult_Ok);
    pdj_engine_destroy(e);
}

TEST(DeckControl, LoadWithNullPathReturnsError) {
    PdjEngine* e = make_engine();
    EXPECT_EQ(pdj_engine_load(e, 0, nullptr), PdjResult_InvalidArg);
    pdj_engine_destroy(e);
}

// ---------------------------------------------------------------------------
// Mixer tests
// ---------------------------------------------------------------------------

TEST(Mixer, SetFaderAndCrossfaderDoNotCrash) {
    PdjEngine* e = make_engine();
    pdj_engine_set_fader(e, 0, 0.8f);
    pdj_engine_set_fader(e, 1, 0.5f);
    pdj_engine_set_crossfader(e, 0.3f);
    // No assertions on values in Phase 0 (no getter yet); just no crash.
    pdj_engine_destroy(e);
}

TEST(Mixer, SetStemGainAllStems) {
    PdjEngine* e = make_engine();
    for (uint32_t stem = 0; stem < 4; ++stem) {
        pdj_engine_set_stem_gain(e, 0, stem, 0.5f);
    }
    // Invalid stem index: should not crash.
    pdj_engine_set_stem_gain(e, 0, 99, 0.5f);
    pdj_engine_destroy(e);
}

// ---------------------------------------------------------------------------
// Null-safety: all public functions must tolerate null engine pointer.
// ---------------------------------------------------------------------------

TEST(NullSafety, AllFunctionsHandleNullEngine) {
    pdj_engine_destroy(nullptr);  // must not crash
    pdj_engine_play(nullptr, 0);
    pdj_engine_pause(nullptr, 0);
    pdj_engine_seek(nullptr, 0, 0);
    pdj_engine_position(nullptr, 0);
    pdj_engine_set_fader(nullptr, 0, 1.0f);
    pdj_engine_set_crossfader(nullptr, 0.5f);
    pdj_engine_set_stem_gain(nullptr, 0, 0, 1.0f);
    pdj_engine_is_playing(nullptr, 0);
    // Reaching here without segfault = pass.
}
