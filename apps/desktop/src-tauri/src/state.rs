//! state.rs — Application state shared between Tauri commands.
//!
//! Each Tauri command accesses the audio engine and the library through
//! this struct via `tauri::State`.  The engine and library are wrapped in
//! `Arc<Mutex<…>>` so that multiple async commands can safely share them.

use std::path::PathBuf;
use std::sync::Mutex;

use pdj_engine_bridge::{Engine, EngineConfig};
use pdj_library::Library;
use tracing::warn;

/// Shared application state.
///
/// Stored as a Tauri-managed singleton via `app.manage(AppState::new(...))`.
pub struct AppState {
    /// The audio engine.  Optional because the audio device may be missing
    /// (e.g. CI / sandbox); the rest of the app should still load.
    pub engine: Mutex<Option<Engine>>,

    /// The local music library.
    pub library: Mutex<Library>,

    /// Path to the database file (for diagnostics).
    pub db_path: PathBuf,
}

impl AppState {
    /// Initialise audio engine and library.
    ///
    /// `db_path` is the path to the SQLite library database.
    /// `engine_config` configures the audio output.
    pub fn init(db_path: PathBuf, engine_config: EngineConfig) -> pdj_core::Result<Self> {
        let library = Library::open(&db_path)?;

        // Try to open the audio engine.  Failures are non-fatal — the UI
        // shell should still launch.
        let engine = match Engine::new(engine_config) {
            Ok(e) => Some(e),
            Err(err) => {
                warn!(?err, "audio engine unavailable — running headless");
                None
            }
        };

        Ok(Self {
            engine:  Mutex::new(engine),
            library: Mutex::new(library),
            db_path,
        })
    }
}
