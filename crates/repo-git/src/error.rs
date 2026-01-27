//! Error types for repo-git

use std::path::PathBuf;

/// Result type for repo-git operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in repo-git operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("Filesystem error: {0}")]
    Fs(#[from] repo_fs::Error),

    #[error("Worktree '{name}' already exists at {path}")]
    WorktreeExists { name: String, path: PathBuf },

    #[error("Worktree '{name}' not found")]
    WorktreeNotFound { name: String },

    #[error("Branch '{name}' not found")]
    BranchNotFound { name: String },

    #[error("Operation '{operation}' not supported in {layout} layout. {hint}")]
    LayoutUnsupported {
        operation: String,
        layout: String,
        hint: String,
    },

    #[error("Invalid branch name: {name}")]
    InvalidBranchName { name: String },

    #[error("Remote '{name}' not found")]
    RemoteNotFound { name: String },

    #[error("No upstream branch configured for '{branch}'")]
    NoUpstreamBranch { branch: String },

    #[error("Merge conflict: {message}")]
    MergeConflict { message: String },

    #[error("Cannot fast-forward: {message}")]
    CannotFastForward { message: String },

    #[error("Push failed: {message}")]
    PushFailed { message: String },

    #[error("Pull failed: {message}")]
    PullFailed { message: String },
}
