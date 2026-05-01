//! pdj-library — Local music library backed by SQLite.
//!
//! Phase 1 scope:
//!   - Open / create the database file with WAL mode.
//!   - Insert and query tracks.
//!   - Scan a folder for audio files (recursive, single-threaded).
//!   - Search by title / artist (FTS5 added in Phase 2).
//!
//! Files live where the user keeps them — the library only stores paths,
//! metadata, and analysis state.  Stems, lyrics, etc. are tracked here
//! but stored in the cache directory by other crates.

pub mod connect;
pub mod import;
pub mod schema;
pub mod search;

pub use connect::Library;
pub use import::{import_file, scan_folder, ImportOutcome, ScanReport};
pub use schema::{AnalysisState, StemsState, Track};
pub use search::{search, Query};
