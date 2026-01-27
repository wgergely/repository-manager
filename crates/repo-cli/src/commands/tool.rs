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
/// Adds a tool to the repository's config.toml
pub fn run_add_tool(path: &Path, name: &str) -> Result<()> {
    println!(
        "{} Adding tool: {}",
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
    let mut manifest = load_manifest(&config_path)?;

    // Check if tool already exists
    if manifest.tools.contains(&name.to_string()) {
        println!(
            "{} Tool {} is already configured.",
            "OK".green().bold(),
            name.cyan()
        );
        return Ok(());
    }

    // Add the tool
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
/// Removes a tool from the repository's config.toml
pub fn run_remove_tool(path: &Path, name: &str) -> Result<()> {
    println!(
        "{} Removing tool: {}",
        "=>".blue().bold(),
        name.cyan()
    );

    let config_path = NormalizedPath::new(path.join(CONFIG_PATH));

    // Load existing manifest
    let mut manifest = load_manifest(&config_path)?;

    // Check if tool exists
    if let Some(pos) = manifest.tools.iter().position(|t| t == name) {
        manifest.tools.remove(pos);
        save_manifest(&config_path, &manifest)?;
        println!("{} Tool {} removed.", "OK".green().bold(), name.cyan());

        // Trigger sync to apply configuration changes
        trigger_sync_and_report(path)?;
    } else {
        println!(
            "{} Tool {} not found in configuration.",
            "WARN".yellow().bold(),
            name.cyan()
        );
    }

    Ok(())
}

/// Run the add-preset command
///
/// Adds a preset to the repository's config.toml
pub fn run_add_preset(path: &Path, name: &str) -> Result<()> {
    println!(
        "{} Adding preset: {}",
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
    let mut manifest = load_manifest(&config_path)?;

    // Check if preset already exists
    if manifest.presets.contains_key(name) {
        println!(
            "{} Preset {} is already configured.",
            "OK".green().bold(),
            name.cyan()
        );
        return Ok(());
    }

    // Add the preset with an empty object
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
/// Removes a preset from the repository's config.toml
pub fn run_remove_preset(path: &Path, name: &str) -> Result<()> {
    println!(
        "{} Removing preset: {}",
        "=>".blue().bold(),
        name.cyan()
    );

    let config_path = NormalizedPath::new(path.join(CONFIG_PATH));

    // Load existing manifest
    let mut manifest = load_manifest(&config_path)?;

    // Check if preset exists
    if manifest.presets.remove(name).is_some() {
        save_manifest(&config_path, &manifest)?;
        println!("{} Preset {} removed.", "OK".green().bold(), name.cyan());
    } else {
        println!(
            "{} Preset {} not found in configuration.",
            "WARN".yellow().bold(),
            name.cyan()
        );
    }

    Ok(())
}

/// Load a manifest from the config file
///
/// If the file doesn't exist, returns an error.
fn load_manifest(path: &NormalizedPath) -> Result<Manifest> {
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
                    eprintln!(
                        "   {} {}",
                        "!".red(),
                        error
                    );
                }
            }
            Ok(())
        }
        Err(e) => {
            eprintln!(
                "{} Sync failed: {}",
                "warning:".yellow().bold(),
                e
            );
            // Don't fail the overall operation - the config change succeeded
            Ok(())
        }
    }
}

/// Generate TOML content from a manifest
///
/// Produces a clean, readable TOML representation of the manifest.
/// Note: tools and rules must come BEFORE [core] section to be parsed as top-level.
fn generate_manifest_toml(manifest: &Manifest) -> String {
    let mut content = String::new();

    // tools array - must be BEFORE [core] section to be top-level
    if !manifest.tools.is_empty() {
        content.push_str("tools = [");
        let tools_str: Vec<String> = manifest.tools.iter().map(|t| format!("\"{}\"", t)).collect();
        content.push_str(&tools_str.join(", "));
        content.push_str("]\n");
    }

    // rules array - must be BEFORE [core] section to be top-level
    if !manifest.rules.is_empty() {
        content.push_str("rules = [");
        let rules_str: Vec<String> = manifest.rules.iter().map(|r| format!("\"{}\"", r)).collect();
        content.push_str(&rules_str.join(", "));
        content.push_str("]\n");
    }

    // Add blank line before [core] if we had top-level keys
    if !manifest.tools.is_empty() || !manifest.rules.is_empty() {
        content.push('\n');
    }

    // [core] section
    content.push_str("[core]\n");
    content.push_str(&format!("mode = \"{}\"\n", manifest.core.mode));

    // [presets] section
    if !manifest.presets.is_empty() {
        content.push('\n');
        content.push_str("[presets]\n");
        for (name, value) in &manifest.presets {
            // Handle preset values
            if value.is_object() && value.as_object().map(|o| o.is_empty()).unwrap_or(false) {
                // Empty object - write as inline table
                content.push_str(&format!("\"{}\" = {{}}\n", name));
            } else {
                // Non-empty value - serialize it
                let toml_value = json_to_toml_value(value);
                content.push_str(&format!("\"{}\" = {}\n", name, toml_value));
            }
        }
    }

    content
}

/// Convert a JSON value to a TOML string representation
fn json_to_toml_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "{}".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("\"{}\"", s),
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(json_to_toml_value).collect();
            format!("[{}]", items.join(", "))
        }
        serde_json::Value::Object(obj) => {
            if obj.is_empty() {
                "{}".to_string()
            } else {
                let pairs: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| format!("{} = {}", k, json_to_toml_value(v)))
                    .collect();
                format!("{{ {} }}", pairs.join(", "))
            }
        }
    }
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
        let result = run_add_tool(path, "eslint");
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
        let result = run_add_tool(path, "eslint");
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
        let result = run_add_tool(path, "eslint");
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
        let result = run_remove_tool(path, "eslint");
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
        let result = run_remove_tool(path, "eslint");
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
        let result = run_add_preset(path, "typescript");
        assert!(result.is_ok());

        // Verify preset was added with [presets] section
        let content = read_config(path);
        assert!(content.contains("[presets]"));
        assert!(content.contains("\"typescript\""));
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
        let result = run_add_preset(path, "typescript");
        assert!(result.is_ok());

        // Verify both presets exist
        let content = read_config(path);
        assert!(content.contains("[presets]"));
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
        let result = run_remove_preset(path, "typescript");
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
        let result = run_remove_preset(path, "typescript");
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
        let result = run_add_tool(path, "eslint");
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
        assert!(content.contains("tools = [\"eslint\", \"prettier\"]"));
        assert!(content.contains("[presets]"));
        assert!(content.contains("\"typescript\" = {}"));
    }
}
