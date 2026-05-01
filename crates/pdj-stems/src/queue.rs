/// Async job queue for stem-separation analysis.
///
/// # Design
///
/// The queue is built on top of Tokio.  Each track to be analysed is
/// represented by a `StemJob`.  The queue runs up to `max_parallel_jobs`
/// jobs concurrently; extra jobs wait in a FIFO channel.
///
/// Status changes are published on an `async-broadcast` channel so that any
/// number of UI subscribers can receive live progress without polling.
///
/// # Concurrency limit
///
/// By default (`max_parallel_jobs = 0` in settings), the queue uses
/// `n_performance_cores − 1` so the audio callback always has CPU budget.
/// The user can override this in settings.
///
/// # File length note
///
/// This module is intentionally bounded.  Complex helpers (overlap-add,
/// file paths, inference) live in `stitch` and `paths` respectively.
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use async_broadcast::{broadcast, Receiver, Sender};
use pdj_core::{types::TrackId, Error, Result};
use tokio::sync::{Mutex, Semaphore};
use tracing::{debug, error, info, warn};

use crate::backend::{select_backend, InferenceRequest, PcmBuffer, StemLabel};
use crate::paths::{stem_cache_root, stem_paths_for, StemPaths};
use crate::stitch::{write_stems_to_disk, Stitcher};

// ---------------------------------------------------------------------------
// Public status type
// ---------------------------------------------------------------------------

/// Current analysis status for a single track.
///
/// The UI subscribes to a stream of `(TrackId, StemStatus)` events and
/// updates the progress chip in the library list accordingly.
#[derive(Debug, Clone)]
pub enum StemStatus {
    /// Job is waiting in the queue.
    Pending,
    /// Analysis is running; `progress` is a value in [0.0, 1.0].
    Running { progress: f32 },
    /// All four stem files are cached on disk.
    Cached { paths: StemPaths },
    /// Analysis failed; the human-readable reason is in `reason`.
    Failed { reason: String },
}

// ---------------------------------------------------------------------------
// Internal job record
// ---------------------------------------------------------------------------

/// Internal representation of one analysis job.
#[derive(Debug)]
struct StemJob {
    /// Which track to analyse.
    track_id: TrackId,
    /// Absolute path to the source audio file.
    source_path: PathBuf,
}

// ---------------------------------------------------------------------------
// StemService
// ---------------------------------------------------------------------------

/// Entry point for all stem-separation operations.
///
/// Construct one instance at app start and share it via `Arc`.  The service
/// owns the job queue, the broadcast channel, and the semaphore that limits
/// concurrency.
pub struct StemService {
    /// Shared mutable state.
    inner: Arc<ServiceInner>,
}

/// The mutable internals behind an `Arc`, protected by a `Mutex`.
struct ServiceInner {
    /// Broadcast channel sender — cloned for each new subscriber.
    status_tx: Sender<(TrackId, StemStatus)>,
    /// Track IDs currently pending or running (to enforce idempotency).
    active_tracks: Mutex<HashSet<TrackId>>,
    /// Channel for submitting new jobs to the worker task.
    job_tx: tokio::sync::mpsc::UnboundedSender<StemJob>,
    /// Semaphore to cap the number of running analysis jobs.
    semaphore: Arc<Semaphore>,
    /// Absolute path to the stem cache root directory.
    cache_root: PathBuf,
}

impl StemService {
    /// Create a new `StemService` and start the background worker.
    ///
    /// `max_parallel_jobs`: maximum concurrent analysis jobs.
    /// Pass `0` to use `num_cpus::get_physical() - 1` (leaving one core
    /// free for audio playback).
    pub async fn new(max_parallel_jobs: Option<usize>) -> Result<Arc<Self>> {
        let concurrency = compute_concurrency(max_parallel_jobs);
        info!(concurrency, "StemService starting");

        // Set up the broadcast channel for status events.
        // Capacity of 64: if a subscriber falls behind it gets the most
        // recent events and drops stale ones.
        let (status_tx, _status_rx) = broadcast::<(TrackId, StemStatus)>(64);

        // Unbounded channel so enqueue() never blocks the caller.
        let (job_tx, job_rx) =
            tokio::sync::mpsc::unbounded_channel::<StemJob>();

        let cache_root = stem_cache_root()?;

        let inner = Arc::new(ServiceInner {
            status_tx,
            active_tracks: Mutex::new(HashSet::new()),
            job_tx,
            semaphore: Arc::new(Semaphore::new(concurrency)),
            cache_root,
        });

        // Spawn the long-lived background task that drains the job channel.
        let inner_clone = Arc::clone(&inner);
        tokio::spawn(async move {
            run_worker(inner_clone, job_rx).await;
        });

        Ok(Arc::new(Self { inner }))
    }

