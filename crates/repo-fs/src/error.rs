//! Error types for repo-fs

use std::path::PathBuf;

/// Result type for repo-fs operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in repo-fs operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse {format} config at {path}: {message}")]
    ConfigParse {
        path: PathBuf,
        format: String,
        message: String,
    },

    #[error("Unsupported config format: {extension}")]
    UnsupportedFormat { extension: String },

    #[error("Layout validation failed: {message}")]
    LayoutValidation { message: String },

    #[error("Could not detect layout mode from filesystem")]
    LayoutDetectionFailed,

    #[error("Config declares {declared:?} mode but filesystem shows {detected:?}")]
    LayoutMismatch {
        declared: super::LayoutMode,
        detected: super::LayoutMode,
    },

    #[error("Network path detected: {path}. Performance may be degraded.")]
    NetworkPathWarning { path: PathBuf },

    #[error("Lock acquisition failed for {path}")]
    LockFailed { path: PathBuf },
}

impl Error {
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
