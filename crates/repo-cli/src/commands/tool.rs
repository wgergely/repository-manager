//! Tool and preset management command implementations
//!
//! Provides add/remove operations for tools and presets in config.toml.

use std::path::Path;

use colored::Colorize;
use serde_json;

use repo_core::{Manifest, SyncEngine};
use repo_fs::NormalizedPath;
use repo_meta::{Registry, ToolRegistry};

use crate::commands::sync::detect_mode;
use crate::error::{CliError, Result};

/// Path to config.toml within a repository
const CONFIG_PATH: &str = ".repository/config.toml";

/// Run the add-tool command
///
/// Adds a tool to the repository's config.toml.
/// When `dry_run` is true, shows what would happen without modifying files.
pub fn run_add_tool(path: &Path, name: &str, dry_run: bool) -> Result<()> {
    let prefix = if dry_run { "[dry run] " } else { "" };
    println!(
        "{}{} Adding tool: {}",
        prefix,
        "=>".blue().bold(),
        name.cyan()
    );

    // Validate tool name
    let tool_registry = ToolRegistry::with_builtins();
    if !tool_registry.is_known(name) {
        eprintln!(
            "{} Unknown tool '{}'. Known tools: {}",
            "warning:".yellow().bold(),
            name,
            tool_registry.list_known().join(", ")
        );
    }

    let config_path = NormalizedPath::new(path.join(CONFIG_PATH));

    // Load existing manifest or create empty one
    let manifest = load_manifest(&config_path)?;

    // Check if tool already exists
    if manifest.tools.contains(&name.to_string()) {
        println!(
            "{}{} Tool {} is already configured.",
            prefix,
            "OK".green().bold(),
            name.cyan()
        );
        return Ok(());
    }

    if dry_run {
        println!("{}Would add tool '{}' to config.toml", prefix, name);
        println!(
            "{}Would trigger sync to generate tool configurations",
            prefix
        );
        return Ok(());
    }

    // Add the tool
    let mut manifest = manifest;
    manifest.tools.push(name.to_string());

    // Save the manifest
    save_manifest(&config_path, &manifest)?;

    println!("{} Tool {} added.", "OK".green().bold(), name.cyan());

    // Trigger sync to apply tool configuration
    trigger_sync_and_report(path)?;

    Ok(())
}

/// Run the remove-tool command
///
/// Removes a tool from the repository's config.toml.
/// When `dry_run` is true, shows what would happen without modifying files.
pub fn run_remove_tool(path: &Path, name: &str, dry_run: bool) -> Result<()> {
    let prefix = if dry_run { "[dry run] " } else { "" };
    println!(
        "{}{} Removing tool: {}",
        prefix,
        "=>".blue().bold(),
        name.cyan()
    );

    let config_path = NormalizedPath::new(path.join(CONFIG_PATH));

    // Load existing manifest
    let manifest = load_manifest(&config_path)?;

    // Check if tool exists
    if let Some(pos) = manifest.tools.iter().position(|t| t == name) {
        if dry_run {
            println!("{}Would remove tool '{}' from config.toml", prefix, name);
            println!("{}Would trigger sync to update tool configurations", prefix);
            return Ok(());
        }

        let mut manifest = manifest;
        manifest.tools.remove(pos);
        save_manifest(&config_path, &manifest)?;
        println!("{} Tool {} removed.", "OK".green().bold(), name.cyan());

        // Trigger sync to apply configuration changes
        trigger_sync_and_report(path)?;
    } else {
        println!(
            "{}{} Tool {} not found in configuration.",
            prefix,
            "WARN".yellow().bold(),
            name.cyan()
        );
    }

    Ok(())
}

