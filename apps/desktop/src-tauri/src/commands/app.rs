//! commands/app.rs — Application-level commands (version, status, keymap, settings).

use std::collections::HashMap;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tauri::State;
use toml::Value;

use crate::state::AppState;

/// Default keymap bundled with the application binary.
/// Loaded from `config/keymap.toml` at compile time.
const DEFAULT_KEYMAP_TOML: &str = include_str!("../../../../../config/keymap.toml");

/// Returns the application version string from Cargo.toml.
#[tauri::command]
pub fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Status payload returned to the UI on startup.
#[derive(Debug, Serialize)]
pub struct AppStatus {
    pub version: String,
    pub audio_running: bool,
    pub library_count: i64,
}

/// Returns whether the audio engine is running and how many tracks the
/// library currently holds.  Used by the UI for the status bar.
#[tauri::command]
pub fn app_status(state: State<'_, AppState>) -> AppStatus {
    let audio_running = state
        .engine
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().map(|e| e.is_running()))
        .unwrap_or(false);

    let library_count = state
        .library
        .lock()
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
    let Ok(val) = src.parse::<Value>() else {
        return out;
    };
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

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

/// User-facing settings exposed to the UI.
///
/// Only the fields that the UI can sensibly change are listed here.
/// All values are read from (and saved to) the user's `settings.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub pitch_range_pct: f32,
    pub music_root: String,
    pub online_lookup: bool,
    pub target_fps: u32,
    pub update_check: bool,
}

impl Default for UiSettings {
    fn default() -> Self {
        // Mirror config/defaults.toml so the UI shows correct defaults
        // when no user override file exists.
        Self {
            sample_rate: 44_100,
            buffer_size: 128,
            pitch_range_pct: 8.0,
            music_root: "~/Music".to_string(),
            online_lookup: false,
            target_fps: 60,
            update_check: false,
        }
    }
}

/// Load current settings, merging defaults with user overrides.
#[tauri::command]
pub fn settings_load() -> UiSettings {
    let Some(dirs) = ProjectDirs::from("io", "PhraseDJ", "PhraseDJ") else {
        return UiSettings::default();
    };
    let path = dirs.config_local_dir().join("settings.toml");
    let Ok(src) = std::fs::read_to_string(&path) else {
        return UiSettings::default();
    };
    // Deserialise into a permissive intermediate Value so partial files work.
    let Ok(val) = src.parse::<Value>() else {
        return UiSettings::default();
    };
    let mut s = UiSettings::default();
    if let Some(audio) = val.get("audio") {
        if let Some(v) = audio.get("sample_rate").and_then(|v| v.as_integer()) {
            s.sample_rate = v as u32;
        }
        if let Some(v) = audio.get("buffer_size").and_then(|v| v.as_integer()) {
            s.buffer_size = v as u32;
        }
        if let Some(v) = audio.get("pitch_range_pct").and_then(|v| v.as_float()) {
            s.pitch_range_pct = v as f32;
        }
    }
    if let Some(lib) = val.get("library") {
        if let Some(v) = lib.get("music_root").and_then(|v| v.as_str()) {
            s.music_root = v.to_string();
        }
    }
    if let Some(lyrics) = val.get("lyrics") {
        if let Some(v) = lyrics.get("online_lookup").and_then(|v| v.as_bool()) {
            s.online_lookup = v;
        }
    }
    if let Some(ui) = val.get("ui") {
        if let Some(v) = ui.get("target_fps").and_then(|v| v.as_integer()) {
            s.target_fps = v as u32;
        }
    }
    if let Some(net) = val.get("network") {
        if let Some(v) = net.get("update_check").and_then(|v| v.as_bool()) {
            s.update_check = v;
        }
    }
    s
}

/// Persist settings to the user's `settings.toml`.
///
/// Writes a minimal TOML with only the fields managed by the UI.
/// Fields absent from the UI (model paths, service URLs, etc.) are not
/// touched — they remain in the existing file or fall back to defaults.
#[tauri::command]
pub fn settings_save(settings: UiSettings) -> Result<(), String> {
    let dirs = ProjectDirs::from("io", "PhraseDJ", "PhraseDJ")
        .ok_or("cannot locate app-support directory")?;
    let dir = dirs.config_local_dir().to_path_buf();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("settings.toml");

    let content = format!(
        r#"# PhraseDJ user settings — managed by the Settings UI.
# Other options are in config/defaults.toml.

[audio]
sample_rate     = {sr}
buffer_size     = {bs}
pitch_range_pct = {pr}

[library]
music_root = "{mr}"

[lyrics]
online_lookup = {ol}

[ui]
target_fps = {fps}

[network]
update_check = {uc}
"#,
        sr = settings.sample_rate,
        bs = settings.buffer_size,
        pr = settings.pitch_range_pct,
        mr = settings.music_root.replace('"', "\\\""),
        ol = settings.online_lookup,
        fps = settings.target_fps,
        uc = settings.update_check,
    );

    std::fs::write(&path, content).map_err(|e| e.to_string())
}
