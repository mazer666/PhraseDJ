/**
 * ringbuffer.hpp — Lock-free single-producer / single-consumer ring buffer.
 *
 * Used to pass decoded audio frames from a decoder thread to the realtime
 * audio callback without any mutex or allocation.
 *
 * Realtime-safety guarantee:
 *   - push() is called by the producer thread only.
 *   - pop() / read_available() are called by the consumer (audio callback) only.
 *   - No locks, no heap allocation after construction.
 *   - Indices are std::atomic with acquire/release ordering.
 *
 * Template parameters:
 *   T    — element type (e.g. float for PCM samples).
 *   Cap  — capacity in elements. Must be a power of two for the mask trick.
 */

#pragma once

#include <atomic>
#include <cstddef>
#include <cstring>
#include <array>
#include <cassert>

namespace pdj {

template <typename T, std::size_t Cap>
class RingBuffer {
    static_assert((Cap & (Cap - 1)) == 0, "RingBuffer capacity must be a power of two");

public:
    RingBuffer() : write_idx_(0), read_idx_(0) {}

    // Non-copyable, non-movable — the buffer is big and fixed in place.
    RingBuffer(const RingBuffer&) = delete;
    RingBuffer& operator=(const RingBuffer&) = delete;

    // ---------------------------------------------------------------------------
    // Producer API  (call only from the producer thread)
    // ---------------------------------------------------------------------------

    /**
     * Write up to `count` elements from `src` into the buffer.
     *
     * Returns the number of elements actually written (≤ count).
     * May write fewer than requested if the buffer is nearly full.
     */
    std::size_t push(const T* src, std::size_t count) noexcept {
        const std::size_t write = write_idx_.load(std::memory_order_relaxed);
        const std::size_t read  = read_idx_.load(std::memory_order_acquire);
        const std::size_t available = Cap - (write - read);   // free slots
        const std::size_t n = std::min(count, available);
        if (n == 0) return 0;

        // The buffer may wrap around; handle both halves.
        const std::size_t offset = write & mask_;
        const std::size_t first  = std::min(n, Cap - offset);
        std::memcpy(buf_.data() + offset, src, first * sizeof(T));
        if (n > first) {
            std::memcpy(buf_.data(), src + first, (n - first) * sizeof(T));
        }

        write_idx_.store(write + n, std::memory_order_release);
        return n;
    }

    // ---------------------------------------------------------------------------
    // Consumer API  (call only from the consumer / audio-callback thread)
    // ---------------------------------------------------------------------------

    /**
     * Read up to `count` elements into `dst`.
     *
     * Returns the number of elements actually read (may be ≤ count).
     */
    std::size_t pop(T* dst, std::size_t count) noexcept {
        const std::size_t read  = read_idx_.load(std::memory_order_relaxed);
        const std::size_t write = write_idx_.load(std::memory_order_acquire);
        const std::size_t filled = write - read;
        const std::size_t n = std::min(count, filled);
        if (n == 0) return 0;

        const std::size_t offset = read & mask_;
        const std::size_t first  = std::min(n, Cap - offset);
        std::memcpy(dst, buf_.data() + offset, first * sizeof(T));
        if (n > first) {
            std::memcpy(dst + first, buf_.data(), (n - first) * sizeof(T));
        }

        read_idx_.store(read + n, std::memory_order_release);
        return n;
    }

    /**
     * How many elements are ready to be read right now.
     *
     * Safe to call from either thread (gives a conservative lower bound from
     * the consumer's perspective).
     */
    std::size_t read_available() const noexcept {
        const std::size_t write = write_idx_.load(std::memory_order_acquire);
        const std::size_t read  = read_idx_.load(std::memory_order_relaxed);
        return write - read;
    }

    /**
     * How many free slots remain (conservative from the producer's perspective).
     */
    std::size_t write_available() const noexcept {
        return Cap - read_available();
    }

    /** True when the buffer contains no data. */
    bool empty() const noexcept { return read_available() == 0; }

    /** Reset to empty.  Only safe to call when both threads are stopped. */
    void reset() noexcept {
        write_idx_.store(0, std::memory_order_relaxed);
        read_idx_.store(0, std::memory_order_relaxed);
    }

    static constexpr std::size_t capacity() noexcept { return Cap; }

private:
    static constexpr std::size_t mask_ = Cap - 1;

    alignas(64) std::array<T, Cap> buf_{};   // cache-line aligned data
    alignas(64) std::atomic<std::size_t> write_idx_;
    alignas(64) std::atomic<std::size_t> read_idx_;
};

} // namespace pdj
