//! Sync, check, and fix command implementations
//!
//! These commands manage synchronization state between the ledger and filesystem.

use std::path::Path;

use colored::Colorize;
use serde_json::json;

use repo_core::config::Manifest;
use repo_core::hooks::{HookContext, HookEvent, run_hooks};
use repo_core::{CheckStatus, Mode, SyncEngine, SyncOptions};
use repo_fs::NormalizedPath;

use crate::context::{RepoContext, detect_context};
use crate::error::{CliError, Result};

/// Resolve the repository root from any path within the repo
///
/// Uses context detection to find the correct root:
/// - In worktrees mode: returns container root
/// - In standard mode: returns repo root
/// - Not in a repo: returns error
pub fn resolve_root(path: &Path) -> Result<NormalizedPath> {
    let context = detect_context(path);

    match context {
        RepoContext::ContainerRoot { path } => Ok(NormalizedPath::new(&path)),
        RepoContext::Worktree { container, .. } => Ok(NormalizedPath::new(&container)),
        RepoContext::StandardRepo { path } => Ok(NormalizedPath::new(&path)),
        RepoContext::NotARepo => Err(CliError::user(
            "Not in a repository. Run 'repo init' to create one.",
        )),
    }
}

/// Detect the repository mode from config.toml
///
/// Delegates to [`repo_core::detect_mode`] which checks filesystem markers
/// (`.gt`, `.git`) and falls back to `.repository/config.toml` via ConfigResolver.
/// Defaults to Standard mode when no indicators are found.
pub fn detect_mode(root: &NormalizedPath) -> Result<Mode> {
    Ok(repo_core::detect_mode(root)?)
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

/// Run the check command
///
/// Validates that the filesystem matches the ledger state.
pub fn run_check(path: &Path) -> Result<()> {
    println!(
        "{} Checking repository configuration...",
        "=>".blue().bold()
    );

    let root = resolve_root(path)?;
    let mode = detect_mode(&root)?;
    let engine = SyncEngine::new(root, mode)?;

    let report = engine.check()?;

    match report.status {
        CheckStatus::Healthy => {
            println!(
                "{} Repository is healthy. No drift detected.",
                "OK".green().bold()
            );
        }
        CheckStatus::Missing => {
            println!("{} Some files are missing:", "MISSING".yellow().bold());
            for item in &report.missing {
                println!(
                    "   {} {} ({}): {}",
                    "-".yellow(),
                    item.file.cyan(),
                    item.tool.dimmed(),
                    item.description
                );
            }
            println!();
            println!("Run {} to repair.", "repo fix".cyan());
        }
        CheckStatus::Drifted => {
            println!("{} Configuration has drifted:", "DRIFTED".red().bold());
            for item in &report.drifted {
                println!(
                    "   {} {} ({}): {}",
                    "!".red(),
                    item.file.cyan(),
                    item.tool.dimmed(),
                    item.description
                );
            }
            if !report.missing.is_empty() {
                println!();
                println!("{} Also missing:", "MISSING".yellow().bold());
                for item in &report.missing {
                    println!(
                        "   {} {} ({}): {}",
                        "-".yellow(),
                        item.file.cyan(),
                        item.tool.dimmed(),
                        item.description
                    );
                }
            }
            println!();
            println!("Run {} to repair.", "repo fix".cyan());
        }
        CheckStatus::Broken => {
            println!("{} Repository is in a broken state:", "BROKEN".red().bold());
            for msg in &report.messages {
                println!("   {} {}", "!".red(), msg);
            }
            println!();
            println!("Manual intervention may be required.");
        }
    }

    Ok(())
}

/// Run the sync command
///
/// Synchronizes configuration from the ledger to the filesystem.
pub fn run_sync(path: &Path, dry_run: bool, json_output: bool) -> Result<()> {
    let root = resolve_root(path)?;
    let mode = detect_mode(&root)?;
    let hooks = load_hooks(root.as_ref());
    let engine = SyncEngine::new(root.clone(), mode)?;

    // Pre-sync hooks
    let hook_context = HookContext::for_sync();
    if let Err(e) = run_hooks(&hooks, HookEvent::PreSync, &hook_context, root.as_ref()) {
        println!("{} Pre-sync hook failed: {}", "warn:".yellow().bold(), e);
    }

    let options = SyncOptions { dry_run };
    let report = engine.sync_with_options(options)?;

    if json_output {
        // JSON output for CI/CD integration
        let output = json!({
            "dry_run": dry_run,
            "success": report.success,
            "has_changes": !report.actions.is_empty(),
            "changes": report.actions.iter()
                .map(|a| {
                    let clean = a.strip_prefix("[dry-run] Would ").unwrap_or(a);
                    json!({
                        "action": clean,
                        "type": categorize_action(clean),
                    })
                })
                .collect::<Vec<_>>(),
            "errors": report.errors,
            "root": root.as_str(),
            "mode": mode.to_string(),
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        // Human-readable colored output
        if dry_run {
            println!("{} Previewing sync (dry-run)...", "=>".blue().bold());
        } else {
            println!(
                "{} Synchronizing tool configurations...",
                "=>".blue().bold()
            );
        }

        if report.success {
            if report.actions.is_empty() {
                println!(
                    "{} Already synchronized. No changes needed.",
                    "OK".green().bold()
                );
            } else {
                let prefix = if dry_run {
                    "Would take actions"
                } else {
                    "Synchronization complete"
                };
                println!("{} {}:", "OK".green().bold(), prefix);
                for action in &report.actions {
                    let clean = action.strip_prefix("[dry-run] Would ").unwrap_or(action);
                    let (prefix_char, colored_action) = format_action(clean);
                    println!("   {} {}", prefix_char, colored_action);
                }
            }
        } else {
            println!("{} Synchronization failed:", "ERROR".red().bold());
            for error in &report.errors {
                println!("   {} {}", "!".red(), error);
            }
            return Err(CliError::user("Synchronization failed"));
        }
    }

    // Post-sync hooks (only after successful sync)
    if report.success
        && let Err(e) = run_hooks(&hooks, HookEvent::PostSync, &hook_context, root.as_ref())
    {
        println!("{} Post-sync hook failed: {}", "warn:".yellow().bold(), e);
    }

    Ok(())
}

/// Categorize an action for JSON output
fn categorize_action(action: &str) -> &'static str {
    let lower = action.to_lowercase();
    if lower.starts_with("create") || lower.contains("created") {
        "create"
    } else if lower.starts_with("update")
        || lower.contains("updated")
        || lower.starts_with("modify")
    {
        "update"
    } else if lower.starts_with("delete")
        || lower.starts_with("remove")
        || lower.contains("deleted")
    {
        "delete"
    } else {
        "other"
    }
}

/// Format an action with colored output
fn format_action(action: &str) -> (colored::ColoredString, colored::ColoredString) {
    let lower = action.to_lowercase();
    if lower.starts_with("create") || lower.contains("created") {
        ("+".green(), action.green())
    } else if lower.starts_with("update")
        || lower.contains("updated")
        || lower.starts_with("modify")
    {
        ("~".yellow(), action.yellow())
    } else if lower.starts_with("delete")
        || lower.starts_with("remove")
        || lower.contains("deleted")
    {
        ("-".red(), action.red())
    } else {
        (" ".normal(), action.normal())
    }
}

/// Run the fix command
///
/// Repairs configuration drift by re-synchronizing.
pub fn run_fix(path: &Path, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("{} Previewing fix (dry-run)...", "=>".blue().bold());
    } else {
        println!("{} Fixing configuration drift...", "=>".blue().bold());
    }

    let root = resolve_root(path)?;
    let mode = detect_mode(&root)?;
    let engine = SyncEngine::new(root, mode)?;

    // First check what's wrong
    let check_report = engine.check()?;

    if check_report.status == CheckStatus::Healthy {
        println!(
            "{} Repository is already healthy. Nothing to fix.",
            "OK".green().bold()
        );
        return Ok(());
    }

    // Now fix it (or simulate)
    let options = SyncOptions { dry_run };
    let report = engine.fix_with_options(options)?;

    if report.success {
        if report.actions.is_empty() {
            let msg = if dry_run {
                "No actions needed."
            } else {
                "Configuration fixed."
            };
            println!("{} {}", "OK".green().bold(), msg);
        } else {
            let prefix = if dry_run {
                "Would take actions"
            } else {
                "Configuration fixed"
            };
            println!("{} {}:", "OK".green().bold(), prefix);
            for action in &report.actions {
                println!("   {} {}", "+".green(), action);
            }
        }
    } else {
        println!("{} Fix operation failed:", "ERROR".red().bold());
        for error in &report.errors {
            println!("   {} {}", "!".red(), error);
        }
        return Err(CliError::user("Fix operation failed"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_minimal_repo(dir: &Path, mode: &str) {
        // Create filesystem marker matching the mode
        if mode == "worktrees" {
            fs::create_dir_all(dir.join(".gt")).unwrap();
            fs::create_dir_all(dir.join("main")).unwrap();
        } else {
            fs::create_dir_all(dir.join(".git")).unwrap();
        }

        // Create .repository directory
        let repo_dir = dir.join(".repository");
        fs::create_dir_all(&repo_dir).unwrap();

        // Create config.toml
        let config_content = format!(
            r#"[core]
mode = "{}"
"#,
            mode
        );
        fs::write(repo_dir.join("config.toml"), config_content).unwrap();
    }

    #[test]
    fn test_check_healthy_repo() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Create a minimal repo
        create_minimal_repo(path, "standard");

        // Check should pass (empty ledger = healthy)
        let result = run_check(path);
        if let Err(ref e) = result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok(), "run_check failed: {:?}", result.err());
    }

    #[test]
    fn test_sync_creates_ledger() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Create a minimal repo
        create_minimal_repo(path, "standard");

        // Ledger should not exist yet
        let ledger_path = path.join(".repository").join("ledger.toml");
        assert!(!ledger_path.exists());

        // Run sync
        let result = run_sync(path, false, false);
        assert!(result.is_ok());

        // Ledger should now exist
        assert!(ledger_path.exists());
    }

    #[test]
    fn test_detect_mode_standard() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Create a minimal repo with standard mode
        create_minimal_repo(path, "standard");

        let root = NormalizedPath::new(path);
        let mode = detect_mode(&root).unwrap();
        assert_eq!(mode, Mode::Standard);
    }

    #[test]
    fn test_detect_mode_worktrees() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Create a minimal repo with worktrees mode
        create_minimal_repo(path, "worktrees");

        let root = NormalizedPath::new(path);
        let mode = detect_mode(&root).unwrap();
        assert_eq!(mode, Mode::Worktrees);
    }

    #[test]
    fn test_detect_mode_no_config() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // No config file and no filesystem markers - defaults to Standard
        let root = NormalizedPath::new(path);
        let mode = detect_mode(&root).unwrap();
        assert_eq!(mode, Mode::Standard);
    }

    #[test]
    fn test_fix_healthy_repo() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Create a minimal repo
        create_minimal_repo(path, "standard");

        // Fix should complete successfully (nothing to fix)
        let result = run_fix(path, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sync_dry_run() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Create a minimal repo
        create_minimal_repo(path, "standard");

        // Run sync in dry-run mode
        let result = run_sync(path, true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fix_dry_run() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Create a minimal repo
        create_minimal_repo(path, "standard");

        // Fix in dry-run mode should complete successfully
        let result = run_fix(path, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_root_standard_repo() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        create_minimal_repo(path, "standard");

        // From root should return root
        let root = resolve_root(path).unwrap();
        assert_eq!(root.as_ref(), path);
    }

    #[test]
    fn test_resolve_root_worktrees_container() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Create worktrees mode config
        let repo_dir = path.join(".repository");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(
            repo_dir.join("config.toml"),
            "[core]\nmode = \"worktrees\"\n",
        )
        .unwrap();

        // From container root should return container root
        let root = resolve_root(path).unwrap();
        assert_eq!(root.as_ref(), path);
    }

    #[test]
    fn test_resolve_root_from_worktree() {
        let temp_dir = TempDir::new().unwrap();
        let container = temp_dir.path();

        // Create worktrees mode config at container root
        let repo_dir = container.join(".repository");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(
            repo_dir.join("config.toml"),
            "[core]\nmode = \"worktrees\"\n",
        )
        .unwrap();

        // Create a worktree directory
        let worktree = container.join("feature-branch");
        let nested = worktree.join("src");
        fs::create_dir_all(&nested).unwrap();

        // From nested inside worktree should return container root
        let root = resolve_root(&nested).unwrap();
        assert_eq!(root.as_ref(), container);
    }

    #[test]
    fn test_resolve_root_not_a_repo() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // No .repository directory - should error
        let result = resolve_root(path);
        assert!(result.is_err());
    }
}
