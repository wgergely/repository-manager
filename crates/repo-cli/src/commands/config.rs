//! Configuration display and tool info commands

use std::path::Path;

use colored::Colorize;
use repo_fs::NormalizedPath;
use repo_tools::{ToolCategory, ToolRegistry};

use crate::commands::tool::load_manifest;
use crate::error::{CliError, Result};

/// Path to config.toml within a repository
const CONFIG_PATH: &str = ".repository/config.toml";

/// Display the current repository configuration
pub fn run_config_show(path: &Path, json: bool) -> Result<()> {
    let config_path = NormalizedPath::new(path.join(CONFIG_PATH));
    let manifest = load_manifest(&config_path)?;

    if json {
        let output = serde_json::json!({
            "mode": manifest.core.mode,
            "tools": manifest.tools,
            "rules": manifest.rules,
            "presets": manifest.presets.keys().collect::<Vec<_>>(),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&output).unwrap_or_default()
        );
        return Ok(());
    }

    println!("{}", "Repository Configuration".bold());
    println!();

    println!("  {:<12} {}", "Mode:".dimmed(), manifest.core.mode);
    println!();

    // Tools
    if manifest.tools.is_empty() {
        println!("  {:<12} {}", "Tools:".dimmed(), "(none)".dimmed());
    } else {
        println!("  {}:", "Tools".dimmed());
        for tool in &manifest.tools {
            println!("    {} {}", "+".green(), tool);
        }
    }
    println!();

    // Presets
    if manifest.presets.is_empty() {
        println!("  {:<12} {}", "Presets:".dimmed(), "(none)".dimmed());
    } else {
        println!("  {}:", "Presets".dimmed());
        for name in manifest.presets.keys() {
            println!("    {} {}", "+".green(), name);
        }
    }
    println!();

    // Rules
    let rule_count = manifest.rules.len();
    if rule_count == 0 {
        println!("  {:<12} {}", "Rules:".dimmed(), "(none)".dimmed());
    } else {
        println!("  {}:", "Rules".dimmed());
        for rule in &manifest.rules {
            println!("    {} {}", "+".green(), rule);
        }
    }

    Ok(())
}

/// Display detailed information about a specific tool
pub fn run_tool_info(path: &Path, name: &str) -> Result<()> {
    let registry = ToolRegistry::with_builtins();

    let reg = registry.get(name).ok_or_else(|| {
        CliError::user(format!(
            "Unknown tool '{}'. Use 'repo list-tools' to see available tools.",
            name
        ))
    })?;

    let category_str = match reg.category {
        ToolCategory::Ide => "IDE",
        ToolCategory::CliAgent => "CLI Agent",
        ToolCategory::Autonomous => "Autonomous Agent",
        ToolCategory::Copilot => "Copilot",
    };

    println!("{}", "Tool Information".bold());
    println!();
    println!("  {:<16} {}", "Name:".dimmed(), reg.name);
    println!("  {:<16} {}", "Slug:".dimmed(), reg.slug);
    println!("  {:<16} {}", "Category:".dimmed(), category_str);
    println!(
        "  {:<16} {}",
        "Config path:".dimmed(),
        reg.definition.integration.config_path
    );

    if !reg.definition.integration.additional_paths.is_empty() {
        for extra in &reg.definition.integration.additional_paths {
            println!("  {:<16} {}", "".dimmed(), extra);
        }
    }

    // Capabilities
    println!();
    println!("  {}:", "Capabilities".dimmed());
    println!(
        "    Instructions:  {}",
        if reg.supports_instructions() {
            "yes".green()
        } else {
            "no".dimmed()
        }
    );
    println!(
        "    MCP:           {}",
        if reg.supports_mcp() {
            "yes".green()
        } else {
            "no".dimmed()
        }
    );
    println!(
        "    Rules dir:     {}",
        if reg.supports_rules_directory() {
            "yes".green()
        } else {
            "no".dimmed()
        }
    );

    // Check if active in current project
    let config_path = NormalizedPath::new(path.join(CONFIG_PATH));
    match load_manifest(&config_path) {
        Ok(manifest) => {
            let is_active = manifest.tools.iter().any(|t| t == name);
            println!();
            if is_active {
                println!(
                    "  {:<16} {}",
                    "Status:".dimmed(),
                    "Active (in current project)".green()
                );
            } else {
                println!("  {:<16} {}", "Status:".dimmed(), "Not active".dimmed());
            }
        }
        Err(_) => {
            // No config.toml found, skip status
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config(dir: &Path, content: &str) {
        let git_dir = dir.join(".git");
        std::fs::create_dir_all(&git_dir).unwrap();
        let repo_dir = dir.join(".repository");
        std::fs::create_dir_all(&repo_dir).unwrap();
        std::fs::write(repo_dir.join("config.toml"), content).unwrap();
    }

    #[test]
    fn test_config_show_runs() {
        let temp_dir = TempDir::new().unwrap();
        create_test_config(
            temp_dir.path(),
            "tools = [\"cursor\", \"claude\"]\n\n[core]\nmode = \"standard\"\n",
        );
        let result = run_config_show(temp_dir.path(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_show_json_runs() {
        let temp_dir = TempDir::new().unwrap();
        create_test_config(
            temp_dir.path(),
            "tools = [\"cursor\"]\n\n[core]\nmode = \"standard\"\n",
        );
        let result = run_config_show(temp_dir.path(), true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_show_no_config() {
        let temp_dir = TempDir::new().unwrap();
        let result = run_config_show(temp_dir.path(), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_info_known_tool() {
        let temp_dir = TempDir::new().unwrap();
        create_test_config(
            temp_dir.path(),
            "tools = [\"claude\"]\n\n[core]\nmode = \"standard\"\n",
        );
        let result = run_tool_info(temp_dir.path(), "claude");
        assert!(result.is_ok());
    }

    #[test]
    fn test_tool_info_unknown_tool() {
        let temp_dir = TempDir::new().unwrap();
        let result = run_tool_info(temp_dir.path(), "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_info_without_config() {
        let temp_dir = TempDir::new().unwrap();
        // No config.toml -- should still show tool info, just skip status
        let result = run_tool_info(temp_dir.path(), "cursor");
        assert!(result.is_ok());
    }
}
