//! Shared git2 helper functions for worktree operations
//!
//! These functions encapsulate common git2 patterns used by multiple layout providers.

use std::path::Path;

use git2::{BranchType, MergeOptions, Repository, WorktreeAddOptions, WorktreePruneOptions};

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
/// Returns the branch name if HEAD points to a branch, or `None` if HEAD is detached.
pub fn get_current_branch(repo: &Repository) -> Result<Option<String>> {
    let head = repo.head()?;

    if head.is_branch() {
        Ok(Some(head.shorthand().unwrap_or("HEAD").to_string()))
    } else {
        Ok(None)
    }
}

/// Push a branch to a remote repository.
///
/// # Arguments
/// * `repo` - The repository to push from
/// * `remote` - Remote name (defaults to "origin" if None)
/// * `branch` - Branch to push (defaults to current branch if None)
/// * `current_branch_fn` - Function to get the current branch name
pub fn push(
    repo: &Repository,
    remote: Option<&str>,
    branch: Option<&str>,
    current_branch_fn: impl FnOnce() -> Result<String>,
) -> Result<()> {
    let remote_name = remote.unwrap_or("origin");
    let branch_name = match branch {
        Some(b) => b.to_string(),
        None => current_branch_fn()?,
    };

    let mut remote = repo
        .find_remote(remote_name)
        .map_err(|_| Error::RemoteNotFound {
            name: remote_name.to_string(),
        })?;

    let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);

    remote
        .push(&[&refspec], None)
        .map_err(|e| Error::PushFailed {
            message: e.message().to_string(),
        })?;

    Ok(())
}

/// Pull changes from a remote repository using fetch + fast-forward.
///
/// # Arguments
/// * `repo` - The repository to pull into
/// * `remote` - Remote name (defaults to "origin" if None)
/// * `branch` - Branch to pull (defaults to current branch if None)
/// * `current_branch_fn` - Function to get the current branch name
/// * `checkout_repo` - Optional different repo for checking out HEAD (e.g., main worktree)
pub fn pull(
    repo: &Repository,
    remote: Option<&str>,
    branch: Option<&str>,
    current_branch_fn: impl FnOnce() -> Result<String>,
    checkout_repo: Option<&Repository>,
) -> Result<()> {
    let remote_name = remote.unwrap_or("origin");
    let branch_name = match branch {
        Some(b) => b.to_string(),
        None => current_branch_fn()?,
    };

    let mut remote = repo
        .find_remote(remote_name)
        .map_err(|_| Error::RemoteNotFound {
            name: remote_name.to_string(),
        })?;

    remote
        .fetch(&[&branch_name], None, None)
        .map_err(|e| Error::PullFailed {
            message: format!("Fetch failed: {}", e.message()),
        })?;

    let fetch_head = repo
        .find_reference("FETCH_HEAD")
        .map_err(|e| Error::PullFailed {
            message: format!("Could not find FETCH_HEAD: {}", e.message()),
        })?;

    let fetch_commit = fetch_head.peel_to_commit().map_err(|e| Error::PullFailed {
        message: format!("Could not resolve FETCH_HEAD: {}", e.message()),
    })?;

    let head = repo.head()?;
    let head_commit = head.peel_to_commit()?;

    let (merge_analysis, _) =
        repo.merge_analysis(&[&repo.find_annotated_commit(fetch_commit.id())?])?;

    if merge_analysis.is_up_to_date() {
        return Ok(());
    }

    if merge_analysis.is_fast_forward() {
        let refname = format!("refs/heads/{}", branch_name);
        let mut reference = repo.find_reference(&refname)?;
        reference.set_target(
            fetch_commit.id(),
            &format!("pull: fast-forward to {}", fetch_commit.id()),
        )?;

        let co_repo = checkout_repo.unwrap_or(repo);
        co_repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        return Ok(());
    }

    Err(Error::CannotFastForward {
        message: format!(
            "Cannot fast-forward {} from {} to {}. Manual merge required.",
            branch_name,
            head_commit.id(),
            fetch_commit.id()
        ),
    })
}

/// Merge a source branch into the current branch.
///
/// # Arguments
/// * `repo` - The repository (used for branch lookup and merge analysis)
/// * `source` - The branch name to merge from
/// * `current_branch_fn` - Function to get the current branch name
/// * `merge_repo` - Optional different repo for performing the merge (e.g., main worktree)
pub fn merge(
    repo: &Repository,
    source: &str,
    current_branch_fn: impl FnOnce() -> Result<String>,
    merge_repo: Option<&Repository>,
) -> Result<()> {
    let source_branch =
        repo.find_branch(source, BranchType::Local)
            .map_err(|_| Error::BranchNotFound {
                name: source.to_string(),
            })?;

    let source_commit = source_branch.get().peel_to_commit()?;
    let annotated_commit = repo.find_annotated_commit(source_commit.id())?;

    let (merge_analysis, _) = repo.merge_analysis(&[&annotated_commit])?;

    if merge_analysis.is_up_to_date() {
        return Ok(());
    }

    if merge_analysis.is_fast_forward() {
        let current_branch = current_branch_fn()?;
        let refname = format!("refs/heads/{}", current_branch);
        let mut reference = repo.find_reference(&refname)?;
        reference.set_target(
            source_commit.id(),
            &format!("merge {}: fast-forward", source),
        )?;

        let co_repo = merge_repo.unwrap_or(repo);
        co_repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        return Ok(());
    }

    // Normal merge
    let mr = merge_repo.unwrap_or(repo);
    let mut merge_opts = MergeOptions::new();
    let annotated_for_merge = mr.find_annotated_commit(source_commit.id())?;
    mr.merge(&[&annotated_for_merge], Some(&mut merge_opts), None)?;

    let mut index = mr.index()?;
    if index.has_conflicts() {
        mr.cleanup_state()?;
        return Err(Error::MergeConflict {
            message: format!("Merge of '{}' resulted in conflicts", source),
        });
    }

    let signature = mr.signature()?;
    let tree_id = index.write_tree()?;
    let tree = mr.find_tree(tree_id)?;
    let head_commit = mr.head()?.peel_to_commit()?;
    let source_commit_in_mr = mr.find_commit(source_commit.id())?;

    let message = format!("Merge branch '{}'", source);
    mr.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &message,
        &tree,
        &[&head_commit, &source_commit_in_mr],
    )?;

    mr.cleanup_state()?;

    Ok(())
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
        assert!(branch == Some("main".to_string()) || branch == Some("master".to_string()));
    }
}
