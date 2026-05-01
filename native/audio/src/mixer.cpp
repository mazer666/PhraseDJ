/**
 * mixer.cpp — Master mixer implementation.
 */

#include "mixer.hpp"

#include <algorithm>
#include <cmath>
#include <cstring>

namespace pdj {

Mixer::Mixer(std::array<Deck*, MAX_DECKS> decks)
    : decks_(decks)
{}

// ---------------------------------------------------------------------------
// process_block  (realtime-safe)
// ---------------------------------------------------------------------------

void Mixer::process_block(float* out, uint32_t frames) noexcept {
    // Zero the output buffer first.
    std::memset(out, 0, static_cast<std::size_t>(frames) * 2 * sizeof(float));

    const float xf  = crossfader_.load();
    const float mg  = master_gain_.load();

    for (int d = 0; d < MAX_DECKS; ++d) {
        if (!decks_[d]) continue;

        // Clear per-deck scratch, then ask the deck to mix into it.
        const uint32_t safe_frames = std::min(frames, SCRATCH);
        std::memset(deck_buf_[d], 0,
                    static_cast<std::size_t>(safe_frames) * 2 * sizeof(float));
        decks_[d]->mix_into(deck_buf_[d], safe_frames);

        // Apply crossfader gain.
        const float xg = xfade_gain(xf, d);

        for (uint32_t i = 0; i < safe_frames * 2; ++i) {
            out[i] += deck_buf_[d][i] * xg * mg;
        }
    }

    // Brick-wall limiter at -0.3 dBFS (≈ 0.966 linear).
    limit(out, frames * 2, 0.966f);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

float Mixer::xfade_gain(float pos, int side) noexcept {
    // Equal-power law: deck A = cos(pos * π/2), deck B = sin(pos * π/2).
    // Evaluated at 0 → A=1,B=0; at 0.5 → A≈0.707,B≈0.707; at 1 → A=0,B=1.
    constexpr float HALF_PI = 1.5707963267948966f;
    if (side == 0) return std::cos(pos * HALF_PI);
    return std::sin(pos * HALF_PI);
}

void Mixer::limit(float* buf, uint32_t samples, float ceil) noexcept {
    for (uint32_t i = 0; i < samples; ++i) {
        if (buf[i] >  ceil) buf[i] =  ceil;
        if (buf[i] < -ceil) buf[i] = -ceil;
    }
}

} // namespace pdj
