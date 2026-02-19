use std::path::PathBuf;

/// Errors that can occur in the extension system.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to parse extension manifest TOML.
    #[error("failed to parse extension manifest: {0}")]
    ManifestParse(#[from] toml::de::Error),

    /// Extension manifest file not found at the expected path.
    #[error("extension manifest not found: {0}")]
    ManifestNotFound(PathBuf),

    /// Invalid semver version string.
    #[error("invalid version '{version}': {source}")]
    InvalidVersion {
        version: String,
        source: semver::Error,
    },

    /// Invalid extension name.
    #[error("invalid extension name '{name}': {reason}")]
    InvalidName { name: String, reason: String },

    /// Failed to serialize extension manifest.
    #[error("failed to serialize extension manifest: {0}")]
    ManifestSerialize(String),

    /// I/O error reading or writing extension files.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Extension not found in registry.
    #[error("unknown extension: {0}")]
    UnknownExtension(String),
}

pub type Result<T> = std::result::Result<T, Error>;
