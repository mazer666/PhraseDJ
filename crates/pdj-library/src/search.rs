//! search.rs — Library search.
//!
//! Phase 1: simple LIKE-based search on title and artist.
//! Phase 2 will switch to FTS5 for full-text search.

use pdj_core::Result;
use serde::{Deserialize, Serialize};

use crate::connect::Library;
use crate::schema::{AnalysisState, StemsState, Track};

/// A search query.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Query {
    /// Free-text fragment matched against title / artist / album.
    pub text: Option<String>,
    /// Inclusive minimum BPM.
    pub bpm_min: Option<f32>,
    /// Inclusive maximum BPM.
    pub bpm_max: Option<f32>,
    /// Maximum number of rows returned (default 200).
    pub limit: Option<u32>,
}

/// Run a search.
pub fn search(lib: &Library, q: &Query) -> Result<Vec<Track>> {
    let mut sql = String::from(
        "SELECT id, path, rel_path, title, artist, album,
                duration_ms, bpm, key,
                imported_at, analyzed_at,
                analysis_state, stems_state
           FROM tracks WHERE 1=1",
    );
    let mut args: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(text) = &q.text {
        let like = format!("%{}%", text.replace('%', "\\%"));
        sql.push_str(" AND (title LIKE ? OR artist LIKE ? OR album LIKE ?)");
        args.push(like.clone().into());
        args.push(like.clone().into());
        args.push(like.into());
    }
    if let Some(min) = q.bpm_min {
        sql.push_str(" AND bpm >= ?");
        args.push((min as f64).into());
    }
    if let Some(max) = q.bpm_max {
        sql.push_str(" AND bpm <= ?");
        args.push((max as f64).into());
    }
    sql.push_str(" ORDER BY artist, album, title");
    sql.push_str(" LIMIT ?");
    args.push((q.limit.unwrap_or(200) as i64).into());

    let mut stmt = lib
        .conn()
        .prepare(&sql)
        .map_err(|e| pdj_core::Error::Database(e.to_string()))?;

    // Convert Vec<Value> into a slice of &dyn ToSql via params_from_iter.
    let rows = stmt
        .query_map(rusqlite::params_from_iter(args.iter()), Track::from_row)
        .map_err(|e| pdj_core::Error::Database(e.to_string()))?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| pdj_core::Error::Database(e.to_string()))?);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connect::Library;
    use crate::schema::Track;

    fn lib_with_three_tracks() -> (tempfile::TempDir, Library) {
        let dir = tempfile::tempdir().unwrap();
        let lib = Library::open(dir.path().join("lib.db")).unwrap();
        let mk = |title: &str, artist: &str, bpm: f32| {
            let mut t = Track::new_from_path(format!("/music/{title}.flac"));
            t.title = Some(title.into());
            t.artist = Some(artist.into());
            t.bpm = Some(bpm);
            lib.insert_track(&t).unwrap();
        };
        mk("Strobe", "Deadmau5", 128.0);
        mk("Levels", "Avicii", 126.0);
        mk("Ghosts", "Deadmau5", 105.0);
        (dir, lib)
    }

    #[test]
    fn empty_query_returns_all() {
        let (_d, lib) = lib_with_three_tracks();
        let res = search(&lib, &Query::default()).unwrap();
        assert_eq!(res.len(), 3);
    }

    #[test]
    fn text_query_filters_by_artist() {
        let (_d, lib) = lib_with_three_tracks();
        let q = Query {
            text: Some("Deadmau5".into()),
            ..Default::default()
        };
        let res = search(&lib, &q).unwrap();
        assert_eq!(res.len(), 2);
    }

    #[test]
    fn bpm_range_filters() {
        let (_d, lib) = lib_with_three_tracks();
        let q = Query {
            bpm_min: Some(120.0),
            bpm_max: Some(127.0),
            ..Default::default()
        };
        let res = search(&lib, &q).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].title.as_deref(), Some("Levels"));
    }
}
