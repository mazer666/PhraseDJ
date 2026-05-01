//! ffi.rs — Raw C bindings to libpdj_audio.
//!
//! These declarations mirror `native/audio/include/pdj_engine.h` exactly.
//! They are unsafe by definition and should only be used from `lib.rs`,
//! which wraps them in a safe Rust facade.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::os::raw::{c_char, c_float, c_int, c_void};

/// Opaque engine handle — the actual struct lives in the C++ side.
#[repr(C)]
pub struct PdjEngine {
    _private: [u8; 0],
}

/// Engine configuration passed to `pdj_engine_create`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PdjEngineConfig {
    pub sample_rate:   u32,
    pub buffer_size:   u32,
    pub channel_count: u32,
}

/// Result codes returned by fallible engine functions.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // discriminants are read across FFI; suppress lint
pub enum PdjResult {
    Ok          = 0,
    InvalidArg  = 1,
    NotReady    = 2,
    Io          = 3,
    Internal    = 4,
}

extern "C" {
    // Lifecycle
    pub fn pdj_engine_create(config: *const PdjEngineConfig) -> *mut PdjEngine;
    pub fn pdj_engine_destroy(engine: *mut PdjEngine);

    // Track loading
    pub fn pdj_engine_load(
        engine: *mut PdjEngine,
        deck_index: u32,
        file_path: *const c_char,
    ) -> PdjResult;

    pub fn pdj_engine_load_stems(
        engine: *mut PdjEngine,
        deck_index: u32,
        path_main: *const c_char,
        path_v: *const c_char,
        path_d: *const c_char,
        path_b: *const c_char,
        path_o: *const c_char,
    ) -> PdjResult;

    // Transport
    pub fn pdj_engine_play(engine: *mut PdjEngine, deck_index: u32) -> PdjResult;
    pub fn pdj_engine_pause(engine: *mut PdjEngine, deck_index: u32) -> PdjResult;
    pub fn pdj_engine_seek(
        engine: *mut PdjEngine,
        deck_index: u32,
        position_frames: u64,
    ) -> PdjResult;
    pub fn pdj_engine_position(engine: *mut PdjEngine, deck_index: u32) -> u64;

    // Mixer
    pub fn pdj_engine_set_fader(engine: *mut PdjEngine, deck_index: u32, value: c_float);
    pub fn pdj_engine_set_crossfader(engine: *mut PdjEngine, value: c_float);
    pub fn pdj_engine_set_master_gain(engine: *mut PdjEngine, value: c_float);
    pub fn pdj_engine_set_stem_gain(
        engine: *mut PdjEngine,
        deck_index: u32,
        stem_index: u32,
        value: c_float,
    );

    // Status queries
    pub fn pdj_engine_is_playing(engine: *mut PdjEngine, deck_index: u32) -> c_int;
    pub fn pdj_engine_is_loaded(engine: *mut PdjEngine, deck_index: u32) -> c_int;
    pub fn pdj_engine_is_running(engine: *mut PdjEngine) -> c_int;

    // BPM analysis
    pub fn pdj_engine_analyse_bpm(
        engine: *mut PdjEngine,
        deck_index: u32,
        samples: *const c_float,
        frame_count: u64,
    ) -> PdjResult;
    pub fn pdj_engine_get_bpm(engine: *mut PdjEngine, deck_index: u32) -> c_float;

    // Tempo control
    pub fn pdj_engine_set_tempo_ratio(
        engine: *mut PdjEngine,
        deck_index: u32,
        ratio: c_float,
    );
    pub fn pdj_engine_get_tempo_ratio(
        engine: *mut PdjEngine,
        deck_index: u32,
    ) -> c_float;

    // Waveform analysis
    pub fn pdj_engine_compute_waveform(
        engine: *mut PdjEngine,
        deck_index: u32,
        num_bins: u32,
        out_min: *mut c_float,
        out_max: *mut c_float,
    ) -> PdjResult;
    pub fn pdj_engine_compute_stem_waveforms(
        engine: *mut PdjEngine,
        deck_index: u32,
        num_bins: u32,
        out_v: *mut c_float,
        out_d: *mut c_float,
        out_b: *mut c_float,
        out_o: *mut c_float,
    ) -> PdjResult;
    pub fn pdj_engine_total_frames(engine: *mut PdjEngine, deck_index: u32) -> u64;
}

// Make `*mut PdjEngine` Send + Sync so we can hold it inside Arc<Mutex<>>.
// All C functions are documented as thread-safe at the engine level
// (atomics + internal mutex for non-RT control), so this is sound.
unsafe impl Send for PdjEngine {}
unsafe impl Sync for PdjEngine {}

// Silence "unused" warnings if c_void is referenced indirectly.
const _: Option<*mut c_void> = None;
