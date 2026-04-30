/// Project-wide error type.
///
/// Each sub-crate wraps its own domain errors into a variant here so the
/// application layer can handle them uniformly.  The `#[from]` attribute on
/// each variant lets Rust convert automatically with `?`.
use thiserror::Error;

/// The canonical result alias used across all PhraseDJ crates.
pub type Result<T> = std::result::Result<T, Error>;

/// All errors that can occur inside PhraseDJ.
#[derive(Debug, Error)]
pub enum Error {
    /// A settings file could not be read or parsed.
    #[error("settings error: {0}")]
    Settings(String),

    /// A required file was not found on disk.
    #[error("file not found: {path}")]
    FileNotFound { path: String },

    /// An I/O operation failed.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A TOML deserialisation error.
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    /// A JSON error (used for macro/schema files).
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// A SQLite database error.  Added when pdj-library is compiled in.
    #[error("database error: {0}")]
    Database(String),

    /// An error from an external library that we wrap as a string to avoid
    /// leaking the dependency into the public API.
    #[error("{0}")]
    Other(String),
}

impl Error {
    /// Convenience constructor for `Error::Other`.
    pub fn other(msg: impl Into<String>) -> Self {
        Error::Other(msg.into())
    }
}
