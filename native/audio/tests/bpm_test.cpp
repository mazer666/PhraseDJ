/**
 * bpm_test.cpp — BPM detection sanity tests.
 *
 * Generates a synthetic click track at a known BPM and checks that the
 * detector recovers the tempo within ±2 BPM tolerance.
 */

#include "bpm.hpp"

#include <gtest/gtest.h>
#include <cmath>
#include <vector>

using pdj::detect_bpm;
using pdj::build_beatgrid;

namespace {

/// Generate a synthetic stereo click-track at `bpm` for `seconds` seconds.
std::vector<float> make_click_track(float bpm, float seconds, uint32_t sr) {
    const auto frames = static_cast<uint64_t>(seconds * static_cast<float>(sr));
    std::vector<float> samples(frames * 2, 0.0f);

    const double beat_frames = 60.0 / static_cast<double>(bpm) *
                               static_cast<double>(sr);
    const uint32_t click_len = sr / 100;   // 10 ms click

    for (double pos = 0; pos < static_cast<double>(frames);
         pos += beat_frames) {
        const auto start = static_cast<uint64_t>(pos);
        for (uint32_t i = 0; i < click_len && (start + i) < frames; ++i) {
            // Decaying impulse on both channels.
            const float v = std::exp(-static_cast<float>(i) / 50.0f) * 0.8f;
            samples[(start + i) * 2]     = v;
            samples[(start + i) * 2 + 1] = v;
        }
    }
    return samples;
}

} // namespace

TEST(Bpm, Detect120Bpm) {
    constexpr uint32_t SR = 44100;
    auto pcm = make_click_track(120.0f, 20.0f, SR);
    auto res = detect_bpm(pcm.data(), pcm.size() / 2, SR);
    EXPECT_NEAR(res.bpm, 120.0f, 2.0f);
    EXPECT_TRUE(res.reliable);
    EXPECT_GT(res.markers.size(), 0u);
}

TEST(Bpm, Detect140Bpm) {
    constexpr uint32_t SR = 44100;
    auto pcm = make_click_track(140.0f, 20.0f, SR);
    auto res = detect_bpm(pcm.data(), pcm.size() / 2, SR);
    EXPECT_NEAR(res.bpm, 140.0f, 2.0f);
}

TEST(Bpm, Detect95Bpm) {
    constexpr uint32_t SR = 44100;
    auto pcm = make_click_track(95.0f, 20.0f, SR);
    auto res = detect_bpm(pcm.data(), pcm.size() / 2, SR);
    EXPECT_NEAR(res.bpm, 95.0f, 2.0f);
}

TEST(Bpm, BuildBeatgridProducesEvenlySpacedMarkers) {
    auto markers = build_beatgrid(120.0f, 0, 44100 * 10, 44100);
    ASSERT_GT(markers.size(), 1u);
    const auto delta = markers[1].frame - markers[0].frame;
    EXPECT_NEAR(static_cast<double>(delta), 22050.0, 5.0);
}

TEST(Bpm, ShortInputDoesNotCrash) {
    std::vector<float> tiny(100, 0.0f);
    auto res = detect_bpm(tiny.data(), 50, 44100);
    EXPECT_FALSE(res.reliable);
    EXPECT_GT(res.bpm, 0.0f);
}
