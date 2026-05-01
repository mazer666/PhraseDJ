/**
 * decoder.cpp — Audio decoder implementation using libsndfile.
 *
 * Supports: WAV, FLAC, AIFF, OGG/Vorbis, and any other format libsndfile
 * handles.  MP3 support is provided by the libsndfile 1.1+ MPEG layer.
 *
 * Output is always resampled to the engine's target sample rate using a
 * simple linear interpolation.  A higher-quality SRC (libsamplerate) can
 * replace this in a later phase without changing the Decoder interface.
 */

#include "decoder.hpp"

#include <algorithm>
#include <cmath>
#include <cstring>
#include <sndfile.h>
#include <stdexcept>
#include <vector>

namespace pdj {

// ---------------------------------------------------------------------------
// Linear resampler helper
// ---------------------------------------------------------------------------

/**
 * A simple linear-interpolation resampler.
 *
 * Quality is acceptable for most DJ use-cases.  Phase 3 may upgrade to
 * a sinc-based resampler for better alias suppression.
 */
class LinearResampler {
public:
    LinearResampler(uint32_t src_rate, uint32_t dst_rate, int channels)
        : ratio_(static_cast<double>(src_rate) / static_cast<double>(dst_rate))
        , channels_(channels)
        , phase_(0.0)
        , prev_(channels, 0.0f)
        , curr_(channels, 0.0f)
    {}

    /**
     * Resample `src` (src_frames * channels_ floats) into `dst`.
     *
     * Returns the number of output frames written.
     */
    uint32_t process(const float* src, uint32_t src_frames,
                     float* dst, uint32_t max_dst_frames) {
        uint32_t out_written = 0;
        uint32_t src_pos = 0;

        while (out_written < max_dst_frames) {
            // Advance phase by the ratio until we need a new source frame.
            while (phase_ >= 1.0) {
                if (src_pos >= src_frames) return out_written;
                for (int c = 0; c < channels_; ++c) {
                    prev_[c] = curr_[c];
                    curr_[c] = src[src_pos * channels_ + c];
                }
                ++src_pos;
                phase_ -= 1.0;
            }

            // Interpolate between prev and curr.
            float t = static_cast<float>(phase_);
            for (int c = 0; c < channels_; ++c) {
                dst[out_written * channels_ + c] =
                    prev_[c] + t * (curr_[c] - prev_[c]);
            }
            ++out_written;
            phase_ += ratio_;
        }
        return out_written;
    }

    void reset() {
        phase_ = 0.0;
        std::fill(prev_.begin(), prev_.end(), 0.0f);
        std::fill(curr_.begin(), curr_.end(), 0.0f);
    }

private:
    double             ratio_;
    int                channels_;
    double             phase_;
    std::vector<float> prev_;
    std::vector<float> curr_;
};

// ---------------------------------------------------------------------------
// SndfileDecoder
// ---------------------------------------------------------------------------

class SndfileDecoder final : public Decoder {
public:
    SndfileDecoder(SNDFILE* sf, SF_INFO info_in,
                   uint32_t target_rate)
        : sf_(sf)
        , info_in_(info_in)
        , target_rate_(target_rate)
        , position_(0)
    {
        // Build the AudioInfo struct (output frame count may differ from
        // source when sample rates differ).
        const double ratio = static_cast<double>(target_rate) /
                             static_cast<double>(info_in.samplerate);
        info_.sample_rate   = target_rate;
        info_.channels      = 2;   // always stereo output
        info_.total_frames  = static_cast<uint64_t>(
            static_cast<double>(info_in.frames) * ratio);
        info_.duration_secs = static_cast<double>(info_in.frames) /
                              static_cast<double>(info_in.samplerate);

        // Only create the resampler if rates differ.
        if (info_in.samplerate != static_cast<int>(target_rate)) {
            resampler_ = std::make_unique<LinearResampler>(
                static_cast<uint32_t>(info_in.samplerate),
                target_rate,
                info_in.channels);
        }

        // Pre-allocate decode scratch buffer (1024 frames).
        scratch_.resize(static_cast<std::size_t>(1024 * info_in.channels));
        // Stereo-mix buffer for upmix/downmix.
        mix_buf_.resize(1024 * 2);
    }

    ~SndfileDecoder() override {
        if (sf_) sf_close(sf_);
    }

    const AudioInfo& info() const override { return info_; }
    uint64_t position() const override { return position_; }

