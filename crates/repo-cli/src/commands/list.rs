//! List commands for tools and presets

use colored::Colorize;
use repo_meta::Registry;
use repo_tools::{ToolCategory, ToolRegistry};

use crate::error::Result;

/// Run the list-tools command
pub fn run_list_tools(category_filter: Option<&str>) -> Result<()> {
    let registry = ToolRegistry::with_builtins();

    // Parse category filter if provided
    let filter: Option<ToolCategory> = match category_filter {
        Some("ide") => Some(ToolCategory::Ide),
        Some("cli-agent") => Some(ToolCategory::CliAgent),
        Some("autonomous") => Some(ToolCategory::Autonomous),
        Some("copilot") => Some(ToolCategory::Copilot),
        Some(other) => {
            eprintln!(
                "{} Unknown category '{}'. Valid: ide, cli-agent, autonomous, copilot",
                "warning:".yellow().bold(),
                other
            );
            None
        }
        None => None,
    };

    println!("{}", "Available Tools".bold());
    println!();

    // Group by category
    let categories = [
        (ToolCategory::Ide, "IDE Tools"),
        (ToolCategory::CliAgent, "CLI Agents"),
        (ToolCategory::Autonomous, "Autonomous Agents"),
        (ToolCategory::Copilot, "Copilots"),
    ];

    for (cat, label) in categories {
        // Skip if filtering and this isn't the category
        if let Some(f) = filter {
            if f != cat {
                continue;
            }
        }

        let tools = registry.by_category(cat);
        if tools.is_empty() {
            continue;
        }

        println!("{}:", label.cyan().bold());
        for slug in tools {
            if let Some(reg) = registry.get(slug) {
                let config = &reg.definition.integration.config_path;
                println!(
                    "  {:<14} {} ({})",
                    slug.green(),
                    reg.name,
                    config.dimmed()
                );
            }
        }
        println!();
    }

    let total = registry.len();
    println!(
        "{} {} tools available. Use {} to add one.",
        "Total:".dimmed(),
        total,
        "repo add-tool <name>".cyan()
    );

    Ok(())
}

/// Run the list-presets command
pub fn run_list_presets() -> Result<()> {
    let registry = Registry::with_builtins();

    println!("{}", "Available Presets".bold());
    println!();

    for preset in registry.list_presets() {
        if let Some(provider) = registry.get_provider(&preset) {
            println!("  {:<16} (provider: {})", preset.green(), provider.dimmed());
        }
    }

    println!();
    println!(
        "{} {} presets available. Use {} to add one.",
        "Total:".dimmed(),
        registry.len(),
        "repo add-preset <name>".cyan()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_tools_runs() {
        let result = run_list_tools(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_tools_with_category() {
        let result = run_list_tools(Some("ide"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_presets_runs() {
        let result = run_list_presets();
        assert!(result.is_ok());
    }
}
