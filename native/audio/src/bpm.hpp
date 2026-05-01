/**
 * bpm.hpp — BPM detection and beatgrid helpers.
 *
 * Uses autocorrelation on the onset-energy envelope to estimate BPM.
 * Runs entirely offline (not in the audio callback); call from a
 * background analysis thread.
 *
 * Accuracy: ±0.5 BPM for most electronic music in the 60–200 BPM range.
 * A manual BPM override and beatgrid tap function are provided for edge cases.
 */

#pragma once

#include <cstdint>
#include <string>
#include <vector>

namespace pdj {

/** A beat position marker. */
struct BeatMarker {
    uint64_t frame;   ///< Position in output-sample-rate frames.
    float    bpm;     ///< Local BPM at this beat (for tracks with tempo changes).
};

/** Result of a BPM analysis run. */
struct BeatgridResult {
    float                    bpm;           ///< Estimated global BPM.
    uint64_t                 first_beat;    ///< Frame of the first detected beat.
    std::vector<BeatMarker>  markers;       ///< All beat markers for the track.
    bool                     reliable;      ///< False if the estimate is uncertain.
};

/**
 * Analyse the audio in `samples` (interleaved stereo, `frame_count` frames)
 * and return a beatgrid estimate.
 *
 * @param samples      Pointer to interleaved stereo float PCM.
 * @param frame_count  Number of stereo frames.
 * @param sample_rate  Sample rate of `samples` (e.g. 44100).
 */
BeatgridResult detect_bpm(const float* samples,
                            uint64_t     frame_count,
                            uint32_t     sample_rate);

/**
 * Build a uniform beatgrid given a BPM, first-beat position, and total length.
 *
 * Useful after the user manually taps or corrects the BPM.
 */
std::vector<BeatMarker> build_beatgrid(float    bpm,
                                        uint64_t first_beat_frame,
                                        uint64_t total_frames,
                                        uint32_t sample_rate);

} // namespace pdj
