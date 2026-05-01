//! pdj-engine-bridge — Safe Rust facade over the C++ pdj_audio engine.
//!
//! The C++ engine is a shared library produced by `native/audio/CMakeLists.txt`.
//! This crate provides a memory-safe Rust API that:
//!
//!  - Owns the engine handle (`Drop` calls `pdj_engine_destroy`).
//!  - Validates indices and converts errors to typed `Error` values.
//!  - Hides all `unsafe` so that downstream crates never need to use it.
//!
//! Thread safety: `Engine` is `Send + Sync`.  All audio-control functions
//! are realtime-safe on the C++ side (atomics + lock-free buffers).

mod ffi;

use std::ffi::CString;
use std::path::Path;

use pdj_core::Error as CoreError;
use thiserror::Error;
use tracing::{debug, warn};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Errors raised by the bridge.
#[derive(Debug, Error)]
pub enum BridgeError {
    /// The engine could not be created (no audio device, bad config).
    #[error("failed to create audio engine")]
    CreateFailed,

    /// A function argument was invalid (e.g. deck index out of range).
    #[error("invalid argument")]
    InvalidArg,

    /// File loading or seeking failed.
    #[error("I/O error from engine")]
    Io,

    /// An unexpected internal error.
    #[error("engine internal error")]
    Internal,

    /// A path could not be converted to a C string (contains a NUL byte).
    #[error("invalid path: contains NUL byte")]
    BadPath,
}

impl From<BridgeError> for CoreError {
    fn from(e: BridgeError) -> Self {
        CoreError::other(e.to_string())
    }
}

/// Convenience alias.
pub type Result<T> = std::result::Result<T, BridgeError>;

/// Configuration for engine creation.
#[derive(Debug, Clone, Copy)]
pub struct EngineConfig {
    pub sample_rate:   u32,
    pub buffer_size:   u32,
    pub channel_count: u32,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self { sample_rate: 44_100, buffer_size: 128, channel_count: 2 }
    }
}

// ---------------------------------------------------------------------------
// Engine wrapper
// ---------------------------------------------------------------------------

/// Safe wrapper around the C++ audio engine.
pub struct Engine {
    handle: *mut ffi::PdjEngine,
}

// SAFETY: All C functions are thread-safe (atomics + internal mutex on C++ side).
unsafe impl Send for Engine {}
unsafe impl Sync for Engine {}

impl Engine {
    /// Create a new engine.  Opens the default audio output device.
    ///
    /// In headless environments (CI, sandbox) the engine is still created
    /// but `is_running()` returns false.  This lets tests run without a
    /// real audio device.
    pub fn new(config: EngineConfig) -> Result<Self> {
        let cfg = ffi::PdjEngineConfig {
            sample_rate:   config.sample_rate,
            buffer_size:   config.buffer_size,
            channel_count: config.channel_count,
        };
        // SAFETY: passing a valid pointer; engine handle returned by C++.
        let handle = unsafe { ffi::pdj_engine_create(&cfg) };
        if handle.is_null() {
            return Err(BridgeError::CreateFailed);
        }
        debug!(?config, "Audio engine created");
        Ok(Self { handle })
    }

    /// True if the audio output stream is running.
    pub fn is_running(&self) -> bool {
        unsafe { ffi::pdj_engine_is_running(self.handle) != 0 }
    }

    // ----- Track loading -------------------------------------------------

    /// Load an audio file onto a deck.
    pub fn load(&self, deck: u32, path: &Path) -> Result<()> {
        check_deck(deck)?;
        let path_str = path.to_string_lossy();
        let cpath = CString::new(path_str.as_ref()).map_err(|_| BridgeError::BadPath)?;
        // SAFETY: handle non-null; path is a valid CString for the call duration.
        let res = unsafe { ffi::pdj_engine_load(self.handle, deck, cpath.as_ptr()) };
        check_result(res)
    }

    // ----- Transport -----------------------------------------------------

    pub fn play(&self, deck: u32) -> Result<()> {
        check_deck(deck)?;
        check_result(unsafe { ffi::pdj_engine_play(self.handle, deck) })
    }

    pub fn pause(&self, deck: u32) -> Result<()> {
        check_deck(deck)?;
        check_result(unsafe { ffi::pdj_engine_pause(self.handle, deck) })
    }

    pub fn seek(&self, deck: u32, position_frames: u64) -> Result<()> {
        check_deck(deck)?;
        check_result(unsafe { ffi::pdj_engine_seek(self.handle, deck, position_frames) })
    }

    pub fn position(&self, deck: u32) -> u64 {
        if check_deck(deck).is_err() { return 0; }
        unsafe { ffi::pdj_engine_position(self.handle, deck) }
    }

    // ----- Mixer ---------------------------------------------------------

    pub fn set_fader(&self, deck: u32, value: f32) {
        if check_deck(deck).is_err() { return; }
        unsafe { ffi::pdj_engine_set_fader(self.handle, deck, value.clamp(0.0, 1.0)) }
    }

    pub fn set_crossfader(&self, value: f32) {
        unsafe { ffi::pdj_engine_set_crossfader(self.handle, value.clamp(0.0, 1.0)) }
    }

