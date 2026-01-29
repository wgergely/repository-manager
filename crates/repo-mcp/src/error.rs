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

    /// Unknown tool requested
    #[error("unknown tool: {0}")]
    UnknownTool(String),

    /// Invalid argument provided
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML parse error
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    /// TOML serialize error
    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    /// Tool not implemented
    #[error("tool not implemented: {0}")]
    NotImplemented(String),

    /// Unknown resource requested
    #[error("unknown resource: {0}")]
    UnknownResource(String),
}