    DecodeResult read_frames(float* out,
                              uint32_t frame_count,
                              uint32_t& frames_read) override {
        frames_read = 0;
        if (!sf_) return DecodeResult::IoError;

        // How many source frames do we need to produce `frame_count` output frames?
        const bool needs_resample =
            (info_in_.samplerate != static_cast<int>(target_rate_));
        const uint32_t src_needed = needs_resample
            ? static_cast<uint32_t>(std::ceil(
                  static_cast<double>(frame_count) *
                  static_cast<double>(info_in_.samplerate) /
                  static_cast<double>(target_rate_)))
            : frame_count;

        const std::size_t scratch_needed =
            static_cast<std::size_t>(src_needed) *
            static_cast<std::size_t>(info_in_.channels);
        if (scratch_.size() < scratch_needed)
            scratch_.resize(scratch_needed);

        // Decode raw frames from libsndfile.
        const sf_count_t got = sf_readf_float(sf_, scratch_.data(),
                                               static_cast<sf_count_t>(src_needed));
        if (got <= 0) return DecodeResult::EndOfFile;

        // Convert to stereo.
        const uint32_t got_u = static_cast<uint32_t>(got);
        if (mix_buf_.size() < static_cast<std::size_t>(got_u * 2))
            mix_buf_.resize(static_cast<std::size_t>(got_u * 2));
        to_stereo(scratch_.data(), got_u,
                  info_in_.channels, mix_buf_.data());

        // Resample if needed.
        if (needs_resample && resampler_) {
            frames_read = resampler_->process(mix_buf_.data(), got_u,
                                              out, frame_count);
        } else {
            frames_read = std::min(got_u, frame_count);
            std::memcpy(out, mix_buf_.data(),
                        static_cast<std::size_t>(frames_read) * 2 * sizeof(float));
        }

        position_ += frames_read;
        return frames_read > 0 ? DecodeResult::Ok : DecodeResult::EndOfFile;
    }

    DecodeResult seek_to(uint64_t frame) override {
        if (!sf_) return DecodeResult::IoError;

        // Convert output frame to source frame.
        const double src_frame =
            static_cast<double>(frame) *
            static_cast<double>(info_in_.samplerate) /
            static_cast<double>(target_rate_);

        if (sf_seek(sf_, static_cast<sf_count_t>(src_frame), SEEK_SET) < 0)
            return DecodeResult::SeekError;

        position_ = frame;
        if (resampler_) resampler_->reset();
        return DecodeResult::Ok;
    }

private:
    /** Mix any channel count down to (or up to) stereo in-place. */
    static void to_stereo(const float* src, uint32_t frames,
                           int src_ch, float* dst) {
        if (src_ch == 2) {
            // Already stereo — straight copy.
            std::memcpy(dst, src,
                static_cast<std::size_t>(frames) * 2 * sizeof(float));
        } else if (src_ch == 1) {
            // Mono → duplicate to both channels.
            for (uint32_t i = 0; i < frames; ++i) {
                dst[i * 2]     = src[i];
                dst[i * 2 + 1] = src[i];
            }
        } else {
            // Multi-channel → mix down to stereo.
            for (uint32_t i = 0; i < frames; ++i) {
                float l = 0.0f, r = 0.0f;
                for (int c = 0; c < src_ch; ++c) {
                    float s = src[i * static_cast<uint32_t>(src_ch) + static_cast<uint32_t>(c)];
                    if (c % 2 == 0) l += s; else r += s;
                }
                const float scale = 1.0f / static_cast<float>((src_ch + 1) / 2);
                dst[i * 2]     = l * scale;
                dst[i * 2 + 1] = r * scale;
            }
        }
    }

    SNDFILE*    sf_;
    SF_INFO     info_in_;
    uint32_t    target_rate_;
    AudioInfo   info_{};
    uint64_t    position_;

    std::unique_ptr<LinearResampler> resampler_;
    std::vector<float> scratch_;
    std::vector<float> mix_buf_;
};

// ---------------------------------------------------------------------------
// Factory
// ---------------------------------------------------------------------------

std::unique_ptr<Decoder> Decoder::open(const std::string& path,
                                        uint32_t target_sample_rate) {
    SF_INFO info{};
    SNDFILE* sf = sf_open(path.c_str(), SFM_READ, &info);
    if (!sf) return nullptr;
    if (info.frames <= 0 || info.channels <= 0 || info.samplerate <= 0) {
        sf_close(sf);
        return nullptr;
    }
    return std::make_unique<SndfileDecoder>(sf, info, target_sample_rate);
}

} // namespace pdj
