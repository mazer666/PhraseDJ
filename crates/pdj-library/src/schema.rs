//! schema.rs — SQLite schema definitions and Rust models.
//!
//! See `specs/07-library.md` for the full design.  Phase 1 ships v1 of the
//! schema; later phases add new columns via numbered migration files.

use pdj_core::TrackId;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Schema SQL — applied on first open and on every version bump.
// ---------------------------------------------------------------------------

/// SQL applied to a fresh database to bring it to schema version 1.
pub const SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS tracks (
    id              TEXT PRIMARY KEY,
    path            TEXT NOT NULL UNIQUE,
    rel_path        TEXT,
    title           TEXT,
    artist          TEXT,
    album           TEXT,
    isrc            TEXT,
    duration_ms     INTEGER,
    sample_rate     INTEGER,
    channels        INTEGER,
    bitrate_kbps    INTEGER,
    bpm             REAL,
    key             TEXT,
    energy          REAL,
    first_beat_ms   INTEGER,
    imported_at     INTEGER NOT NULL,
    analyzed_at     INTEGER,
    analysis_state  TEXT NOT NULL DEFAULT 'raw',
    stems_state     TEXT NOT NULL DEFAULT 'pending',
    lyrics_state    TEXT NOT NULL DEFAULT 'pending',
    notes           TEXT
);
CREATE INDEX IF NOT EXISTS idx_tracks_artist ON tracks(artist);
CREATE INDEX IF NOT EXISTS idx_tracks_bpm    ON tracks(bpm);
CREATE INDEX IF NOT EXISTS idx_tracks_key    ON tracks(key);
"#;

/// Schema version this build of `pdj-library` writes and reads.
pub const CURRENT_VERSION: i64 = 1;

// ---------------------------------------------------------------------------
// Rust models
// ---------------------------------------------------------------------------

/// Analysis state for beatgrid / key / energy detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnalysisState {
    /// Just imported, no analysis run yet.
    Raw,
    /// Beatgrid + BPM detected, deeper analysis pending.
    Beatgrid,
    /// All analysis fields populated.
    Full,
    /// Analysis attempted but failed; see `notes` column for details.
    Failed,
}

impl AnalysisState {
    pub fn as_str(self) -> &'static str {
        match self {
            AnalysisState::Raw => "raw",
            AnalysisState::Beatgrid => "beatgrid",
            AnalysisState::Full => "full",
            AnalysisState::Failed => "failed",
        }
    }
    pub fn parse(s: &str) -> Self {
        match s {
            "beatgrid" => AnalysisState::Beatgrid,
            "full" => AnalysisState::Full,
            "failed" => AnalysisState::Failed,
            _ => AnalysisState::Raw,
        }
    }
}

/// State of stem-separation cache for a track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StemsState {
    Pending,
    Running,
    Cached,
    Failed,
}

impl StemsState {
    pub fn as_str(self) -> &'static str {
        match self {
            StemsState::Pending => "pending",
            StemsState::Running => "running",
            StemsState::Cached => "cached",
            StemsState::Failed => "failed",
        }
    }
    pub fn parse(s: &str) -> Self {
        match s {
            "running" => StemsState::Running,
            "cached" => StemsState::Cached,
            "failed" => StemsState::Failed,
            _ => StemsState::Pending,
        }
    }
}

/// One row in the `tracks` table.  Optional fields mirror SQL nullability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: TrackId,
    pub path: String,
    pub rel_path: Option<String>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration_ms: Option<u64>,
    pub bpm: Option<f32>,
    pub key: Option<String>,
    pub imported_at: i64,
    pub analyzed_at: Option<i64>,
    pub analysis_state: AnalysisState,
    pub stems_state: StemsState,
}

impl Track {
    /// Build a minimal Track for a freshly imported file.
    pub fn new_from_path(path: impl Into<String>) -> Self {
        Self {
            id: TrackId::new(),
            path: path.into(),
            rel_path: None,
            title: None,
            artist: None,
            album: None,
            duration_ms: None,
            bpm: None,
            key: None,
            imported_at: now_unix_ms(),
            analyzed_at: None,
            analysis_state: AnalysisState::Raw,
            stems_state: StemsState::Pending,
        }
    }

    /// Map a rusqlite Row to a Track.
    ///
    /// Assumes the SELECT query includes columns in the standard order:
    /// id, path, rel_path, title, artist, album, duration_ms, bpm, key,
    /// imported_at, analyzed_at, analysis_state, stems_state.
    pub fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        let id_s: String = row.get(0)?;
        let id = TrackId::parse(&id_s).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?;
        Ok(Track {
            id,
            path: row.get(1)?,
            rel_path: row.get(2)?,
            title: row.get(3)?,
            artist: row.get(4)?,
            album: row.get(5)?,
            duration_ms: row.get(6)?,
            bpm: row.get(7)?,
            key: row.get(8)?,
            imported_at: row.get(9)?,
            analyzed_at: row.get(10)?,
            analysis_state: AnalysisState::parse(&row.get::<_, String>(11)?),
            stems_state: StemsState::parse(&row.get::<_, String>(12)?),
        })
    }
}

/// Current Unix time in milliseconds.
pub fn now_unix_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
