/**
 * decoder.hpp — Audio file decoder interface.
 *
 * Decodes audio files (FLAC, WAV, AIFF, MP3, OGG) to interleaved 32-bit
 * float PCM at a fixed output sample rate.  Decoding is synchronous and
 * designed to run on a background thread, feeding a RingBuffer consumed by
 * the audio callback.
 *
 * The output format is always:
 *   - 32-bit float
 *   - Interleaved stereo (2 channels).  Mono sources are up-mixed.
 *   - Sample rate = whatever the engine was opened with.
 */

#pragma once

#include <cstddef>
#include <cstdint>
#include <memory>
#include <string>

namespace pdj {

/** Metadata read from the file without a full decode. */
struct AudioInfo {
    uint64_t    total_frames;  ///< Total PCM frames in the file.
    uint32_t    sample_rate;   ///< Native sample rate of the file.
    int         channels;      ///< Channel count in the file.
    double      duration_secs; ///< Derived: total_frames / sample_rate.
};

/** Result codes returned by Decoder operations. */
enum class DecodeResult {
    Ok,            ///< Success.
    EndOfFile,     ///< No more data; decoding is complete.
    FormatError,   ///< File format not recognised or corrupt.
    IoError,       ///< Filesystem error.
    SeekError,     ///< Seek position out of range.
};

/**
 * Single-file audio decoder.
 *
 * Usage:
 *   1. Construct with open().
 *   2. Call read_frames() in a loop to fill a RingBuffer.
 *   3. Call seek_to() for scrubbing / cue jumps.
 */
class Decoder {
public:
    virtual ~Decoder() = default;

    /** Open a file.  Returns nullptr on format or I/O error. */
    static std::unique_ptr<Decoder> open(const std::string& path,
                                          uint32_t target_sample_rate);

    /** Metadata (valid after successful open). */
    virtual const AudioInfo& info() const = 0;

    /**
     * Read up to `frame_count` interleaved stereo frames into `out`.
     *
     * `out` must point to at least `frame_count * 2` floats.
     * Sets `frames_read` to the number of frames actually written.
     */
    virtual DecodeResult read_frames(float*   out,
                                      uint32_t frame_count,
                                      uint32_t& frames_read) = 0;

    /**
     * Seek to `frame` (absolute, in output sample-rate frames).
     *
     * Thread-safe only if called while no read_frames() is in progress.
     */
    virtual DecodeResult seek_to(uint64_t frame) = 0;

    /** Current read position in output frames. */
    virtual uint64_t position() const = 0;
};

} // namespace pdj
