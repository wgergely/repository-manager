//! Error types for repo-content operations.

use thiserror::Error;

/// Errors that can occur during content operations.
#[derive(Debug, Error)]
pub enum Error {
    /// IO error during file operations.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Parse error for content formats.
    #[error("Parse error: {0}")]
    Parse(String),

    /// Edit operation failed.
    #[error("Edit failed: {0}")]
    EditFailed(String),
}

/// Result type alias for repo-content operations.
pub type Result<T> = std::result::Result<T, Error>;
