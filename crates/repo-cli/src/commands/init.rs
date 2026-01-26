//! Init command implementation
//!
//! Initializes a new repository with Repository Manager configuration.

use std::path::Path;
use std::process::Command;

use colored::Colorize;

use crate::error::{CliError, Result};

/// Run the init command
///
/// Initializes a repository with the specified mode, tools, and presets.
pub fn run_init(path: &Path, mode: &str, tools: &[String], presets: &[String]) -> Result<()> {
    println!(
        "{} Initializing repository in {} mode...",
        "=>".blue().bold(),
        mode.cyan()
    );

    if !tools.is_empty() {
        println!("   Tools: {}", tools.join(", ").yellow());
    }
    if !presets.is_empty() {
        println!("   Presets: {}", presets.join(", ").yellow());
    }

    init_repository(path, mode, tools, presets)?;

    println!("{} Repository initialized!", "OK".green().bold());
    Ok(())
}

/// Initialize a repository with the given configuration
///
/// This function:
/// - Creates the `.repository` directory
/// - Creates `config.toml` with the specified mode, tools, and presets
/// - Initializes git if `.git` doesn't exist
/// - For worktrees mode, creates the `main/` directory
pub fn init_repository(
    path: &Path,
    mode: &str,
    tools: &[String],
    presets: &[String],
) -> Result<()> {
    // Validate mode (accept both "worktree" and "worktrees")
    let is_worktree_mode = mode == "worktree" || mode == "worktrees";
    if mode != "standard" && !is_worktree_mode {
        return Err(CliError::user(format!(
            "Invalid mode '{}'. Must be 'standard' or 'worktree'.",
            mode
        )));
    }

    // Create .repository directory
    let repo_dir = path.join(".repository");
    std::fs::create_dir_all(&repo_dir)?;

    // Generate and write config.toml
    let config_content = generate_config(mode, tools, presets);
    let config_path = repo_dir.join("config.toml");
    std::fs::write(&config_path, config_content)?;

    // Initialize git if .git doesn't exist
    let git_dir = path.join(".git");
    if !git_dir.exists() {
        init_git(path)?;
    }

    // For worktree mode, create main/ directory
    if is_worktree_mode {
        let main_dir = path.join("main");
        if !main_dir.exists() {
            std::fs::create_dir_all(&main_dir)?;
        }
    }

    Ok(())
}

/// Generate the config.toml content
pub fn generate_config(mode: &str, tools: &[String], presets: &[String]) -> String {
    let mut config = String::new();

    // [core] section
    config.push_str("[core]\n");
    config.push_str(&format!("mode = \"{}\"\n", mode));

    // Add tools if present
    if !tools.is_empty() {
        config.push('\n');
        config.push_str("[tools]\n");
        for tool in tools {
            config.push_str(&format!("{} = {{}}\n", tool));
        }
    }

    // Add presets if present
    if !presets.is_empty() {
        config.push('\n');
        config.push_str("[presets]\n");
        for preset in presets {
            config.push_str(&format!("{} = {{}}\n", preset));
        }
    }

    config
}

