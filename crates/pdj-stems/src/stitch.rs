/// Overlap-add segment stitching and FLAC writing.
///
/// # The overlap-add problem
///
/// HTDemucs runs inference on short overlapping windows of audio (e.g. 8 s
/// with 1 s of overlap on each edge).  The edges of each window are often
/// slightly inaccurate due to transformer context limits.  Overlap-add (OLA)
/// discards the unreliable edges and cross-fades adjacent windows to produce
/// a seamless output.
///
/// # How this module is used
///
/// The job queue calls:
/// 1. `Stitcher::new()` to create a fresh accumulator.
/// 2. `Stitcher::add_segment()` for each processed window.
/// 3. `Stitcher::finalise()` to flush the last window and get the full stems.
/// 4. `write_stems_to_disk()` to save the four FLAC files.
///
/// # Cross-fade shape
///
/// We use a half-Hann window (cosine ramp) for the blend.  This is smoother
/// than a linear fade and avoids a +3 dB amplitude bump at the seam.
use std::path::Path;

use pdj_core::Result;

use crate::backend::PcmBuffer;
use crate::paths::StemPaths;

// ---------------------------------------------------------------------------
// Stitcher
// ---------------------------------------------------------------------------

/// Accumulates processed segments and blends them with overlap-add.
///
/// Create one `Stitcher` per track analysis.  Feed it segments in order
/// with `add_segment()`, then call `finalise()` to get the complete stems.
pub struct Stitcher {
    /// Number of channels in the audio (1 = mono, 2 = stereo).
    channels: u16,
    /// Sample rate (Hz) — stored so the output buffer carries it.
    sample_rate: u32,
    /// Overlap length in *samples per channel* (not frames × channels).
    overlap_samples: usize,
    /// Accumulated output for each of the four stems.
    ///
    /// Index matches `StemLabel::ALL` order:
    ///   0 = vocals, 1 = drums, 2 = bass, 3 = other.
    accumulated: [Vec<f32>; 4],
}

impl Stitcher {
    /// Create a new stitcher.
    ///
    /// `overlap_samples` is the number of *frames* (not samples) to blend
    /// between adjacent segments.  A value of `sample_rate` (e.g. 44 100)
    /// gives a 1-second crossfade.
    pub fn new(channels: u16, sample_rate: u32, overlap_frames: usize) -> Self {
        Self {
            channels,
            sample_rate,
            overlap_samples: overlap_frames * channels as usize,
            accumulated: Default::default(),
        }
    }

    /// Add one processed segment to the accumulator.
    ///
    /// `stems` must be in `StemLabel::ALL` order: vocals, drums, bass, other.
    /// The function applies a half-Hann fade-out on the leading edge and
    /// blends it with the previous segment's trailing edge.
    pub fn add_segment(&mut self, stems: [PcmBuffer; 4]) {
        for (label_idx, stem_buf) in stems.iter().enumerate() {
            let acc = &mut self.accumulated[label_idx];
            let new_samples = &stem_buf.samples;

            if acc.is_empty() {
                // First segment — just append.
                acc.extend_from_slice(new_samples);
            } else {
                // Blend: the last `overlap_samples` in `acc` are cross-faded
                // with the first `overlap_samples` of the new segment.
                let blend_len = self.overlap_samples.min(acc.len()).min(new_samples.len());
                let acc_len = acc.len();
                let fade = half_hann_fade(blend_len);

                for i in 0..blend_len {
                    // fade_out for acc, fade_in for new.
                    let alpha = fade[i]; // 0 → 1 (new signal weight)
                    let acc_index = acc_len - blend_len + i;
                    acc[acc_index] = acc[acc_index] * (1.0 - alpha) + new_samples[i] * alpha;
                }

                // Append the non-overlapping tail of the new segment.
                acc.extend_from_slice(&new_samples[blend_len..]);
            }
        }
    }

    /// Consume the stitcher and return the four complete stem buffers.
    ///
    /// Call this only once, after all segments have been added.
    pub fn finalise(self) -> [PcmBuffer; 4] {
        let [v, d, b, o] = self.accumulated;
        let make_buf = |samples: Vec<f32>| PcmBuffer {
            samples,
            channels: self.channels,
            sample_rate: self.sample_rate,
        };
        [make_buf(v), make_buf(d), make_buf(b), make_buf(o)]
    }
}

// ---------------------------------------------------------------------------
// FLAC writing
// ---------------------------------------------------------------------------