    pub fn set_master_gain(&self, value: f32) {
        unsafe { ffi::pdj_engine_set_master_gain(self.handle, value.clamp(0.0, 1.5)) }
    }

    pub fn set_stem_gain(&self, deck: u32, stem: u32, value: f32) {
        if check_deck(deck).is_err() { return; }
        if stem >= 4 { return; }
        unsafe { ffi::pdj_engine_set_stem_gain(self.handle, deck, stem, value.clamp(0.0, 1.5)) }
    }

    // ----- Status --------------------------------------------------------

    pub fn is_playing(&self, deck: u32) -> bool {
        if check_deck(deck).is_err() { return false; }
        unsafe { ffi::pdj_engine_is_playing(self.handle, deck) != 0 }
    }

    pub fn is_loaded(&self, deck: u32) -> bool {
        if check_deck(deck).is_err() { return false; }
        unsafe { ffi::pdj_engine_is_loaded(self.handle, deck) != 0 }
    }

    // ----- BPM -----------------------------------------------------------

    /// Analyse PCM samples for BPM.  Blocks — call from a worker thread.
    pub fn analyse_bpm(&self, deck: u32, samples: &[f32], frame_count: u64) -> Result<()> {
        check_deck(deck)?;
        if samples.len() < (frame_count.saturating_mul(2)) as usize {
            return Err(BridgeError::InvalidArg);
        }
        let res = unsafe {
            ffi::pdj_engine_analyse_bpm(self.handle, deck, samples.as_ptr(), frame_count)
        };
        check_result(res)
    }

    pub fn get_bpm(&self, deck: u32) -> f32 {
        if check_deck(deck).is_err() { return 0.0; }
        unsafe { ffi::pdj_engine_get_bpm(self.handle, deck) }
    }

    // ----- Tempo ratio ---------------------------------------------------

    /// Set the vinyl-style playback speed for a deck.
    ///
    /// `ratio` is clamped to [0.5, 2.0] in the C++ layer.
    /// 1.0 = normal speed, 1.05 = +5 %, 0.95 = −5 %.
    pub fn set_tempo_ratio(&self, deck: u32, ratio: f32) {
        if check_deck(deck).is_err() { return; }
        unsafe { ffi::pdj_engine_set_tempo_ratio(self.handle, deck, ratio) }
    }

    /// Return the current tempo ratio for a deck (1.0 if unset).
    pub fn get_tempo_ratio(&self, deck: u32) -> f32 {
        if check_deck(deck).is_err() { return 1.0; }
        unsafe { ffi::pdj_engine_get_tempo_ratio(self.handle, deck) }
    }

    /// Nudge the playback speed by a small delta (positive = faster).
    ///
    /// The new ratio is clamped to [0.5, 2.0].
    /// Typical delta: ±0.01 (1 %) per key-press.
    pub fn nudge_tempo(&self, deck: u32, delta: f32) {
        if check_deck(deck).is_err() { return; }
        let current = self.get_tempo_ratio(deck);
        let next = (current + delta).clamp(0.5, 2.0);
        self.set_tempo_ratio(deck, next);
    }

    /// Synchronise `deck`'s tempo to the opposite deck's BPM.
    ///
    /// Reads both BPMs, computes the required ratio, and applies it.
    /// Returns `Err` if either deck has no BPM data yet.
    pub fn sync_tempo(&self, deck: u32) -> Result<()> {
        check_deck(deck)?;
        let other = 1 - deck;
        let this_bpm  = self.get_bpm(deck);
        let other_bpm = self.get_bpm(other);
        if this_bpm <= 0.0 || other_bpm <= 0.0 {
            warn!(deck, this_bpm, other_bpm, "sync_tempo: BPM not available");
            return Err(BridgeError::Internal);
        }
        let ratio = other_bpm / this_bpm;
        debug!(deck, ratio, "sync_tempo: applying ratio");
        self.set_tempo_ratio(deck, ratio);
        Ok(())
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        // SAFETY: pdj_engine_destroy(NULL) is documented as safe.
        unsafe { ffi::pdj_engine_destroy(self.handle) }
        self.handle = std::ptr::null_mut();
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const MAX_DECK_INDEX: u32 = 1;

fn check_deck(deck: u32) -> Result<()> {
    if deck > MAX_DECK_INDEX {
        warn!(deck, "deck index out of range");
        return Err(BridgeError::InvalidArg);
    }
    Ok(())
}

fn check_result(r: ffi::PdjResult) -> Result<()> {
    match r {
        ffi::PdjResult::Ok => Ok(()),
        ffi::PdjResult::InvalidArg => Err(BridgeError::InvalidArg),
        ffi::PdjResult::Io => Err(BridgeError::Io),
        ffi::PdjResult::NotReady | ffi::PdjResult::Internal => Err(BridgeError::Internal),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_deck_accepts_valid_indices() {
        assert!(check_deck(0).is_ok());
        assert!(check_deck(1).is_ok());
        assert!(check_deck(2).is_err());
        assert!(check_deck(99).is_err());
    }
}
