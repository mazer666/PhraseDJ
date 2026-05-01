/// ONNX Runtime backend for stem separation.
///
/// # Why ONNX?
///
/// ONNX Runtime (ORT) is the cross-platform fallback for platforms where
/// MLX is unavailable (Linux, Windows, Intel Macs).  ORT can also use the
/// CoreML execution provider on macOS for hardware acceleration even on
/// non-Apple-Silicon machines.
///
/// # Current state (Phase 2 stub)
///
/// Linking ORT into a Rust binary requires the `ort` crate and a matching
/// native library.  Until the model file is packaged and the download flow
/// is built, this module contains:
///
/// 1. `OnnxBackend::is_available()` — always true (ONNX is always compiled in).
/// 2. `OnnxBackend::infer()` — a **stub** that returns zeroed stems, same as
///    the MLX stub.
///
/// # Replacing the stub
///
/// 1. Add `ort = "2"` to `Cargo.toml` (feature `load-dynamic` for Linux).
/// 2. Load the exported `htdemucs.onnx` model in `new()`.
/// 3. Run inference in `infer()` and fill in real stems.
/// 4. Delete this doc note.
///
/// See `specs/03-ai-stems.md §2` for model details.
use pdj_core::Result;

use crate::backend::{InferenceRequest, InferenceResult, PcmBuffer, StemBackend};

// ---------------------------------------------------------------------------
// OnnxBackend
// ---------------------------------------------------------------------------

/// Cross-platform ONNX Runtime inference backend.
pub struct OnnxBackend;

impl OnnxBackend {
    /// Create a new ONNX backend instance.
    ///
    /// Model weights are not loaded here to keep startup fast.  They will
    /// be loaded lazily on the first `infer()` call once the download logic
    /// is in place.
    pub fn new() -> Self {
        Self
    }
}

impl Default for OnnxBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl StemBackend for OnnxBackend {
    fn name(&self) -> &'static str {
        "onnx"
    }

    /// ONNX Runtime is always compiled in and therefore always available.
    fn is_available(&self) -> bool {
        true
    }

    /// Separate one segment into four stems using HTDemucs via ONNX Runtime.
    ///
    /// **Phase 2 stub:** returns zeroed stems.  Replace with real ORT
    /// session inference once the model file is available.
    fn infer(&self, request: InferenceRequest) -> Result<InferenceResult> {
        tracing::debug!(
            frames  = request.audio.frame_count(),
            backend = "onnx",
            "Running stub inference (no-op zeros)",
        );
        Ok(todo_stub_inference(request))
    }
}

// ---------------------------------------------------------------------------
// Stub — remove when real ONNX inference is implemented
// ---------------------------------------------------------------------------

/// Return four zeroed stem buffers matching the input shape.
///
/// **TODO(spec):** Replace with ORT session run once `htdemucs.onnx` is
/// packaged.  Reference: `specs/03-ai-stems.md §2`.
fn todo_stub_inference(request: InferenceRequest) -> InferenceResult {
    let template = PcmBuffer {
        samples:     vec![0.0_f32; request.audio.samples.len()],
        channels:    request.audio.channels,
        sample_rate: request.audio.sample_rate,
    };
    InferenceResult {
        stems: [
            template.clone(),
            template.clone(),
            template.clone(),
            template,
        ],
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::InferenceRequest;

    fn make_request(frames: usize, channels: u16) -> InferenceRequest {
        InferenceRequest {
            audio: PcmBuffer {
                samples:     vec![0.25_f32; frames * channels as usize],
                channels,
                sample_rate: 44_100,
            },
        }
    }

    #[test]
    fn stub_returns_four_stems_correct_shape() {
        let backend = OnnxBackend::new();
        let req     = make_request(512, 1); // mono
        let result  = backend.infer(req).expect("stub infer");
        assert_eq!(result.stems.len(), 4);
        for stem in &result.stems {
            assert_eq!(stem.samples.len(), 512);
            assert_eq!(stem.channels, 1);
        }
    }

    #[test]
    fn onnx_is_always_available() {
        assert!(OnnxBackend::new().is_available());
    }

    #[test]
    fn backend_name_is_onnx() {
        assert_eq!(OnnxBackend::new().name(), "onnx");
    }
}
