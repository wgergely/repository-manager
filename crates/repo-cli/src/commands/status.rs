//! Status command implementation

use std::path::Path;

use colored::Colorize;
use repo_core::Manifest;
use repo_fs::NormalizedPath;

use crate::error::{CliError, Result};

const CONFIG_PATH: &str = ".repository/config.toml";

/// Run the status command
pub fn run_status(path: &Path) -> Result<()> {
    let config_path = NormalizedPath::new(path.join(CONFIG_PATH));
    let native_path = config_path.to_native();

    // Check if repo is initialized
    if !native_path.exists() {
        println!("{}", "Not a repository".red().bold());
        println!();
        println!("Run {} to initialize.", "repo init".cyan());
        return Ok(());
    }

    // Load manifest
    let content = std::fs::read_to_string(&native_path)?;
    let manifest = Manifest::parse(&content).map_err(|e| CliError::user(e.to_string()))?;

    // Display status
    println!("{}", "Repository Status".bold());
    println!();

    println!("{}:   {}", "Path".dimmed(), path.display());
    println!("{}:   {}", "Mode".dimmed(), manifest.core.mode.cyan());
    println!("{}:   {}", "Config".dimmed(), CONFIG_PATH);
    println!();

    // Tools
    println!("{}:", "Enabled Tools".bold());
    if manifest.tools.is_empty() {
        println!("  {} (use {} to add)", "None".dimmed(), "repo add-tool".cyan());
    } else {
        for tool in &manifest.tools {
            let config_exists = check_tool_config_exists(path, tool);
            let status = if config_exists {
                "in sync".green()
            } else {
                "missing config".yellow()
            };
            println!("  {} {} ({})", "+".green(), tool.cyan(), status);
        }
    }
    println!();

    // Presets
    println!("{}:", "Presets".bold());
    if manifest.presets.is_empty() {
        println!("  {} (use {} to add)", "None".dimmed(), "repo add-preset".cyan());
    } else {
        for (name, _value) in &manifest.presets {
            println!("  {} {}", "+".green(), name.cyan());
        }
    }
    println!();

    // Rules
    println!("{}:", "Rules".bold());
    if manifest.rules.is_empty() {
        println!("  {} (use {} to add)", "None".dimmed(), "repo add-rule".cyan());
    } else {
        println!("  {} active rules", manifest.rules.len());
        for rule in &manifest.rules {
            println!("  {} {}", "+".green(), rule);
        }
    }

    Ok(())
}

/// Check if a tool's config file exists
fn check_tool_config_exists(path: &Path, tool: &str) -> bool {
    let config_file = match tool {
        "claude" => "CLAUDE.md",
        "cursor" => ".cursorrules",
        "aider" => ".aider.conf.yml",
        "gemini" => "GEMINI.md",
        "cline" => ".clinerules",
        "roo" => ".roorules",
        "copilot" => ".github/copilot-instructions.md",
        "vscode" => ".vscode/settings.json",
        "zed" => ".zed/settings.json",
        "jetbrains" => ".idea/.junie/guidelines.md",
        "windsurf" => ".windsurfrules",
        "antigravity" => ".antigravity/rules.md",
        "amazonq" => ".amazonq/rules.md",
        _ => return false,
    };
    path.join(config_file).exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo(dir: &Path) {
        std::fs::create_dir_all(dir.join(".repository")).unwrap();
        std::fs::write(
            dir.join(".repository/config.toml"),
            r#"tools = ["claude"]

[core]
mode = "standard"
"#,
        )
        .unwrap();
    }

    #[test]
    fn test_status_not_initialized() {
        let temp = TempDir::new().unwrap();
        let result = run_status(temp.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_status_initialized() {
        let temp = TempDir::new().unwrap();
        create_test_repo(temp.path());
        let result = run_status(temp.path());
        assert!(result.is_ok());
    }
}
