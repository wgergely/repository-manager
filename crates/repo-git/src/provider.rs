//! Layout provider trait for git operations

use crate::Result;
use repo_fs::NormalizedPath;

/// Information about a worktree.
#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    /// Worktree name (matches branch name or slug)
    pub name: String,

    /// Filesystem path to the worktree
    pub path: NormalizedPath,

    /// Branch checked out in this worktree
    pub branch: String,

    /// Whether this is the main/primary worktree
    pub is_main: bool,
}

/// Trait for layout-agnostic git operations.
///
/// Implementations handle the specifics of each layout mode
/// (Container, InRepoWorktrees, Classic).
pub trait LayoutProvider {
    /// Path to the git database (.gt or .git)
    fn git_database(&self) -> &NormalizedPath;

    /// Path to the main branch worktree
    fn main_worktree(&self) -> &NormalizedPath;

    /// Compute path for a feature worktree by name
    fn feature_worktree(&self, name: &str) -> NormalizedPath;

    /// List all worktrees in this layout
    fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>>;

    /// Create a new feature worktree
    ///
    /// - `name`: Branch/worktree name
    /// - `base`: Optional base branch (defaults to current HEAD)
    fn create_feature(&self, name: &str, base: Option<&str>) -> Result<NormalizedPath>;

    /// Remove a feature worktree
    fn remove_feature(&self, name: &str) -> Result<()>;

    /// Get the current branch name
    fn current_branch(&self) -> Result<String>;

    /// Push the current branch to a remote repository.
    ///
    /// # Arguments
    /// * `remote` - Remote name (defaults to "origin" if None)
    /// * `branch` - Branch name to push (defaults to current branch if None)
    ///
    /// # Note
    /// This method requires credentials for authenticated remotes.
    /// Currently only supports remotes that don't require authentication
    /// or use credential helpers configured in git config.
    fn push(&self, remote: Option<&str>, branch: Option<&str>) -> Result<()>;

    /// Pull changes from a remote repository.
    ///
    /// # Arguments
    /// * `remote` - Remote name (defaults to "origin" if None)
    /// * `branch` - Branch name to pull (defaults to current branch if None)
    ///
    /// # Note
    /// Performs a fetch followed by a fast-forward merge.
    /// Returns an error if fast-forward is not possible.
    fn pull(&self, remote: Option<&str>, branch: Option<&str>) -> Result<()>;

    /// Merge a branch into the current branch.
    ///
    /// # Arguments
    /// * `source` - The branch name to merge from
    ///
    /// # Note
    /// Attempts a fast-forward merge first. If that's not possible,
    /// creates a merge commit. Returns an error if there are conflicts.
    fn merge(&self, source: &str) -> Result<()>;
}
