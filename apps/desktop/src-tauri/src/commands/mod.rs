//! commands/mod.rs — Bundle and re-export all Tauri commands.
//!
//! Each command is a thin Rust function annotated with `#[tauri::command]`
//! that the React frontend can call via `invoke('command_name', ...)`.
//!
//! Heavier logic lives in the relevant crate (`pdj-library`,
//! `pdj-engine-bridge`, etc.); commands only do parameter conversion and
//! error mapping.

pub mod app;
pub mod deck;
pub mod library;
pub mod mixer;

// Re-exported list passed to `tauri::generate_handler!`.
//
// Adding a new command means: write the function in the right module, add
// `pub use` here, and append to `invoke_handler!` in `lib.rs`.
pub use app::*;
pub use deck::*;
pub use library::*;
pub use mixer::*;
