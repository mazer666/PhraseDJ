# 05 — Transitions and Macros

## 1. Goals

- Record a manual transition once, replay it with the same feel anytime.
- Allow editing recorded transitions on a beat-locked timeline.
- Trigger macros half-automatically (user starts, app finishes) or fully
  automatically.
- Make transitions a first-class citizen of the library — searchable, sharable
  per JSON file, attachable to track pairs.

## 2. Concepts

| Term | Meaning |
|---|---|
| **Event** | One control change — `set_fader(deck, value, t)`, `set_filter(...)`, `cue_press`, etc. |
| **Macro** | A time-ordered sequence of events anchored to bars / beats |
| **Transition** | A macro that involves both decks and changes the dominant audio source |
| **Anchor** | A reference point in the beatgrid (e.g. "8 bars before drop on deck B") |
| **Trigger** | The moment the macro starts running — can be manual or scheduled |

## 3. Recording

- The macro recorder is enabled from the toolbar; a small red dot appears.
- Every control event from the user (UI, MIDI, keyboard) is captured into a
  ring buffer with monotonic timestamps.
- When the recorder is stopped, the ring buffer is converted into a
  beat-relative event list using the active beatgrid of each deck.
- The user names the macro and confirms which track pair it applies to
  (the apps' default is "the two tracks currently loaded").

Storage format (JSON):

```json
{
  "id": "8c5b1d92-…",
  "name": "8-bar filter+stem swap",
  "version": 1,
  "tracks": ["track-a-uuid", "track-b-uuid"],
  "anchor": { "deck": "B", "bar_offset_from_drop": -8 },
  "duration_bars": 8,
  "events": [
    { "t_bar": 0.00, "ch": "fader.A",   "v": 1.00, "ease": "linear" },
    { "t_bar": 4.00, "ch": "fader.A",   "v": 0.00, "ease": "cosine" },
    { "t_bar": 0.00, "ch": "fader.B",   "v": 0.00, "ease": "linear" },
    { "t_bar": 4.00, "ch": "fader.B",   "v": 1.00, "ease": "cosine" },
    { "t_bar": 0.00, "ch": "stem.A.vox","v": 1.00, "ease": "linear" },
    { "t_bar": 2.00, "ch": "stem.A.vox","v": 0.00, "ease": "cosine" }
  ]
}
```

Files live at
`<app-support>/PhraseDJ/macros/<track-a>__<track-b>__<macro-id>.json`.

## 4. Replay

- The replay engine schedules events on the audio engine via realtime-safe
  setters at sample-accurate timing relative to the chosen anchor.
- All eases are computed in advance into a small interpolation table to keep
  the realtime path branch-free.
- A macro can be paused, cancelled, or trimmed mid-flight.
- "Apply on next phrase" — the engine waits for the configured anchor before
  starting (e.g. next 4-bar boundary).

## 5. Modes

- **Half-auto.** User initiates the transition (presses `T`); the macro then
  runs to completion.
- **Full-auto.** When both decks have a saved transition for this pair, the
  app can chain tracks autonomously (DJ-set mode).
- **Suggest.** A subtle hint (border highlight on the matching deck) when a
  good moment to fire the macro is approaching.

## 6. Editing

The Macro mode UI shows the events on a timeline:

- Lanes for the most-touched channels (fader, filter, stems) auto-promoted
  to top.
- Drag points to retime; right-click to change ease curve; double-click to
  delete.
- Multi-select with rubber-band; align to nearest bar / beat / 1/16.
- Versioning: each save increments `version`; older versions kept for undo
  history (last 5 in memory, last 1 on disk).

## 7. Phrase awareness

- The phrase detector emits markers (intro / verse / chorus / drop / outro)
  at import time (`pdj-stems` companion model).
- Macros can reference phrases instead of bar offsets, e.g.
  `"anchor": { "deck": "B", "phrase": "drop", "bar_offset": -8 }`.
- Useful for transitions that should adapt across remixes / extended
  versions.

## 8. Replication & sharing

- A macro JSON contains track UUIDs that map to canonical metadata (title,
  artist, ISRC if present).
- Sharing a macro = sharing a JSON file; on import, PhraseDJ resolves track
  identity by ISRC first, then by title+artist+duration heuristic.
- Macros are not coupled to the user's specific audio files, but they are
  not playable without similar tracks.

## 9. Crate `pdj-macros` API

```rust
pub fn record_start(pair: TrackPair) -> Result<RecorderId>;
pub fn record_stop(id: RecorderId, name: &str) -> Result<MacroId>;
pub fn list_for_pair(pair: TrackPair) -> Vec<MacroSummary>;
pub fn apply(id: MacroId, mode: ApplyMode) -> Result<RunHandle>;
pub fn cancel(handle: RunHandle) -> Result<()>;
pub fn export(id: MacroId, dest: &Path) -> Result<()>;
pub fn import(src: &Path) -> Result<MacroId>;
```

Internally:

- `mod recorder` — captures events, time-aligns to beatgrid
- `mod replay` — schedules events into the engine
- `mod store` — JSON I/O, indexes, integrity checks
- `mod schema` — version migrations
- `mod ease` — easing curves table

Each file ≤ 400 lines.

## 10. Edge cases

- **Tempo drift between recording and replay.** Use bar-relative timestamps
  + per-deck beatgrid to keep events musical even at different BPMs.
- **Macro longer than remaining track.** Auto-truncate, raise a warning at
  edit time.
- **Conflicting macros for the same pair.** UI lets the user pick a default
  per pair; others remain available manually.
- **Live override.** During replay the user can grab any control — replay
  yields that channel until the macro ends.

## 11. Testing

- Unit tests for ease tables, schema migration, and beat-relative time math.
- Integration test: record a synthetic macro, save, reload, replay, compare
  resulting fader trajectory against the recorded one within ε < 0.5 %.
- Long-haul test (Phase 4 hardening): chain 50 macros over 4 hours; assert
  no drift accumulates.
