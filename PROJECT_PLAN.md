# PhraseDJ Project Plan

> Realistic plan for a **hobby project with roughly 5 hours of development
> time per week**, AI-assisted. Expected duration to a live-capable MVP:
> **18–24 months**.

---

## Guardrails

- **Pace:** ~5 h/week ≈ 20 h/month
- **Method:** vibe coding with Claude Code as the main agent, Gemini for
  second opinions
- **Mode:** incremental, in phases each producing visible output
- **Definition of Done per phase:** runnable, tested, documented, merged to main

## Success metrics

| Metric | Target |
|---|---|
| Audio latency (round-trip) | < 10 ms at 44.1 kHz / 128 samples |
| Crash rate | 0 crashes in a 4 h live set (Phase 4) |
| Stem separation, 7-min track | < 20 s on M2 |
| App start time | < 2 s until ready to play |
| Test coverage | ≥ 70 % on core modules |
| File-length violations | 0 |

---

## Phase 0 — Foundation (Month 1)

**Goal:** Repository skeleton in place, build pipeline running, "Hello PhraseDJ"
shows up as an empty Tauri window.

- [x] Specs and plans (this state)
- [x] Repo layout (`apps/`, `crates/`, `native/`, `ml/`, `config/`)
- [x] Tauri 2 skeleton with React/TS/Vite
- [x] Cargo workspace with a `pdj-core` stub
- [x] CMake skeleton for `native/audio`
- [x] CI on GitHub Actions: `cargo test`, `pnpm test`, `ctest`, lint
- [x] `config/defaults.toml` + loader in `pdj-core`
- [x] CONTRIBUTING.md, ISSUE/PR templates

**Outcome:** an empty window that launches on macOS.

---

## Phase 1 — Audio core + 2 decks (Months 2–8)

**Goal:** play two tracks simultaneously, crossfader and volume work, keyboard
control runs.

- [x] CoreAudio integration via a PortAudio wrapper in C++
- [x] FFI bridge `pdj-engine-bridge` (Rust ↔ C++)
- [x] Decoders for MP3, FLAC, WAV, AAC (libsndfile + lightweight FFmpeg)
- [x] Lock-free ring buffer for the audio callback
- [x] Two deck instances (Play/Pause/Cue/Seek)
- [x] Crossfader, channel faders, master output
- [x] Waveform renderer (canvas-based overview; `wgpu` upgrade planned Phase 2)
- [x] Beatgrid detection (BPM via spectral auto-correlation)
- [x] Tempo sync between decks (vinyl-style pitch ratio + nudge)
- [x] Keyboard mapping driven by `keymap.toml` (with user-override merging)
- [x] Local library: SQLite schema, drag & drop import, folder scan
- [x] Settings UI (with reset-to-default)
- [x] Latency benchmark as a CI check

**Milestone 1 (M8):** "I can load two tracks from my local library, play
them in sync via keyboard, and mix them with the crossfader."

---

## Phase 2 — AI stems (Months 9–13)

**Goal:** four stems per deck, controllable via sliders instead of EQ in stem
mode. Background analysis on library import.

- [x] `pdj-stems` crate with an async job queue
- [x] `StemBackend` trait + `MlxBackend` stub (Apple Silicon detection real; inference stubbed)
- [x] `OnnxBackend` stub (cross-platform fallback, always available)
- [x] Port HTDemucs to MLX (4-stem) — real inference (mlx_rs integrated)
- [x] ONNX Runtime fallback (Linux/Windows) — real inference (ort session wired)
- [x] Background analysis on import; cache stems in app-support folder (queue + LRU path helpers)
- [x] Overlap-add stitching + WAV/FLAC writing (`stitch` module)
- [x] Cache path layout (`paths` module; `StemPaths`)
- [x] Stem player in the audio engine (4 parallel streams per deck)
- [x] Stem mixer UI (4 faders per deck instead of 3-band EQ)
- [x] Stem waveform (multi-colour: vocals/drums/bass/other)
- [x] Memory-budget guard (cap parallel stems)

**Milestone 2 (M13):** "During a mix I can isolate the vocals from deck A and
the drums from deck B and layer them."

---

## Phase 3 — Macros + transition recorder (Months 14–17)

**Goal:** manual mixes are recorded, saved, editable, and recallable next time.

- [ ] `pdj-macros` crate: event recorder for all control events
- [ ] Persistence as JSON next to track metadata
- [ ] Replay engine anchored on the beatgrid (not absolute time)
- [ ] Macro editor UI (timeline with curves, DAW-automation style)
- [ ] "Apply macro" button with half-auto and full-auto modes
- [ ] Phrase-detection model (intro / verse / chorus / drop / outro)
- [ ] Transition suggestions: "start transition at chorus 2 of deck A"
- [ ] Library view "transitions involving this track"

**Milestone 3 (M17):** "I mix two tracks by hand, save the transition, and
next time PhraseDJ replays it on a single button press."

---

## Phase 4 — Lyrics + MIDI + live hardening (Months 18–22)

**Goal:** karaoke-grade lyrics, external hardware works, bar-ready stability.

- [ ] LRC parser + tag-based lyrics loader
- [ ] **Online lyrics lookup** via LRCLib (or comparable open service),
      opt-in toggle, transparent network log
- [ ] `whisper.cpp` via FFI for forced alignment (offline default)
- [ ] Lyrics overlay with progress mask (Apple-Music style)
- [ ] MIDI input layer + learn wizard
- [ ] Mouse gestures (vertical swipe = filter, horizontal = scrub)
- [ ] Auto-save of mix state every 10 s
- [ ] Crash recovery on relaunch
- [ ] Panic stop (immediate clean audio kill)
- [ ] Output-device hot-swap without crash
- [ ] 4-hour stress test in CI (synthetic run)

**Milestone 4 (M22):** "I can play live with a USB controller, an external
audio interface and two hours of material, without fear of crash or data loss."

---

## Phase 5 — Plugins + polish (Months 23–24, then cross-platform)

**Goal:** CLAP plugins run, JS scripting open, MCP bridge documented. Linux
build attempted.

- [ ] CLAP host integration
- [ ] Built-in plugins: 3-band EQ, filter, echo, reverb, beat roll
- [ ] QuickJS scripting API
- [ ] MCP server for PhraseDJ (library inspect, macro generation)
- [ ] Linux port via PortAudio + ONNX
- [ ] Windows evaluation
- [ ] Beta program

**Milestone 5 (M24+):** Public Beta v0.9.

---

## Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Audio latency target unreachable | medium | high | Early benchmarks in Phase 1, configurable buffer size |
| MLX model port fails | medium | medium | ONNX fallback planned from day 1 |
| File complexity explodes | high | medium | 600-line rule, monthly refactor slot |
| Motivation fades | high | high | Small wins every 4 weeks, visible UI early |
| License conflict from a third-party lib | low | high | SBOM check in CI, MPL-2.0 compatibility audit |
| Live crash at first gig | medium | high | Phase-4 hardening, stress test, panic stop |
| Online lyrics service unavailable | medium | low | Offline Whisper alignment is the always-working path |

---

## Cadence

- **Weekly:** one focused coding block (3 h) + one review/plan block (2 h)
- **Monthly:** retro, phase progress, risk-list refresh
- **Per phase:** record a short demo video — both motivation and documentation
