//! Skills marketplace command implementations
//!
//! Provides list/search/install operations for discovering and installing
//! extensions from the built-in registry.

use colored::Colorize;
use repo_extensions::ExtensionRegistry;

use crate::error::Result;

/// List all available skills in the registry.
pub fn run_skills_list(registry: &ExtensionRegistry) -> Result<()> {
    let names = registry.known_extensions();

    if names.is_empty() {
        println!("No skills available.");
        return Ok(());
    }

    println!("{} Available skills:", "=>".blue().bold());
    println!();

    for name in &names {
        if let Some(entry) = registry.get(name) {
            println!(
                "  {} {}",
                entry.name.cyan().bold(),
                format!("- {}", entry.description).dimmed()
            );
            println!("    {}", entry.source.dimmed());
        }
    }

    println!();
    println!(
        "{} {} skill(s) available. Use {} to add one.",
        "OK".green().bold(),
        names.len(),
        "repo skills install <name>".cyan()
    );

    Ok(())
}

/// Search skills by name or description substring.
pub fn run_skills_search(registry: &ExtensionRegistry, query: &str) -> Result<()> {
    let query_lower = query.to_lowercase();
    let names = registry.known_extensions();

    let matches: Vec<&str> = names
        .iter()
        .filter(|name| {
            if let Some(entry) = registry.get(name) {
                entry.name.to_lowercase().contains(&query_lower)
                    || entry.description.to_lowercase().contains(&query_lower)
            } else {
                false
            }
        })
        .map(|s| s.as_str())
        .collect();

    if matches.is_empty() {
        println!(
            "{} No skills matching '{}'.",
            "WARN".yellow().bold(),
            query
        );
        return Ok(());
    }

    println!(
        "{} Skills matching '{}':",
        "=>".blue().bold(),
        query.cyan()
    );
    println!();

    for name in &matches {
        if let Some(entry) = registry.get(name) {
            println!(
                "  {} {}",
                entry.name.cyan().bold(),
                format!("- {}", entry.description).dimmed()
            );
            println!("    {}", entry.source.dimmed());
        }
    }

    println!();
    println!(
        "{} {} result(s).",
        "OK".green().bold(),
        matches.len()
    );

    Ok(())
}

/// Install a skill by name from the registry.
pub fn run_skills_install(registry: &ExtensionRegistry, name: &str) -> Result<()> {
    let entry = match registry.get(name) {
        Some(e) => e,
        None => {
            println!(
                "{} Skill '{}' not found in the registry.",
                "ERROR".red().bold(),
                name
            );
            println!(
                "Use {} to see available skills.",
                "repo skills list".cyan()
            );
            return Ok(());
        }
    };

    println!(
        "{} Installing skill: {} ({})",
        "=>".blue().bold(),
        entry.name.cyan().bold(),
        entry.description
    );

    // Delegate to the extension install machinery
    crate::commands::extension::handle_extension_install(&entry.source, false)?;

    println!(
        "{} Skill '{}' installed.",
        "OK".green().bold(),
        name
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_extensions::{ExtensionEntry, ExtensionRegistry};

    fn test_registry() -> ExtensionRegistry {
        let mut registry = ExtensionRegistry::new();
        registry.register(ExtensionEntry {
            name: "alpha".to_string(),
            description: "Alpha framework for testing".to_string(),
            source: "https://example.com/alpha.git".to_string(),
        });
        registry.register(ExtensionEntry {
            name: "beta-tools".to_string(),
            description: "Beta developer tools".to_string(),
            source: "https://example.com/beta.git".to_string(),
        });
        registry
    }

    #[test]
    fn test_skills_list_empty_registry() {
        let registry = ExtensionRegistry::new();
        let result = run_skills_list(&registry);
        assert!(result.is_ok());
    }

    #[test]
    fn test_skills_list_with_entries() {
        let registry = test_registry();
        let result = run_skills_list(&registry);
        assert!(result.is_ok());
    }

    #[test]
    fn test_skills_search_by_name() {
        let registry = test_registry();
        let result = run_skills_search(&registry, "alpha");
        assert!(result.is_ok());
    }

    #[test]
    fn test_skills_search_by_description() {
        let registry = test_registry();
        let result = run_skills_search(&registry, "developer");
        assert!(result.is_ok());
    }

    #[test]
    fn test_skills_search_no_match() {
        let registry = test_registry();
        let result = run_skills_search(&registry, "nonexistent");
        assert!(result.is_ok());
    }

    #[test]
    fn test_skills_search_case_insensitive() {
        let registry = test_registry();
        let result = run_skills_search(&registry, "ALPHA");
        assert!(result.is_ok());
    }

    #[test]
    fn test_skills_install_not_found() {
        let registry = test_registry();
        let result = run_skills_install(&registry, "nonexistent");
        assert!(result.is_ok());
    }
}
