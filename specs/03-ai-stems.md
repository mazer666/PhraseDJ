# 03 — AI: Stem Separation

## 1. Goals

- Split every imported track into 4 stems: vocals, drums, bass, other.
- Run on-device, no cloud, no API keys.
- Process a 7-minute track in under 20 s on Apple Silicon (M2 baseline).
- Cache results so the cost is paid once per track.
- Be the foundation for Phase 2 features (stem mixer, multi-colour waveform,
  AI transition suggestions).

## 2. Model

- **HTDemucs** (Hybrid Transformer Demucs), pretrained 4-stem variant.
- Sample rate 44.1 kHz, stereo, segmented inference (overlap-add).
- Two backends:
  - **MLX backend** on macOS / Apple Silicon — preferred path
  - **ONNX Runtime backend** as cross-platform fallback (CPU + CoreML EP on Mac)
- Optional later: 6-stem (guitar / piano in addition). Off by default to
  keep storage and compute predictable.

## 3. Pipeline

```
import event ─► analysis queue ─► stem job
                                     │
                              ┌──────▼─────┐
                              │ load PCM   │
                              └──────┬─────┘
                                     ▼
                              ┌────────────┐
                              │ segment    │  (e.g. 8 s windows, 1 s overlap)
                              └──────┬─────┘
                                     ▼
                              ┌────────────┐
                              │ inference  │  (MLX or ONNX)
                              └──────┬─────┘
                                     ▼
                              ┌────────────┐
                              │ overlap-   │  (cross-fade segment edges)
                              │ add stitch │
                              └──────┬─────┘
                                     ▼
                              ┌────────────┐
                              │ write 4    │  (FLAC, 16-bit, 44.1k)
                              │ stem files │
                              └──────┬─────┘
                                     ▼
                              update DB row, emit event
```

## 4. Storage

- Stems live under
  `<app-support>/PhraseDJ/cache/stems/<track_uuid>/{vocals,drums,bass,other}.flac`.
- One FLAC per stem, 16-bit, mono or stereo to match source.
- Cache size budget enforced by an LRU policy with default 50 GB
  (configurable). When full, least-recently-used tracks are pruned and
  the corresponding DB column is reset to "needs reanalysis".
- A "Re-analyse" action in the UI clears and rebuilds.

## 5. Performance budget

| Hardware | Target time / 7-min track | Notes |
|---|---|---|
| Apple M1 | < 30 s | MLX |
| Apple M2 | < 20 s | MLX |
| Apple M3 / M4 | < 12 s | MLX |
| x86-64, no GPU | < 3 min | ONNX CPU |
| x86-64 + CUDA EP | < 30 s | optional |

The job queue throttles concurrency to `n_perf_cores − 1` to keep audio
playback glitch-free. A "playing now" guard pauses analysis if RT load
exceeds 60 %.

## 6. Crate `pdj-stems`

Public surface (Rust):

```rust
/// Status of stem separation for a single track.
pub enum StemStatus {
    Pending,
    Running { progress: f32 },
    Cached { paths: StemPaths },
    Failed { reason: String },
}

/// Submits a track for analysis. Idempotent.
pub fn enqueue(track: TrackId) -> Result<()>;

/// Cancels a running or pending job.
pub fn cancel(track: TrackId) -> Result<()>;

/// Subscribes to status updates (broadcast channel).
pub fn subscribe() -> impl Stream<Item = (TrackId, StemStatus)>;

/// Convenience: blocks until done (for tests).
pub fn wait(track: TrackId) -> Result<StemPaths>;
```

Internally split into:

- `mod backend` — trait `StemBackend`
- `mod mlx` — MLX implementation (compiled on macOS)
- `mod onnx` — ONNX implementation (always compiled)
- `mod queue` — job scheduling, throttling
- `mod stitch` — overlap-add and write FLACs
- `mod paths` — cache layout helpers

Each file ≤ 400 lines.

## 7. Quality gates

- Unit test with a 30-second reference track checks that:
  - 4 stems are produced
  - Sum of stems ≈ original (RMS error < -40 dB)
  - Vocals stem has higher centroid than bass stem
- Snapshot test: SHA-256 of the produced stems must match the saved baseline
  to one-bit precision per backend, per OS, per arch (caught in CI).
- Latency test: end-to-end import → stems-cached for the reference track
  must finish under target time per hardware class.

## 8. UX implications

- Library list shows a small chip per row: ⏳ pending, 🔄 running %, ✅ ready,
  ⚠ failed.
- A track without stems can still be played — stem mixer just shows a
  "Stems analysing… (45 %)" overlay.
- Background analysis can be paused from the status bar (e.g. before going on
  stage).

## 9. Privacy

- All model weights ship with the binary (or are downloaded once at first
  start with explicit consent dialog showing source URL and SHA).
- Audio never leaves the device.
- The download URL and integrity hash live in `config/defaults.toml`.

## 10. Future model work (out of MVP)

- 6-stem extraction (guitar / piano)
- Real-time stem separation for live input
- AI key correction per stem
- Vocal-removal "instrumental on demand" mode

These remain non-goals until 1.0 ships.
