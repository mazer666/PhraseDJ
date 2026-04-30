# 06 — Lyrics

## 1. Goals

- Display synchronised, karaoke-style lyrics for the currently playing track.
- Work primarily offline; reach the network only as an opt-in fallback.
- Be source-honest: always show where the lyrics came from.
- Never block playback on lyrics availability.

## 2. Source priority (resolution chain)

For a given track, the resolver tries in order:

1. **Embedded LRC tag** (`USLT` in ID3v2, `LYRICS` in Vorbis comments)
2. **Sidecar `.lrc` file** next to the audio file
3. **Cached lyrics** under `<app-support>/PhraseDJ/cache/lyrics/<track-uuid>.lrc`
4. **Online lookup** (LRCLib by default) — opt-in toggle
5. **Local Whisper alignment** — runs `whisper.cpp` on-device to transcribe
   the vocals stem (which the stem pipeline already produced) and aligns the
   words to the audio

Each successful source updates the cache so the next playback starts
instantly.

## 3. Online lookup (opt-in exception)

Network access is disabled at first launch. The first time the user opens a
track without local lyrics, a one-time consent dialog explains:

- which service is queried (LRCLib, public, no account)
- what data is sent (artist, title, duration, optional album — never audio)
- where the response is cached
- a "Always allow" / "Allow once" / "Never" choice

Once enabled, every outbound request is logged in
`<app-support>/PhraseDJ/network.log` and surfaced in a small
"Network activity" view in settings. The toggle is reversible at any time.

If the lookup returns nothing, the resolver continues to step 5.

### LRCLib request shape (illustrative)

```
GET https://lrclib.net/api/get?
    artist_name=<artist>&
    track_name=<title>&
    album_name=<album>&
    duration=<seconds>
```

Response is a JSON object containing `syncedLyrics` (LRC) or
`plainLyrics` (no timestamps). Synced lyrics go straight to the cache;
plain lyrics fall through to step 5 for alignment.

## 4. Local Whisper alignment

- `whisper.cpp` linked via FFI in `pdj-lyrics`.
- Uses a small multilingual model by default (`ggml-base.en` for English,
  configurable in `defaults.toml`).
- Runs on the **isolated vocals stem** that `pdj-stems` already produced —
  this dramatically improves accuracy compared to running Whisper on the
  full mix.
- Output is converted to LRC (`[mm:ss.xx] line`).
- If the source had plain lyrics, the alignment uses them as transcript
  hint (forced alignment) for higher accuracy.

## 5. Display

- Mode 1 — **Inline**: scrolling text beneath the deck, current line larger
  and brighter.
- Mode 2 — **Karaoke overlay**: full-width strip across the screen with a
  word-level progress mask animating with the beat.
- Mode 3 — **Off**.

Switchable via a single toolbar button or hotkey (`Y`).

## 6. Source indicator

A small badge next to the lyrics panel shows the active source:

- `TAG` — embedded
- `LRC` — sidecar / file
- `NET` — online (with hostname tooltip)
- `ASR` — local Whisper alignment
- `MIX` — combined (e.g. plain network + local alignment)

The badge colour matches the network-traffic policy: blue for offline-only,
amber for any source that involved a network call.

## 7. Crate `pdj-lyrics` API

```rust
pub enum LyricsSource { Tag, Sidecar, Cache, Network, Whisper, Mixed }

pub struct Lyrics {
    pub track_id: TrackId,
    pub source: LyricsSource,
    pub language: Option<String>,
    pub lines: Vec<LyricLine>,
}

pub struct LyricLine {
    pub start_ms: u64,
    pub end_ms:   u64,
    pub text:     String,
    pub words:    Option<Vec<WordTiming>>,
}

pub fn resolve(track: TrackId, options: ResolveOptions) -> Result<Lyrics>;
pub fn invalidate(track: TrackId) -> Result<()>;
pub fn export_lrc(track: TrackId, dest: &Path) -> Result<()>;
```

Internal modules:

- `mod tag`        — read embedded lyrics
- `mod sidecar`    — discover and parse `.lrc` files
- `mod online`     — HTTP client (reqwest), strict timeout, no retries on 4xx
- `mod whisper`    — FFI to whisper.cpp
- `mod align`      — word-level alignment from plain text + audio
- `mod render`     — convert to view model used by the UI

Each file ≤ 400 lines.

## 8. Privacy and resilience

- Outbound requests have a 5-second timeout. No retries beyond two with
  exponential backoff.
- A network failure never blocks playback.
- A "panic kill" toggle in settings stops all in-flight network operations
  and disables future ones until re-enabled.
- The full network log is exportable as a plain CSV for transparency.

## 9. Edge cases

- **Instrumental tracks** — resolver returns an empty `Lyrics` with source
  `Tag` so the UI shows "No lyrics".
- **Wrong-language Whisper** — language detection runs on the first 10 s of
  the vocals stem; mismatch falls back to a multilingual model.
- **Mismatched online result** — if the returned duration differs from the
  local file by more than 3 seconds, the result is discarded.
- **Karaoke timing drift** — re-alignment is offered as a one-click action
  from the lyrics panel.

## 10. Testing

- Unit tests for tag readers and LRC parsers with malformed inputs.
- Integration test: a track without lyrics → online lookup mocked → cache
  hit on second call.
- Determinism test: Whisper alignment on a fixed sample produces the same
  LRC bytes across runs (within an ε for floating-point boundaries).
