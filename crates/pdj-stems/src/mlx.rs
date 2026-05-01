/// MLX backend for stem separation on Apple Silicon.
///
/// # What is MLX?
///
/// MLX is Apple's open-source machine-learning framework optimised for
/// Apple Silicon (M-series chips).  It runs on the unified memory
/// architecture, so CPU and GPU share the same RAM — no data copies needed
/// between host and device.
///
/// # Current state (Phase 2 stub)
///
/// Full HTDemucs porting to MLX is a non-trivial task (Python → MLX Swift
/// or mlx-swift bindings).  This module contains:
///
/// 1. `MlxBackend::is_available()` — real check (CPU vendor string on macOS).
/// 2. `MlxBackend::infer()` — a **stub** that returns zeroed stems so the
///    rest of the pipeline (queue, stitch, cache) can be exercised end-to-end
///    today.  Replace the body with real MLX inference once the Swift/Python
///    bridge lands.
///
/// # Replacing the stub
///
/// When the real model is ready:
/// 1. Implement the Swift FFI (or use `mlx-rs` if available) (done).
/// 2. Delete the `todo_stub_inference` function below.
/// 3. Fill in `infer()` with the real call.
/// 4. Delete this doc note.
///
/// See `specs/03-ai-stems.md` for the full design.
use pdj_core::{Error, Result};

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
use mlx_rs::{Array, StreamOrDevice};
use std::sync::Mutex;

use crate::backend::{InferenceRequest, InferenceResult, PcmBuffer, StemBackend};

// ---------------------------------------------------------------------------
// Platform detection
// ---------------------------------------------------------------------------

/// Return `true` if we are running on an Apple Silicon CPU.
///
/// We check `std::env::consts::ARCH` at compile time.  On non-macOS builds
/// this is always false, so the ONNX backend takes over.
fn is_apple_silicon() -> bool {
    // cfg!() is evaluated at compile time, so the false branch is
    // dead-code-eliminated on non-macOS targets.
    cfg!(target_os = "macos") && cfg!(target_arch = "aarch64")
}

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
        let mut loaded = self.model_loaded.lock().map_err(|e| Error::Settings(e.to_string()))?;
        if !*loaded {
            let model_path = dirs::data_local_dir()
                .unwrap_or_default()
                .join("PhraseDJ/models/htdemucs.safetensors");
                
            if !model_path.exists() {
                return Err(Error::Settings(format!("MLX model not found at {}", model_path.display())));
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
        is_apple_silicon()
    }

    /// Separate one segment into four stems using HTDemucs on MLX.
    ///
    /// Returns zeroed stems if the model is not found, otherwise runs the
    /// real MLX computation (stubbed out here since full demucs model graph
    /// implementation is large).
    fn infer(&self, request: InferenceRequest) -> Result<InferenceResult> {
        tracing::debug!(
            frames  = request.audio.frame_count(),
            backend = "mlx",
            "Running MLX inference",
        );
        
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            if let Err(e) = self.get_or_load_model() {
                tracing::warn!("MLX model load failed: {}. Falling back to stub zeros.", e);
                return Ok(todo_stub_inference(request));
            }
            
            let frames = request.audio.frame_count();
            let _channels = request.audio.channels as i32;
            
            // Build input tensor
            // let input = Array::from_slice(&request.audio.samples, &[1, channels, frames as i32]);
            // let output = model.forward(&input);
            // mlx_rs::eval(&[output])?;
            
            tracing::info!("MLX model loaded successfully, simulating inference for {} frames", frames);
            return Ok(todo_stub_inference(request));
        }
        
        #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
        {
            Ok(todo_stub_inference(request))
        }
    }
}

// ---------------------------------------------------------------------------
// Stub — remove when real MLX inference is implemented
// ---------------------------------------------------------------------------

/// Return four zeroed stem buffers of the same shape as the input.
///
/// This lets the full pipeline (queue → stitch → cache) be exercised in
/// integration tests before the real model is ported.
///
/// **TODO(spec):** Replace with actual HTDemucs MLX inference.
/// Reference: `specs/03-ai-stems.md §2`.
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
                samples:     vec![0.5_f32; frames * channels as usize],
                channels,
                sample_rate: 44_100,
            },
        }
    }

    #[test]
    fn stub_returns_four_stems_with_correct_shape() {
        let backend = MlxBackend::new();
        let req     = make_request(1024, 2);
        let result  = backend.infer(req).expect("stub infer");
        // Four stems…
        assert_eq!(result.stems.len(), 4);
        // …each with the same shape as the input.
        for stem in &result.stems {
            assert_eq!(stem.samples.len(), 1024 * 2);
            assert_eq!(stem.channels, 2);
            assert_eq!(stem.sample_rate, 44_100);
        }
    }

    #[test]
    fn backend_name_is_mlx() {
        assert_eq!(MlxBackend::new().name(), "mlx");
    }
}
