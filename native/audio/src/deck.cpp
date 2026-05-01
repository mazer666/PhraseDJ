/**
 * deck.cpp — Deck implementation.
 *
 * Keeps a ring buffer prefilled from a background thread so the audio
 * callback never has to wait for disk I/O or decoding.
 */

#include "deck.hpp"

#include <algorithm>
#include <chrono>
#include <cstring>
#include <vector>

namespace pdj {

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

Deck::Deck() = default;

Deck::~Deck() {
    // Signal the prefetch thread to stop and wait for it to exit.
    prefetch_run_.store(false);
    wake_prefetch();
    if (prefetch_thread_.joinable()) prefetch_thread_.join();
}

// ---------------------------------------------------------------------------
// Load
// ---------------------------------------------------------------------------

bool Deck::load(const std::string& path, uint32_t sample_rate) {
    // Stop the old prefetch thread before swapping the decoder.
    prefetch_run_.store(false);
    wake_prefetch();
    if (prefetch_thread_.joinable()) prefetch_thread_.join();

    // Open the new file.
    auto dec = Decoder::open(path, sample_rate);
    if (!dec) return false;

    // Swap decoder and reset state (single caller, no RT contention here).
    state_.playing.store(false);
    sample_rate_ = sample_rate;
    audio_info_  = dec->info();
    decoder_     = std::move(dec);
    position_.store(0);
    eof_.store(false);
    loaded_.store(true);
    ring_.reset();

    // Launch new prefetch thread.
    prefetch_run_.store(true);
    prefetch_thread_ = std::thread(&Deck::prefetch_loop, this);

    return true;
}

// ---------------------------------------------------------------------------
// Transport controls
// ---------------------------------------------------------------------------

void Deck::play()  { state_.playing.store(true);  }
void Deck::pause() { state_.playing.store(false); }

void Deck::seek(uint64_t frame) {
    seek_target_.store(frame);
    seek_pending_.store(true);
    wake_prefetch();
}

uint64_t Deck::position() const { return position_.load(); }

const AudioInfo* Deck::audio_info() const {
    return loaded_.load() ? &audio_info_ : nullptr;
}

// ---------------------------------------------------------------------------
// Audio callback — mix_into  (realtime-safe)
// ---------------------------------------------------------------------------

uint32_t Deck::mix_into(float* out, uint32_t frames) noexcept {
    if (!state_.playing.load()) return 0;

    // Pop from the ring buffer into our pre-allocated scratch.
    const uint32_t to_read = std::min(frames, SCRATCH_FRAMES);
    const std::size_t samples = ring_.pop(scratch_, to_read * 2);
    const uint32_t frames_got = static_cast<uint32_t>(samples) / 2;

    if (frames_got == 0) return 0;

    // Apply channel gain and accumulate into the output buffer.
    const float g = state_.gain.load();
    for (uint32_t i = 0; i < frames_got * 2; ++i) {
        out[i] += scratch_[i] * g;
    }

    position_.fetch_add(frames_got);
    return frames_got;
}

// ---------------------------------------------------------------------------
// Prefetch thread
// ---------------------------------------------------------------------------

void Deck::prefetch_loop() {
    // Buffer we decode into (on heap — fine since this is not the RT thread).
    constexpr uint32_t CHUNK = 2048;   // frames per decode call
    std::vector<float> buf(CHUNK * 2);

    while (prefetch_run_.load()) {
        // Handle pending seek.
        if (seek_pending_.load()) {
            seek_pending_.store(false);
            const uint64_t target = seek_target_.load();
            if (decoder_) {
                decoder_->seek_to(target);
                position_.store(target);
                eof_.store(false);
                ring_.reset();
            }
        }

        // Fill the ring buffer up to 75 % capacity.
        if (!eof_.load() && decoder_) {
            while (ring_.write_available() > CHUNK * 2) {
                uint32_t got = 0;
                const auto res = decoder_->read_frames(buf.data(), CHUNK, got);
                if (got > 0) {
                    ring_.push(buf.data(), got * 2);
                }
                if (res == DecodeResult::EndOfFile) {
                    eof_.store(true);
                    state_.playing.store(false);
                    break;
                }
                if (res != DecodeResult::Ok) break;
            }
        }

        // Sleep until woken or a 20 ms refresh.
        {
            std::unique_lock<std::mutex> lock(prefetch_mutex_);
            prefetch_cv_.wait_for(lock, std::chrono::milliseconds(20),
                [this] { return !prefetch_run_.load() ||
                                seek_pending_.load() ||
                                ring_.write_available() > DECK_RING_CAP / 2; });
        }
    }
}

void Deck::wake_prefetch() {
    std::lock_guard<std::mutex> lock(prefetch_mutex_);
    prefetch_cv_.notify_one();
}

} // namespace pdj
