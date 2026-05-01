//! commands/library.rs — Library import / search / list commands.

use std::path::PathBuf;

use pdj_library::{scan_folder, search, ImportOutcome, Query, ScanReport, Track};
use tauri::State;

use crate::state::AppState;

/// Import a single audio file into the library.  Idempotent.
#[tauri::command]
pub fn library_import_file(path: String, state: State<'_, AppState>)
    -> Result<String, String>
{
    let lib = state.library.lock().map_err(|e| e.to_string())?;
    let outcome = pdj_library::import_file(&lib, PathBuf::from(&path))
        .map_err(|e| e.to_string())?;
        
    // Enqueue for background stem separation.
    if let Err(e) = state.stems.submit(pdj_stems::QueueJob {
        track: outcome.id(),
        path:  PathBuf::from(path),
    }) {
        tracing::warn!("Failed to queue stem separation: {}", e);
    }

    Ok(match outcome {
        ImportOutcome::Added(id)    => format!("added:{id}"),
        ImportOutcome::Existing(id) => format!("existing:{id}"),
    })
}

/// Recursively scan a folder for audio files.
#[tauri::command]
pub fn library_scan_folder(path: String, state: State<'_, AppState>)
    -> Result<ScanReport, String>
{
    let lib = state.library.lock().map_err(|e| e.to_string())?;
    scan_folder(&lib, PathBuf::from(path)).map_err(|e| e.to_string())
}

/// Return the most-recently-imported tracks.
#[tauri::command]
pub fn library_recent(limit: u32, state: State<'_, AppState>)
    -> Result<Vec<Track>, String>
{
    let lib = state.library.lock().map_err(|e| e.to_string())?;
    lib.recent(limit).map_err(|e| e.to_string())
}

/// Search the library.
#[tauri::command]
pub fn library_search(query: Query, state: State<'_, AppState>)
    -> Result<Vec<Track>, String>
{
    let lib = state.library.lock().map_err(|e| e.to_string())?;
    search(&lib, &query).map_err(|e| e.to_string())
}
