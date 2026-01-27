//! Error types for repo-cli

/// Result type for CLI operations
pub type Result<T> = std::result::Result<T, CliError>;

/// Errors that can occur in CLI operations
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)] // User variant will be used as CLI is implemented
pub enum CliError {
    /// Error from repo-core
    #[error(transparent)]
    Core(#[from] repo_core::Error),

    /// Error from repo-fs
    #[error(transparent)]
    Fs(#[from] repo_fs::Error),

    /// Standard I/O error
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Interactive prompt error
    #[error("Interactive prompt error: {0}")]
    Dialoguer(#[from] dialoguer::Error),

    /// User-facing error with a message
    #[error("{message}")]
    User { message: String },
}

impl CliError {
    /// Create a new user error with the given message
    #[allow(dead_code)] // Will be used as CLI commands are implemented
    pub fn user(message: impl Into<String>) -> Self {
        Self::User {
            message: message.into(),
        }
    }
}
