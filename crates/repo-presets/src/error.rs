//! Error types for repo-presets

use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Filesystem error: {0}")]
    Fs(#[from] repo_fs::Error),

    #[error("Meta error: {0}")]
    Meta(#[from] repo_meta::Error),

    #[error("Command failed: {command}")]
    CommandFailed { command: String },

    #[error("Command not found: {command}")]
    CommandNotFound { command: String },

    #[error("Environment creation failed at {path}: {message}")]
    EnvCreationFailed { path: PathBuf, message: String },

    #[error("Python not found. Install Python or uv first.")]
    PythonNotFound,

    #[error("uv not found. Install uv: https://docs.astral.sh/uv/")]
    UvNotFound,

    #[error("Preset check failed: {message}")]
    CheckFailed { message: String },
}
