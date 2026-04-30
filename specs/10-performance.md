# 10 — Performance

## 1. Budgets

| Subsystem | Budget |
|---|---|
| Audio round-trip latency | < 10 ms at 44.1 kHz / 128-sample buffer |
| Audio callback worst case | < 30 % of buffer duration on M2 |
| UI frame time | < 8 ms (target 120 fps) |
| Cold start (window visible) | < 2 s |
| Track load + first sound | < 200 ms |
| Library search (10 k tracks) | < 50 ms |
| Stem analysis (7 min, M2) | < 20 s |
| Memory (idle, 2 decks loaded) | < 600 MB |
| Memory (analysing) | < 1.5 GB peak |

## 2. Audio safety rules

(See also `02-audio-engine.md` §8.)

- No allocation in the realtime callback.
- No locks. Use SPSC ring buffers and atomic flags.
- No syscalls that may block (no `printf`, no `fwrite`, no `mmap` of new
  pages).
- All FP math finite — denormal flush enabled (`FTZ` + `DAZ`).
- Each audio path has a bounded loop count proven by inspection; tests
  enforce no allocation via `mtrace`-style hooks.

## 3. Profiling

Required tooling on macOS:

- **Instruments** (Time Profiler, Allocations, System Trace).
- **Tracy** (compile-time switch `--features tracy`) for in-app timing
  scopes in Rust and C++.
- **Cargo flamegraph** for offline runs.

Required tooling cross-platform:

- **perf** + **hotspot** on Linux.
- A CI benchmark job compares results against a stored baseline; >10 %
  regression breaks the build.

## 4. Benchmarks

A suite under `tests/bench/` covers:

- `audio_callback_latency.rs` — measure callback duration on a synthetic mix.
- `stem_separation_e2e.rs` — wall-clock for the reference 7-min track.
- `library_search_p99.rs` — 10 k synthetic rows, 50 ms p99 target.
- `cold_start.rs` — invokes the binary headlessly and measures first
  ready-to-play timestamp.

Benchmarks are run in CI on macOS-13 self-hosted runners (M-series) and on
GitHub-hosted Linux runners (with realistic margins for the latter).

## 5. Memory

- Pre-decoded PCM is bounded per track to `min(file_pcm_size, 200 MB)`.
- Stems beyond what's currently loaded on a deck are demand-paged from
  disk via the engine's prefetch ring buffer.
- Analysis workers use a memory cap (default 1 GB) with task admission
  control: bigger jobs wait.

## 6. Concurrency

- Tokio runtime configured with N-1 worker threads on multi-core machines.
- Audio thread is its own OS thread with realtime priority (`mach_set_ts`
  on macOS) and is never scheduled on an efficiency core.
- Workers (decoder, analysis) yield often and check for cancellation tokens.

## 7. Frame budget for the UI

- Waveform updates at the display refresh rate, not the audio rate.
- Heavy paints batched per frame; GPU upload paths use `wgpu` staging
  buffers.
- React renders memoised; expensive panels are virtualised (library list).

## 8. Warm-state optimisation

- App keeps recently used tracks decoded and warm in a small LRU.
- Settings cache invalidation is event-driven, not polled.
- Database connections live in a small pool; queries use prepared statements.

## 9. Anti-patterns to avoid

- ❌ Doing analysis work synchronously in a Tauri command handler.
- ❌ Logging in the audio callback.
- ❌ Allocating per-frame on the UI hot path.
- ❌ Holding a database transaction across user input.
- ❌ Spawning a thread per track instead of using the pool.

## 10. Definition of "fast enough"

A change is allowed to merge if all of these hold:

1. No regression > 5 % on any existing benchmark.
2. New code paths come with a benchmark or are demonstrably out of any hot path.
3. Audio safety tests stay green.
4. Memory peak doesn't grow more than 5 % on the standard scenario.
