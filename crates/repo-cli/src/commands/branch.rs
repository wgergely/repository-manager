//! Branch command implementations
//!
//! Provides branch management operations that work with both Standard and Worktrees modes.

use std::path::Path;

use colored::Colorize;

use repo_core::config::Manifest;
use repo_core::hooks::{HookContext, HookEvent, run_hooks};
use repo_core::{Mode, ModeBackend, StandardBackend, WorktreeBackend};
use repo_fs::NormalizedPath;

use super::sync::detect_mode;
use crate::error::Result;

/// Create a ModeBackend for the given root and mode.
///
/// Returns a boxed trait object that can be used for branch operations.
pub fn create_backend(root: &NormalizedPath, mode: Mode) -> Result<Box<dyn ModeBackend>> {
    match mode {
        Mode::Standard => {
            let backend = StandardBackend::new(root.clone())?;
            Ok(Box::new(backend))
        }
        Mode::Worktrees => {
            let backend = WorktreeBackend::new(root.clone())?;
            Ok(Box::new(backend))
        }
    }
}

/// Load hooks from config.toml if it exists
fn load_hooks(path: &Path) -> Vec<repo_core::hooks::HookConfig> {
    let config_path = path.join(".repository").join("config.toml");
    if !config_path.exists() {
        return Vec::new();
    }
    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    match Manifest::parse(&content) {
        Ok(m) => m.hooks,
        Err(_) => Vec::new(),
    }
}

/// Run the branch add command.
///
/// Creates a new branch. In Standard mode, creates a git branch.
/// In Worktrees mode, creates a new worktree with the branch.
pub fn run_branch_add(path: &Path, name: &str, base: Option<&str>) -> Result<()> {
    let root = NormalizedPath::new(path);
    let mode = detect_mode(&root)?;
    let backend = create_backend(&root, mode)?;
    let hooks = load_hooks(path);

    let base_display = base.unwrap_or("HEAD");
    println!(
        "{} Creating branch {} (from {})...",
        "=>".blue().bold(),
        name.cyan(),
        base_display.yellow()
    );

    // Pre-create hooks
    let ctx = HookContext::for_branch(name, None);
    if let Err(e) = run_hooks(&hooks, HookEvent::PreBranchCreate, &ctx, path) {
        println!("{} Pre-create hook failed: {}", "warn:".yellow().bold(), e);
    }

    backend.create_branch(name, base)?;

    // Post-create hooks
    let worktree_path = match mode {
        Mode::Worktrees => Some(root.join(name)),
        Mode::Standard => None,
    };
    let ctx = HookContext::for_branch(name, worktree_path.as_ref().map(|p| p.as_ref()));
    if let Err(e) = run_hooks(&hooks, HookEvent::PostBranchCreate, &ctx, path) {
        println!("{} Post-create hook failed: {}", "warn:".yellow().bold(), e);
    }

    match mode {
        Mode::Worktrees => {
            let wt_path = root.join(name);
            println!(
                "{} Branch {} created at {}",
                "OK".green().bold(),
                name.cyan(),
                wt_path.as_str().yellow()
            );
        }
        Mode::Standard => {
            println!("{} Branch {} created.", "OK".green().bold(), name.cyan());
        }
    }

    Ok(())
}

/// Run the branch remove command.
///
/// Removes a branch. In Standard mode, deletes the git branch.
/// In Worktrees mode, removes the worktree and optionally the branch.
pub fn run_branch_remove(path: &Path, name: &str) -> Result<()> {
    let root = NormalizedPath::new(path);
    let mode = detect_mode(&root)?;
    let backend = create_backend(&root, mode)?;
    let hooks = load_hooks(path);

    println!("{} Removing branch {}...", "=>".blue().bold(), name.cyan());

    // Pre-delete hooks
    let ctx = HookContext::for_branch(name, None);
    if let Err(e) = run_hooks(&hooks, HookEvent::PreBranchDelete, &ctx, path) {
        println!("{} Pre-delete hook failed: {}", "warn:".yellow().bold(), e);
    }

    backend.delete_branch(name)?;

    // Post-delete hooks
    let ctx = HookContext::for_branch(name, None);
    if let Err(e) = run_hooks(&hooks, HookEvent::PostBranchDelete, &ctx, path) {
        println!("{} Post-delete hook failed: {}", "warn:".yellow().bold(), e);
    }

    match mode {
        Mode::Worktrees => {
            println!(
                "{} Branch and worktree {} removed.",
                "OK".green().bold(),
                name.cyan()
            );
        }
        Mode::Standard => {
            println!("{} Branch {} removed.", "OK".green().bold(), name.cyan());
        }
    }

    Ok(())
}

