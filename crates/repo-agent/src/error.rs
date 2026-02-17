//! Error types for agent operations

use std::path::PathBuf;

/// Errors that can occur during agent operations
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    /// Python interpreter not found or version too old
    #[error("Python 3.13+ not found. Install Python 3.13 or later for agent features.")]
    PythonNotFound,

    /// Python version is too old
    #[error("Python {found} is too old. Agent features require Python 3.13+.")]
    PythonVersionTooOld {
        /// The version that was found
        found: String,
    },

    /// Vaultspec framework directory not found
    #[error("Vaultspec not found at {path}. Run 'repo agent init' to set up agent orchestration.")]
    VaultspecNotFound {
        /// The path that was searched
        path: PathBuf,
    },

    /// I/O error during discovery or execution
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Error parsing command output
    #[error("Failed to parse output: {0}")]
    ParseError(String),

    /// Subprocess exited with non-zero status
    #[error("Command failed (exit code {code}): {stderr}")]
    CommandFailed {
        /// Exit code from the subprocess
        code: i32,
        /// Captured stderr output
        stderr: String,
    },
}

/// Result type alias for agent operations
pub type Result<T> = std::result::Result<T, AgentError>;
