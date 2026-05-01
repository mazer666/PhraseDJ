//! connect.rs — Database connection management for the local library.
//!
//! Opens the SQLite file with safe pragmas (WAL mode, foreign keys),
//! applies schema migrations on first open, and exposes a small set of
//! CRUD helpers used by the rest of the crate.

use std::path::{Path, PathBuf};

use pdj_core::{Error, Result, TrackId};
use rusqlite::{params, Connection, OptionalExtension};
use thiserror::Error;
use tracing::{debug, info};

use crate::schema::{self, AnalysisState, StemsState, Track, CURRENT_VERSION, SCHEMA_V1};

/// Library-specific errors.  Wrapped into `pdj_core::Error::Database`.
#[derive(Debug, Error)]
pub enum LibraryError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("schema version {found} is newer than supported {supported}")]
    UnsupportedSchema { found: i64, supported: i64 },

    #[error("track not found: {0}")]
    NotFound(TrackId),
}

impl From<LibraryError> for Error {
    fn from(e: LibraryError) -> Self {
        Error::Database(e.to_string())
    }
}

// ---------------------------------------------------------------------------
// Library handle
// ---------------------------------------------------------------------------

/// A handle to the local music library.
///
/// `Library` owns one SQLite connection.  Multiple instances may exist; SQLite
/// handles concurrency via WAL.  For Phase 1 a single connection is enough;
/// a connection pool can be added later if needed.
pub struct Library {
    conn: Connection,
    path: PathBuf,
}

impl Library {
    /// Open or create the library at `path`.
    ///
    /// Applies pending migrations if the schema version is older than
    /// `CURRENT_VERSION`.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&path).map_err(LibraryError::Sqlite)?;

        // Recommended pragmas for a desktop app.
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(LibraryError::Sqlite)?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(LibraryError::Sqlite)?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(LibraryError::Sqlite)?;
        conn.busy_timeout(std::time::Duration::from_secs(5))
            .map_err(LibraryError::Sqlite)?;

