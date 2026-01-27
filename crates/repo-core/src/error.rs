//! Error types for repo-core

use std::path::PathBuf;

/// Result type for repo-core operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in repo-core operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Configuration file not found at expected path
    #[error("Configuration not found at {path}")]
    ConfigNotFound { path: PathBuf },

    /// Invalid repository mode specified
    #[error("Invalid mode: {mode}")]
    InvalidMode { mode: String },

    /// Error in ledger operations
    #[error("Ledger error: {message}")]
    LedgerError { message: String },

    /// Intent not found in ledger
    #[error("Intent not found: {id}")]
    IntentNotFound { id: String },

    /// Projection failed for a tool
    #[error("Projection failed for {tool}: {reason}")]
    ProjectionFailed { tool: String, reason: String },

    /// Synchronization error
    #[error("Sync error: {message}")]
    SyncError { message: String },

    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),

    // Transparent wrappers for underlying crate errors
    /// Filesystem error from repo-fs
    #[error(transparent)]
    Fs(#[from] repo_fs::Error),

    /// Git error from repo-git
    #[error(transparent)]
    Git(#[from] repo_git::Error),

    /// Metadata error from repo-meta
    #[error(transparent)]
    Meta(#[from] repo_meta::Error),

    /// Tools error from repo-tools
    #[error(transparent)]
    Tools(#[from] repo_tools::Error),

    /// Presets error from repo-presets
    #[error(transparent)]
    Presets(#[from] repo_presets::Error),

    /// Content error from repo-content
    #[error(transparent)]
    Content(#[from] repo_content::Error),

    /// Standard I/O error
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// TOML deserialization error
    #[error(transparent)]
    TomlDe(#[from] toml::de::Error),

    /// TOML serialization error
    #[error(transparent)]
    TomlSer(#[from] toml::ser::Error),
}