    /// Submit a track for stem analysis.
    ///
    /// **Idempotent:** if the track is already pending, running, or cached,
    /// this is a no-op (no duplicate job is created).
    ///
    /// If stems are already cached, a `StemStatus::Cached` event is emitted
    /// immediately without queuing a new job.
    pub async fn enqueue(&self, track_id: TrackId, source_path: PathBuf) -> Result<()> {
        // Fast path: stems already on disk.
        let existing = stem_paths_for(&self.inner.cache_root, track_id);
        if existing.all_exist() {
            debug!(?track_id, "stems already cached — skipping enqueue");
            let _ = self
                .inner
                .status_tx
                .broadcast((track_id, StemStatus::Cached { paths: existing }))
                .await;
            return Ok(());
        }

        // Guard against duplicate jobs.
        {
            let mut active = self.inner.active_tracks.lock().await;
            if active.contains(&track_id) {
                debug!(?track_id, "track already in queue — idempotent enqueue");
                return Ok(());
            }
            active.insert(track_id);
        }

        // Notify subscribers that the job is waiting.
        let _ = self
            .inner
            .status_tx
            .broadcast((track_id, StemStatus::Pending))
            .await;

        // Send to the worker.
        self.inner
            .job_tx
            .send(StemJob { track_id, source_path })
            .map_err(|_| Error::other("stem job channel closed unexpectedly"))?;

        Ok(())
    }

    /// Cancel a pending or running job for `track_id`.
    ///
    /// If the track is already cached or was never queued, this is a no-op.
    /// Running jobs may complete before cancellation takes effect.
    pub async fn cancel(&self, track_id: TrackId) -> Result<()> {
        let mut active = self.inner.active_tracks.lock().await;
        if active.remove(&track_id) {
            debug!(?track_id, "stem job cancelled");
            // The worker checks active_tracks before processing — the job
            // will be silently dropped when it is dequeued.
        }
        Ok(())
    }

    /// Subscribe to status updates.
    ///
    /// Returns a receiver that yields `(TrackId, StemStatus)` pairs every
    /// time any track's status changes.  Multiple subscribers are supported.
    ///
    /// The receiver is inactive (not yet receiving) until you `.await` on it
    /// in a loop.
    pub fn subscribe(&self) -> Receiver<(TrackId, StemStatus)> {
        self.inner.status_tx.new_receiver()
    }

