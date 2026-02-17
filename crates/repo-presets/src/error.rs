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

    #[error("Virtual environment creation failed at {path}")]
    VenvCreationFailed { path: String },

    #[error("Preset check failed: {message}")]
    CheckFailed { message: String },

    #[error("Failed to clone git repository {url}: {message}")]
    GitClone { url: String, message: String },

    #[error("Failed to read plugin manifest at {path}: {message}")]
    PluginManifest { path: String, message: String },

    #[error("Failed to update Claude settings: {0}")]
    ClaudeSettings(String),

    #[error("Plugin not installed")]
    PluginNotInstalled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_clone_error_display() {
        let err = Error::GitClone {
            url: "https://github.com/anthropics/claude-code-plugins".to_string(),
            message: "network error".to_string(),
        };
        assert!(err.to_string().contains("plugins"));
        assert!(err.to_string().contains("network error"));
    }

    #[test]
    fn test_plugin_manifest_error_display() {
        let err = Error::PluginManifest {
            path: "/path/to/plugin.json".to_string(),
            message: "invalid JSON".to_string(),
        };
        assert!(err.to_string().contains("plugin.json"));
    }
}
