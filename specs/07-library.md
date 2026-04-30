# 07 — Library

## 1. Goals

- Manage a local music collection without copying or moving files.
- Persist analysis results (BPM, key, energy, phrase markers, stems, lyrics).
- Stay fast at 100 000+ tracks.
- Survive crashes — never corrupt the database.

## 2. Storage

- **SQLite** at `<app-support>/PhraseDJ/library.db`, opened via `rusqlite`
  with WAL mode, foreign keys on, busy timeout 5 s.
- Audio files stay where they are; the DB stores absolute paths plus a
  resolved relative path against a configurable "music root" for
  portability.
- Caches (stems, lyrics, waveform peaks) live under
  `<app-support>/PhraseDJ/cache/`.

## 3. Schema (v1)

```sql
CREATE TABLE tracks (
  id              TEXT PRIMARY KEY,        -- UUID
  path            TEXT NOT NULL UNIQUE,    -- absolute path
  rel_path        TEXT,                    -- relative to music_root, if any
  title           TEXT,
  artist          TEXT,
  album           TEXT,
  isrc            TEXT,                    -- if present in tags
  duration_ms     INTEGER,
  sample_rate     INTEGER,
  channels        INTEGER,
  bitrate_kbps    INTEGER,
  bpm             REAL,
  key             TEXT,                    -- camelot or note
  energy          REAL,                    -- 0..1
  first_beat_ms   INTEGER,
  imported_at     INTEGER,                 -- unix ms
  analyzed_at     INTEGER,
  analysis_state  TEXT NOT NULL,           -- 'raw' | 'beatgrid' | 'full' | 'failed'
  stems_state     TEXT NOT NULL,           -- 'pending' | 'running' | 'cached' | 'failed'
  lyrics_state    TEXT NOT NULL,           -- same vocabulary
  notes           TEXT
);
CREATE INDEX idx_tracks_artist ON tracks(artist);
CREATE INDEX idx_tracks_bpm    ON tracks(bpm);
CREATE INDEX idx_tracks_key    ON tracks(key);

CREATE TABLE phrase_markers (
  track_id    TEXT NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
  start_ms    INTEGER NOT NULL,
  end_ms      INTEGER NOT NULL,
  kind        TEXT NOT NULL,    -- 'intro' | 'verse' | 'chorus' | 'drop' | 'outro' | 'break'
  confidence  REAL,
  PRIMARY KEY (track_id, start_ms)
);

CREATE TABLE cue_points (
  track_id    TEXT NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
  slot        INTEGER NOT NULL,            -- 0..7
  position_ms INTEGER NOT NULL,
  label       TEXT,
  color       TEXT,
  PRIMARY KEY (track_id, slot)
);

CREATE TABLE crates (
  id     TEXT PRIMARY KEY,
  name   TEXT NOT NULL,
  parent TEXT REFERENCES crates(id) ON DELETE SET NULL,
  smart_query TEXT                         -- JSON for smart crates
);

CREATE TABLE crate_tracks (
  crate_id TEXT NOT NULL REFERENCES crates(id) ON DELETE CASCADE,
  track_id TEXT NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
  added_at INTEGER NOT NULL,
  PRIMARY KEY (crate_id, track_id)
);

CREATE TABLE macros (
  id          TEXT PRIMARY KEY,
  track_a     TEXT NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
  track_b     TEXT NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
  name        TEXT NOT NULL,
  json_path   TEXT NOT NULL,               -- on-disk macro file
  duration_bars INTEGER NOT NULL,
  created_at  INTEGER NOT NULL
);
```

Migrations live in `crates/pdj-library/migrations/` numbered `0001_init.sql`,
`0002_*.sql`, etc. The library refuses to start if it sees a newer schema
version than the binary supports.

## 4. Import

- Drag-and-drop on the library drawer.
- Folder watcher (opt-in) that picks up new files in user-chosen directories.
- Manual scan of a folder (recursive).
- Duplicate detection by path; tag-based re-association if a known file moved.
- Imports add a row with `analysis_state='raw'` and enqueue beatgrid + stems
  + lyrics jobs.

## 5. Search and filters

- Full-text on title / artist / album using SQLite FTS5 virtual table.
- Range filters on BPM, energy, length.
- Equality filters on key (with Camelot wheel mapping for harmonic mixing).
- Smart crates store their query as JSON (canonicalised AST) so the same
  filter can be reused everywhere.

## 6. Backup and integrity

- Nightly SQLite `VACUUM INTO` snapshot to `library-backup-<date>.db`,
  rolling 7 copies.
- A "Verify library" action checks every track path exists; missing files
  are flagged but not removed.
- Integrity check (`PRAGMA integrity_check`) on every startup; if it fails,
  the app refuses to write and offers to restore from the latest backup.

## 7. Crate `pdj-library` API

```rust
pub fn open(path: &Path) -> Result<Library>;
pub fn import_file(&self, path: &Path) -> Result<TrackId>;
pub fn scan_folder(&self, root: &Path) -> Result<ScanReport>;
pub fn get(&self, id: TrackId) -> Result<Track>;
pub fn search(&self, query: &Query) -> Result<Vec<TrackSummary>>;
pub fn update_analysis(&self, id: TrackId, patch: AnalysisPatch) -> Result<()>;
pub fn watch(&self) -> impl Stream<Item = LibraryEvent>;
```

Internal modules:

- `mod schema`    — version constants, migrations
- `mod connect`   — pool, pragmas, busy timeout
- `mod import`    — file discovery + tag reading
- `mod search`    — FTS / range queries
- `mod crates`    — crate / smart crate logic
- `mod backup`    — snapshot helpers

Each file ≤ 400 lines.

## 8. Settings keys (in `defaults.toml`)

```toml
[library]
music_root = "~/Music"
follow_symlinks = false
ignore_files = ["*.tmp", "*.part"]
backup_count = 7
```

## 9. Privacy

- Cover art and metadata never leave the device unless lyrics lookup is
  enabled (see `06-lyrics.md`).
- The library DB is unencrypted by default; a future opt-in encryption
  feature is tracked but out of MVP.

## 10. Testing

- In-memory SQLite for unit tests; throwaway temp DBs for integration.
- A 10 000-row synthetic dataset benchmark verifies search response < 50 ms.
- Property tests for query AST round-trip (parse → serialise → parse).