        let mut lib = Self { conn, path };
        lib.migrate()?;
        info!(path = ?lib.path, "library opened");
        Ok(lib)
    }

    /// Run schema migrations up to `CURRENT_VERSION`.
    fn migrate(&mut self) -> Result<()> {
        // Always create v1 baseline if missing (idempotent).
        self.conn
            .execute_batch(SCHEMA_V1)
            .map_err(LibraryError::Sqlite)?;

        // Read the current schema version.
        let v: Option<i64> = self
            .conn
            .query_row("SELECT version FROM schema_version", [], |r| r.get(0))
            .optional()
            .map_err(LibraryError::Sqlite)?;

        match v {
            None => {
                // First run — record version.
                self.conn
                    .execute(
                        "INSERT INTO schema_version (version) VALUES (?)",
                        params![CURRENT_VERSION],
                    )
                    .map_err(LibraryError::Sqlite)?;
            }
            Some(v) if v > CURRENT_VERSION => {
                return Err(LibraryError::UnsupportedSchema {
                    found: v,
                    supported: CURRENT_VERSION,
                }
                .into());
            }
            Some(_) => {
                // Future migrations will go here.  Phase 1 has only v1.
                debug!(version = v, "schema is up to date");
            }
        }
        Ok(())
    }

    // ----- CRUD ----------------------------------------------------------

    /// Insert a track row.  Returns the assigned id.
    pub fn insert_track(&self, track: &Track) -> Result<TrackId> {
        self.conn
            .execute(
                "INSERT INTO tracks (
                id, path, rel_path, title, artist, album,
                duration_ms, bpm, key,
                imported_at, analyzed_at,
                analysis_state, stems_state
            ) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)",
                params![
                    track.id.to_string(),
                    track.path,
                    track.rel_path,
                    track.title,
                    track.artist,
                    track.album,
                    track.duration_ms,
                    track.bpm,
                    track.key,
                    track.imported_at,
                    track.analyzed_at,
                    track.analysis_state.as_str(),
                    track.stems_state.as_str(),
                ],
            )
            .map_err(LibraryError::Sqlite)?;
        Ok(track.id)
    }

    /// Fetch one track by id.
    pub fn get_track(&self, id: TrackId) -> Result<Track> {
        let id_s = id.to_string();
        let track = self
            .conn
            .query_row(
                "SELECT id, path, rel_path, title, artist, album,
                    duration_ms, bpm, key,
                    imported_at, analyzed_at,
                    analysis_state, stems_state
             FROM tracks WHERE id = ?",
                params![id_s],
                Track::from_row,
            )
            .optional()
            .map_err(LibraryError::Sqlite)?
            .ok_or_else(|| Error::Database(format!("track not found: {id}")))?;
        Ok(track)
    }

    /// List the most recent N tracks.
    pub fn recent(&self, limit: u32) -> Result<Vec<Track>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, path, rel_path, title, artist, album,
                    duration_ms, bpm, key,
                    imported_at, analyzed_at,
                    analysis_state, stems_state
             FROM tracks ORDER BY imported_at DESC LIMIT ?",
            )
            .map_err(LibraryError::Sqlite)?;
        let rows = stmt
            .query_map(params![limit], Track::from_row)
            .map_err(LibraryError::Sqlite)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(LibraryError::Sqlite)?);
        }
        Ok(out)
    }

    /// Update analysis fields after the engine has finished BPM detection.
    pub fn set_bpm(&self, id: TrackId, bpm: f32) -> Result<()> {
        self.conn
            .execute(
                "UPDATE tracks
                SET bpm = ?,
                    analysis_state = ?,
                    analyzed_at = ?
              WHERE id = ?",
                params![
                    bpm,
                    AnalysisState::Beatgrid.as_str(),
                    schema::now_unix_ms(),
                    id.to_string(),
                ],
            )
            .map_err(LibraryError::Sqlite)?;
        Ok(())
    }

    /// Update stems-cache state after the analyser finishes.
    pub fn set_stems_state(&self, id: TrackId, state: StemsState) -> Result<()> {
        self.conn
            .execute(
                "UPDATE tracks SET stems_state = ? WHERE id = ?",
                params![state.as_str(), id.to_string()],
            )
            .map_err(LibraryError::Sqlite)?;
        Ok(())
    }

    /// Delete a track from the library (does not remove the audio file).
    pub fn delete(&self, id: TrackId) -> Result<()> {
        self.conn
            .execute("DELETE FROM tracks WHERE id = ?", params![id.to_string()])
            .map_err(LibraryError::Sqlite)?;
        Ok(())
    }

    /// Total track count.
    pub fn count(&self) -> Result<i64> {
        let n: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM tracks", [], |r| r.get(0))
            .map_err(LibraryError::Sqlite)?;
        Ok(n)
    }

    /// Path the library was opened from.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Borrow the underlying connection (used by sibling modules).
    pub(crate) fn conn(&self) -> &Connection {
        &self.conn
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn open_temp() -> Library {
        let dir = tempfile::tempdir().unwrap();
        Library::open(dir.path().join("library.db")).unwrap()
    }

    #[test]
    fn open_and_count_zero() {
        let lib = open_temp();
        assert_eq!(lib.count().unwrap(), 0);
    }

    #[test]
    fn insert_and_get_roundtrip() {
        let lib = open_temp();
        let mut t = Track::new_from_path("/music/song.flac");
        t.title = Some("Test Track".into());
        t.artist = Some("Test Artist".into());
        let id = lib.insert_track(&t).unwrap();
        let got = lib.get_track(id).unwrap();
        assert_eq!(got.title.as_deref(), Some("Test Track"));
        assert_eq!(got.artist.as_deref(), Some("Test Artist"));
        assert_eq!(lib.count().unwrap(), 1);
    }

    #[test]
    fn set_bpm_updates_state() {
        let lib = open_temp();
        let t = Track::new_from_path("/music/song.flac");
        let id = lib.insert_track(&t).unwrap();
        lib.set_bpm(id, 128.5).unwrap();
        let got = lib.get_track(id).unwrap();
        assert!(got.bpm.is_some());
        assert!((got.bpm.unwrap() - 128.5).abs() < 0.01);
        assert_eq!(got.analysis_state, AnalysisState::Beatgrid);
    }

    #[test]
    fn delete_removes_track() {
        let lib = open_temp();
        let t = Track::new_from_path("/music/song.flac");
        let id = lib.insert_track(&t).unwrap();
        lib.delete(id).unwrap();
        assert_eq!(lib.count().unwrap(), 0);
    }
}
