/**
 * deck.hpp — One playback deck.
 *
 * A Deck owns a Decoder and a RingBuffer.  A background thread keeps the
 * RingBuffer filled.  The realtime audio callback reads from the buffer
 * without any locking.
 *
 * Thread model:
 *   - Loader thread: calls load(), prefill().
 *   - Prefetch thread: calls fill_buffer() continuously.
 *   - Audio callback (RT): calls read_frames(), set_*() setters.
 *   - Control thread (Rust/UI): calls play(), pause(), seek(), set_*().
 *
 * Setters that are realtime-safe use atomics.  Non-RT operations (load,
 * seek) signal the prefetch thread and do work there.
 */

#pragma once

#include "decoder.hpp"
#include "ringbuffer.hpp"

#include <atomic>
#include <condition_variable>
#include <memory>
#include <mutex>
#include <string>
#include <thread>

namespace pdj {

// Ring buffer large enough for ~2 seconds of stereo float at 48 kHz.
// 2 * 48000 * 2 = 192 000 ≈ 262 144 = 2^18
constexpr std::size_t DECK_RING_CAP = 1 << 18;  // 262 144 floats (~1 MB)

/**
 * Playback state of a deck — read by the audio callback atomically.
 */
struct DeckPlayState {
    std::atomic<bool>     playing{false};
    std::atomic<float>    gain{1.0f};    ///< Channel fader (0–1).
    std::atomic<float>    pitch{1.0f};   ///< Playback rate (1.0 = normal).

    // Stem gains: [0]=vocals [1]=drums [2]=bass [3]=other.
    std::atomic<float>    stem_gain[4];

    DeckPlayState() {
        for (auto& g : stem_gain) g.store(1.0f);
    }
};

/**
 * One playback deck.
 */
class Deck {
public:
    Deck();
    ~Deck();

    // Non-copyable.
    Deck(const Deck&) = delete;
    Deck& operator=(const Deck&) = delete;

    // ------------------------------------------------------------------
    // Control API  (called from the Rust/UI thread)
    // ------------------------------------------------------------------

    /** Load a file.  Stops any current playback and starts prefetching. */
    bool load(const std::string& path, uint32_t sample_rate);

    /** Load 4 stems. */
    bool load_stems(const std::string& path_main,
                    const std::string& path_v,
                    const std::string& path_d,
                    const std::string& path_b,
                    const std::string& path_o,
                    uint32_t sample_rate);

    /** Start playback. */
    void play();

    /** Pause playback.  Position is retained. */
    void pause();

    /**
     * Seek to `frame` (in output sample-rate frames).
     * Issues a seek on the decoder and re-fills the ring buffer.
     */
    void seek(uint64_t frame);

    /** Return the playback position in output frames. */
    uint64_t position() const;

    /** Return metadata if a file is loaded; nullptr otherwise. */
    const AudioInfo* audio_info() const;

    // ------------------------------------------------------------------
    // Realtime-safe setters  (called from the audio callback)
    // ------------------------------------------------------------------

    void set_gain(float v)             noexcept { state_.gain.store(v);         }
    void set_pitch(float v)            noexcept { state_.pitch.store(v);        }
    float get_pitch()            const noexcept { return state_.pitch.load();   }
    void set_stem_gain(int s, float v) noexcept {
        if (s >= 0 && s < 4) state_.stem_gain[s].store(v);
    }
    void set_playing(bool v)           noexcept { state_.playing.store(v);      }

    // ------------------------------------------------------------------
    // Audio callback API  (called from the RT thread only)
    // ------------------------------------------------------------------

    /**
     * Mix `frames` stereo frames into `out` (interleaved L/R floats).
     *
     * Applies channel gain.  Returns the number of frames actually mixed
     * (may be less if the buffer runs dry — caller should zero-fill the rest).
     *
     * Realtime-safe: no allocation, no locks, no I/O.
     */
    uint32_t mix_into(float* out, uint32_t frames) noexcept;

    bool is_playing()  const noexcept { return state_.playing.load(); }
    bool is_loaded()   const noexcept { return loaded_.load();        }

private:
    /** Background thread that keeps the ring buffer full. */
    void prefetch_loop();

    /** Trigger the prefetch thread to wake and fill. */
    void wake_prefetch();

    DeckPlayState  state_;
    std::unique_ptr<Decoder>  decoders_[4];
    RingBuffer<float, DECK_RING_CAP> rings_[4];

    AudioInfo                 audio_info_{};
    std::atomic<uint64_t>     position_{0};
    std::atomic<bool>         loaded_{false};
    std::atomic<bool>         has_stems_{false};
    std::atomic<bool>         eof_{false};

    // Prefetch thread synchronisation.
    std::thread               prefetch_thread_;
    std::mutex                prefetch_mutex_;
    std::condition_variable   prefetch_cv_;
    std::atomic<bool>         prefetch_run_{false};
    std::atomic<bool>         seek_pending_{false};
    std::atomic<uint64_t>     seek_target_{0};

    uint32_t  sample_rate_{44100};

    // Small mix scratch buffer (no alloc in callback).
    static constexpr uint32_t SCRATCH_FRAMES = 4096;
    float scratch_[SCRATCH_FRAMES * 2]{};
    float stem_scratch_[SCRATCH_FRAMES * 2]{};
};

} // namespace pdj
