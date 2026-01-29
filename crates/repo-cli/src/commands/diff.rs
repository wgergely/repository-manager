//! Diff command implementation
//!
//! Previews what changes sync would make without applying them.

use std::path::Path;

use colored::Colorize;
use serde_json::json;

use repo_core::{Mode, SyncEngine, SyncOptions};
use repo_fs::NormalizedPath;

use super::sync::{detect_mode, resolve_root};
use crate::error::Result;

/// Run the diff command
///
/// Shows what changes sync would make without applying them.
/// This is essentially a sync with dry_run=true, but with diff-style output.
pub fn run_diff(path: &Path, json: bool) -> Result<()> {
    let root = resolve_root(path)?;
    let mode = detect_mode(&root)?;
    let engine = SyncEngine::new(root.clone(), mode)?;

    // Run sync in dry-run mode to see what would change
    let options = SyncOptions { dry_run: true };
    let report = engine.sync_with_options(options)?;

    if json {
        // JSON output for CI/CD integration
        let json_output = json!({
            "has_changes": !report.actions.is_empty(),
            "changes": report.actions.iter()
                .map(|a| {
                    // Strip "[dry-run] Would " prefix if present
                    let clean = a.strip_prefix("[dry-run] Would ").unwrap_or(a);
                    json!({
                        "action": clean,
                        "raw": a
                    })
                })
                .collect::<Vec<_>>(),
            "errors": report.errors,
            "success": report.success,
        });
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else {
        // Human-readable diff-style output
        print_diff_output(&report.actions, &report.errors, &root, mode);
    }

    Ok(())
}

/// Print human-readable diff-style output
fn print_diff_output(actions: &[String], errors: &[String], root: &NormalizedPath, mode: Mode) {
    if actions.is_empty() && errors.is_empty() {
        println!(
            "{} No changes needed. Repository is in sync.",
            "OK".green().bold()
        );
        return;
    }

    println!(
        "{} {} ({})",
        "Diff".blue().bold(),
        root.as_str().yellow(),
        mode.to_string().cyan()
    );
    println!();

    if !actions.is_empty() {
        println!("{}", "Changes that would be made:".bold());
        println!();

        for action in actions {
            // Format the action as a diff-style line
            let clean = action.strip_prefix("[dry-run] Would ").unwrap_or(action);

            // Determine the type of change and color accordingly
            let (prefix, colored_action) = if clean.starts_with("create") || clean.starts_with("Created") {
                ("+".green(), clean.green())
            } else if clean.starts_with("update") || clean.starts_with("Updated") || clean.starts_with("modify") {
                ("~".yellow(), clean.yellow())
            } else if clean.starts_with("delete") || clean.starts_with("remove") || clean.starts_with("Deleted") {
                ("-".red(), clean.red())
            } else {
                (" ".normal(), clean.normal())
            };

            println!("  {} {}", prefix, colored_action);
        }
    }

    if !errors.is_empty() {
        println!();
        println!("{}", "Errors:".red().bold());
        for error in errors {
            println!("  {} {}", "!".red(), error);
        }
    }

    println!();
    println!(
        "Run {} to apply these changes.",
        "repo sync".cyan()
    );
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

        let repo_dir = dir.join(".repository");
        fs::create_dir_all(&repo_dir).unwrap();
        let config_content = format!(
            r#"[core]
mode = "{}"
"#,
            mode
        );
        fs::write(repo_dir.join("config.toml"), config_content).unwrap();
    }

    #[test]
    fn test_diff_basic() {
        let temp_dir = TempDir::new().unwrap();
        create_minimal_repo(temp_dir.path(), "standard");

        let result = run_diff(temp_dir.path(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_diff_json() {
        let temp_dir = TempDir::new().unwrap();
        create_minimal_repo(temp_dir.path(), "standard");

        let result = run_diff(temp_dir.path(), true);
        assert!(result.is_ok());
    }
}
