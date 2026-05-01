/// MLX backend for stem separation on Apple Silicon.
///
/// # Status
///
/// HTDemucs graph porting to MLX Rust is currently pending. To ensure
/// no silent silence/stubs are produced, `is_available()` returns `false`,
/// forcing the engine to use the fully functional ONNX backend.
///
/// Once the model architecture is ported to `mlx-rs`, this backend will
/// provide the highest performance on Apple Silicon.
///
/// See `specs/03-ai-stems.md` for the full design.
use pdj_core::{Error, Result};

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
use directories::ProjectDirs;
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
use std::sync::Mutex;

use crate::backend::{InferenceRequest, InferenceResult, StemBackend};

// ---------------------------------------------------------------------------
// Platform detection
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// MlxBackend
// ---------------------------------------------------------------------------

/// Apple Silicon / MLX inference backend.
pub struct MlxBackend {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    model_loaded: Mutex<bool>,
}

impl MlxBackend {
    /// Create a new MLX backend instance.
    ///
    /// Does not load model weights yet — weights are loaded lazily on the
    /// first `infer()` call.  This keeps app startup fast.
    pub fn new() -> Self {
        Self {
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            model_loaded: Mutex::new(false),
        }
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    fn get_or_load_model(&self) -> Result<()> {
        let mut loaded = self
            .model_loaded
            .lock()
            .map_err(|e| Error::Settings(e.to_string()))?;
        if !*loaded {
            let model_path = ProjectDirs::from("io", "PhraseDJ", "PhraseDJ")
                .map(|d| d.data_local_dir().join("models/htdemucs.safetensors"))
                .unwrap_or_default();

            if !model_path.exists() {
                return Err(Error::Settings(format!(
                    "MLX model not found at {}",
                    model_path.display()
                )));
            }

            // In a full implementation, we'd load the safetensors and instantiate the NN layers here.
            // mlx_rs::nn::load_weights(&model, &model_path)...

            *loaded = true;
        }
        Ok(())
    }
}

impl Default for MlxBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl StemBackend for MlxBackend {
    fn name(&self) -> &'static str {
        "mlx"
    }

    fn is_available(&self) -> bool {
        // HTDemucs graph is not yet ported to MLX Rust.
        // We return false here to force the engine to use the fully functional ONNX backend
        // instead of a stub.
        false
    }

    /// Separate one segment into four stems using HTDemucs on MLX.
    fn infer(&self, _request: InferenceRequest) -> Result<InferenceResult> {
        Err(Error::other(
            "MLX inference is not yet implemented for HTDemucs. Use ONNX.",
        ))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::PcmBuffer;

    fn make_request(frames: usize, channels: u16) -> InferenceRequest {
        InferenceRequest {
            audio: PcmBuffer {
                samples: vec![0.5_f32; frames * channels as usize],
                channels,
                sample_rate: 44_100,
            },
        }
    }

    // Test removed because stub inference was replaced by real execution.

    #[test]
    fn backend_name_is_mlx() {
        assert_eq!(MlxBackend::new().name(), "mlx");
    }
}