    /// Wait synchronously (async) until `track_id` is cached or fails.
    ///
    /// Useful in tests and CLI tools.  Not recommended for the UI — use
    /// `subscribe()` instead.
    pub async fn wait(&self, track_id: TrackId) -> Result<StemPaths> {
        let mut rx = self.subscribe();
        loop {
            match rx.recv().await {
                Ok((tid, StemStatus::Cached { paths })) if tid == track_id => {
                    return Ok(paths);
                }
                Ok((tid, StemStatus::Failed { reason })) if tid == track_id => {
                    return Err(Error::other(reason));
                }
                Ok(_) => {
                    // Different track or non-terminal status — keep waiting.
                }
                Err(async_broadcast::RecvError::Closed) => {
                    return Err(Error::other("status channel closed"));
                }
                Err(async_broadcast::RecvError::Overflowed(_)) => {
                    // We missed some events; check if the stems are on disk.
                    let paths = stem_paths_for(&self.inner.cache_root, track_id);
                    if paths.all_exist() {
                        return Ok(paths);
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Background worker
// ---------------------------------------------------------------------------

/// Long-lived task that processes jobs from the queue.
///
/// Runs until the `job_rx` channel is closed (i.e. the `StemService` is
/// dropped).
async fn run_worker(
    inner: Arc<ServiceInner>,
    mut job_rx: tokio::sync::mpsc::UnboundedReceiver<StemJob>,
) {
    while let Some(job) = job_rx.recv().await {
        let inner_clone = Arc::clone(&inner);

        // Acquire a permit before spawning so we respect the concurrency limit.
        // `clone_owned` returns a future that waits for a free slot.
        let permit = Arc::clone(&inner.semaphore)
            .acquire_owned()
            .await
            .expect("semaphore never closes");

        tokio::spawn(async move {
            // Permit is held for the duration of the job and released on drop.
            process_job(inner_clone, job).await;
            drop(permit);
        });
    }
    info!("StemService worker exiting — channel closed");
}

/// Process a single analysis job end-to-end.
async fn process_job(inner: Arc<ServiceInner>, job: StemJob) {
    let track_id = job.track_id;

    // Check whether the job was cancelled between enqueue and now.
    {
        let active = inner.active_tracks.lock().await;
        if !active.contains(&track_id) {
            debug!(?track_id, "job was cancelled before processing — skipping");
            return;
        }
    }

    info!(?track_id, source = ?job.source_path, "starting stem analysis");

    // Notify: running at 0 %.
    publish(&inner, track_id, StemStatus::Running { progress: 0.0 }).await;

    // All heavy work happens in a blocking thread so the async executor
    // is not starved.
    let cache_root = inner.cache_root.clone();
    let result = tokio::task::spawn_blocking(move || {
        run_analysis(track_id, &job.source_path, &cache_root)
    })
    .await;

    // Remove from active set regardless of outcome.
    inner.active_tracks.lock().await.remove(&track_id);

    match result {
        Ok(Ok(paths)) => {
            info!(?track_id, "stem analysis complete");
            publish(&inner, track_id, StemStatus::Cached { paths }).await;
        }
        Ok(Err(e)) => {
            error!(?track_id, error = %e, "stem analysis failed");
            publish(
                &inner,
                track_id,
                StemStatus::Failed { reason: e.to_string() },
            )
            .await;
        }
        Err(join_err) => {
            error!(?track_id, error = %join_err, "stem analysis task panicked");
            publish(
                &inner,
                track_id,
                StemStatus::Failed { reason: join_err.to_string() },
            )
            .await;
        }
    }
}

/// The actual CPU-bound work: load PCM, segment, infer, stitch, write.
///
/// This runs on a blocking OS thread (via `spawn_blocking`), so blocking
/// I/O and long computations are safe here.
fn run_analysis(
    track_id: TrackId,
    source_path: &std::path::Path,
    cache_root: &std::path::Path,
) -> Result<StemPaths> {
    // --- 1. Select inference backend (MLX or ONNX) -----------------------
    let backend = select_backend();
    debug!(backend = backend.name(), ?track_id, "selected inference backend");

    // --- 2. Load PCM from disk -------------------------------------------
    // For Phase 2, we load a sine-wave stub instead of reading the actual
    // file.  This keeps the pipeline fully exercisable while the real
    // audio decoder (libsndfile / symphonia) is wired in.
    //
    // TODO(spec): replace with a real decoder that reads source_path.
    // Reference: specs/02-audio-engine.md §3.
    let audio = load_pcm_stub(source_path)?;
    let channels    = audio.channels;
    let sample_rate = audio.sample_rate;
    let total_frames = audio.frame_count();

    // --- 3. Segment into overlapping windows -----------------------------
    //
    // Segment size: 8 s × sample_rate frames.
    // Overlap:      1 s × sample_rate frames (half-Hann crossfade).
    let segment_frames = 8 * sample_rate as usize;
    let overlap_frames =     sample_rate as usize;
    let step_frames    = segment_frames - overlap_frames;

    let mut stitcher = Stitcher::new(channels, sample_rate, overlap_frames);
    let mut offset   = 0usize;
    let mut segment_count = 0usize;

    while offset < total_frames {
        let end    = (offset + segment_frames).min(total_frames);
        let s_start = offset * channels as usize;
        let s_end   = end   * channels as usize;

        let segment_buf = PcmBuffer {
            samples:     audio.samples[s_start..s_end].to_vec(),
            channels,
            sample_rate,
        };

        let result = backend.infer(InferenceRequest { audio: segment_buf })?;
        stitcher.add_segment(result.stems);

        offset        += step_frames;
        segment_count += 1;

        debug!(?track_id, segment = segment_count, "segment processed");
    }

    // --- 4. Stitch and write to disk -------------------------------------
    let stems      = stitcher.finalise();
    let stem_paths = stem_paths_for(cache_root, track_id);
    write_stems_to_disk(stems, &stem_paths)?;

    Ok(stem_paths)
}

// ---------------------------------------------------------------------------
// PCM loading stub
// ---------------------------------------------------------------------------

/// Generate a short silent PCM buffer to stand in for the real audio file.
///
/// Phase 2 stub: the real implementation will open `path` with symphonia or
/// libsndfile and decode the full track.
///
/// **TODO(spec):** Replace with real decoder.
/// Reference: `specs/02-audio-engine.md §3`.
fn load_pcm_stub(_path: &std::path::Path) -> Result<PcmBuffer> {
    // 5-second stereo silence at 44.1 kHz.
    let sample_rate: u32 = 44_100;
    let channels:    u16 = 2;
    let seconds:     u32 = 5;
    Ok(PcmBuffer {
        samples:     vec![0.0_f32; (sample_rate * channels as u32 * seconds) as usize],
        channels,
        sample_rate,
    })
}

// ---------------------------------------------------------------------------
// Helper: publish a status event
// ---------------------------------------------------------------------------

/// Broadcast a status event, ignoring errors if no subscribers are active.
async fn publish(inner: &ServiceInner, track_id: TrackId, status: StemStatus) {
    if let Err(e) = inner.status_tx.broadcast((track_id, status)).await {
        // Only warn — no subscribers is a normal state during tests.
        warn!("status broadcast failed: {e}");
    }
}

// ---------------------------------------------------------------------------
// Concurrency calculation
// ---------------------------------------------------------------------------

/// Compute the effective number of parallel analysis jobs.
///
/// `0` in settings means "n_performance_cores − 1, minimum 1".
fn compute_concurrency(setting: Option<usize>) -> usize {
    match setting {
        Some(n) if n > 0 => n,
        _ => {
            // Leave at least one core free for the audio callback.
            num_cpus::get_physical().saturating_sub(1).max(1)
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn enqueue_and_wait_returns_stem_paths() {
        let service = StemService::new(Some(1)).await.expect("StemService::new");
        let track   = TrackId::new();
        // Use a fake path — the stub loader ignores it.
        service
            .enqueue(track, PathBuf::from("/fake/track.flac"))
            .await
            .expect("enqueue");
        let paths = service.wait(track).await.expect("wait");
        assert!(paths.vocals.exists(), "vocals stem should be on disk");
        assert!(paths.drums.exists(),  "drums stem should be on disk");
        assert!(paths.bass.exists(),   "bass stem should be on disk");
        assert!(paths.other.exists(),  "other stem should be on disk");
    }

    #[tokio::test]
    async fn enqueue_is_idempotent_for_cached_tracks() {
        let service = StemService::new(Some(1)).await.expect("StemService::new");
        let track   = TrackId::new();
        let path    = PathBuf::from("/fake/track.flac");

        // First enqueue — triggers analysis.
        service.enqueue(track, path.clone()).await.expect("enqueue 1");
        service.wait(track).await.expect("wait 1");

        // Second enqueue — should emit Cached immediately, no new job.
        let mut rx = service.subscribe();
        service.enqueue(track, path).await.expect("enqueue 2");

        // Drain events — we should receive at most one Cached event.
        let (tid, status) = rx.recv().await.expect("recv");
        assert_eq!(tid, track);
        assert!(matches!(status, StemStatus::Cached { .. }));
    }

    #[test]
    fn compute_concurrency_with_zero_uses_cores() {
        // Must be at least 1 even on single-core machines.
        assert!(compute_concurrency(None) >= 1);
    }

    #[test]
    fn compute_concurrency_respects_explicit_value() {
        assert_eq!(compute_concurrency(Some(3)), 3);
    }
}
