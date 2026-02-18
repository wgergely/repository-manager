//! Interactive prompts for CLI commands
//!
//! Uses dialoguer for terminal-based interactive selection.

use colored::Colorize;
use dialoguer::{Confirm, Input, MultiSelect, Select};
use repo_meta::Registry;
use repo_tools::ToolRegistry;

use crate::commands::init::InitConfig;
use crate::error::Result;

/// Available repository modes
const MODES: &[&str] = &["worktrees", "standard"];

/// Run interactive init prompts
///
/// Prompts the user for project configuration and returns an InitConfig.
pub fn interactive_init(default_name: &str) -> Result<InitConfig> {
    println!();

    // Project name
    let name: String = Input::new()
        .with_prompt("Project name")
        .default(if default_name == "." {
            "my-project".to_string()
        } else {
            default_name.to_string()
        })
        .interact_text()?;

    // Mode selection
    let mode_idx = Select::new()
        .with_prompt("Repository mode")
        .items(MODES)
        .default(0)
        .interact()?;
    let mode = MODES[mode_idx].to_string();

    // Tool selection (multi-select) - dynamically from ToolRegistry
    let tool_registry = ToolRegistry::with_builtins();
    let available_tools = tool_registry.list();
    let tool_indices = MultiSelect::new()
        .with_prompt("Select tools (space to toggle, enter to confirm)")
        .items(&available_tools)
        .interact()?;
    let tools: Vec<String> = tool_indices
        .iter()
        .map(|&i| available_tools[i].to_string())
        .collect();

    // Preset selection (multi-select) - dynamically from Registry
    let preset_registry = Registry::with_builtins();
    let available_presets = preset_registry.list_presets();
    let preset_indices = MultiSelect::new()
        .with_prompt("Select presets (space to toggle, enter to confirm)")
        .items(&available_presets)
        .interact()?;
    let presets: Vec<String> = preset_indices
        .iter()
        .map(|&i| available_presets[i].clone())
        .collect();

    // Remote URL (optional)
    let add_remote = Confirm::new()
        .with_prompt("Add a git remote?")
        .default(false)
        .interact()?;

    let remote = if add_remote {
        let url: String = Input::new().with_prompt("Remote URL").interact_text()?;
        if url.trim().is_empty() {
            None
        } else {
            Some(url)
        }
    } else {
        None
    };

    // Show summary and confirm
    println!();
    println!("{}", "Summary:".bold());
    println!("  {}: {}", "Project".dimmed(), name.cyan());
    println!("  {}: {}", "Mode".dimmed(), mode.cyan());
    if tools.is_empty() {
        println!("  {}: {}", "Tools".dimmed(), "(none)".dimmed());
    } else {
        println!("  {}: {}", "Tools".dimmed(), tools.join(", ").cyan());
    }
    if presets.is_empty() {
        println!("  {}: {}", "Presets".dimmed(), "(none)".dimmed());
    } else {
        println!("  {}: {}", "Presets".dimmed(), presets.join(", ").cyan());
    }
    match &remote {
        Some(url) => println!("  {}: {}", "Remote".dimmed(), url.cyan()),
        None => println!("  {}: {}", "Remote".dimmed(), "(none)".dimmed()),
    }
    println!();

    let proceed = Confirm::new()
        .with_prompt("Proceed?")
        .default(true)
        .interact()?;

    if !proceed {
        return Err(crate::error::CliError::user("Init cancelled by user."));
    }

    Ok(InitConfig {
        name,
        mode,
        tools,
        presets,
        remote,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_available_modes() {
        assert!(MODES.contains(&"worktrees"));
        assert!(MODES.contains(&"standard"));
    }

    #[test]
    fn test_tool_registry_has_tools() {
        let registry = ToolRegistry::with_builtins();
        let tools = registry.list();
        assert!(tools.contains(&"vscode"));
        assert!(tools.contains(&"cursor"));
        assert!(tools.contains(&"claude"));
        assert!(tools.len() >= 10, "Should have at least 10 tools");
    }

    #[test]
    fn test_preset_registry_has_presets() {
        let registry = Registry::with_builtins();
        let presets = registry.list_presets();
        assert!(!presets.is_empty(), "Should have presets available");
    }
}
