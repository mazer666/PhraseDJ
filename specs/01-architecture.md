# 01 — Architecture

## 1. Layered overview

```
┌──────────────────────────────────────────────────────┐
│  UI Layer (React + TypeScript, Tauri-hosted)         │
│  decks · waveforms · macro editor · lyrics · settings│
└──────────────────────┬───────────────────────────────┘
                       │ Tauri IPC (typed commands + events)
┌──────────────────────▼───────────────────────────────┐
│  App Layer (Rust, Tauri backend)                     │
│  pdj-library · pdj-macros · pdj-lyrics · pdj-midi    │
│  pdj-stems · pdj-plugins · pdj-engine-bridge         │
└──┬──────────────┬───────────────┬───────────────┬────┘
   │ FFI          │ MLX / ONNX    │ FS / SQLite   │ HTTP (opt-in)
┌──▼────────────┐ ┌▼────────────┐ ┌▼────────────┐ ┌▼────────────┐
│ C++ Audio     │ │ AI Inference│ │ Local store │ │ Lyrics API  │
│ Engine        │ │ HTDemucs etc│ │ tracks db   │ │ LRCLib      │
│ CoreAudio /   │ │             │ │ macros json │ │             │
│ PortAudio     │ │             │ │ stems cache │ │             │
└───────────────┘ └─────────────┘ └─────────────┘ └─────────────┘
```

## 2. Technology choices

| Layer | Choice | Why |
|---|---|---|
| UI shell | Tauri 2 | Native window, small binary, no Chromium bundle, Rust-bridged |
| UI framework | React 18 + TypeScript + Vite | Fast iteration, strong AI-coding support |
| State | Zustand + TanStack Query | Minimal, no Redux ceremony |
| Charts/Waveform | `wgpu` via Tauri-side rendering OR canvas/WebGL | Metal acceleration on macOS |
| App logic | Rust, Cargo workspace | Memory safety, performance, FFI |
| Audio engine | C++20 | CoreAudio API, mature DSP libs |
| Audio I/O | CoreAudio (mac) + PortAudio (cross) | Lowest latency on each platform |
| Decoders | libsndfile + minimal FFmpeg | MP3, FLAC, WAV, AAC |
| Beat / key | aubio (eval) or in-house | LGPL — wrap behind FFI |
| AI runtime | Apple MLX (mac) + ONNX Runtime (cross) | Native performance + portability |
| Stem model | HTDemucs (4-stem) | Best quality/speed trade-off |
| Lyrics ASR | whisper.cpp | C++, embeddable, offline |
| DB | SQLite via `rusqlite` | Zero-conf, single file |
| Plugins | CLAP standard | License-friendly, modern |
| Scripting | QuickJS via `rquickjs` | Tiny, embeddable JS engine |
| Tests | `cargo test`, Vitest, GoogleTest | Per-language standards |

## 3. Cargo workspace layout

```
crates/
  pdj-core/              shared types, error enum, Result alias, settings loader
  pdj-library/           SQLite, metadata, scanning
  pdj-engine-bridge/    C-FFI to native audio engine; safe Rust facade
  pdj-stems/             stem separation orchestration
  pdj-macros/            transition recorder, replay, persistence
  pdj-lyrics/            tag/LRC parsing, online lookup, Whisper bridge
  pdj-midi/              MIDI input, mapping, learn mode
  pdj-plugins/           CLAP host, QuickJS, MCP server
apps/desktop/src-tauri/  Tauri commands, app wiring, window management
```

Every crate has a tight public surface in `src/lib.rs`. Internal modules
remain `pub(crate)` unless explicitly part of the API.

## 4. Process model

PhraseDJ runs as a single OS process with these threads:

| Thread | Owner | Realtime? | Purpose |
|---|---|---|---|
| main / UI | Tauri | no | event loop, window |
| audio callback | C++ engine | **yes** | mix decks, write samples to device |
| disk I/O | tokio runtime | no | decode, prefetch, library scan |
| analysis worker(s) | tokio + MLX | no | stems, beatgrid, phrase, key, ASR |
| network worker | tokio | no | opt-in lyrics lookup |
| MIDI listener | midir | low-latency | hardware events |

Inter-thread communication: lock-free SPSC ring buffers (audio ↔ control),
`tokio::sync::mpsc` (workers ↔ app), Tauri events (app → UI).

## 5. Data flow: track import

```
  drag&drop / scan
        │
        ▼
  pdj-library  ── insert track row (status = "raw")
        │
        ▼
  analysis queue ─► beatgrid ─► key/energy ─► phrase markers
        │                                      │
        ▼                                      ▼
  pdj-stems  ─► HTDemucs  ─► cache 4× WAV  ─► library row updated
        │
        ▼
  pdj-lyrics ─► tag → LRC → online → whisper alignment ─► .lrc cached
```

All steps emit progress events the UI subscribes to. The user can play
the track at any point — missing analysis just disables features
(e.g. stem mixer is greyed out until stems are cached).

## 6. Settings architecture

- `config/defaults.toml` — shipped, versioned, read-only at runtime
- `<app-support>/PhraseDJ/settings.toml` — user overrides, written on change
- `<app-support>/PhraseDJ/keymap.toml` — keyboard / MIDI mapping
- `pdj-core::config` — single loader, hot-reloads on file change in dev mode
- Schema validated via JSON Schema in `config/schema.json`

A settings-UI editor never edits TOML directly — it goes through
`pdj-core::config` and writes via the loader's atomic-rename helper.

## 7. Error handling

- Each crate defines its own `Error` enum with `thiserror`.
- `pdj-core::Result<T>` is the project-wide alias.
- Tauri commands return `Result<T, AppError>` mapped to a typed JSON error.
- The UI shows errors via a shared toast component; never raw stack traces.

## 8. Logging

- `tracing` crate with structured fields.
- Console default, rolling-file appender in app-support folder.
- Audio-callback uses a lock-free log queue drained by a non-RT thread.
- Default level: `info`; `debug` enabled by `--verbose` or settings flag.

## 9. Cross-platform readiness

Even though macOS is first, no module hardcodes Apple-specific paths or APIs
outside of:

- `native/audio/coreaudio_backend.cpp` (alternative: `portaudio_backend.cpp`)
- `pdj-stems/src/mlx.rs` (alternative: `pdj-stems/src/onnx.rs`)
- Settings paths (use `directories` crate)

CI builds at minimum the Linux target from Phase 2 onwards to catch drift.

## 10. Key architectural rules

1. UI never speaks to the audio engine directly — always via Tauri commands
   handled by Rust.
2. Audio engine never speaks to the database, network, or filesystem during
   playback. Pre-loaded buffers only.
3. AI inference and any decoding happen on worker threads and feed the engine
   via lock-free buffers.
4. No crate may exceed 600 lines per file (rule from `LLM.md`).
5. No module may import another module that's "below" it in the layered diagram.
