//! commands/deck.rs — Deck control commands.
//!
//! All commands return a `Result<_, String>` so the React frontend can
//! display errors as toast messages.  Internal errors are stringified
//! before crossing the IPC boundary.

use std::path::PathBuf;

use serde::Serialize;
use tauri::State;

use crate::state::AppState;

/// Waveform peak data sent to the UI for canvas rendering.
#[derive(Debug, Serialize)]
pub struct WaveformData {
    /// Number of bins (length of both peak arrays).
    pub num_bins: u32,
    /// Minimum (negative) amplitude per bin.
    pub peaks_min: Vec<f32>,
    /// Maximum (positive) amplitude per bin.
    pub peaks_max: Vec<f32>,
    /// Stem peaks if available [vocals, drums, bass, other]. All are positive.
    pub stem_peaks: Option<Vec<Vec<f32>>>,
    /// Total frames in the track at the engine's sample rate.
    pub total_frames: u64,
}

/// One-shot snapshot of a deck's state.  Sent on every UI poll.
#[derive(Debug, Serialize)]
pub struct DeckState {
    pub deck: u32,
    pub loaded: bool,
    pub playing: bool,
    pub position: u64,
    pub bpm: f32,
    /// Current playback speed ratio (1.0 = normal, 1.05 = +5 %).
    pub tempo_ratio: f32,
}

/// Load a file onto a deck (`deck` = 0 or 1).
#[tauri::command]
pub async fn deck_load(deck: u32, path: String, state: State<'_, AppState>) -> Result<String, String> {
    let p = PathBuf::from(&path);

    // Auto-import to get TrackId (or find existing)
    let track_id = {
        let lib = state.library.lock().map_err(|e| e.to_string())?;
        pdj_library::import_file(&lib, &p)
            .map(|outcome| outcome.id())
            .map_err(|e| e.to_string())?
    };

    // Enqueue for background stem separation. If it's already cached or running,
    // the queue handles idempotency internally.
    if let Err(e) = state.stems.enqueue(track_id, p.clone()).await {
        tracing::warn!("Failed to queue stem separation: {}", e);
    }

    // Check if stems exist
    let has_stems = {
        if let Ok(root) = pdj_stems::paths::stem_cache_root() {
            let paths = pdj_stems::paths::stem_paths_for(&root, track_id);
            if paths.all_exist() {
                Some(paths)
            } else {
                None
            }
        } else {
            None
        }
    };

    let guard = state.engine.lock().map_err(|e| e.to_string())?;
    let engine = guard.as_ref().ok_or("audio engine not available")?;

    if let Some(stems) = has_stems {
        let arr = [
            stems.vocals.as_path(),
            stems.drums.as_path(),
            stems.bass.as_path(),
            stems.other.as_path(),
        ];
        match engine.load_stems(deck, &p, &arr) {
            Ok(()) => return Ok(track_id.to_string()),
            Err(e) => {
                tracing::warn!(deck, error = %e, "stem loading failed, falling back to normal load");
                // Fall through to normal load below.
            }
        }
    }

    engine.load(deck, &p).map_err(|e| e.to_string())?;
    Ok(track_id.to_string())
}

/// Toggle play/pause on a deck.
#[tauri::command]
pub fn deck_play(deck: u32, state: State<'_, AppState>) -> Result<(), String> {
    let guard = state.engine.lock().map_err(|e| e.to_string())?;
    let engine = guard.as_ref().ok_or("audio engine not available")?;
    engine.play(deck).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn deck_pause(deck: u32, state: State<'_, AppState>) -> Result<(), String> {
    let guard = state.engine.lock().map_err(|e| e.to_string())?;
    let engine = guard.as_ref().ok_or("audio engine not available")?;
    engine.pause(deck).map_err(|e| e.to_string())
}

/// Seek a deck to an absolute frame position.
#[tauri::command]
pub fn deck_seek(deck: u32, position: u64, state: State<'_, AppState>) -> Result<(), String> {
    let guard = state.engine.lock().map_err(|e| e.to_string())?;
    let engine = guard.as_ref().ok_or("audio engine not available")?;
    engine.seek(deck, position).map_err(|e| e.to_string())
}

/// Return current state of a deck (called by the UI poll loop).
#[tauri::command]
pub fn deck_state(deck: u32, state: State<'_, AppState>) -> Result<DeckState, String> {
    let guard = state.engine.lock().map_err(|e| e.to_string())?;
    let engine = guard.as_ref().ok_or("audio engine not available")?;
    Ok(DeckState {
        deck,
        loaded: engine.is_loaded(deck),
        playing: engine.is_playing(deck),
        position: engine.position(deck),
        bpm: engine.get_bpm(deck),
        tempo_ratio: engine.get_tempo_ratio(deck),
    })
}

/// Set the playback speed ratio for a deck (1.0 = normal, clamped to 0.5–2.0).
#[tauri::command]
pub fn deck_set_tempo_ratio(
    deck: u32,
    ratio: f32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let guard = state.engine.lock().map_err(|e| e.to_string())?;
    let engine = guard.as_ref().ok_or("audio engine not available")?;
    engine.set_tempo_ratio(deck, ratio);
    Ok(())
}

/// Synchronise a deck's tempo to the other deck's BPM.
///
/// Both decks must have BPM data from analysis; returns an error otherwise.
#[tauri::command]
pub fn deck_sync(deck: u32, state: State<'_, AppState>) -> Result<(), String> {
    let guard = state.engine.lock().map_err(|e| e.to_string())?;
    let engine = guard.as_ref().ok_or("audio engine not available")?;
    engine.sync_tempo(deck).map_err(|e| e.to_string())
}

/// Nudge the playback speed by a small delta.
///
/// `delta` is added to the current tempo ratio; clamped to [0.5, 2.0].
/// Typical usage: ±0.01 per key-press for vinyl-style pitch-bend.
#[tauri::command]
pub fn deck_nudge_tempo(deck: u32, delta: f32, state: State<'_, AppState>) -> Result<(), String> {
    let guard = state.engine.lock().map_err(|e| e.to_string())?;
    let engine = guard.as_ref().ok_or("audio engine not available")?;
    engine.nudge_tempo(deck, delta);
    Ok(())
}

/// Compute and return waveform peak data for a loaded deck.
///
/// `bins` controls how many vertical bars the canvas will draw.
/// 800 is a good default for a full-width overview waveform.
/// Blocking — Tauri runs this on a thread pool, not the main thread.
#[tauri::command]
pub async fn deck_waveform(
    deck: u32,
    bins: u32,
    state: State<'_, AppState>,
) -> Result<WaveformData, String> {
    let guard = state.engine.lock().map_err(|e| e.to_string())?;
    let engine = guard.as_ref().ok_or("audio engine not available")?;
    let total_frames = engine.total_frames(deck);
    let peaks = engine
        .compute_waveform(deck, bins)
        .map_err(|e| e.to_string())?;

    // Try to get stem peaks too, but don't fail if they aren't loaded.
    let stem_peaks = if let Ok(sp) = engine.compute_stem_waveforms(deck, bins) {
        Some(vec![sp.vocals, sp.drums, sp.bass, sp.other])
    } else {
        None
    };

    Ok(WaveformData {
        num_bins: bins,
        peaks_min: peaks.peaks_min,
        peaks_max: peaks.peaks_max,
        stem_peaks,
        total_frames,
    })
}