/// Write four stem buffers to FLAC files at the paths given in `stem_paths`.
///
/// Each stem is stored as a single-file FLAC: 16-bit PCM at the source
/// sample rate (usually 44 100 Hz), preserving the original channel count.
///
/// **Note:** This function uses the `hound` crate to write WAV files.
/// Until a pure-Rust FLAC encoder is integrated, the extension is `.flac`
/// but the actual data is 16-bit WAV.  The queue code already writes these
/// files to the correct `.flac` paths.  When real FLAC encoding lands,
/// this function's signature stays the same — only the internals change.
pub fn write_stems_to_disk(stems: [PcmBuffer; 4], paths: &StemPaths) -> Result<()> {
    let stem_files = [
        (&stems[0], &paths.vocals),
        (&stems[1], &paths.drums),
        (&stems[2], &paths.bass),
        (&stems[3], &paths.other),
    ];

    for (buf, path) in stem_files {
        write_wav_stem(buf, path)?;
    }
    Ok(())
}

/// Write a single PCM buffer to a WAV file at `path`.
///
/// Samples are clamped to [-1.0, 1.0] and converted to 16-bit PCM before
/// writing.
fn write_wav_stem(buf: &PcmBuffer, path: &Path) -> Result<()> {
    // Create parent directories if they do not exist yet.
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let spec = hound::WavSpec {
        channels: buf.channels,
        sample_rate: buf.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(path, spec)
        .map_err(|e| pdj_core::Error::other(format!("WAV write error: {e}")))?;

    for &sample in &buf.samples {
        // Clamp to [-1, 1] before converting to i16 to avoid wrap-around.
        let clamped = sample.clamp(-1.0, 1.0);
        let as_i16 = (clamped * i16::MAX as f32) as i16;
        writer
            .write_sample(as_i16)
            .map_err(|e| pdj_core::Error::other(format!("WAV sample write: {e}")))?;
    }

    writer
        .finalize()
        .map_err(|e| pdj_core::Error::other(format!("WAV finalize: {e}")))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Half-Hann fade helper
// ---------------------------------------------------------------------------

/// Generate a `len`-point half-Hann window rising from 0 to 1.
///
/// Used as the blend weight for the incoming segment.  The outgoing
/// segment uses `(1 - weight)`.
///
/// Hann is chosen over linear because it avoids a 3 dB amplitude boost at
/// the centre of the crossfade.
fn half_hann_fade(len: usize) -> Vec<f32> {
    if len <= 1 {
        return vec![1.0; len];
    }
    (0..len)
        .map(|i| {
            // Rising half-Hann: 0.5 * (1 - cos(π * i / (len - 1)))
            let phase = std::f32::consts::PI * i as f32 / (len - 1) as f32;
            0.5 * (1.0 - phase.cos())
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_stem(value: f32, frames: usize, channels: u16) -> PcmBuffer {
        PcmBuffer {
            samples: vec![value; frames * channels as usize],
            channels,
            sample_rate: 44_100,
        }
    }

    fn make_four_stems(value: f32, frames: usize) -> [PcmBuffer; 4] {
        [
            make_stem(value, frames, 2),
            make_stem(value, frames, 2),
            make_stem(value, frames, 2),
            make_stem(value, frames, 2),
        ]
    }

    #[test]
    fn stitcher_single_segment_passes_through_unchanged() {
        let mut stitcher = Stitcher::new(2, 44_100, 0);
        stitcher.add_segment(make_four_stems(0.5, 100));
        let result = stitcher.finalise();
        // 100 frames × 2 channels = 200 samples.
        assert_eq!(result[0].samples.len(), 200);
        // Value should be 0.5 (no blending with only one segment).
        for s in &result[0].samples {
            assert!((s - 0.5).abs() < 1e-5, "sample was {s}");
        }
    }

    #[test]
    fn stitcher_two_segments_no_overlap_concatenates() {
        let mut stitcher = Stitcher::new(1, 44_100, 0); // mono, no overlap
        stitcher.add_segment([
            make_stem(0.1, 10, 1),
            make_stem(0.1, 10, 1),
            make_stem(0.1, 10, 1),
            make_stem(0.1, 10, 1),
        ]);
        stitcher.add_segment([
            make_stem(0.9, 10, 1),
            make_stem(0.9, 10, 1),
            make_stem(0.9, 10, 1),
            make_stem(0.9, 10, 1),
        ]);
        let result = stitcher.finalise();
        // 10 + 10 = 20 frames.
        assert_eq!(result[0].samples.len(), 20);
    }

    #[test]
    fn half_hann_starts_at_zero_and_ends_near_one() {
        let fade = half_hann_fade(100);
        assert_eq!(fade.len(), 100);
        assert!(fade[0].abs() < 1e-5, "start should be ~0");
        assert!((fade[99] - 1.0).abs() < 1e-5, "end should be ~1");
    }

    #[test]
    fn write_stems_to_disk_creates_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = StemPaths {
            vocals: dir.path().join("vocals.flac"),
            drums: dir.path().join("drums.flac"),
            bass: dir.path().join("bass.flac"),
            other: dir.path().join("other.flac"),
        };
        let stems = make_four_stems(0.0, 64);
        write_stems_to_disk(stems, &paths).expect("write");
        assert!(paths.vocals.exists());
        assert!(paths.drums.exists());
        assert!(paths.bass.exists());
        assert!(paths.other.exists());
    }
}
