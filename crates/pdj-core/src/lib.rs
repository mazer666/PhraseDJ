/// pdj-core — shared foundations for PhraseDJ.
///
/// This crate is the only one that all other crates are allowed to depend on.
/// It provides:
///   - `Error` / `Result` — project-wide error types
///   - `config` — settings loader (reads TOML, merges user overrides)
///   - `types` — shared IDs and value types (TrackId, etc.)
///
/// Nothing in this crate may depend on audio, UI, or network code.
pub mod config;
pub mod error;
pub mod types;

// Re-export the most common items at the crate root for convenience.
pub use config::{AudioSettings, LibrarySettings, LyricsSettings, NetworkSettings, Settings, StemsSettings, UiSettings};
pub use error::{Error, Result};
pub use types::TrackId;
