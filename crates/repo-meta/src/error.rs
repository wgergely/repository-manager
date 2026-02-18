//! Error types for repo-meta

use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Filesystem error: {0}")]
    Fs(#[from] repo_fs::Error),

    #[error("Configuration not found at {path}")]
    ConfigNotFound { path: PathBuf },

    #[error("Invalid configuration at {path}: {message}")]
    InvalidConfig { path: PathBuf, message: String },

    #[error("Config file too large: {path} is {size} bytes (max {max})")]
    ConfigTooLarge { path: PathBuf, size: u64, max: u64 },

    #[error("Preset not found: {id}")]
    PresetNotFound { id: String },

    #[error("Tool not found: {id}")]
    ToolNotFound { id: String },

    #[error("Rule not found: {id}")]
    RuleNotFound { id: String },

    #[error("Provider not registered for preset: {preset_id}")]
    ProviderNotRegistered { preset_id: String },

    #[error("Invalid mode: {mode}")]
    InvalidMode { mode: String },
}
