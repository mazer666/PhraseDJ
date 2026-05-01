/**
 * mixer.hpp — Master mixer: sums decks, applies crossfader, brick-wall limiter.
 *
 * Called from the audio callback once per buffer.  Fully realtime-safe:
 * all parameters are read from atomics, no allocation, no I/O.
 */

#pragma once

#include "deck.hpp"

#include <algorithm>
#include <array>
#include <atomic>
#include <cstdint>

namespace pdj {

constexpr int MAX_DECKS = 2;

/**
 * Master mixer.
 *
 * Holds references to the two decks.  The audio callback calls
 * process_block() to produce the final output buffer.
 */
class Mixer {
public:
    explicit Mixer(std::array<Deck*, MAX_DECKS> decks);

    // ------------------------------------------------------------------
    // Realtime-safe setters
    // ------------------------------------------------------------------

    /** Master output gain (0–1). */
    void set_master_gain(float v) noexcept { master_gain_.store(v); }

    /**
     * Crossfader position (0.0 = only Deck A, 1.0 = only Deck B).
     * Uses an equal-power law so the perceived volume stays constant.
     */
    void set_crossfader(float v) noexcept {
        crossfader_.store(std::clamp(v, 0.0f, 1.0f));
    }

    // ------------------------------------------------------------------
    // Audio callback
    // ------------------------------------------------------------------

    /**
     * Mix all decks into `out` (interleaved stereo, `frames` frames).
     *
     * Clears `out` first.  Applies crossfader, master gain, and a
     * brick-wall limiter at -0.3 dBFS true peak.
     *
     * Realtime-safe: no allocation, no locks, no I/O.
     */
    void process_block(float* out, uint32_t frames) noexcept;

private:
    /** Equal-power crossfader gain for one side (0=left, 1=right). */
    static float xfade_gain(float pos, int side) noexcept;

    /** Inline brick-wall limiter at the given ceiling. */
    static void limit(float* buf, uint32_t samples, float ceil) noexcept;

    std::array<Deck*, MAX_DECKS> decks_;
    std::atomic<float>           crossfader_{0.5f};
    std::atomic<float>           master_gain_{1.0f};

    // Per-deck scratch buffers (no alloc in callback).
    static constexpr uint32_t SCRATCH = 4096;
    float deck_buf_[MAX_DECKS][SCRATCH * 2]{};
};

} // namespace pdj
