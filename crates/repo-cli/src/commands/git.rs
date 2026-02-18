//! Git command implementations (push, pull, merge)
//!
//! These commands use repo-git's free functions for network operations
//! and LayoutProvider for repo/branch discovery.

use std::path::Path;

use colored::Colorize;
use git2::Repository;

use repo_fs::NormalizedPath;
use repo_git::{ClassicLayout, ContainerLayout, LayoutProvider};

use super::sync::detect_mode;
use crate::error::Result;
use repo_core::Mode;

/// Create a LayoutProvider for git operations based on detected mode.
fn create_git_provider(root: &NormalizedPath, mode: Mode) -> Result<Box<dyn LayoutProvider>> {
    match mode {
        Mode::Standard => {
            let layout = ClassicLayout::new(root.clone())?;
            Ok(Box::new(layout))
        }
        Mode::Worktrees => {
            let layout = ContainerLayout::new(root.clone(), Default::default())?;
            Ok(Box::new(layout))
        }
    }
}

/// Run the push command.
///
/// Pushes the current branch to the specified remote.
pub fn run_push(path: &Path, remote: Option<&str>, branch: Option<&str>) -> Result<()> {
    let root = NormalizedPath::new(path);
    let mode = detect_mode(&root)?;
    let provider = create_git_provider(&root, mode)?;
    let repo =
        Repository::open(provider.main_worktree().to_native()).map_err(repo_git::Error::from)?;

    let remote_name = remote.unwrap_or("origin");
    let branch_display = branch.unwrap_or("current branch");

    println!(
        "{} Pushing {} to {}...",
        "=>".blue().bold(),
        branch_display.cyan(),
        remote_name.yellow()
    );

    let current_branch_fn = || provider.current_branch();
    repo_git::push(&repo, remote, branch, current_branch_fn)?;

    println!(
        "{} Successfully pushed to {}",
        "OK".green().bold(),
        remote_name.yellow()
    );

    Ok(())
}

/// Run the pull command.
///
/// Pulls changes from the specified remote.
pub fn run_pull(path: &Path, remote: Option<&str>, branch: Option<&str>) -> Result<()> {
    let root = NormalizedPath::new(path);
    let mode = detect_mode(&root)?;
    let provider = create_git_provider(&root, mode)?;
    let repo =
        Repository::open(provider.main_worktree().to_native()).map_err(repo_git::Error::from)?;

    let remote_name = remote.unwrap_or("origin");
    let branch_display = branch.unwrap_or("current branch");

    println!(
        "{} Pulling {} from {}...",
        "=>".blue().bold(),
        branch_display.cyan(),
        remote_name.yellow()
    );

    let current_branch_fn = || provider.current_branch();
    repo_git::pull(&repo, remote, branch, current_branch_fn, None)?;

    println!(
        "{} Successfully pulled from {}",
        "OK".green().bold(),
        remote_name.yellow()
    );

    Ok(())
}

/// Run the merge command.
///
/// Merges the source branch into the current branch.
pub fn run_merge(path: &Path, source: &str) -> Result<()> {
    let root = NormalizedPath::new(path);
    let mode = detect_mode(&root)?;
    let provider = create_git_provider(&root, mode)?;
    let repo =
        Repository::open(provider.main_worktree().to_native()).map_err(repo_git::Error::from)?;

    println!(
        "{} Merging {} into current branch...",
        "=>".blue().bold(),
        source.cyan()
    );

    let current_branch_fn = || provider.current_branch();
    repo_git::merge(&repo, source, current_branch_fn, None)?;

    println!(
        "{} Successfully merged {}",
        "OK".green().bold(),
        source.cyan()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    // Integration tests require real git repos - tested in mission_tests.rs
}
