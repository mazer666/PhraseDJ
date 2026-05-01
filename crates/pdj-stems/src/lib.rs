/// pdj-stems — HTDemucs-based stem separation for PhraseDJ.
///
/// # Overview
///
/// This crate splits audio tracks into four stems:
///   - **vocals** — lead vocals and backing vocals
///   - **drums**  — kick, snare, hi-hats, cymbals
///   - **bass**   — bass guitar and sub-bass
///   - **other**  — everything else (guitars, synths, strings, …)
///
/// Processing happens in the background so playback is never interrupted.
/// Results are cached on disk.  The same track is never analysed twice.
///
/// # Quick start
///
/// ```rust,no_run
/// use pdj_stems::{StemService, StemStatus};
/// use pdj_core::types::TrackId;
///
/// # tokio_test::block_on(async {
/// let service = StemService::new(None).await.unwrap();
/// let track   = TrackId::new();
/// service.enqueue(track, "/path/to/track.flac".into()).await.unwrap();
///
/// let mut rx = service.subscribe();
/// while let Ok((tid, status)) = rx.recv().await {
///     if tid == track {
///         match status {
///             StemStatus::Cached { paths } => {
///                 println!("vocals at {:?}", paths.vocals);
///                 break;
///             }
///             StemStatus::Failed { reason } => {
///                 eprintln!("error: {reason}");
///                 break;
///             }
///             _ => {}
///         }
///     }
/// }
/// # });
/// ```
///
/// # Module layout
///
/// | Module    | Responsibility                                           |
/// |-----------|----------------------------------------------------------|
/// | `backend` | `StemBackend` trait that both MLX and ONNX implement     |
/// | `mlx`     | Apple Silicon / MLX implementation (macOS only)          |
/// | `onnx`    | ONNX Runtime fallback (all platforms)                    |
/// | `queue`   | Async job scheduler with concurrency throttling          |
/// | `stitch`  | Overlap-add segment stitching + FLAC writing             |
/// | `paths`   | Cache directory layout helpers                           |
pub mod backend;
pub mod mlx;
pub mod onnx;
pub mod paths;
pub mod queue;
pub mod stitch;

// Re-export the main entry points so callers only need to import from
// `pdj_stems` directly.
pub use queue::{StemService, StemStatus};
pub use paths::StemPaths;
