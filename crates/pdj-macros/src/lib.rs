//! pdj-macros — Transition recording, editing, and automated replay.
//!
//! This crate implements the macro engine that allows recording user
//! control events (faders, EQs, stems) and replaying them beat-locked
//! to the audio engine.

use pdj_core::{Result, TrackId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Macro {
    pub id: String,
    pub name: String,
    pub tracks: [TrackId; 2],
    pub events: Vec<MacroEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroEvent {
    /// Time in bars from the anchor point.
    pub t_bar: f32,
    /// Semantic channel name (e.g. "deck.A.fader").
    pub channel: String,
    /// Value (usually 0.0 to 1.0).
    pub value: f32,
}

/// Start recording a new macro for the given track pair.
pub fn record_start(_tracks: [TrackId; 2]) -> Result<String> {
    // Phase 2: Implement event capturing from the engine bridge.
    Ok("new-recorder-id".to_string())
}

/// Stop recording and save the macro.
pub fn record_stop(_recorder_id: &str, _name: &str) -> Result<Macro> {
    Err(pdj_core::Error::other("Not implemented"))
}

pub mod recorder;
pub mod replay;
pub mod store;
