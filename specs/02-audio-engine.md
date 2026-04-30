# 02 — Audio Engine

## 1. Goals

- Round-trip latency < 10 ms at 44.1 kHz / 128-sample buffer on a typical Mac.
- Glitch-free playback while CPU runs stems and analysis in the background.
- Two decks playing 4 stems each = up to 8 parallel streams.
- Sample-accurate scrubbing, looping, and beat-locked transitions.
- Hot-swap of output device without crash.

## 2. Architecture

```
              ┌─────────────────────────────────────┐
              │         Mixer (sums all decks)      │
              └────────┬────────────────────┬───────┘
                       │                    │
        ┌──────────────▼─────┐    ┌─────────▼────────┐
        │ Deck A             │    │ Deck B           │
        │ ┌────┐┌────┐┌────┐┌────┐│ ┌────┐ ...        │
        │ │Voc ││Drm ││Bas ││Oth ││ │Voc │            │
        │ └────┘└────┘└────┘└────┘│ └────┘            │
        │  4× ring buffer fed     │  same              │
        │  by decoder thread      │                    │
        └─────────────────────────┘─────────────────┘
                       ▲
                       │ pre-decoded PCM frames
              ┌────────┴────────┐
              │ Decoder workers │  (libsndfile / FFmpeg)
              └────────┬────────┘
                       │
                  files / cache
```

The audio callback runs in CoreAudio's realtime thread. It only reads from
the SPSC ring buffers, applies fader/EQ/effect, sums into the master, and
writes to the device buffer.

## 3. Public engine API (FFI surface)

Defined in `native/audio/include/pdj_engine.h` and wrapped in
`pdj-engine-bridge`. Examples:

```c
PdjEngine*   pdj_engine_create(const PdjEngineConfig* cfg);
void         pdj_engine_destroy(PdjEngine*);
PdjResult    pdj_engine_load(PdjEngine*, uint32_t deck, const char* path);
PdjResult    pdj_engine_play(PdjEngine*, uint32_t deck);
PdjResult    pdj_engine_pause(PdjEngine*, uint32_t deck);
PdjResult    pdj_engine_seek(PdjEngine*, uint32_t deck, uint64_t frame);
void         pdj_engine_set_fader(PdjEngine*, uint32_t deck, float value);
void         pdj_engine_set_crossfader(PdjEngine*, float value);
void         pdj_engine_set_stem_gain(PdjEngine*, uint32_t deck, uint32_t stem, float g);
PdjStatus    pdj_engine_status(PdjEngine*);
```

All setters are realtime-safe (atomic store). All loaders run on a worker
thread and signal completion via a callback.

## 4. Buffer sizes and latency

Latency in milliseconds:

```
Δt = (buffer_size / sample_rate) × 1000
```

| Sample rate | Buffer | One-way latency |
|---|---|---|
| 44 100 Hz | 64  | ~1.5 ms |
| 44 100 Hz | 128 | ~2.9 ms |
| 44 100 Hz | 256 | ~5.8 ms |
| 48 000 Hz | 128 | ~2.7 ms |
| 96 000 Hz | 128 | ~1.3 ms |

Default ships with **44.1 kHz / 128 samples**. Configurable in
`config/defaults.toml`. The settings UI exposes a clear "low / balanced /
safe" preset on top of the raw values.

## 5. Decoders

| Format | Library | Notes |
|---|---|---|
| WAV / AIFF / FLAC | libsndfile | First class |
| MP3 | minimp3 (C, header-only) | Pre-decode whole file at import for cheap seek |
| AAC / M4A | minimp4 + minimal FFmpeg | Avoid full FFmpeg if possible |
| OGG / Opus | libsndfile (Vorbis) + opusfile | Optional |

Tracks are decoded **fully on import** when reasonable size (< 100 MB PCM),
otherwise streamed in 1-second chunks via a prefetch ring buffer.

## 6. Beat detection and beatgrid

- BPM detection via aubio's tempo algorithm (LGPL, dynamically linked).
- Manual override available; user can drag the beatgrid in the UI.
- Beatgrid stored as `(first_beat_frame: u64, bpm: f32)` plus optional
  per-region override list for tracks with tempo changes.
- Sync between decks always operates on the beatgrid, not raw time.

## 7. Pitch / tempo control

- Independent rate and key control via `RubberBand` (GPL is incompatible —
  use the BSD-licensed Soundtouch or the recently relicensed RubberBand v3
  if licensing permits; otherwise build a small in-house PSOLA + WSOLA).
- Vinyl-style "DJ-like" pitch slider with adjustable range (±8 % default,
  ±16 %, ±100 %).
- Keylock toggle per deck.

## 8. Realtime safety rules

Inside the audio callback:

- No allocation. Buffers are pre-sized at engine start.
- No mutex. Use atomics or lock-free SPSC ring buffers.
- No blocking syscall. No `printf`, no `std::cout`.
- Bounded loops only. Loop count known statically or by sample budget.
- All FP math finite — guard against NaN propagation in user effects.

A unit test (`engine_realtime_test`) compiles with sanitisers and asserts
no allocation occurs during a 1-second mix using `mtrace`-like hooks.

## 9. Device handling

- Default output uses CoreAudio's "default output device" with subscription
  to default-device-changed events.
- Hot-swap path: mute master → reconfigure → unmute, all within 200 ms.
- Headphones / cue output via second device or split-stereo on the same
  device (configurable).
- Auto-recover on device unplug: route to default device, post a UI toast.

## 10. Master output processing chain

```
[deck mix] → [crossfader] → [master gain] → [limiter] → [device]
```

The limiter is an inline brick-wall set to -0.3 dBFS true peak by default,
to protect against clipping when several stems collide on a Drop.

## 11. Files and length budget

| File | Approx. lines | Responsibility |
|---|---|---|
| `engine.cpp` | ≤ 400 | lifecycle, public C API |
| `mixer.cpp` | ≤ 400 | sum decks, master fx |
| `deck.cpp` | ≤ 400 | per-deck state, fader, EQ |
| `decoder.cpp` | ≤ 400 | format dispatch |
| `ringbuffer.hpp` | ≤ 200 | header-only SPSC buffer |
| `coreaudio_backend.cpp` | ≤ 400 | device callback |
| `portaudio_backend.cpp` | ≤ 400 | cross-platform alternative |

If any file approaches the limit, split by responsibility (e.g. `deck_eq.cpp`,
`deck_fx.cpp`).

## 12. Open questions

- Final pitch-shift library decision (`spec` ticket once Phase 1 starts).
- Whether to ship a custom DSP kernel for stems mixing or reuse mixer code.
- Hardware AU / VST host support after CLAP — likely never, to stay clean.
