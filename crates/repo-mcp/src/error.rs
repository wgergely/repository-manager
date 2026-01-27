//! Error types for the MCP server

use thiserror::Error;

/// Result type alias for MCP operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during MCP server operations
#[derive(Debug, Error)]
pub enum Error {
    /// Error from the core repository logic
    #[error("core error: {0}")]
    Core(#[from] repo_core::Error),

    /// Error during JSON serialization/deserialization
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Invalid tool arguments
    #[error("invalid arguments: {message}")]
    InvalidArguments { message: String },

    /// Resource not found
    #[error("resource not found: {uri}")]
    ResourceNotFound { uri: String },

    /// Server not initialized
    #[error("server not initialized")]
    NotInitialized,
}
