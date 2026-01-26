//! Sync, check, and fix command implementations
//!
//! These commands manage synchronization state between the ledger and filesystem.

use std::path::Path;

use colored::Colorize;

use repo_core::{CheckStatus, ConfigResolver, Mode, SyncEngine};
use repo_fs::NormalizedPath;

use crate::error::{CliError, Result};

/// Detect the repository mode from config.toml
///
/// Reads the mode from `.repository/config.toml` using ConfigResolver.
/// Defaults to Standard mode if no config exists.
pub fn detect_mode(root: &NormalizedPath) -> Result<Mode> {
    let resolver = ConfigResolver::new(root.clone());

    if !resolver.has_config() {
        // No config file, default to standard mode
        return Ok(Mode::Standard);
    }

    let config = resolver.resolve()?;
    let mode: Mode = config.mode.parse().map_err(|e: repo_core::Error| {
        CliError::user(format!("Invalid mode in config: {}", e))
    })?;

    Ok(mode)
}

/// Run the check command
///
/// Validates that the filesystem matches the ledger state.
pub fn run_check(path: &Path) -> Result<()> {
    println!(
        "{} Checking repository configuration...",
        "=>".blue().bold()
    );

    let root = NormalizedPath::new(path);
    let mode = detect_mode(&root)?;
    let engine = SyncEngine::new(root, mode)?;

    let report = engine.check()?;

    match report.status {
        CheckStatus::Healthy => {
            println!("{} Repository is healthy. No drift detected.", "OK".green().bold());
        }
        CheckStatus::Missing => {
            println!(
                "{} Some files are missing:",
                "MISSING".yellow().bold()
            );
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
            println!(
                "{} Configuration has drifted:",
                "DRIFTED".red().bold()
            );
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
            println!(
                "{} Repository is in a broken state:",
                "BROKEN".red().bold()
            );
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
pub fn run_sync(path: &Path) -> Result<()> {
    println!(
        "{} Synchronizing tool configurations...",
        "=>".blue().bold()
    );

    let root = NormalizedPath::new(path);
    let mode = detect_mode(&root)?;
    let engine = SyncEngine::new(root, mode)?;

    let report = engine.sync()?;

    if report.success {
        if report.actions.is_empty() {
            println!("{} Already synchronized. No changes needed.", "OK".green().bold());
        } else {
            println!("{} Synchronization complete:", "OK".green().bold());
            for action in &report.actions {
                println!("   {} {}", "+".green(), action);
            }
        }
    } else {
        println!("{} Synchronization failed:", "ERROR".red().bold());
        for error in &report.errors {
            println!("   {} {}", "!".red(), error);
        }
        return Err(CliError::user("Synchronization failed"));
    }

    Ok(())
}

/// Run the fix command
///
/// Repairs configuration drift by re-synchronizing.
pub fn run_fix(path: &Path) -> Result<()> {
    println!(
        "{} Fixing configuration drift...",
        "=>".blue().bold()
    );

    let root = NormalizedPath::new(path);
    let mode = detect_mode(&root)?;
    let engine = SyncEngine::new(root, mode)?;

    // First check what's wrong
    let check_report = engine.check()?;

    if check_report.status == CheckStatus::Healthy {
        println!("{} Repository is already healthy. Nothing to fix.", "OK".green().bold());
        return Ok(());
    }

    // Now fix it
    let report = engine.fix()?;

    if report.success {
        if report.actions.is_empty() {
            println!("{} Configuration fixed.", "OK".green().bold());
        } else {
            println!("{} Configuration fixed:", "OK".green().bold());
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
        // Create .git directory to simulate git repo
        let git_dir = dir.join(".git");
        fs::create_dir_all(&git_dir).unwrap();

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
        let result = run_sync(path);
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

        // No config file - should default to standard
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
        let result = run_fix(path);
        assert!(result.is_ok());
    }
}
