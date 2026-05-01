/// Stem backend trait — the contract that both MLX and ONNX must satisfy.
///
/// A `StemBackend` receives raw PCM audio and returns four separated stem
/// buffers.  All heavy work happens here; the job queue calls this trait
/// and does not care which backend is active.
///
/// # Threading
///
/// Backends are called from a blocking Tokio task (`spawn_blocking`), so
/// they are allowed to block the current OS thread.  They must **not** hold
/// async locks or spawn sub-tasks.
///
/// # Adding a new backend
///
/// 1. Create `src/my_backend.rs` implementing `StemBackend`.
/// 2. Register it in `src/lib.rs`.
/// 3. Add a feature flag if the new backend requires an optional C library.

use pdj_core::Result;

// ---------------------------------------------------------------------------
// Stem label
// ---------------------------------------------------------------------------

/// Which of the four standard stems a buffer belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StemLabel {
    Vocals,
    Drums,
    Bass,
    Other,
}

impl StemLabel {
    /// The four standard stem labels in a fixed order (used for indexing).
    pub const ALL: [StemLabel; 4] = [
        StemLabel::Vocals,
        StemLabel::Drums,
        StemLabel::Bass,
        StemLabel::Other,
    ];

    /// The file-system-safe name for this stem.
    pub fn as_str(self) -> &'static str {
        match self {
            StemLabel::Vocals => "vocals",
            StemLabel::Drums => "drums",
            StemLabel::Bass => "bass",
            StemLabel::Other => "other",
        }
    }
}

impl std::fmt::Display for StemLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// PCM buffer helpers
// ---------------------------------------------------------------------------

/// An interleaved, 32-bit floating-point PCM buffer.
///
/// Channels are interleaved: for stereo, even indices are left, odd are right.
#[derive(Debug, Clone)]
pub struct PcmBuffer {
    /// Raw PCM samples, interleaved across channels.
    pub samples: Vec<f32>,
    /// Number of audio channels (1 = mono, 2 = stereo).
    pub channels: u16,
    /// Sample rate in Hz (e.g. 44_100).
    pub sample_rate: u32,
}

impl PcmBuffer {
    /// Number of *frames* (time steps) regardless of channel count.
    pub fn frame_count(&self) -> usize {
        self.samples.len() / self.channels as usize
    }
}

// ---------------------------------------------------------------------------
// Inference request / response
// ---------------------------------------------------------------------------

/// One segment of audio sent to a backend for inference.
///
/// The backend must return four separated stems for the same time window.
/// The segment boundaries are set by the overlap-add logic in `stitch`.
#[derive(Debug)]
pub struct InferenceRequest {
    /// Audio to separate.
    pub audio: PcmBuffer,
}

/// Four separated stems returned by a backend for one segment.
#[derive(Debug)]
pub struct InferenceResult {
    /// The separated stems, indexed by `StemLabel::ALL` order.
    /// Each entry has the same `channels` and `sample_rate` as the request.
    pub stems: [PcmBuffer; 4],
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// The interface every inference backend must implement.
///
/// Implementations live in `mod mlx` (Apple Silicon) and `mod onnx`
/// (cross-platform fallback).
pub trait StemBackend: Send + Sync {
    /// A short name shown in logs and settings ("mlx" or "onnx").
    fn name(&self) -> &'static str;

    /// Return `true` if this backend can run on the current machine.
    ///
    /// For example, the MLX backend checks that we are on Apple Silicon.
    /// The ONNX backend is always available.
    fn is_available(&self) -> bool;

    /// Separate a single audio segment into four stems.
    ///
    /// This function is called from a blocking thread.  It may take up to
    /// several seconds per segment.  Progress is not reported per-segment —
    /// only per-track (handled at the queue level).
    fn infer(&self, request: InferenceRequest) -> Result<InferenceResult>;
}

// ---------------------------------------------------------------------------
// Backend selector
// ---------------------------------------------------------------------------

/// Select the best available backend.
///
/// MLX is preferred on macOS/Apple Silicon.  Falls back to ONNX on all
/// other platforms or if MLX is unavailable.
pub fn select_backend() -> Box<dyn StemBackend> {
    let mlx = crate::mlx::MlxBackend::new();
    if mlx.is_available() {
        tracing::info!("Stem backend: MLX (Apple Silicon)");
        return Box::new(mlx);
    }
    let onnx = crate::onnx::OnnxBackend::new();
    tracing::info!("Stem backend: ONNX (CPU)");
    Box::new(onnx)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stem_label_names_are_stable() {
        // These strings are used as directory names in the cache.
        // Changing them would break existing user caches — keep stable.
        assert_eq!(StemLabel::Vocals.as_str(), "vocals");
        assert_eq!(StemLabel::Drums.as_str(), "drums");
        assert_eq!(StemLabel::Bass.as_str(), "bass");
        assert_eq!(StemLabel::Other.as_str(), "other");
    }

    #[test]
    fn pcm_frame_count_matches_samples_divided_by_channels() {
        let buf = PcmBuffer {
            samples: vec![0.0_f32; 200],
            channels: 2,
            sample_rate: 44_100,
        };
        // 200 samples / 2 channels = 100 frames
        assert_eq!(buf.frame_count(), 100);
    }
}
