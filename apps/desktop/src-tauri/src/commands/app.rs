//! commands/app.rs — Application-level commands (version, status, keymap).

use std::collections::HashMap;

use directories::ProjectDirs;
use serde::Serialize;
use tauri::State;
use toml::Value;

use crate::state::AppState;

/// Default keymap bundled with the application binary.
/// Loaded from `config/keymap.toml` at compile time.
const DEFAULT_KEYMAP_TOML: &str =
    include_str!("../../../../../config/keymap.toml");

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

/// Load the keymap, merging the bundled default with optional user overrides.
///
/// Returns a flat `{key: intent}` map.  The user's file at
/// `$APP_SUPPORT/keymap.toml` takes precedence for any key it defines.
#[tauri::command]
pub fn keymap_load() -> HashMap<String, String> {
    // Parse the bundled default first.
    let mut map = parse_keymap_toml(DEFAULT_KEYMAP_TOML);

    // Attempt to load user overrides from the app-support directory.
    if let Some(dirs) = ProjectDirs::from("io", "PhraseDJ", "PhraseDJ") {
        let user_file = dirs.config_local_dir().join("keymap.toml");
        if let Ok(contents) = std::fs::read_to_string(&user_file) {
            let overrides = parse_keymap_toml(&contents);
            // User keys win over defaults.
            map.extend(overrides);
        }
    }

    map
}

/// Parse a `[keymap]` TOML section into a flat key → intent map.
fn parse_keymap_toml(src: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let Ok(val) = src.parse::<Value>() else { return out };
    let Some(table) = val.get("keymap").and_then(|v| v.as_table()) else {
        return out;
    };
    for (k, v) in table {
        if let Some(intent) = v.as_str() {
            out.insert(k.clone(), intent.to_string());
        }
    }
    out
}
