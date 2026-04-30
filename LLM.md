# LLM.md — Guide for AI Agents (Vibe Coding)

This document is the **binding work instruction** for **Claude Code, Gemini Code
Assist** and similar AI agents that produce or modify code in this repository.
Pull requests violating these rules will be rejected.

Humans are welcome to read it as well, to understand how the project is being
developed AI-assisted.

---

## 1. Project in 60 seconds

PhraseDJ is a native, local, open-source DJ application for macOS (later also
Linux/Windows). It uses local AI for stem separation, phrase analysis and
lyrics synchronization. Stack: Tauri 2 + Rust + C++ audio engine + React UI.

Before any non-trivial change, read at least:
- `specs/00-overview.md` — vision, scope, non-goals
- `specs/01-architecture.md` — module boundaries
- the spec for the domain you are touching

## 2. Golden rules

### Rule 1 — File length
**No source file may exceed 600 logical lines. Target: 400.**
If a file grows: split it. One file = one responsibility. Comments, blank
lines and imports do not count toward the 600-line limit; actual code does.
Test files may be longer but should still stay modular.

### Rule 2 — Beginner readability
The maintainer is a **programming beginner**. Code must be written so that a
person with basic programming knowledge can read it:

- Speaking identifiers (`crossfaderPosition`, not `xfP`)
- Short functions (≤ 40 lines as a rule of thumb)
- **Doc comments on every public function / struct / module**
- Inline comments wherever the logic isn't obvious
- Complex algorithms get a block comment explaining *what, why, source*
- Prefer clear multi-line code over clever one-liners

Example (Rust):

```rust
/// Smoothly moves the crossfader from its current value toward `target`.
///
/// `duration_ms` controls the transition time. A value of 0 sets it
/// immediately. Used internally by macro replay so transitions sound
/// human-mixed instead of robotic.
pub fn ease_crossfader(target: f32, duration_ms: u32) -> Result<()> {
    // Cosine easing — linear fader moves are audibly abrupt
    // (see specs/05-transitions-macros.md).
    ...
}
```

### Rule 3 — Externalise settings
**No magic numbers in code.** All configuration values (paths, default BPM,
audio buffer size, model names, hotkeys, colours, …) belong in:

- `config/defaults.toml` — shipped defaults (versioned in repo)
- `~/Library/Application Support/PhraseDJ/settings.toml` — user overrides
- `~/Library/Application Support/PhraseDJ/keymap.toml` — keyboard / MIDI mapping

Code reads settings through a single `config` module. Hard-coding a value is
a spec violation.

### Rule 4 — Tests are mandatory
Every new module or public function gets at least **one unit test**. Audio
code additionally gets integration tests against reference samples.

AI agents are excellent at writing tests — use that. Production code and test
code grow together, in the same PR.

### Rule 5 — Local-first, with one explicit exception
PhraseDJ is **local-first**. No module may reach the network without an
explicit, opt-in user action. No telemetry, ever.

**Exception — lyrics:** lyrics may be fetched from public open lyrics
databases (e.g. LRCLib) as a fallback when local recognition isn't available
or didn't produce a good alignment. This network access:

- is **opt-in** (toggle in settings, default on but visible)
- happens **only on explicit track import / lyrics request**
- sends **only metadata** (title, artist, duration) — never the audio file
- is **clearly logged** in a network-activity panel for transparency
- has a hard fallback: local Whisper alignment must always work offline

Update checks and plugin-marketplace browsing (later phases) follow the same
rules: opt-in, transparent, no silent traffic.

### Rule 6 — Realtime-audio discipline
Inside the audio callback (Rust or C++):
- **No allocations** (no `malloc`, `Vec::push`, `Box::new`)
- **No locks** (use lock-free structures or atomics)
- **No I/O** (no files, no logging, no sleeping)
- **No unbounded loops**

If you work in the audio path and are unsure: **ask, re-read
`02-audio-engine.md` and `10-performance.md`, write a latency-measuring test.**

## 3. Directory layout and module boundaries

```
PhraseDJ/
├── apps/desktop/             ← Tauri shell + React UI
│   ├── src/                  ← frontend (TS/React)
│   └── src-tauri/            ← Rust backend of the app
├── crates/
│   ├── pdj-core/             ← shared types, errors, settings loader
│   ├── pdj-library/          ← SQLite, metadata, import/scan
│   ├── pdj-engine-bridge/   ← FFI to the C++ audio engine
│   ├── pdj-stems/            ← MLX/ONNX wrapper for Demucs
│   ├── pdj-macros/           ← transition recorder, replay, persistence
│   ├── pdj-lyrics/           ← LRC parser, Whisper bridge, online lookup
│   ├── pdj-midi/             ← MIDI input, learn mode, mapping
│   └── pdj-plugins/          ← CLAP host, JS scripting, MCP bridge
├── native/audio/             ← C++ audio engine
├── ml/                       ← model definitions and conversion scripts
├── config/                   ← defaults.toml, schema.json
├── plugins/                  ← built-in CLAP plugins
├── specs/                    ← binding specs
└── tests/                    ← cross-module integration tests
```

**A module boundary is sacred.** If `pdj-lyrics` suddenly wants to manipulate
audio buffers, it's an architectural violation. Talk to the maintainer before
weakening boundaries.

## 4. Per-language style cheatsheet

### Rust
- Edition 2021, `clippy::pedantic` as default lint set
- `cargo fmt` must pass
- Errors typed via `thiserror`; `anyhow` only in binaries
- Async where I/O-bound (`tokio`), sync where CPU-bound

### TypeScript / React
- Strict mode on, no `any`
- Functional components, hooks, no class state
- State: Zustand or TanStack Query, no Redux
- ESLint + Prettier with the standard config

### C++ (audio engine)
- C++20, `-Wall -Wextra -Wpedantic -Werror`
- `clang-tidy` with the modernize set
- No exceptions inside the audio callback
- RAII everywhere, no `new`/`delete` outside smart pointers

## 5. Workflow for AI agents

1. **Read** the relevant spec chapter and existing tests.
2. **Plan** in a short TODO list what you will change.
3. **Implement** one module at a time, 100–200 lines per iteration.
4. **Test** — `cargo test`, `pnpm test`, `ctest` (for C++).
5. **Document** public APIs.
6. **Check file lengths** — split if > 400 lines.
7. **Commit** with conventional commits (`feat:`, `fix:`, `docs:`, `refactor:`).

## 6. Combining AI agents

| Agent | Recommended use |
|---|---|
| **Claude Code** | Architecture refactors, multi-file changes, tests, specs |
| **Gemini Code Assist** | Second opinion, code reviews, audio DSP, performance |

For larger design decisions: **ask both, decide with the human.**

## 7. MCP integration

PhraseDJ exposes its own parameters via the Model Context Protocol so that
AI agents can interact with the app at runtime (inspect library, propose
transitions, generate macros). Details: `specs/09-plugin-system.md`.

## 8. Common mistakes to avoid

- ❌ "I'll quickly add Spotify support." → **non-goal, do not.**
- ❌ A 1200-line file → **600-line rule.**
- ❌ `let buffer_size = 256;` mid-code → **`config/defaults.toml`.**
- ❌ `unwrap()` in production code → typed error.
- ❌ Silent network call → only the documented lyrics-lookup exception is allowed.
- ❌ No tests → PR rejection.

## 9. When something is unclear

Add a `// TODO(spec):` comment, open an issue, **and ask the maintainer**
instead of guessing. A small correct change beats a big change that gets
reverted later.
