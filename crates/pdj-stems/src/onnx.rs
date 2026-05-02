use ndarray::{Array3, Ix4};
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::Value;
use pdj_core::{Error, Result};
/// ONNX Runtime backend for stem separation.
///
/// # Why ONNX?
///
/// ONNX Runtime (ORT) is the cross-platform fallback for platforms where
/// MLX is unavailable (Linux, Windows, Intel Macs).  ORT can also use the
/// CoreML execution provider on macOS for hardware acceleration even on
/// non-Apple-Silicon machines.
///
/// # Implementation
///
/// The backend lazily loads the HTDemucs ONNX model on first inference.
/// Input audio is deinterleaved into an ndarray tensor of shape
/// `[batch=1, channels, frames]`, run through the ORT session, and the
/// output `[1, 4, channels, frames]` is re-interleaved back into PCM
/// buffers.
///
/// The model file must be placed at
/// `<app-support>/PhraseDJ/models/htdemucs.onnx`.
///
/// See `specs/03-ai-stems.md §2` for model details.
use std::sync::Mutex;

use crate::backend::{InferenceRequest, InferenceResult, PcmBuffer, StemBackend};

// ---------------------------------------------------------------------------
// OnnxBackend
// ---------------------------------------------------------------------------

/// Cross-platform ONNX Runtime inference backend.
pub struct OnnxBackend {
    session: Mutex<Option<Session>>,
}

impl OnnxBackend {
    /// Create a new ONNX backend instance.
    ///
    /// Model weights are not loaded here to keep startup fast.  They will
    /// be loaded lazily on the first `infer()` call once the download logic
    /// is in place.
    pub fn new() -> Self {
        // Initialize ort environment (thread-safe, idempotent).
        let _ = ort::init().commit();
        Self {
            session: Mutex::new(None),
        }
    }

    fn get_or_load_session(&self) -> Result<std::sync::MutexGuard<'_, Option<Session>>> {
        let mut guard = self
            .session
            .lock()
            .map_err(|e| Error::Settings(e.to_string()))?;
        if guard.is_none() {
            let model_path = crate::paths::model_path()?;

            if !model_path.exists() {
                // If model doesn't exist, we fallback to returning an error which
                // could trigger a stub or gracefully fail.
                return Err(Error::Settings(format!(
                    "ONNX model not found at {}",
                    model_path.display()
                )));
            }

            let session = Session::builder()
                .map_err(|e| Error::other(format!("Session builder error: {}", e)))?
                .with_optimization_level(GraphOptimizationLevel::Level3)
                .map_err(|e| Error::other(format!("Opt level error: {}", e)))?
                .with_intra_threads(4)
                .map_err(|e| Error::other(format!("Threads error: {}", e)))?
                .commit_from_file(model_path)
                .map_err(|e| Error::other(format!("Model load error: {}", e)))?;

            *guard = Some(session);
        }
        Ok(guard)
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
    /// Returns zeroed stems if the model is not found, otherwise runs the
    /// real ORT session inference.
    fn infer(&self, request: InferenceRequest) -> Result<InferenceResult> {
        tracing::debug!(
            frames = request.audio.frame_count(),
            backend = "onnx",
            "Running ONNX inference",
        );

        let mut session_guard = match self.get_or_load_session() {
            Ok(g) => g,
            Err(e) => {
                tracing::error!("ONNX model load failed: {}", e);
                return Err(Error::other(format!("Model missing or invalid: {}", e)));
            }
        };

        let session = session_guard.as_mut().unwrap();

        let frames = request.audio.frame_count();
        let channels = request.audio.channels as usize;

        // Deinterleave: HTDemucs expects [batch=1, channels, frames]
        let mut deinterleaved = vec![0.0f32; frames * channels];
        for c in 0..channels {
            for f in 0..frames {
                deinterleaved[c * frames + f] = request.audio.samples[f * channels + c];
            }
        }

        let input_tensor = Array3::from_shape_vec((1, channels, frames), deinterleaved)
            .map_err(|e| Error::other(format!("Shape error: {}", e)))?;

        // ort 2.0 requires explicit Value creation from ndarray.
        let input_value = Value::from_array(input_tensor)
            .map_err(|e| Error::other(format!("Failed to create input tensor: {}", e)))?;

        // Use the first input name from the model (usually "input" or "audio").
        let input_name = session.inputs()[0].name().to_string();
        let outputs = session
            .run(ort::inputs![
                input_name.as_str() => input_value,
            ])
            .map_err(|e| Error::other(format!("Inference failed: {}", e)))?;

        // Extract [1, 4, channels, frames]
        let out_value = &outputs[0];
        let (out_shape, out_data) = out_value
            .try_extract_tensor::<f32>()
            .map_err(|e| Error::other(format!("Extraction failed: {}", e)))?;

        let shape: Vec<usize> = out_shape.iter().map(|&d| d as usize).collect();
        let out_view = ndarray::ArrayViewD::from_shape(shape, out_data)
            .map_err(|e| Error::other(format!("ArrayView creation failed: {}", e)))?
            .into_dimensionality::<Ix4>()
            .map_err(|e| Error::other(format!("Output dimension mismatch: {}", e)))?;

        // HTDemucs native output order: 0=drums, 1=bass, 2=other, 3=vocals
        // StemLabel::ALL target order: 0=vocals, 1=drums, 2=bass, 3=other
        let demucs_to_stemlabel = [3, 0, 1, 2];

        let mut stems = Vec::new();
        for &stem_idx in &demucs_to_stemlabel {
            let mut interleaved = vec![0.0f32; frames * channels];
            for c in 0..channels {
                for f in 0..frames {
                    // Indexing into [batch, stem, channel, frame] -> [0, stem_idx, c, f]
                    let val = out_view[[0, stem_idx, c, f]];
                    interleaved[f * channels + c] = val;
                }
            }
            stems.push(PcmBuffer {
                samples: interleaved,
                channels: request.audio.channels,
                sample_rate: request.audio.sample_rate,
            });
        }

        tracing::info!("ONNX inference completed for {} frames", frames);
        let stems: [PcmBuffer; 4] = stems.try_into().map_err(|v: Vec<PcmBuffer>| {
            Error::other(format!("Expected 4 stems, got {}", v.len()))
        })?;

        Ok(InferenceResult { stems })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn onnx_is_always_available() {
        assert!(OnnxBackend::new().is_available());
    }

    #[test]
    fn backend_name_is_onnx() {
        assert_eq!(OnnxBackend::new().name(), "onnx");
    }
}
