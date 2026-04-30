# 11 — Testing and QA

## 1. Pyramid

```
        ┌──────────────┐
        │  E2E / live  │   small, slow — Phase 4+
        └──────────────┘
       ┌────────────────┐
       │  integration   │   per crate + cross-crate
       └────────────────┘
     ┌────────────────────┐
     │   unit (largest)   │   every module
     └────────────────────┘
```

## 2. Per-language tooling

| Language | Test framework | Mocking | Coverage |
|---|---|---|---|
| Rust | `cargo test` + `proptest` | `mockall` where needed | `cargo llvm-cov` |
| TypeScript | Vitest + React Testing Library | MSW for HTTP | `vitest --coverage` |
| C++ | GoogleTest + GoogleBench | manual fakes | `llvm-cov` |

## 3. Audio-specific tests

- **Reference samples** in `tests/fixtures/audio/` (royalty-free, included
  in repo with their licences).
- **Stem fidelity test**: separates a reference track and asserts
  `RMS(sum_of_stems − original) < -40 dB`.
- **Latency test**: measures callback duration with a synthetic mix at the
  default buffer size; budget defined in `10-performance.md`.
- **No-allocation test**: links a hook that traps `malloc` during a 1 s
  callback run and fails the test if hit.
- **Determinism test**: same input → same output across runs (modulo
  documented float-boundary ε).

## 4. Library and macros

- In-memory SQLite for unit tests, throwaway temp DBs for integration.
- Macro round-trip property test: record → save → load → replay produces a
  fader trajectory within ε of the original.

## 5. UI

- Component tests with React Testing Library on every interactive piece.
- Snapshot tests for theme tokens to catch accidental palette drift.
- Storybook (later phase) for visual review.
- Manual UI smoke checklist run before each phase milestone.

## 6. End-to-end (Phase 4+)

- A headless harness drives the app via Tauri commands and asserts on emitted
  events.
- A 4-hour stress test plays a synthetic playlist with random cue points,
  scratches, and macro firings; assert no leaks and no audio dropouts.
- Crash-recovery test: kill the process mid-set, restart, confirm
  auto-saved state restores.

## 7. CI gates (must pass to merge)

1. Format: `cargo fmt`, `prettier --check`, `clang-format`.
2. Lint: `cargo clippy -D warnings`, ESLint, `clang-tidy`.
3. Build: all targets green on macOS; Linux from Phase 2; Windows eval Phase 5.
4. Test: full suite green on macOS; subset on Linux.
5. Coverage: ≥ 70 % on core crates, ≥ 60 % on UI.
6. File-length linter: no `*.rs`, `*.ts(x)`, `*.cpp`, `*.hpp` over 600 logical
   lines (custom script in `tools/check_file_size.sh`).
7. Benchmark: no regression > 5 % on the tracked suite.
8. SBOM: licence scan green.

## 8. Test data and licensing

- All audio fixtures must be licensed for redistribution (CC0 / CC-BY).
- Each fixture has a `LICENSE.txt` next to it citing source and licence.
- LRC fixtures synthesised in tests where possible to avoid copyright
  questions.

## 9. Bug reports and reproducibility

- Issue template asks for OS, app version, hardware, audio device, and the
  contents of `network.log` if relevant.
- A "diagnostic dump" action exports a sanitised state snapshot the user can
  attach: settings, recent log, library counts, no track paths.

## 10. Manual QA checklist (per release)

- Cold start under 2 s.
- Load track, play, pause, seek, scratch — sounds clean.
- Sync two decks of mismatched BPM — no audible drift over 5 minutes.
- Stem mix with all 4 stems toggled — no clicks at toggle.
- Apply a saved macro — output matches reference recording within ε.
- Lyrics: tag, LRC, online (with consent), Whisper — all sources display.
- MIDI learn on three controls — bindings persist across restart.
- Plugin host: load a CLAP plugin, run for 5 minutes, close cleanly.
- Pull network cable mid-lyrics-fetch — no UI hang, fallback works.
- Pull audio interface mid-set — auto-recover to default device.
