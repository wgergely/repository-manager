//! Shared git2 helper functions for worktree operations
//!
//! These functions encapsulate common git2 patterns used by multiple layout providers.

use std::path::Path;

use git2::{BranchType, Repository, WorktreeAddOptions, WorktreePruneOptions};

use crate::{Error, Result};

/// Create a new worktree with an associated branch.
///
/// This creates a new local branch based on `base` (or HEAD if None),
/// then creates a worktree at `worktree_path` checked out to that branch.
///
/// # Arguments
/// * `repo` - The repository to create the worktree in
/// * `worktree_path` - Where to create the worktree
/// * `branch_name` - Name for both the branch and worktree
/// * `base` - Optional base branch name (uses HEAD if None)
pub fn create_worktree_with_branch(
    repo: &Repository,
    worktree_path: &Path,
    branch_name: &str,
    base: Option<&str>,
) -> Result<()> {
    // Get the commit to base the new branch on
    let base_commit = match base {
        Some(base_name) => {
            let branch = repo
                .find_branch(base_name, BranchType::Local)
                .map_err(|_| Error::BranchNotFound {
                    name: base_name.to_string(),
                })?;
            branch.get().peel_to_commit()?
        }
        None => {
            let head = repo.head()?;
            head.peel_to_commit()?
        }
    };

    // Create a new branch for the feature worktree
    let new_branch = repo.branch(branch_name, &base_commit, false)?;
    let new_branch_ref = new_branch.into_reference();

    // Create worktree with the new branch
    let mut opts = WorktreeAddOptions::new();
    opts.reference(Some(&new_branch_ref));

    repo.worktree(branch_name, worktree_path, Some(&opts))?;

    Ok(())
}

/// Remove a worktree and optionally its associated branch.
///
/// This prunes the worktree (removing the working directory and git references),
/// then attempts to delete the branch. Branch deletion failures are logged but
/// not treated as errors (the branch may be in use elsewhere).
///
/// # Arguments
/// * `repo` - The repository containing the worktree
/// * `name` - The name of the worktree/branch to remove
pub fn remove_worktree_and_branch(repo: &Repository, name: &str) -> Result<()> {
    let wt = repo
        .find_worktree(name)
        .map_err(|_| Error::WorktreeNotFound {
            name: name.to_string(),
        })?;

    // Configure prune options to remove valid worktrees and their directories
    let mut prune_opts = WorktreePruneOptions::new();
    prune_opts.valid(true); // Allow pruning valid (existing) worktrees
    prune_opts.working_tree(true); // Also remove the working tree directory

    // Prune the worktree (removes directory and git references)
    wt.prune(Some(&mut prune_opts))?;

    // Also try to delete the branch
    if let Ok(mut branch) = repo.find_branch(name, BranchType::Local)
        && let Err(e) = branch.delete()
    {
        tracing::warn!(
            branch = %name,
            error = %e,
            "Failed to delete branch after worktree removal"
        );
    }

    Ok(())
}

/// Get the current branch name from a repository.
///
/// Returns the branch name if HEAD points to a branch, or "HEAD" if detached.
pub fn get_current_branch(repo: &Repository) -> Result<String> {
    let head = repo.head()?;

    if head.is_branch() {
        Ok(head.shorthand().unwrap_or("HEAD").to_string())
    } else {
        Ok("HEAD".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_get_current_branch_on_main() {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();

        // Create an initial commit so HEAD points to a branch
        let sig = repo.signature().unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Initial", &tree, &[])
            .unwrap();

        let branch = get_current_branch(&repo).unwrap();
        // Default branch is either "main" or "master" depending on git config
        assert!(branch == "main" || branch == "master");
    }
}
