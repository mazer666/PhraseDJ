//! commands/mixer.rs — Mixer (fader, crossfader, master, stems) commands.

use tauri::State;

use crate::state::AppState;

/// Set the per-deck channel fader (0.0 – 1.0).
#[tauri::command]
pub fn mixer_set_fader(deck: u32, value: f32, state: State<'_, AppState>)
    -> Result<(), String>
{
    let guard = state.engine.lock().map_err(|e| e.to_string())?;
    let engine = guard.as_ref().ok_or("audio engine not available")?;
    engine.set_fader(deck, value);
    Ok(())
}

/// Set the crossfader position (0.0 = Deck A, 1.0 = Deck B).
#[tauri::command]
pub fn mixer_set_crossfader(value: f32, state: State<'_, AppState>)
    -> Result<(), String>
{
    let guard = state.engine.lock().map_err(|e| e.to_string())?;
    let engine = guard.as_ref().ok_or("audio engine not available")?;
    engine.set_crossfader(value);
    Ok(())
}

/// Set the master output gain (0.0 – 1.5).
#[tauri::command]
pub fn mixer_set_master_gain(value: f32, state: State<'_, AppState>)
    -> Result<(), String>
{
    let guard = state.engine.lock().map_err(|e| e.to_string())?;
    let engine = guard.as_ref().ok_or("audio engine not available")?;
    engine.set_master_gain(value);
    Ok(())
}

/// Set the gain for a specific stem (0=vocals 1=drums 2=bass 3=other).
#[tauri::command]
pub fn mixer_set_stem_gain(deck: u32, stem: u32, value: f32,
                            state: State<'_, AppState>) -> Result<(), String>
{
    let guard = state.engine.lock().map_err(|e| e.to_string())?;
    let engine = guard.as_ref().ok_or("audio engine not available")?;
    engine.set_stem_gain(deck, stem, value);
    Ok(())
}
