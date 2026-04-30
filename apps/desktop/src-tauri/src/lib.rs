/// PhraseDJ Tauri application library.
///
/// Exports the `run` function called by `main.rs`.  All Tauri commands are
/// registered here.  Commands are the bridge between the React frontend and
/// the Rust backend (pdj-core and other crates).
///
/// Phase 0: minimal setup – just gets a window on screen.
/// Phase 1: audio engine commands added here.
use tracing_subscriber::{fmt, EnvFilter};

/// Initialise logging and start the Tauri event loop.
///
/// This function never returns unless Tauri panics.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Set up structured logging.  The RUST_LOG environment variable
    // controls the level; default is "info".
    fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new("info")
        }))
        .init();

    tracing::info!("PhraseDJ starting");

    tauri::Builder::default()
        // Register Tauri plugins.
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        // Register application commands (empty for Phase 0).
        .invoke_handler(tauri::generate_handler![
            app_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running PhraseDJ");
}

/// Returns the application version string from Cargo.toml.
///
/// The React frontend calls this on startup to show the version in the header.
#[tauri::command]
fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