/// Run the add-preset command
///
/// Adds a preset to the repository's config.toml.
/// When `dry_run` is true, shows what would happen without modifying files.
pub fn run_add_preset(path: &Path, name: &str, dry_run: bool) -> Result<()> {
    let prefix = if dry_run { "[dry run] " } else { "" };
    println!(
        "{}{} Adding preset: {}",
        prefix,
        "=>".blue().bold(),
        name.cyan()
    );

    // Validate preset name
    let registry = Registry::with_builtins();
    if !registry.has_provider(name) {
        eprintln!(
            "{} Unknown preset '{}'. Known presets: {}",
            "warning:".yellow().bold(),
            name,
            registry.list_presets().join(", ")
        );
    }

    let config_path = NormalizedPath::new(path.join(CONFIG_PATH));

    // Load existing manifest or create empty one
    let manifest = load_manifest(&config_path)?;

    // Check if preset already exists
    if manifest.presets.contains_key(name) {
        println!(
            "{}{} Preset {} is already configured.",
            prefix,
            "OK".green().bold(),
            name.cyan()
        );
        return Ok(());
    }

    if dry_run {
        println!("{}Would add preset '{}' to config.toml", prefix, name);
        return Ok(());
    }

    // Add the preset with an empty object
    let mut manifest = manifest;
    manifest
        .presets
        .insert(name.to_string(), serde_json::json!({}));

    // Save the manifest
    save_manifest(&config_path, &manifest)?;

    println!("{} Preset {} added.", "OK".green().bold(), name.cyan());
    Ok(())
}

/// Run the remove-preset command
///
/// Removes a preset from the repository's config.toml.
/// When `dry_run` is true, shows what would happen without modifying files.
pub fn run_remove_preset(path: &Path, name: &str, dry_run: bool) -> Result<()> {
    let prefix = if dry_run { "[dry run] " } else { "" };
    println!(
        "{}{} Removing preset: {}",
        prefix,
        "=>".blue().bold(),
        name.cyan()
    );

    let config_path = NormalizedPath::new(path.join(CONFIG_PATH));

    // Load existing manifest
    let manifest = load_manifest(&config_path)?;

    // Check if preset exists
    if manifest.presets.contains_key(name) {
        if dry_run {
            println!("{}Would remove preset '{}' from config.toml", prefix, name);
            return Ok(());
        }

        let mut manifest = manifest;
        manifest.presets.remove(name);
        save_manifest(&config_path, &manifest)?;
        println!("{} Preset {} removed.", "OK".green().bold(), name.cyan());
    } else {
        println!(
            "{}{} Preset {} not found in configuration.",
            prefix,
            "WARN".yellow().bold(),
            name.cyan()
        );
    }

    Ok(())
}

/// Load a manifest from the config file
///
/// If the file doesn't exist, returns an error.
pub fn load_manifest(path: &NormalizedPath) -> Result<Manifest> {
    let native_path = path.to_native();

    if !native_path.exists() {
        return Err(CliError::user(format!(
            "Config file not found: {}. Run 'repo init' first.",
            path
        )));
    }

    let content = std::fs::read_to_string(&native_path)?;
    let manifest = Manifest::parse(&content)?;
    Ok(manifest)
}

/// Save a manifest to the config file
///
/// Writes the manifest back to config.toml in a clean format.
pub fn save_manifest(path: &NormalizedPath, manifest: &Manifest) -> Result<()> {
    let native_path = path.to_native();

    // Ensure parent directory exists
    if let Some(parent) = native_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Generate TOML content
    let content = generate_manifest_toml(manifest);

    std::fs::write(&native_path, content)?;
    Ok(())
}

