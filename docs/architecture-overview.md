# Architecture Overview

**Audience:** Contributors and maintainers  
**Owner:** Core maintainers  
**Last reviewed:** 2026-05-02

## Purpose
Explain how PhraseDJ components interact so contributors can make changes safely.

## System map

```text
React UI (apps/desktop/src)
   ↓ invokes
Tauri commands (apps/desktop/src-tauri/src/commands)
   ↓ mutate/read
Rust app state (apps/desktop/src-tauri/src/state.rs)
   ↓ bridges to
Native audio engine (native/audio/src/*.cpp)
```

## Main components

### 1) Desktop UI (TypeScript + React)
- Presents decks, waveform/crossfader, settings, status.
- Maintains UI-local state and issues API/command calls.

### 2) Tauri host (Rust)
- Registers command handlers in `src-tauri/src/commands`.
- Coordinates state and app orchestration.
- Bridges frontend intents to native audio/backend logic.

### 3) Native audio engine (C++)
- Handles decode, mix, BPM utilities, and low-level backend integration.
- Built/tested through CMake targets (`make test-cpp`).

### 4) Shared configuration
- Defaults and keymaps in `config/defaults.toml` and `config/keymap.toml`.
- Schema in `config/schema.json`.

## Boundaries and invariants
- UI should not embed hardcoded runtime constants when config exists.
- Command interfaces are the contract boundary between UI and backend.
- Native audio behavior changes require C++ tests and integration sanity checks.

## Where to go deeper
- `specs/01-architecture.md`
- `specs/02-audio-engine.md`
- `specs/04-ui-ux.md`
