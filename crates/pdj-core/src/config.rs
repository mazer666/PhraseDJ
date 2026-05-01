/// Settings loader for PhraseDJ.
///
/// # How settings work
///
/// There are two layers:
///   1. `config/defaults.toml` – shipped defaults, read-only at runtime.
///   2. `<app-support>/PhraseDJ/settings.toml` – user overrides, written
///      when the user changes something in the Settings UI.
///
/// This module merges the two layers.  User values win over defaults.
/// Neither file is required; missing = use the other layer's values.
///
/// All constants (buffer sizes, paths, colours, …) must come from here.
/// Hard-coding a value in business logic is a spec violation.
use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::{Error, Result};

// ---------------------------------------------------------------------------
// Settings struct — mirrors config/defaults.toml exactly.
// ---------------------------------------------------------------------------

/// Top-level settings for PhraseDJ.
///
/// The fields are grouped by subsystem (audio, library, ui, …).
/// Each field has a `#[serde(default)]` so that a partial user file still
/// works — missing keys fall back to `Default::default()`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub audio: AudioSettings,
    pub library: LibrarySettings,
    pub stems: StemsSettings,
    pub lyrics: LyricsSettings,
    pub ui: UiSettings,
    pub network: NetworkSettings,
}

/// Audio engine configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioSettings {
    /// Audio output sample rate in Hz.  44100 or 48000 are common.
    pub sample_rate: u32,
    /// Frames per buffer.  Smaller = lower latency but higher CPU risk.
    pub buffer_size: u32,
    /// Output device name.  Empty string means "system default".
    pub output_device: String,
    /// Optional second device used for headphone cue output.
    pub cue_device: String,
    /// Vinyl-style pitch range in percent.  Default ±8 %.
    pub pitch_range_pct: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            sample_rate: 44_100,
            buffer_size: 128,
            output_device: String::new(),
            cue_device: String::new(),
            pitch_range_pct: 8.0,
        }
    }
}

/// Local library settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LibrarySettings {
    /// Root folder for relative path resolution.
    pub music_root: String,
    /// Follow symlinks when scanning folders.
    pub follow_symlinks: bool,
    /// Glob patterns for files to skip during scan.
    pub ignore_files: Vec<String>,
    /// Number of nightly database backup snapshots to keep.
    pub backup_count: u32,
    /// Maximum stem cache size in gigabytes before LRU pruning.
    pub stem_cache_gb: f32,
}

impl Default for LibrarySettings {
    fn default() -> Self {
        Self {
            music_root: String::from("~/Music"),
            follow_symlinks: false,
            ignore_files: vec!["*.tmp".into(), "*.part".into()],
            backup_count: 7,
            stem_cache_gb: 50.0,
        }
    }
}

/// Stem separation settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StemsSettings {
    /// AI model to use.  "htdemucs" is the default.
    pub model: String,
    /// Number of stems (4 or 6).
    pub stem_count: u8,
    /// Whether to run analysis automatically on every import.
    pub auto_analyse: bool,
    /// Maximum parallel analysis jobs.
    ///
    /// `0` means "n_performance_cores − 1" (auto), which leaves one core
    /// free for real-time audio playback.
    pub max_parallel_jobs: u8,
}

impl Default for StemsSettings {
    fn default() -> Self {
        Self {
            model: String::from("htdemucs"),
            stem_count: 4,
            auto_analyse: true,
            max_parallel_jobs: 0,
        }
    }
}

/// Lyrics settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LyricsSettings {
    /// Allow fetching lyrics from the internet (opt-in).
    pub online_lookup: bool,
    /// Primary online service URL.
    pub online_service_url: String,
    /// Whisper model name for local alignment.
    pub whisper_model: String,
    /// Request timeout for online lookups in seconds.
    pub online_timeout_secs: u64,
}

impl Default for LyricsSettings {
    fn default() -> Self {
        Self {
            online_lookup: false,
            online_service_url: String::from("https://lrclib.net/api/get"),
            whisper_model: String::from("ggml-base.en"),
            online_timeout_secs: 5,
        }
    }
}

/// UI display settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiSettings {
    /// Target display refresh rate for waveform rendering (60 or 120).
    pub target_fps: u32,
    /// Default mode on startup: "classic", "stem", or "macro".
    pub default_mode: String,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            target_fps: 60,
            default_mode: String::from("classic"),
        }
    }
}

/// Network and privacy settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkSettings {
    /// Check for updates on startup (opt-in).
    pub update_check: bool,
}

// ---------------------------------------------------------------------------
// Loader
// ---------------------------------------------------------------------------

/// Loaded and merged settings.
pub struct Config {
    pub settings: Settings,
    /// Path to the user settings file (may or may not exist yet).
    pub user_path: PathBuf,
}

