//! commands/deck.rs — Deck control commands.
//!
//! All commands return a `Result<_, String>` so the React frontend can
//! display errors as toast messages.  Internal errors are stringified
//! before crossing the IPC boundary.

use std::path::PathBuf;

use serde::Serialize;
use tauri::State;

use crate::state::AppState;

/// One-shot snapshot of a deck's state.  Sent on every UI poll.
#[derive(Debug, Serialize)]
pub struct DeckState {
    pub deck:         u32,
    pub loaded:       bool,
    pub playing:      bool,
    pub position:     u64,
    pub bpm:          f32,
    /// Current playback speed ratio (1.0 = normal, 1.05 = +5 %).
    pub tempo_ratio:  f32,
}

/// Load a file onto a deck (`deck` = 0 or 1).
#[tauri::command]
pub fn deck_load(deck: u32, path: String, state: State<'_, AppState>)
    -> Result<(), String>
{
    let p = PathBuf::from(&path);
    let guard = state.engine.lock().map_err(|e| e.to_string())?;
    let engine = guard.as_ref().ok_or("audio engine not available")?;
    engine.load(deck, &p).map_err(|e| e.to_string())
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
pub fn deck_seek(deck: u32, position: u64, state: State<'_, AppState>)
    -> Result<(), String>
{
    let guard = state.engine.lock().map_err(|e| e.to_string())?;
    let engine = guard.as_ref().ok_or("audio engine not available")?;
    engine.seek(deck, position).map_err(|e| e.to_string())
}

/// Return current state of a deck (called by the UI poll loop).
#[tauri::command]
pub fn deck_state(deck: u32, state: State<'_, AppState>)
    -> Result<DeckState, String>
{
    let guard = state.engine.lock().map_err(|e| e.to_string())?;
    let engine = guard.as_ref().ok_or("audio engine not available")?;
    Ok(DeckState {
        deck,
        loaded:      engine.is_loaded(deck),
        playing:     engine.is_playing(deck),
        position:    engine.position(deck),
        bpm:         engine.get_bpm(deck),
        tempo_ratio: engine.get_tempo_ratio(deck),
    })
}

/// Set the playback speed ratio for a deck (1.0 = normal, clamped to 0.5–2.0).
#[tauri::command]
pub fn deck_set_tempo_ratio(deck: u32, ratio: f32,
                             state: State<'_, AppState>) -> Result<(), String>
{
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
pub fn deck_nudge_tempo(deck: u32, delta: f32,
                         state: State<'_, AppState>) -> Result<(), String>
{
    let guard = state.engine.lock().map_err(|e| e.to_string())?;
    let engine = guard.as_ref().ok_or("audio engine not available")?;
    engine.nudge_tempo(deck, delta);
    Ok(())
}
