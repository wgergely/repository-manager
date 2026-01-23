//! Error types for repo-tools

use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Filesystem error: {0}")]
    Fs(#[from] repo_fs::Error),

    #[error("Block error: {0}")]
    Block(#[from] repo_blocks::error::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Tool config not found at {path}")]
    ConfigNotFound { path: PathBuf },

    #[error("Sync failed for {tool}: {message}")]
    SyncFailed { tool: String, message: String },
}
