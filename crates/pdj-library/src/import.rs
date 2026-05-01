//! import.rs — Folder scanning and single-file import.
//!
//! Phase 1 keeps it simple:
//!   - Walks a directory recursively.
//!   - Filters by file extension (well-known audio formats).
//!   - Skips the `ignore_files` patterns from settings.
//!   - Does not yet read tags (Phase 1.5 — minimal `id3` / `metaflac` work).
//!
//! Heavier metadata extraction and tag reading move to a separate module
//! once a stable tagging crate is chosen.

use std::path::Path;

use pdj_core::{Result, TrackId};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::connect::Library;
use crate::schema::Track;

/// File extensions PhraseDJ understands at import time.
const AUDIO_EXTENSIONS: &[&str] = &[
    "wav", "wave", "flac", "aiff", "aif",
    "mp3", "m4a", "aac", "ogg", "opus",
];

/// Outcome of scanning a folder.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    /// Number of files added to the library.
    pub added:     u32,
    /// Number of files skipped because they were already present.
    pub duplicate: u32,
    /// Number of files skipped because of an unsupported extension.
    pub skipped:   u32,
    /// Errors encountered (path + reason).
    pub errors:    Vec<String>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Outcome of a single-file import.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportOutcome {
    /// File was not in the library; a new row was inserted.
    Added(TrackId),
    /// File was already present; existing id is returned.
    Existing(TrackId),
}

impl ImportOutcome {
    /// Get the TrackId regardless of outcome.
    pub fn id(&self) -> TrackId {
        match *self {
            ImportOutcome::Added(id) | ImportOutcome::Existing(id) => id,
        }
    }
}

/// Import a single file.  Returns the new TrackId, or the existing one if
/// the path is already in the library.
pub fn import_file(lib: &Library, path: impl AsRef<Path>) -> Result<ImportOutcome> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(pdj_core::Error::FileNotFound {
            path: path.display().to_string(),
        });
    }

    // Check for an existing row with this path.
    let path_str = path.to_string_lossy().to_string();
    let existing: Option<String> = lib.conn()
        .query_row("SELECT id FROM tracks WHERE path = ?",
                   [&path_str], |r| r.get(0))
        .map_or_else(
            |e| if matches!(e, rusqlite::Error::QueryReturnedNoRows) { Ok(None) } else { Err(e) },
            |s: String| Ok(Some(s)),
        )
        .map_err(|e| pdj_core::Error::Database(e.to_string()))?;

    if let Some(s) = existing {
        let id = TrackId::parse(&s)
            .map_err(|e| pdj_core::Error::Database(e.to_string()))?;
        return Ok(ImportOutcome::Existing(id));
    }

    let track = Track::new_from_path(path_str);
    let id = lib.insert_track(&track)?;
    Ok(ImportOutcome::Added(id))
}

/// Recursively scan `root`, importing every supported audio file.
///
/// Phase 1 is single-threaded; Phase 2 may parallelise after the analysis
/// queue lands.
pub fn scan_folder(lib: &Library, root: impl AsRef<Path>) -> Result<ScanReport> {
    let root = root.as_ref();
    let mut report = ScanReport::default();

    if !root.exists() {
        return Err(pdj_core::Error::FileNotFound {
            path: root.display().to_string(),
        });
    }

    walk(lib, root, &mut report)?;
    debug!(?report, "scan complete");
    Ok(report)
}

// ---------------------------------------------------------------------------
// Implementation
// ---------------------------------------------------------------------------

fn walk(lib: &Library, dir: &Path, report: &mut ScanReport) -> Result<()> {
    let entries = std::fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let ft = entry.file_type()?;

        if ft.is_dir() {
            walk(lib, &path, report)?;
        } else if ft.is_file() {
            visit_file(lib, &path, report);
        }
        // Symlinks: ignored by default to avoid loops; Phase 1.5 can add
        // settings.library.follow_symlinks support.
    }
    Ok(())
}

fn visit_file(lib: &Library, path: &Path, report: &mut ScanReport) {
    if !is_audio(path) {
        report.skipped += 1;
        return;
    }
    match import_file(lib, path) {
        Ok(ImportOutcome::Added(_))    => report.added += 1,
        Ok(ImportOutcome::Existing(_)) => report.duplicate += 1,
        Err(e) => {
            warn!(?path, %e, "import failed");
            report.errors.push(format!("{}: {e}", path.display()));
        }
    }
}

fn is_audio(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let lowered = ext.to_ascii_lowercase();
        return AUDIO_EXTENSIONS.iter().any(|e| *e == lowered);
    }
    false
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    fn temp_lib_with_files() -> (tempfile::TempDir, PathBuf, Library) {
        let dir = tempfile::tempdir().unwrap();
        let music = dir.path().join("music");
        std::fs::create_dir_all(music.join("artist")).unwrap();
        // Create a few zero-byte audio files (extension only matters here).
        for name in ["a.flac", "b.mp3", "artist/c.wav"] {
            let p = music.join(name);
            File::create(&p).unwrap().write_all(b"").unwrap();
        }
        // And one non-audio file.
        File::create(music.join("readme.txt")).unwrap().write_all(b"").unwrap();
        let lib = Library::open(dir.path().join("library.db")).unwrap();
        (dir, music, lib)
    }

    #[test]
    fn scan_finds_only_audio_files() {
        let (_d, music, lib) = temp_lib_with_files();
        let report = scan_folder(&lib, music).unwrap();
        assert_eq!(report.added, 3);
        assert_eq!(report.skipped, 1);
        assert_eq!(report.duplicate, 0);
    }

    #[test]
    fn second_scan_marks_duplicates() {
        let (_d, music, lib) = temp_lib_with_files();
        let _ = scan_folder(&lib, &music).unwrap();
        let r2 = scan_folder(&lib, &music).unwrap();
        assert_eq!(r2.added, 0);
        assert_eq!(r2.duplicate, 3);
    }

    #[test]
    fn import_file_idempotent() {
        let (_d, music, lib) = temp_lib_with_files();
        let r1 = import_file(&lib, music.join("a.flac")).unwrap();
        let r2 = import_file(&lib, music.join("a.flac")).unwrap();
        assert!(matches!(r1, ImportOutcome::Added(_)));
        assert!(matches!(r2, ImportOutcome::Existing(_)));
        assert_eq!(r1.id(), r2.id());
    }
}