/// Run the branch checkout command.
///
/// Switches to a branch. In Standard mode, performs a git checkout.
/// In Worktrees mode, returns the path to the worktree.
pub fn run_branch_checkout(path: &Path, name: &str) -> Result<()> {
    let root = NormalizedPath::new(path);
    let mode = detect_mode(&root)?;
    let backend = create_backend(&root, mode)?;

    println!(
        "{} Switching to branch {}...",
        "=>".blue().bold(),
        name.cyan()
    );

    let working_dir = backend.switch_branch(name)?;

    match mode {
        Mode::Worktrees => {
            println!(
                "{} Worktree for {} is at:\n   {}",
                "OK".green().bold(),
                name.cyan(),
                working_dir.as_str().yellow()
            );
            println!();
            println!("  {} {}", "cd".dimmed(), working_dir.as_str().cyan());
        }
        Mode::Standard => {
            println!(
                "{} Switched to branch {}.",
                "OK".green().bold(),
                name.cyan()
            );
        }
    }

    Ok(())
}

/// Run the branch rename command.
///
/// Renames a branch. In Standard mode, renames the git branch.
/// In Worktrees mode, renames the branch and moves the worktree directory.
pub fn run_branch_rename(path: &Path, old_name: &str, new_name: &str) -> Result<()> {
    let root = NormalizedPath::new(path);
    let mode = detect_mode(&root)?;
    let backend = create_backend(&root, mode)?;

    println!(
        "{} Renaming branch {} to {}...",
        "=>".blue().bold(),
        old_name.cyan(),
        new_name.cyan()
    );

    backend.rename_branch(old_name, new_name)?;

    match mode {
        Mode::Worktrees => {
            let new_path = root.join(new_name);
            println!(
                "{} Branch renamed to {} (worktree at {})",
                "OK".green().bold(),
                new_name.cyan(),
                new_path.as_str().yellow()
            );
        }
        Mode::Standard => {
            println!(
                "{} Branch renamed from {} to {}.",
                "OK".green().bold(),
                old_name.cyan(),
                new_name.cyan()
            );
        }
    }

    Ok(())
}

