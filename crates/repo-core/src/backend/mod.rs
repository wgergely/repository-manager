//! Mode backend abstraction
//!
//! Provides a unified interface for repository operations across different
//! repository layouts (Standard and Worktrees).

mod standard;
mod worktree;

pub use standard::StandardBackend;
pub use worktree::WorktreeBackend;

use crate::Result;
use repo_fs::NormalizedPath;

/// Information about a branch in the repository.
#[derive(Debug, Clone)]
pub struct BranchInfo {
    /// Branch name
    pub name: String,

    /// Filesystem path to the worktree (Some for worktrees mode, None for standard)
    pub path: Option<NormalizedPath>,

    /// Whether this is the currently active branch/worktree
    pub is_current: bool,

    /// Whether this is the main/primary branch
    pub is_main: bool,
}

impl BranchInfo {
    /// Create a new BranchInfo for standard mode (no path).
    pub fn standard(name: impl Into<String>, is_current: bool, is_main: bool) -> Self {
        Self {
            name: name.into(),
            path: None,
            is_current,
            is_main,
        }
    }

    /// Create a new BranchInfo for worktree mode (with path).
    pub fn worktree(
        name: impl Into<String>,
        path: NormalizedPath,
        is_current: bool,
        is_main: bool,
    ) -> Self {
        Self {
            name: name.into(),
            path: Some(path),
            is_current,
            is_main,
        }
    }
}

/// Trait for mode-specific repository operations.
///
/// This trait abstracts the differences between Standard and Worktree modes,
/// providing a unified interface for branch and worktree management.
pub trait ModeBackend: Send + Sync {
    /// Get the path to the configuration root (`.repository` directory).
    ///
    /// For Standard mode: `{repo}/.repository`
    /// For Worktrees mode: `{container}/.repository` (shared across worktrees)
    fn config_root(&self) -> NormalizedPath;

    /// Get the current working directory for this backend.
    ///
    /// For Standard mode: The repository root
    /// For Worktrees mode: The active worktree directory
    fn working_dir(&self) -> &NormalizedPath;

    /// Create a new branch.
    ///
    /// In Standard mode, this creates a branch but does not switch to it.
    /// In Worktrees mode, this creates a new worktree with the branch.
    ///
    /// # Arguments
    /// - `name`: Branch name
    /// - `base`: Optional base branch/commit (defaults to current HEAD)
    fn create_branch(&self, name: &str, base: Option<&str>) -> Result<()>;

    /// Delete a branch.
    ///
    /// In Standard mode, this deletes the branch.
    /// In Worktrees mode, this removes the worktree and optionally the branch.
    fn delete_branch(&self, name: &str) -> Result<()>;

    /// List all branches in the repository.
    ///
    /// Returns branch information including paths for worktree mode.
    fn list_branches(&self) -> Result<Vec<BranchInfo>>;

    /// Switch to a branch and return the working directory path.
    ///
    /// In Standard mode, this performs a `git checkout`.
    /// In Worktrees mode, this returns the path to the worktree (creating it if needed).
    fn switch_branch(&self, name: &str) -> Result<NormalizedPath>;

    /// Rename a branch.
    ///
    /// In Standard mode, this renames the git branch.
    /// In Worktrees mode, this renames both the branch and moves the worktree directory.
    fn rename_branch(&self, old_name: &str, new_name: &str) -> Result<()>;
}
