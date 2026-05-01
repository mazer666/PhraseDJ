//! commands/app.rs — Application-level commands (version, status).

use serde::Serialize;
use tauri::State;

use crate::state::AppState;

/// Returns the application version string from Cargo.toml.
#[tauri::command]
pub fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Status payload returned to the UI on startup.
#[derive(Debug, Serialize)]
pub struct AppStatus {
    pub version:        String,
    pub audio_running:  bool,
    pub library_count:  i64,
}

/// Returns whether the audio engine is running and how many tracks the
/// library currently holds.  Used by the UI for the status bar.
#[tauri::command]
pub fn app_status(state: State<'_, AppState>) -> AppStatus {
    let audio_running = state.engine.lock()
        .ok()
        .and_then(|guard| guard.as_ref().map(|e| e.is_running()))
        .unwrap_or(false);

    let library_count = state.library.lock()
        .ok()
        .and_then(|lib| lib.count().ok())
        .unwrap_or(0);

    AppStatus {
        version: env!("CARGO_PKG_VERSION").to_string(),
        audio_running,
        library_count,
    }
}
