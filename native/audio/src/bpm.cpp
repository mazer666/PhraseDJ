/**
 * bpm.cpp — BPM detection via autocorrelation of onset energy.
 *
 * Algorithm (simplified):
 *   1. Downsample to ~11 025 Hz (every 4th frame from 44.1 kHz).
 *   2. Compute RMS energy in 10 ms windows.
 *   3. Half-wave rectify the energy delta (onset strength).
 *   4. Autocorrelate the onset signal over the 60–200 BPM range.
 *   5. Pick the peak; vote with its harmonics for robustness.
 *
 * This avoids any external dependencies (aubio, essentia).  A Phase 3
 * upgrade can replace this with a trained onset detector for better
 * accuracy on unusual tracks.
 */

#include "bpm.hpp"

#include <algorithm>
#include <cmath>
#include <numeric>
#include <vector>

namespace pdj {

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

static constexpr uint32_t DOWNSAMPLE_FACTOR = 4;    // 44100 → ~11025 Hz
static constexpr uint32_t WINDOW_MS         = 10;   // energy window in ms
static constexpr float    BPM_MIN           = 60.0f;
static constexpr float    BPM_MAX           = 200.0f;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Mix stereo to mono and downsample by `factor`. */
static std::vector<float> downsample_mono(const float* samples,
                                           uint64_t     frames,
                                           uint32_t     factor) {
    std::vector<float> out;
    out.reserve(frames / factor + 1);
    for (uint64_t i = 0; i < frames; i += factor) {
        float s = (samples[i * 2] + samples[i * 2 + 1]) * 0.5f;
        out.push_back(s);
    }
    return out;
}

/** Compute RMS energy in non-overlapping windows. */
static std::vector<float> energy_envelope(const std::vector<float>& mono,
                                           uint32_t window_samples) {
    std::vector<float> env;
    const std::size_t n = mono.size();
    env.reserve(n / window_samples + 1);
    for (std::size_t i = 0; i < n; i += window_samples) {
        float sum = 0.0f;
        const std::size_t end = std::min(i + window_samples, n);
        for (std::size_t j = i; j < end; ++j) sum += mono[j] * mono[j];
        env.push_back(std::sqrt(sum / static_cast<float>(end - i)));
    }
    return env;
}

/** Half-wave-rectified first derivative = onset strength. */
static std::vector<float> onset_strength(const std::vector<float>& env) {
    std::vector<float> onset(env.size(), 0.0f);
    for (std::size_t i = 1; i < env.size(); ++i) {
        float diff = env[i] - env[i - 1];
        onset[i] = (diff > 0.0f) ? diff : 0.0f;
    }
    return onset;
}

/** Autocorrelation of `sig` at lag `lag` (sum-of-products). */
static float autocorr(const std::vector<float>& sig, std::size_t lag) {
    if (lag >= sig.size()) return 0.0f;
    float sum = 0.0f;
    const std::size_t n = sig.size() - lag;
    for (std::size_t i = 0; i < n; ++i) sum += sig[i] * sig[i + lag];
    return sum;
}

// ---------------------------------------------------------------------------
// detect_bpm
// ---------------------------------------------------------------------------

BeatgridResult detect_bpm(const float* samples,
                            uint64_t     frame_count,
                            uint32_t     sample_rate) {
    BeatgridResult res{};
    res.bpm      = 120.0f;  // safe default
    res.reliable = false;

    if (frame_count < static_cast<uint64_t>(sample_rate)) {
        // Too short to analyse meaningfully.
        res.markers = build_beatgrid(res.bpm, 0, frame_count, sample_rate);
        return res;
    }

    // Use at most 60 seconds for speed.
    const uint64_t analyse_frames = std::min(frame_count,
        static_cast<uint64_t>(sample_rate) * 60);

    const uint32_t ds_rate = sample_rate / DOWNSAMPLE_FACTOR;
    const uint32_t win_samples = std::max(1u, ds_rate * WINDOW_MS / 1000u);

    auto mono   = downsample_mono(samples, analyse_frames, DOWNSAMPLE_FACTOR);
    auto env    = energy_envelope(mono, win_samples);
    auto onset  = onset_strength(env);

    // BPM → lag in onset-envelope samples.
    // lag = (60 / bpm) * (ds_rate / win_samples)
    const float env_fps = static_cast<float>(ds_rate) /
                          static_cast<float>(win_samples);

    const std::size_t lag_min = static_cast<std::size_t>(
        std::floor(60.0f / BPM_MAX * env_fps));
    const std::size_t lag_max = static_cast<std::size_t>(
        std::ceil(60.0f / BPM_MIN * env_fps));

    // Find the lag with the highest autocorrelation.
    float best_corr = -1.0f;
    std::size_t best_lag = (lag_min + lag_max) / 2;

    for (std::size_t lag = lag_min; lag <= lag_max && lag < onset.size(); ++lag) {
        // Sum autocorrelation at lag and its first harmonic for robustness.
        float c = autocorr(onset, lag);
        if (lag * 2 < onset.size()) c += 0.5f * autocorr(onset, lag * 2);
        if (c > best_corr) { best_corr = c; best_lag = lag; }
    }

    if (best_lag > 0 && best_corr > 0.0f) {
        res.bpm      = 60.0f * env_fps / static_cast<float>(best_lag);
        res.reliable = true;
    }

    // Clamp to a sensible range (handles halving/doubling artefacts).
    while (res.bpm < BPM_MIN) res.bpm *= 2.0f;
    while (res.bpm > BPM_MAX) res.bpm /= 2.0f;

    // Estimate first beat from the first onset above a threshold.
    float threshold = 0.0f;
    for (float v : onset) threshold += v;
    threshold = (threshold / static_cast<float>(onset.size())) * 2.0f;

    uint64_t first_env_window = 0;
    for (std::size_t i = 0; i < onset.size(); ++i) {
        if (onset[i] > threshold) { first_env_window = i; break; }
    }

    res.first_beat = static_cast<uint64_t>(first_env_window) *
                     static_cast<uint64_t>(win_samples) *
                     static_cast<uint64_t>(DOWNSAMPLE_FACTOR);

    res.markers = build_beatgrid(res.bpm, res.first_beat,
                                  frame_count, sample_rate);
    return res;
}

// ---------------------------------------------------------------------------
// build_beatgrid
// ---------------------------------------------------------------------------

std::vector<BeatMarker> build_beatgrid(float    bpm,
                                        uint64_t first_beat_frame,
                                        uint64_t total_frames,
                                        uint32_t sample_rate) {
    std::vector<BeatMarker> markers;
    if (bpm <= 0.0f || sample_rate == 0) return markers;

    const double beat_frames = 60.0 / static_cast<double>(bpm) *
                               static_cast<double>(sample_rate);
    uint64_t pos = first_beat_frame;
    while (pos < total_frames) {
        markers.push_back(BeatMarker{ pos, bpm });
        pos += static_cast<uint64_t>(beat_frames + 0.5);
    }
    return markers;
}

} // namespace pdj