/// Initialize git in the given directory
fn init_git(path: &Path) -> Result<()> {
    let output = Command::new("git")
        .arg("init")
        .current_dir(path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CliError::user(format!("Failed to initialize git: {}", stderr)));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_creates_repository_structure() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        let result = init_repository(path, "standard", &[], &[]);
        assert!(result.is_ok());

        // Verify .repository directory exists
        let repo_dir = path.join(".repository");
        assert!(repo_dir.exists(), ".repository directory should exist");
        assert!(repo_dir.is_dir(), ".repository should be a directory");

        // Verify config.toml exists
        let config_path = repo_dir.join("config.toml");
        assert!(config_path.exists(), "config.toml should exist");

        // Verify config content
        let config_content = std::fs::read_to_string(&config_path).unwrap();
        assert!(config_content.contains("[core]"));
        assert!(config_content.contains("mode = \"standard\""));
    }

    #[test]
    fn test_init_with_tools() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        let tools = vec!["eslint".to_string(), "prettier".to_string()];
        let result = init_repository(path, "standard", &tools, &[]);
        assert!(result.is_ok());

        // Verify tools in config
        let config_path = path.join(".repository").join("config.toml");
        let config_content = std::fs::read_to_string(&config_path).unwrap();
        assert!(config_content.contains("[tools]"));
        assert!(config_content.contains("eslint = {}"));
        assert!(config_content.contains("prettier = {}"));
    }

    #[test]
    fn test_init_with_presets() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        let presets = vec!["typescript".to_string(), "react".to_string()];
        let result = init_repository(path, "standard", &[], &presets);
        assert!(result.is_ok());

        // Verify presets in config
        let config_path = path.join(".repository").join("config.toml");
        let config_content = std::fs::read_to_string(&config_path).unwrap();
        assert!(config_content.contains("[presets]"));
        assert!(config_content.contains("typescript = {}"));
        assert!(config_content.contains("react = {}"));
    }

    #[test]
    fn test_generate_config() {
        // Test basic config
        let config = generate_config("standard", &[], &[]);
        assert_eq!(config, "[core]\nmode = \"standard\"\n");

        // Test with tools
        let tools = vec!["eslint".to_string()];
        let config = generate_config("standard", &tools, &[]);
        assert!(config.contains("[core]\nmode = \"standard\"\n"));
        assert!(config.contains("[tools]\n"));
        assert!(config.contains("eslint = {}"));

        // Test with presets
        let presets = vec!["typescript".to_string()];
        let config = generate_config("standard", &[], &presets);
        assert!(config.contains("[core]\nmode = \"standard\"\n"));
        assert!(config.contains("[presets]\n"));
        assert!(config.contains("typescript = {}"));

        // Test with both tools and presets
        let tools = vec!["eslint".to_string(), "prettier".to_string()];
        let presets = vec!["typescript".to_string()];
        let config = generate_config("worktree", &tools, &presets);
        assert!(config.contains("[core]\nmode = \"worktree\"\n"));
        assert!(config.contains("[tools]\n"));
        assert!(config.contains("eslint = {}"));
        assert!(config.contains("prettier = {}"));
        assert!(config.contains("[presets]\n"));
        assert!(config.contains("typescript = {}"));
    }

    #[test]
    fn test_init_worktree_mode_creates_main_directory() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        let result = init_repository(path, "worktree", &[], &[]);
        assert!(result.is_ok());

        // Verify main/ directory exists for worktree mode
        let main_dir = path.join("main");
        assert!(main_dir.exists(), "main/ directory should exist for worktree mode");
        assert!(main_dir.is_dir(), "main should be a directory");
    }

    #[test]
    fn test_init_standard_mode_does_not_create_main_directory() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        let result = init_repository(path, "standard", &[], &[]);
        assert!(result.is_ok());

        // Verify main/ directory does NOT exist for standard mode
        let main_dir = path.join("main");
        assert!(!main_dir.exists(), "main/ directory should NOT exist for standard mode");
    }

    #[test]
    fn test_init_invalid_mode() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        let result = init_repository(path, "invalid", &[], &[]);
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_msg = format!("{}", err);
        assert!(err_msg.contains("Invalid mode"));
    }

    #[test]
    fn test_init_initializes_git() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Ensure .git doesn't exist
        assert!(!path.join(".git").exists());

        let result = init_repository(path, "standard", &[], &[]);
        assert!(result.is_ok());

        // Verify .git was created
        let git_dir = path.join(".git");
        assert!(git_dir.exists(), ".git directory should exist after init");
    }

    #[test]
    fn test_init_does_not_reinitialize_git() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Pre-create .git directory to simulate existing repo
        std::fs::create_dir(path.join(".git")).unwrap();
        std::fs::write(path.join(".git").join("marker"), "test").unwrap();

        let result = init_repository(path, "standard", &[], &[]);
        assert!(result.is_ok());

        // Verify marker file still exists (git was not reinitialized)
        let marker = path.join(".git").join("marker");
        assert!(marker.exists(), "marker file should still exist - git should not be reinitialized");
    }
}
