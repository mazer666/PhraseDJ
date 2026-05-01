/// Cache directory layout helpers for stem files.
///
/// # Layout on disk
///
/// ```text
/// <app-support>/PhraseDJ/cache/stems/
///   └── <track_uuid>/
///         ├── vocals.flac
///         ├── drums.flac
///         ├── bass.flac
///         └── other.flac
/// ```
///
/// This module provides functions to derive those paths from a `TrackId`
/// and to check whether all four stem files are already present (i.e. the
/// analysis was completed in a previous session).
///
/// All path logic is centralised here so that a rename only requires
/// touching this one file.
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use pdj_core::{types::TrackId, Error, Result};

// ---------------------------------------------------------------------------
// StemPaths
// ---------------------------------------------------------------------------

/// Absolute paths to the four cached stem files for one track.
///
/// These files use the FLAC container, 16-bit PCM, at the source sample
/// rate (usually 44 100 Hz).
#[derive(Debug, Clone)]
pub struct StemPaths {
    /// Vocals (lead + backing).
    pub vocals: PathBuf,
    /// Drums (kick, snare, hats, cymbals).
    pub drums: PathBuf,
    /// Bass (bass guitar, sub-bass).
    pub bass: PathBuf,
    /// Other (guitars, synths, strings, …).
    pub other: PathBuf,
}

impl StemPaths {
    /// Return `true` if all four FLAC files exist on disk.
    pub fn all_exist(&self) -> bool {
        self.vocals.exists() && self.drums.exists() && self.bass.exists() && self.other.exists()
    }
}

// ---------------------------------------------------------------------------
// Root cache directory
// ---------------------------------------------------------------------------

/// Absolute path to the stem cache root directory.
///
/// On macOS: `~/Library/Application Support/io.PhraseDJ.PhraseDJ/cache/stems`
///
/// The directory is created if it does not exist yet.
pub fn stem_cache_root() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("io", "PhraseDJ", "PhraseDJ")
        .ok_or_else(|| Error::Settings("cannot determine app-support directory".into()))?;

    // We put stems under <data-dir>/cache/stems/ so they can be wiped
    // without touching settings or the database.
    let root = dirs.data_local_dir().join("cache").join("stems");
    std::fs::create_dir_all(&root)?;
    Ok(root)
}

/// Absolute path to the HTDemucs ONNX model file.
///
/// On macOS: `~/Library/Application Support/io.PhraseDJ.PhraseDJ/models/htdemucs.onnx`
pub fn model_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("io", "PhraseDJ", "PhraseDJ")
        .ok_or_else(|| Error::Settings("cannot determine app-support directory".into()))?;
    Ok(dirs.data_local_dir().join("models").join("htdemucs.onnx"))
}

// ---------------------------------------------------------------------------
// Per-track directory
// ---------------------------------------------------------------------------

/// Absolute path to the per-track stem directory.
///
/// The directory is created if it does not exist.
pub fn track_stem_dir(root: &Path, track: TrackId) -> Result<PathBuf> {
    let dir = root.join(track.to_string());
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Build `StemPaths` for a track given the cache root directory.
///
/// Does **not** check whether the files exist — call `StemPaths::all_exist`
/// for that.
pub fn stem_paths_for(root: &Path, track: TrackId) -> StemPaths {
    let track_id_s = track.to_string();
    let track_dir = root.join(track_id_s);

    let stem_path = |name: &str| track_dir.join(format!("{name}.flac"));

    StemPaths {
        vocals: stem_path("vocals"),
        drums: stem_path("drums"),
        bass: stem_path("bass"),
        other: stem_path("other"),
    }
}

// ---------------------------------------------------------------------------
// LRU cache pruning helpers
// ---------------------------------------------------------------------------

/// Total size of all FLAC files under `root`, in bytes.
///
/// Used by the LRU pruner to decide whether the cache budget is exceeded.
pub fn cache_size_bytes(root: &Path) -> u64 {
    walkdir_size(root)
}

/// Recursively sum the sizes of all files under `dir`.
fn walkdir_size(dir: &Path) -> u64 {
    // We avoid pulling in `walkdir` for this simple case.
    let mut total: u64 = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                total += walkdir_size(&path);
            } else if let Ok(meta) = std::fs::metadata(&path) {
                total += meta.len();
            }
        }
    }
    total
}

/// Remove the stem directory for a single track (all four FLAC files).
///
/// Called by the LRU pruner when the cache is full.
pub fn remove_track_stems(root: &Path, track: TrackId) -> Result<()> {
    let dir = root.join(track.to_string());
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stem_paths_for_uses_expected_filenames() {
        let root = PathBuf::from("/tmp/stems");
        let track = TrackId::new();
        let paths = stem_paths_for(&root, track);

        // Every path must end in the expected stem name + .flac.
        assert!(paths.vocals.ends_with("vocals.flac"));
        assert!(paths.drums.ends_with("drums.flac"));
        assert!(paths.bass.ends_with("bass.flac"));
        assert!(paths.other.ends_with("other.flac"));
    }

    #[test]
    fn all_exist_returns_false_when_files_are_missing() {
        let root = PathBuf::from("/tmp/stems");
        let track = TrackId::new();
        let paths = stem_paths_for(&root, track);
        // Files don't actually exist — should report false.
        assert!(!paths.all_exist());
    }

    #[test]
    fn cache_size_on_empty_dir_is_zero() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert_eq!(cache_size_bytes(dir.path()), 0);
    }
}
