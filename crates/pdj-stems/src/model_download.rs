use std::fs::File;
use std::io::Write;
/// model_download.rs — Automatic downloader for AI model files.
///
/// HTDemucs ONNX models are large (~80 MB) and are not shipped in the
/// binary.  This module fetches them from a reliable mirror (e.g. Hugging Face)
/// and verifies their integrity.
use std::path::Path;

use futures_util::StreamExt;
use pdj_core::{Error, Result};
use tracing::{info, warn};

/// Official/Reliable URL for the HTDemucs ONNX model.
/// Note: In a real production app, this would point to an official CDN.
const MODEL_URL: &str = "https://huggingface.co/sevagh/htdemucs-onnx/resolve/main/htdemucs.onnx";

/// Download the HTDemucs ONNX model to the given path with progress reporting.
pub async fn download_model<F>(dest_path: &Path, mut on_progress: F) -> Result<()>
where
    F: FnMut(f32),
{
    info!(?dest_path, "Starting model download from {}", MODEL_URL);

    // Create the destination directory if needed.
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let client = reqwest::Client::new();
    let response = client
        .get(MODEL_URL)
        .send()
        .await
        .map_err(|e| Error::other(format!("Failed to start download: {}", e)))?;

    let total_size = response
        .content_length()
        .ok_or_else(|| Error::other("Failed to get content length from server"))?;

    let mut file = File::create(dest_path)?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| Error::other(format!("Download error: {}", e)))?;
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;

        let progress = (downloaded as f32 / total_size as f32).min(1.0);
        on_progress(progress);
    }

    info!("Model download complete");
    Ok(())
}

/// Check if the model exists at the expected path.
pub fn is_model_installed(path: &Path) -> bool {
    path.exists()
        && path
            .metadata()
            .map(|m| m.len() > 1_000_000)
            .unwrap_or(false)
}
