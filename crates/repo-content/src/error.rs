//! Error types for repo-content

use std::ops::Range;
use uuid::Uuid;

/// Result type for repo-content operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in repo-content operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to parse {format} content: {message}")]
    ParseError { format: String, message: String },

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Block not found: {uuid}")]
    BlockNotFound { uuid: Uuid },

    #[error("Invalid block marker at byte {position}: {message}")]
    InvalidBlockMarker { position: usize, message: String },

    #[error("Block markers overlap at range {0:?}")]
    OverlappingBlocks(Range<usize>),

    #[error("Path not found: {path}")]
    PathNotFound { path: String },

    #[error("Cannot set path in {format} document: {reason}")]
    PathSetFailed {
        format: String,
        path: String,
        reason: String,
    },

    #[error("Checksum mismatch for block {uuid}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        uuid: Uuid,
        expected: String,
        actual: String,
    },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl Error {
    pub fn parse(format: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ParseError {
            format: format.into(),
            message: message.into(),
        }
    }
}