impl Config {
    /// Load settings from the shipped defaults file and the user override file.
    ///
    /// `defaults_path` points to `config/defaults.toml` (relative to the
    /// binary's resource dir).  Pass `None` to use only built-in `Default`
    /// values (useful in tests).
    pub fn load(defaults_path: Option<&std::path::Path>) -> Result<Self> {
        // Start from Rust's Default, so we always have every field.
        let mut settings = Settings::default();

        // Layer 1: shipped defaults.toml (if provided).
        if let Some(path) = defaults_path {
            if path.exists() {
                let text = std::fs::read_to_string(path)?;
                let from_file: Settings = toml::from_str(&text)?;
                settings = from_file;
                debug!(?path, "Loaded shipped defaults");
            } else {
                warn!(?path, "defaults.toml not found – using built-in defaults");
            }
        }

        // Layer 2: user overrides in app-support directory.
        let user_path = user_settings_path()?;
        if user_path.exists() {
            let text = std::fs::read_to_string(&user_path)?;
            // Parse as a partial TOML value and merge field-by-field so a
            // missing key in the user file keeps the default.
            merge_user_overrides(&mut settings, &text)?;
            debug!(?user_path, "Merged user settings");
        }

        Ok(Config {
            settings,
            user_path,
        })
    }

    /// Write the current settings to the user override file atomically.
    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.user_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text =
            toml::to_string_pretty(&self.settings).map_err(|e| Error::Settings(e.to_string()))?;
        // Atomic write via a temp file + rename.
        let tmp = self.user_path.with_extension("toml.tmp");
        std::fs::write(&tmp, text)?;
        std::fs::rename(&tmp, &self.user_path)?;
        Ok(())
    }
}

/// Return the path to the user settings file, creating intermediate
/// directories if needed.
pub fn user_settings_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("io", "PhraseDJ", "PhraseDJ")
        .ok_or_else(|| Error::Settings("cannot determine app-support directory".into()))?;
    Ok(dirs.config_local_dir().join("settings.toml"))
}

/// Merge a partial TOML string (user overrides) onto an existing Settings.
///
/// Fields absent from `text` are left unchanged so the user file can be
/// sparse.
fn merge_user_overrides(base: &mut Settings, text: &str) -> Result<()> {
    // Parse into a generic Value so we only overwrite present keys.
    let user_val: toml::Value = toml::from_str(text)?;
    let base_str = toml::to_string(base).map_err(|e| Error::Settings(e.to_string()))?;
    let mut base_val: toml::Value = toml::from_str(&base_str)?;
    merge_toml(&mut base_val, user_val);
    *base = base_val
        .try_into()
        .map_err(|e: toml::de::Error| Error::Toml(e))?;
    Ok(())
}

/// Recursively merge `src` into `dst`.  Only present keys in `src` win.
fn merge_toml(dst: &mut toml::Value, src: toml::Value) {
    match (dst, src) {
        (toml::Value::Table(d), toml::Value::Table(s)) => {
            for (k, v) in s {
                merge_toml(d.entry(k).or_insert(toml::Value::Boolean(false)), v);
            }
        }
        (dst, src) => *dst = src,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sensible() {
        let cfg = Config::load(None).expect("load with no files");
        let s = &cfg.settings;
        // Audio defaults.
        assert_eq!(s.audio.sample_rate, 44_100);
        assert_eq!(s.audio.buffer_size, 128);
        // Library defaults.
        assert!(!s.library.follow_symlinks);
        assert_eq!(s.library.backup_count, 7);
        // Privacy defaults: network OFF by default.
        assert!(!s.lyrics.online_lookup);
        assert!(!s.network.update_check);
    }

    #[test]
    fn user_override_merges_correctly() {
        let user_toml = r#"
[audio]
buffer_size = 256
"#;
        let mut settings = Settings::default();
        merge_user_overrides(&mut settings, user_toml).expect("merge");
        // User value wins.
        assert_eq!(settings.audio.buffer_size, 256);
        // Unchanged field keeps default.
        assert_eq!(settings.audio.sample_rate, 44_100);
    }

    #[test]
    fn save_and_reload_is_roundtrip() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let path = dir.path().join("settings.toml");
        let mut cfg = Config::load(None).expect("load");
        // Mutate a value.
        cfg.settings.audio.buffer_size = 512;
        cfg = Config {
            settings: cfg.settings,
            user_path: path.clone(),
        };
        cfg.save().expect("save");
        // Reload.
        let text = std::fs::read_to_string(&path).expect("read");
        let mut reloaded = Settings::default();
        merge_user_overrides(&mut reloaded, &text).expect("merge");
        assert_eq!(reloaded.audio.buffer_size, 512);
    }
}