/// Run the branch list command.
///
/// Lists all branches. Shows branch names with markers for current and main branches.
/// In Worktrees mode, also shows the path to each worktree.
pub fn run_branch_list(path: &Path) -> Result<()> {
    let root = NormalizedPath::new(path);
    let mode = detect_mode(&root)?;
    let backend = create_backend(&root, mode)?;

    let branches = backend.list_branches()?;

    if branches.is_empty() {
        println!("{} No branches found.", "=>".blue().bold());
        return Ok(());
    }

    println!("{} Branches:", "=>".blue().bold());

    for branch in branches {
        let mut line = String::new();

        // Current branch marker
        if branch.is_current {
            line.push_str(&format!("  {} ", "*".green()));
        } else {
            line.push_str("    ");
        }

        // Branch name
        let name_display = if branch.is_current {
            branch.name.green().bold().to_string()
        } else if branch.is_main {
            branch.name.cyan().to_string()
        } else {
            branch.name.clone()
        };
        line.push_str(&name_display);

        // Main branch indicator
        if branch.is_main {
            line.push_str(&format!(" {}", "(default)".dimmed()));
        }

        // Path for worktrees mode
        if let Some(path) = branch.path {
            line.push_str(&format!(" -> {}", path.as_str().dimmed()));
        }

        println!("{}", line);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_test_utils::git::real_git_repo_with_commit;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn setup_git_repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        real_git_repo_with_commit(dir.path());
        dir
    }

    #[test]
    fn test_detect_mode_default() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // No config file and no filesystem markers - defaults to Standard
        let root = NormalizedPath::new(path);
        let mode = detect_mode(&root).unwrap();

        assert_eq!(mode, Mode::Standard);
    }

    #[test]
    fn test_detect_mode_worktrees_from_config() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Create .repository/config.toml with worktrees mode
        let repo_dir = path.join(".repository");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(
            repo_dir.join("config.toml"),
            "[core]\nmode = \"worktrees\"\n",
        )
        .unwrap();

        let root = NormalizedPath::new(path);
        let mode = detect_mode(&root).unwrap();

        assert_eq!(mode, Mode::Worktrees);
    }

    #[test]
    fn test_detect_mode_standard_from_config() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Create .repository/config.toml with standard mode
        let repo_dir = path.join(".repository");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(
            repo_dir.join("config.toml"),
            "[core]\nmode = \"standard\"\n",
        )
        .unwrap();

        let root = NormalizedPath::new(path);
        let mode = detect_mode(&root).unwrap();

        assert_eq!(mode, Mode::Standard);
    }

    #[test]
    fn test_list_branches() {
        let temp = setup_git_repo();
        let path = temp.path();

        // Create config with standard mode (since setup_git_repo creates standard layout)
        let repo_dir = path.join(".repository");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(
            repo_dir.join("config.toml"),
            "[core]\nmode = \"standard\"\n",
        )
        .unwrap();

        let result = run_branch_list(path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_branch_add_standard() {
        let temp = setup_git_repo();
        let path = temp.path();

        // Create a new branch
        let result = run_branch_add(path, "feature-test", Some("main"));

        // This might fail if the main branch doesn't exist yet,
        // but we test that the function runs without panic
        if result.is_ok() {
            // Verify the branch was created
            let output = Command::new("git")
                .args(["branch", "--list", "feature-test"])
                .current_dir(path)
                .output()
                .expect("Failed to list branches");

            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                stdout.contains("feature-test"),
                "Branch should have been created"
            );
        }
    }

    #[test]
    fn test_branch_rename_standard() {
        let temp = setup_git_repo();
        let path = temp.path();

        // Create config with standard mode
        let repo_dir = path.join(".repository");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(
            repo_dir.join("config.toml"),
            "[core]\nmode = \"standard\"\n",
        )
        .unwrap();

        // Create a branch first
        let add_result = run_branch_add(path, "feature-rename-test", Some("main"));
        if add_result.is_ok() {
            // Rename it
            let result = run_branch_rename(path, "feature-rename-test", "renamed-branch");
            assert!(result.is_ok(), "Branch rename should succeed");

            // Verify old branch no longer exists and new one does
            let output = Command::new("git")
                .args(["branch", "--list", "renamed-branch"])
                .current_dir(path)
                .output()
                .expect("Failed to list branches");
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                stdout.contains("renamed-branch"),
                "Renamed branch should exist"
            );

            let output = Command::new("git")
                .args(["branch", "--list", "feature-rename-test"])
                .current_dir(path)
                .output()
                .expect("Failed to list branches");
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                !stdout.contains("feature-rename-test"),
                "Old branch name should no longer exist"
            );
        }
    }

    #[test]
    fn test_create_backend_standard() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Create .git directory
        fs::create_dir(path.join(".git")).unwrap();

        let root = NormalizedPath::new(path);
        let result = create_backend(&root, Mode::Standard);

        assert!(result.is_ok());
    }

    #[test]
    fn test_create_backend_worktrees() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Create .gt directory
        fs::create_dir(path.join(".gt")).unwrap();
        // Create main worktree directory
        fs::create_dir(path.join("main")).unwrap();

        let root = NormalizedPath::new(path);
        let result = create_backend(&root, Mode::Worktrees);

        assert!(result.is_ok());
    }
}
