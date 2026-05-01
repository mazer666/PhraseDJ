/// PhraseDJ Tauri application library.
///
/// Wires together the audio engine, the local library, and the Tauri
/// command surface used by the React frontend.
pub mod commands;
pub mod state;

use std::path::PathBuf;

use async_broadcast::Receiver;
use directories::ProjectDirs;
use pdj_core::types::TrackId;
use pdj_engine_bridge::EngineConfig;
use pdj_stems::StemStatus;
use serde::Serialize;
use tauri::{Emitter, Manager};
use tracing::{error, warn};
use tracing_subscriber::{fmt, EnvFilter};

use crate::state::AppState;

/// Payload emitted as a Tauri event when a stem job changes status.
#[derive(Debug, Clone, Serialize)]
struct StemEvent {
    track_id: String,
    status: String,         // "pending" | "running" | "cached" | "failed"
    progress: Option<f32>,  // only set when status == "running"
    reason: Option<String>, // only set when status == "failed"
}

/// Initialise logging and start the Tauri event loop.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tracing::info!("PhraseDJ starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Initialise the AppState (audio engine + library) inside `setup`
            // so we can use directories from the Tauri context.
            let db_path = app_support_path("library.db");
            let engine_config = EngineConfig::default();

            match AppState::init(db_path, engine_config) {
                Ok(state) => {
                    // Start forwarding stem status events to the frontend.
                    let rx = state.stems.subscribe();
                    let handle = app.handle().clone();
                    tokio::spawn(forward_stem_events(rx, handle));

                    app.manage(state);
                }
                Err(e) => {
                    error!(?e, "failed to initialise app state");
                    // Fall back to a minimal state so the UI can still load
                    // and surface the error to the user.
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_version,
            commands::app_status,
            commands::keymap_load,
            commands::settings_load,
            commands::settings_save,
            commands::deck_load,
            commands::deck_play,
            commands::deck_pause,
            commands::deck_seek,
            commands::deck_state,
            commands::deck_set_tempo_ratio,
            commands::deck_sync,
            commands::deck_nudge_tempo,
            commands::deck_waveform,
            commands::mixer_set_fader,
            commands::mixer_set_crossfader,
            commands::mixer_set_master_gain,
            commands::mixer_set_stem_gain,
            commands::library_import_file,
            commands::library_scan_folder,
            commands::library_recent,
            commands::library_search,
        ])
        .run(tauri::generate_context!())
        .expect("error while running PhraseDJ");
}

/// Compute a path under the OS-specific app-support folder, creating
/// intermediate directories.
fn app_support_path(file: &str) -> PathBuf {
    let dirs = ProjectDirs::from("io", "PhraseDJ", "PhraseDJ").expect("no app-support dir");
    let dir = dirs.config_local_dir().to_path_buf();
    let _ = std::fs::create_dir_all(&dir);
    dir.join(file)
}

/// Background task that listens on the stem broadcast channel and
/// emits Tauri events to all connected windows.
///
/// The React frontend listens for `"stem-status"` events and can
/// hot-swap stems into a playing deck without a manual reload.
async fn forward_stem_events(mut rx: Receiver<(TrackId, StemStatus)>, handle: tauri::AppHandle) {
    loop {
        match rx.recv().await {
            Ok((track_id, status)) => {
                let event = match &status {
                    StemStatus::Pending => StemEvent {
                        track_id: track_id.to_string(),
                        status: "pending".into(),
                        progress: None,
                        reason: None,
                    },
                    StemStatus::ModelDownloading { progress } => StemEvent {
                        track_id: track_id.to_string(),
                        status: "model_downloading".into(),
                        progress: Some(*progress),
                        reason: None,
                    },
                    StemStatus::Running { progress } => StemEvent {
                        track_id: track_id.to_string(),
                        status: "running".into(),
                        progress: Some(*progress),
                        reason: None,
                    },
                    StemStatus::Cached { .. } => StemEvent {
                        track_id: track_id.to_string(),
                        status: "cached".into(),
                        progress: None,
                        reason: None,
                    },
                    StemStatus::Failed { reason } => StemEvent {
                        track_id: track_id.to_string(),
                        status: "failed".into(),
                        progress: None,
                        reason: Some(reason.clone()),
                    },
                };
                if let Err(e) = handle.emit("stem-status", event) {
                    warn!("Failed to emit stem-status event: {}", e);
                }
            }
            Err(async_broadcast::RecvError::Closed) => {
                tracing::info!("Stem status channel closed — stopping event forwarder");
                break;
            }
            Err(async_broadcast::RecvError::Overflowed(_)) => {
                // Missed some events, not critical — UI will catch up on next poll.
            }
        }
    }
}
