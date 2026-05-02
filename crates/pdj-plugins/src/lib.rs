//! pdj-plugins — CLAP host, JS scripting sandbox, and MCP bridge.
//!
//! This crate provides the extensibility layer for PhraseDJ.

use pdj_core::Result;

/// Common interface for all plugin types (CLAP, JS, MCP).
pub trait Plugin {
    /// Unique identifier for the plugin (e.g. "io.phrasedj.echo").
    fn id(&self) -> &str;
    /// Human-readable name.
    fn name(&self) -> &str;
    /// Current version.
    fn version(&self) -> &str;
}

pub mod clap;
pub mod js;
pub mod mcp;

/// Discovery service for all local plugins.
pub fn scan_plugins() -> Result<()> {
    // Phase 2: Implement scanning for .clap bundles and .js scripts.
    Ok(())
}
