//! Layout provider trait and types
//!
//! This module will be implemented in Task 2.3.

use std::path::PathBuf;

use crate::Result;

/// Information about a worktree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorktreeInfo {
    /// Name of the worktree (usually the branch name).
    pub name: String,
    /// Path to the worktree directory.
    pub path: PathBuf,
    /// Branch checked out in the worktree.
    pub branch: Option<String>,
    /// Whether this is the main worktree.
    pub is_main: bool,
}

/// Trait for different worktree layout strategies.
pub trait LayoutProvider: Send + Sync {
    /// List all worktrees managed by this layout.
    fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>>;

    /// Create a new worktree for the given branch.
    fn create_worktree(&self, branch: &str) -> Result<WorktreeInfo>;

    /// Remove a worktree by name.
    fn remove_worktree(&self, name: &str) -> Result<()>;

    /// Get information about a specific worktree.
    fn get_worktree(&self, name: &str) -> Result<Option<WorktreeInfo>>;
}