/// Trigger sync after tool/preset changes and print the results
///
/// This function runs the sync engine to apply any configuration changes
/// resulting from adding or removing tools/presets.
fn trigger_sync_and_report(path: &Path) -> Result<()> {
    let root = NormalizedPath::new(path);
    let mode = detect_mode(&root)?;
    let engine = SyncEngine::new(root, mode)?;

    match engine.sync() {
        Ok(report) => {
            if !report.actions.is_empty() {
                for action in &report.actions {
                    println!("   {} {}", "+".green(), action);
                }
            }
            if !report.success {
                for error in &report.errors {
                    eprintln!("   {} {}", "!".red(), error);
                }
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("{} Sync failed: {}", "warning:".yellow().bold(), e);
            // Don't fail the overall operation - the config change succeeded
            Ok(())
        }
    }
}

/// Generate TOML content from a manifest
///
/// Delegates to `Manifest::to_toml()` for the shared serialization logic.
fn generate_manifest_toml(manifest: &Manifest) -> String {
    manifest.to_toml()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Helper to create a test config.toml and .git directory
    fn create_test_config(dir: &Path, content: &str) {
        // Create .git directory to simulate git repo (required for sync)
        let git_dir = dir.join(".git");
        std::fs::create_dir_all(&git_dir).unwrap();

        let repo_dir = dir.join(".repository");
        std::fs::create_dir_all(&repo_dir).unwrap();
        std::fs::write(repo_dir.join("config.toml"), content).unwrap();
    }

    /// Helper to read the config.toml content
    fn read_config(dir: &Path) -> String {
        std::fs::read_to_string(dir.join(".repository/config.toml")).unwrap()
    }

    #[test]
    fn test_add_tool() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Create initial config
        create_test_config(
            path,
            r#"[core]
mode = "standard"
"#,
        );

        // Add a tool
        let result = run_add_tool(path, "eslint", false);
        assert!(result.is_ok());

        // Verify tool was added
        let content = read_config(path);
        assert!(content.contains("tools = [\"eslint\"]"));
    }

    #[test]
    fn test_add_tool_to_existing_tools() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Create config with existing tool (tools must be BEFORE [core])
        create_test_config(
            path,
            r#"tools = ["prettier"]

[core]
mode = "standard"
"#,
        );

        // Add another tool
        let result = run_add_tool(path, "eslint", false);
        assert!(result.is_ok());

        // Verify both tools exist
        let content = read_config(path);
        assert!(content.contains("prettier"));
        assert!(content.contains("eslint"));
    }

    #[test]
    fn test_add_duplicate_tool() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Create config with existing tool (tools must be BEFORE [core])
        create_test_config(
            path,
            r#"tools = ["eslint"]

[core]
mode = "standard"
"#,
        );

        // Add duplicate tool - should succeed without duplicating
        let result = run_add_tool(path, "eslint", false);
        assert!(result.is_ok());

        // Parse and verify only one instance
        let content = read_config(path);
        let manifest = Manifest::parse(&content).unwrap();
        assert_eq!(manifest.tools.len(), 1);
    }

    #[test]
    fn test_remove_tool() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Create config with tools (tools must be BEFORE [core])
        create_test_config(
            path,
            r#"tools = ["eslint", "prettier"]

[core]
mode = "standard"
"#,
        );

        // Remove a tool
        let result = run_remove_tool(path, "eslint", false);
        assert!(result.is_ok());

        // Verify tool was removed
        let content = read_config(path);
        assert!(!content.contains("\"eslint\""));
        assert!(content.contains("\"prettier\""));
    }

    #[test]
    fn test_remove_nonexistent_tool() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Create config without the tool (tools must be BEFORE [core])
        create_test_config(
            path,
            r#"tools = ["prettier"]

[core]
mode = "standard"
"#,
        );

        // Remove non-existent tool - should succeed with warning
        let result = run_remove_tool(path, "eslint", false);
        assert!(result.is_ok());

        // Config should be unchanged
        let content = read_config(path);
        assert!(content.contains("\"prettier\""));
    }

    #[test]
    fn test_add_preset() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Create initial config
        create_test_config(
            path,
            r#"[core]
mode = "standard"
"#,
        );

        // Add a preset
        let result = run_add_preset(path, "typescript", false);
        assert!(result.is_ok());

        // Verify preset was added (toml::to_string_pretty uses sub-table headers)
        let content = read_config(path);
        assert!(content.contains("typescript"));
    }

    #[test]
    fn test_add_preset_to_existing_presets() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Create config with existing preset
        create_test_config(
            path,
            r#"[core]
mode = "standard"

[presets]
"react" = {}
"#,
        );

        // Add another preset
        let result = run_add_preset(path, "typescript", false);
        assert!(result.is_ok());

        // Verify both presets exist (toml::to_string_pretty uses sub-table headers)
        let content = read_config(path);
        assert!(content.contains("react"));
        assert!(content.contains("typescript"));
    }

    #[test]
    fn test_remove_preset() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Create config with presets
        create_test_config(
            path,
            r#"[core]
mode = "standard"

[presets]
"typescript" = {}
"react" = {}
"#,
        );

        // Remove a preset
        let result = run_remove_preset(path, "typescript", false);
        assert!(result.is_ok());

        // Verify preset was removed
        let content = read_config(path);
        assert!(!content.contains("typescript"));
        assert!(content.contains("react"));
    }

    #[test]
    fn test_remove_nonexistent_preset() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Create config without the preset
        create_test_config(
            path,
            r#"[core]
mode = "standard"

[presets]
"react" = {}
"#,
        );

        // Remove non-existent preset - should succeed with warning
        let result = run_remove_preset(path, "typescript", false);
        assert!(result.is_ok());

        // Config should still have react
        let content = read_config(path);
        assert!(content.contains("react"));
    }

    #[test]
    fn test_add_tool_no_config() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // No config.toml exists
        let result = run_add_tool(path, "eslint", false);
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_msg = format!("{}", err);
        assert!(err_msg.contains("Config file not found"));
    }

    #[test]
    fn test_save_manifest_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        let config_path = NormalizedPath::new(path.join(".repository/config.toml"));
        let manifest = Manifest::empty();

        let result = save_manifest(&config_path, &manifest);
        assert!(result.is_ok());

        // Verify file was created
        assert!(path.join(".repository/config.toml").exists());
    }

    #[test]
    fn test_generate_manifest_toml() {
        let mut manifest = Manifest::empty();
        manifest.core.mode = "worktrees".to_string();
        manifest.tools = vec!["eslint".to_string(), "prettier".to_string()];
        manifest
            .presets
            .insert("typescript".to_string(), serde_json::json!({}));

        let content = generate_manifest_toml(&manifest);

        assert!(content.contains("[core]"));
        assert!(content.contains("mode = \"worktrees\""));
        // toml::to_string_pretty formats arrays multi-line
        assert!(content.contains("\"eslint\""));
        assert!(content.contains("\"prettier\""));
        // toml::to_string_pretty formats empty map values as sub-table headers
        assert!(content.contains("typescript"));
    }

    #[test]
    fn test_add_tool_dry_run_does_not_modify_config() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        let initial_config = "[core]\nmode = \"standard\"\n";
        create_test_config(path, initial_config);

        let result = run_add_tool(path, "eslint", true);
        assert!(result.is_ok());

        // Config should be unchanged
        let content = read_config(path);
        assert_eq!(content, initial_config);
        assert!(!content.contains("eslint"));
    }

    #[test]
    fn test_remove_tool_dry_run_does_not_modify_config() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        let initial_config = "tools = [\"eslint\", \"prettier\"]\n\n[core]\nmode = \"standard\"\n";
        create_test_config(path, initial_config);

        let result = run_remove_tool(path, "eslint", true);
        assert!(result.is_ok());

        // Config should still contain eslint
        let content = read_config(path);
        assert!(content.contains("eslint"));
        assert!(content.contains("prettier"));
    }

    #[test]
    fn test_add_preset_dry_run_does_not_modify_config() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        let initial_config = "[core]\nmode = \"standard\"\n";
        create_test_config(path, initial_config);

        let result = run_add_preset(path, "typescript", true);
        assert!(result.is_ok());

        // Config should be unchanged
        let content = read_config(path);
        assert_eq!(content, initial_config);
        assert!(!content.contains("typescript"));
    }

    #[test]
    fn test_remove_preset_dry_run_does_not_modify_config() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        let initial_config =
            "[core]\nmode = \"standard\"\n\n[presets]\n\"typescript\" = {}\n\"react\" = {}\n";
        create_test_config(path, initial_config);

        let result = run_remove_preset(path, "typescript", true);
        assert!(result.is_ok());

        // Config should still contain typescript
        let content = read_config(path);
        assert!(content.contains("typescript"));
        assert!(content.contains("react"));
    }
}
