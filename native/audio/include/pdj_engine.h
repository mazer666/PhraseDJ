/**
 * pdj_engine.h — Public C ABI for the PhraseDJ audio engine.
 *
 * This header is the contract between the C++ audio engine and the Rust
 * pdj-engine-bridge crate.  Every function uses C linkage so that Rust's
 * FFI can bind it without a C++ name-mangling layer.
 *
 * Rules:
 *   - No C++ types in this header (no std::, no templates, no references).
 *   - All returned pointers are owned by the caller unless stated otherwise.
 *   - Functions that can fail return PdjResult; PdjResult_Ok means success.
 */

#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* -------------------------------------------------------------------------
   Result codes
------------------------------------------------------------------------- */

/** Numeric result codes returned by fallible engine functions. */
typedef enum {
    PdjResult_Ok          = 0, /**< Success. */
    PdjResult_InvalidArg  = 1, /**< A function argument was out of range. */
    PdjResult_NotReady    = 2, /**< Engine not yet initialised. */
    PdjResult_Io          = 3, /**< File or device I/O error. */
    PdjResult_Internal    = 4, /**< Unexpected internal error. */
} PdjResult;

/* -------------------------------------------------------------------------
   Engine configuration
------------------------------------------------------------------------- */

/** Configuration passed to pdj_engine_create. */
typedef struct {
    uint32_t sample_rate;  /**< Output sample rate in Hz (e.g. 44100). */
    uint32_t buffer_size;  /**< Frames per audio callback buffer (e.g. 128). */
    uint32_t channel_count;/**< Output channel count (2 = stereo). */
} PdjEngineConfig;

/* -------------------------------------------------------------------------
   Engine lifecycle
------------------------------------------------------------------------- */

/** Opaque handle to the audio engine.  Caller owns and must call destroy. */
typedef struct PdjEngine PdjEngine;

/**
 * Create and initialise a new audio engine instance.
 *
 * Returns NULL if initialisation fails (e.g. no audio device available).
 * The caller is responsible for calling pdj_engine_destroy.
 */
PdjEngine* pdj_engine_create(const PdjEngineConfig* config);

/** Release all resources held by the engine.  The pointer must not be used
    after this call. */
void pdj_engine_destroy(PdjEngine* engine);

/* -------------------------------------------------------------------------
   Deck control  (deck_index: 0 = Deck A, 1 = Deck B)
------------------------------------------------------------------------- */

/**
 * Load an audio file onto a deck.
 *
 * Returns immediately; decoding runs on a background thread.
 * The deck emits a "ready" event via the registered callback when done.
 */
PdjResult pdj_engine_load(PdjEngine* engine,
                           uint32_t   deck_index,
                           const char* file_path);

/** Start playback on a deck. */
PdjResult pdj_engine_play(PdjEngine* engine, uint32_t deck_index);

/** Pause playback.  Position is retained. */
PdjResult pdj_engine_pause(PdjEngine* engine, uint32_t deck_index);

/**
 * Seek to an absolute position in frames.
 *
 * Sample-accurate.  No audible click if called while paused; a small
 * smoothing ramp is applied if called during playback.
 */
PdjResult pdj_engine_seek(PdjEngine* engine,
                           uint32_t   deck_index,
                           uint64_t   position_frames);

/** Return the current playback position in frames. */
uint64_t pdj_engine_position(PdjEngine* engine, uint32_t deck_index);

/* -------------------------------------------------------------------------
   Mixer controls  (all values normalised 0.0–1.0)
------------------------------------------------------------------------- */

/** Set the channel fader for a deck (0.0 = mute, 1.0 = unity). */
void pdj_engine_set_fader(PdjEngine* engine, uint32_t deck_index, float value);

/** Set the crossfader position (0.0 = Deck A, 1.0 = Deck B). */
void pdj_engine_set_crossfader(PdjEngine* engine, float value);

/** Set the gain for an individual stem on a deck.
    stem_index: 0=vocals 1=drums 2=bass 3=other */
void pdj_engine_set_stem_gain(PdjEngine* engine,
                               uint32_t   deck_index,
                               uint32_t   stem_index,
                               float      value);

/* -------------------------------------------------------------------------
   Status query
------------------------------------------------------------------------- */

/** Returns 1 if the deck is currently playing, 0 otherwise. */
int pdj_engine_is_playing(PdjEngine* engine, uint32_t deck_index);

#ifdef __cplusplus
} /* extern "C" */
#endif
