//! Status command implementation
//!
//! Shows an overview of the repository status including mode, root, tools, rules, and sync status.

use std::path::Path;

use colored::Colorize;
use serde_json::json;

use repo_core::{CheckStatus, ConfigResolver, Mode, RuleRegistry, SyncEngine};
use repo_fs::NormalizedPath;

use super::sync::{detect_mode, resolve_root};
use crate::error::Result;

/// Status information for JSON output
#[derive(Debug)]
pub struct StatusInfo {
    /// Repository mode (standard or worktrees)
    pub mode: String,
    /// Repository root path
    pub root: String,
    /// Active tools
    pub tools: Vec<String>,
    /// Number of active rules
    pub rules_count: usize,
    /// Sync status (healthy, missing, drifted, broken)
    pub sync_status: String,
    /// Whether the repository has local overrides
    pub has_local_overrides: bool,
}

/// Run the status command
///
/// Shows repository status overview including mode, root, tools, rules count, and sync status.
pub fn run_status(path: &Path, json: bool) -> Result<()> {
    let root = resolve_root(path)?;
    let mode = detect_mode(&root)?;
    let engine = SyncEngine::new(root.clone(), mode)?;

    // Load configuration
    let resolver = ConfigResolver::new(root.clone());
    let config = resolver.resolve()?;

    // Get sync status
    let check_report = engine.check()?;
    let sync_status = match check_report.status {
        CheckStatus::Healthy => "healthy",
        CheckStatus::Missing => "missing",
        CheckStatus::Drifted => "drifted",
        CheckStatus::Broken => "broken",
    };

    // Count rules
    let rules_dir = root.join(".repository/rules");
    let rules_count = count_rules(&rules_dir);

    let status_info = StatusInfo {
        mode: mode.to_string(),
        root: root.as_str().to_string(),
        tools: config.tools.clone(),
        rules_count,
        sync_status: sync_status.to_string(),
        has_local_overrides: resolver.has_local_overrides(),
    };

    if json {
        // JSON output for scripting
        let json_output = json!({
            "mode": status_info.mode,
            "root": status_info.root,
            "tools": status_info.tools,
            "rules_count": status_info.rules_count,
            "sync_status": status_info.sync_status,
            "has_local_overrides": status_info.has_local_overrides,
        });
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else {
        // Human-readable colored output
        print_human_status(&status_info, &mode);
    }

    Ok(())
}

/// Count the number of rule files in the rules directory
fn count_rules(rules_dir: &NormalizedPath) -> usize {
    // Try to load the registry
    let registry_path = rules_dir.join("registry.toml");
    if let Ok(registry) = RuleRegistry::load(registry_path.as_ref().to_path_buf()) {
        return registry.all_rules().len();
    }

    // Fall back to counting .md files in the rules directory
    if rules_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(rules_dir.as_ref()) {
            return entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .is_some_and(|ext| ext == "md")
                })
                .count();
        }
    }

    0
}

/// Print human-readable status output
fn print_human_status(status: &StatusInfo, mode: &Mode) {
    println!("{}", "Repository Status".bold().underline());
    println!();

    // Mode
    let mode_display = match mode {
        Mode::Standard => "standard".cyan(),
        Mode::Worktrees => "worktrees".magenta(),
    };
    println!("  {}: {}", "Mode".bold(), mode_display);

    // Root
    println!("  {}: {}", "Root".bold(), status.root.yellow());

    // Tools
    if status.tools.is_empty() {
        println!("  {}: {}", "Tools".bold(), "none".dimmed());
    } else {
        println!(
            "  {}: {}",
            "Tools".bold(),
            status.tools.join(", ").green()
        );
    }

    // Rules
    if status.rules_count == 0 {
        println!("  {}: {}", "Rules".bold(), "none".dimmed());
    } else {
        println!(
            "  {}: {} active",
            "Rules".bold(),
            status.rules_count.to_string().green()
        );
    }

    // Sync status
    let sync_display = match status.sync_status.as_str() {
        "healthy" => "healthy".green(),
        "missing" => "missing files".yellow(),
        "drifted" => "drifted".red(),
        "broken" => "broken".red().bold(),
        _ => status.sync_status.as_str().normal(),
    };
    println!("  {}: {}", "Sync".bold(), sync_display);

    // Local overrides
    if status.has_local_overrides {
        println!(
            "  {}: {}",
            "Overrides".bold(),
            "local overrides active".cyan()
        );
    }

    println!();
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
        // Note: tools must be at top level, not inside [core] section
        let config_content = format!(
            r#"tools = ["cursor"]

[core]
mode = "{}"
"#,
            mode
        );
        fs::write(repo_dir.join("config.toml"), config_content).unwrap();
    }

    #[test]
    fn test_status_basic() {
        let temp_dir = TempDir::new().unwrap();
        create_minimal_repo(temp_dir.path(), "standard");

        let result = run_status(temp_dir.path(), false);
        if let Err(e) = &result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok(), "run_status failed: {:?}", result.err());
    }

    #[test]
    fn test_status_json() {
        let temp_dir = TempDir::new().unwrap();
        create_minimal_repo(temp_dir.path(), "standard");

        let result = run_status(temp_dir.path(), true);
        if let Err(e) = &result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok(), "run_status json failed: {:?}", result.err());
    }

    #[test]
    fn test_count_rules_empty() {
        let temp_dir = TempDir::new().unwrap();
        let rules_dir = NormalizedPath::new(temp_dir.path().join("rules"));

        let count = count_rules(&rules_dir);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_count_rules_with_files() {
        let temp_dir = TempDir::new().unwrap();
        let rules_dir = temp_dir.path().join("rules");
        fs::create_dir_all(&rules_dir).unwrap();
        fs::write(rules_dir.join("rule1.md"), "# Rule 1").unwrap();
        fs::write(rules_dir.join("rule2.md"), "# Rule 2").unwrap();

        let normalized = NormalizedPath::new(&rules_dir);
        let count = count_rules(&normalized);
        assert_eq!(count, 2);
    }
}
