//! Governance command implementations
//!
//! Provides lint, diff, export, and import operations for config governance.

use std::fs;
use std::path::Path;

use colored::Colorize;

use crate::error::{CliError, Result};

/// Run the rules-lint command
///
/// Checks the configuration for consistency issues.
pub fn run_rules_lint(path: &Path, json: bool) -> Result<()> {
    let config_path = path.join(".repository").join("config.toml");
    if !config_path.exists() {
        return Err(CliError::user(
            "No .repository/config.toml found. Run 'repo init' first.",
        ));
    }

    let content = fs::read_to_string(&config_path)?;
    let manifest = repo_core::Manifest::parse(&content)
        .map_err(|e| CliError::user(format!("Failed to parse config: {}", e)))?;

    // Get available tools from the tools registry
    let registry = repo_tools::ToolRegistry::with_builtins();
    let available_tools: Vec<String> = registry.list().iter().map(|s| s.to_string()).collect();

    let warnings = repo_core::governance::lint_rules(&manifest, &available_tools);

    if json {
        let output = serde_json::to_string_pretty(&warnings)?;
        println!("{}", output);
        return Ok(());
    }

    if warnings.is_empty() {
        println!("{} Configuration is clean.", "OK".green().bold());
        return Ok(());
    }

    println!("{} Found {} issue(s):", "=>".blue().bold(), warnings.len());
    for w in &warnings {
        let prefix = match w.level {
            repo_core::WarnLevel::Info => "info".cyan(),
            repo_core::WarnLevel::Warning => "warn".yellow(),
            repo_core::WarnLevel::Error => "error".red(),
        };
        if let Some(ref tool) = w.tool {
            println!("  [{}] {}: {}", prefix, tool.bold(), w.message);
        } else {
            println!("  [{}] {}", prefix, w.message);
        }
    }

    Ok(())
}

/// Run the rules-diff command
///
/// Shows drift between expected and actual config state.
pub fn run_rules_diff(path: &Path, json: bool) -> Result<()> {
    let config_path = path.join(".repository").join("config.toml");
    if !config_path.exists() {
        return Err(CliError::user(
            "No .repository/config.toml found. Run 'repo init' first.",
        ));
    }

    let content = fs::read_to_string(&config_path)?;
    let manifest = repo_core::Manifest::parse(&content)
        .map_err(|e| CliError::user(format!("Failed to parse config: {}", e)))?;

    let drifts = repo_core::governance::diff_configs(path, &manifest)
        .map_err(|e| CliError::user(format!("Failed to compute diff: {}", e)))?;

    if json {
        let output = serde_json::to_string_pretty(&drifts)?;
        println!("{}", output);
        return Ok(());
    }

    if drifts.is_empty() {
        println!("{} No configuration drift detected.", "OK".green().bold());
        return Ok(());
    }

    println!("{} Found {} drift(s):", "=>".blue().bold(), drifts.len());
    for d in &drifts {
        let prefix = match d.drift_type {
            repo_core::DriftType::Modified => "modified".yellow(),
            repo_core::DriftType::Missing => "missing".red(),
            repo_core::DriftType::Extra => "extra".cyan(),
        };
        println!(
            "  [{}] {} - {} ({})",
            prefix,
            d.tool.bold(),
            d.config_path.display(),
            d.details
        );
    }

    Ok(())
}

/// Run the rules-export command
///
/// Exports rules to AGENTS.md format.
pub fn run_rules_export(path: &Path, format: &str) -> Result<()> {
    if format != "agents" {
        return Err(CliError::user(format!(
            "Unsupported export format '{}'. Supported: agents",
            format
        )));
    }

    let output = repo_core::governance::export_agents_md(path)
        .map_err(|e| CliError::user(format!("Failed to export: {}", e)))?;

    print!("{}", output);
    Ok(())
}

/// Run the rules-import command
///
/// Imports rules from an AGENTS.md file.
pub fn run_rules_import(path: &Path, file: &str) -> Result<()> {
    let file_path = Path::new(file);
    if !file_path.exists() {
        return Err(CliError::user(format!("File not found: {}", file)));
    }

    let content = fs::read_to_string(file_path)?;
    let rules = repo_core::governance::import_agents_md(&content);

    if rules.is_empty() {
        println!("{} No rules found in file.", "WARN".yellow().bold());
        return Ok(());
    }

    let rules_dir = path.join(".repository").join("rules");
    fs::create_dir_all(&rules_dir)?;

    println!(
        "{} Importing {} rule(s)...",
        "=>".blue().bold(),
        rules.len()
    );

    for (id, rule_content) in &rules {
        let rule_path = rules_dir.join(format!("{}.md", id));
        fs::write(&rule_path, rule_content)?;
        println!("   {} {}", "+".green(), id);
    }

    println!("{} Import complete.", "OK".green().bold());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo(dir: &Path) {
        let repo_dir = dir.join(".repository");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(
            repo_dir.join("config.toml"),
            "tools = [\"claude\"]\n\n[core]\nmode = \"standard\"\n",
        )
        .unwrap();
    }

    #[test]
    fn test_rules_lint_no_repo() {
        let temp = TempDir::new().unwrap();
        let result = run_rules_lint(temp.path(), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_rules_lint_basic() {
        let temp = TempDir::new().unwrap();
        create_test_repo(temp.path());
        let result = run_rules_lint(temp.path(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rules_lint_json() {
        let temp = TempDir::new().unwrap();
        create_test_repo(temp.path());
        let result = run_rules_lint(temp.path(), true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rules_diff_no_repo() {
        let temp = TempDir::new().unwrap();
        let result = run_rules_diff(temp.path(), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_rules_diff_basic() {
        let temp = TempDir::new().unwrap();
        create_test_repo(temp.path());
        let result = run_rules_diff(temp.path(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rules_export_empty() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(".repository")).unwrap();
        let result = run_rules_export(temp.path(), "agents");
        assert!(result.is_ok());
    }

    #[test]
    fn test_rules_export_unsupported_format() {
        let temp = TempDir::new().unwrap();
        let result = run_rules_export(temp.path(), "xml");
        assert!(result.is_err());
    }

    #[test]
    fn test_rules_import_missing_file() {
        let temp = TempDir::new().unwrap();
        let result = run_rules_import(temp.path(), "/nonexistent/AGENTS.md");
        assert!(result.is_err());
    }

    #[test]
    fn test_rules_import_roundtrip() {
        let temp = TempDir::new().unwrap();
        let rules_dir = temp.path().join(".repository/rules");
        fs::create_dir_all(&rules_dir).unwrap();

        // Create some rules
        fs::write(rules_dir.join("alpha.md"), "Alpha rule.").unwrap();
        fs::write(rules_dir.join("beta.md"), "Beta rule.").unwrap();

        // Export
        let exported = repo_core::governance::export_agents_md(temp.path()).unwrap();

        // Write to file
        let agents_file = temp.path().join("AGENTS.md");
        fs::write(&agents_file, &exported).unwrap();

        // Import into new location
        let temp2 = TempDir::new().unwrap();
        fs::create_dir_all(temp2.path().join(".repository")).unwrap();
        let result = run_rules_import(temp2.path(), agents_file.to_str().unwrap());
        assert!(result.is_ok());

        // Verify imported rules exist
        let imported_rules_dir = temp2.path().join(".repository/rules");
        assert!(imported_rules_dir.join("alpha.md").exists());
        assert!(imported_rules_dir.join("beta.md").exists());
    }
}
