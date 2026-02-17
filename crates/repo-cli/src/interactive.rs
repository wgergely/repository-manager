//! Interactive prompts for CLI commands
//!
//! Uses dialoguer for terminal-based interactive selection.

use dialoguer::{Confirm, Input, MultiSelect, Select};

use crate::commands::init::InitConfig;
use crate::error::Result;

/// Available repository modes
const MODES: &[&str] = &["worktrees", "standard"];

/// Available tools for selection
const AVAILABLE_TOOLS: &[&str] = &[
    "vscode",
    "cursor",
    "claude",
    "windsurf",
    "gemini",
    "antigravity",
];

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

    // Tool selection (multi-select)
    let tool_indices = MultiSelect::new()
        .with_prompt("Select tools (space to toggle, enter to confirm)")
        .items(AVAILABLE_TOOLS)
        .interact()?;
    let tools: Vec<String> = tool_indices
        .iter()
        .map(|&i| AVAILABLE_TOOLS[i].to_string())
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

    Ok(InitConfig {
        name,
        mode,
        tools,
        presets: Vec::new(), // Could add preset selection later
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
    fn test_available_tools() {
        assert!(AVAILABLE_TOOLS.contains(&"vscode"));
        assert!(AVAILABLE_TOOLS.contains(&"cursor"));
        assert!(AVAILABLE_TOOLS.contains(&"claude"));
    }
}
