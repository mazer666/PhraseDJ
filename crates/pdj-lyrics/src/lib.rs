//! pdj-lyrics — Lyrics discovery, synchronization, and alignment.
//!
//! This crate implements the lyrics resolution chain:
//! 1. Embedded tags (ID3v2 USLT, etc.)
//! 2. Sidecar .lrc files
//! 3. Local cache
//! 4. Online lookup (LRCLib)
//! 5. Local Whisper alignment (on vocals stem)

use pdj_core::{Result, TrackId};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LyricsSource {
    Tag,
    Sidecar,
    Cache,
    Network,
    Whisper,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricLine {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lyrics {
    pub track_id: TrackId,
    pub source: LyricsSource,
    pub lines: Vec<LyricLine>,
}

/// Options for the lyrics resolver.
#[derive(Debug, Clone, Default)]
pub struct ResolveOptions {
    pub allow_network: bool,
    pub allow_asr: bool,
}

/// Resolve lyrics for a given track using the priority chain.
pub async fn resolve(track: TrackId, _options: ResolveOptions) -> Result<Lyrics> {
    // Phase 2 implementation will check tags and sidecars first.
    Err(pdj_core::Error::other(format!(
        "Lyrics not found for track {}",
        track
    )))
}

/// Export lyrics to an LRC file.
pub fn export_lrc(_lyrics: &Lyrics, _dest: &Path) -> Result<()> {
    Ok(())
}
