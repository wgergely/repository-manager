//! Extension command implementations
//!
//! Provides stub handlers for extension lifecycle operations.

use colored::Colorize;
use repo_extensions::ExtensionRegistry;

use crate::error::Result;

/// Handle `repo extension install <source> [--no-activate]`
pub fn handle_extension_install(source: &str, no_activate: bool) -> Result<()> {
    println!(
        "{} Installing extension from {}...",
        "=>".blue().bold(),
        source.cyan()
    );

    if no_activate {
        println!(
            "   {} Extension will not be activated after install.",
            "note:".yellow()
        );
    }

    // TODO: Implement actual install logic (clone, validate manifest, activate)
    println!(
        "{} Extension install from {} (stub - not yet implemented)",
        "OK".green().bold(),
        source.cyan()
    );

    Ok(())
}

/// Handle `repo extension add <name>`
pub fn handle_extension_add(name: &str) -> Result<()> {
    let registry = ExtensionRegistry::with_known();

    if let Some(entry) = registry.get(name) {
        println!(
            "{} Adding known extension {} ({})",
            "=>".blue().bold(),
            name.cyan(),
            entry.description.dimmed()
        );
        println!("   Source: {}", entry.source.yellow());
        // TODO: Implement actual add logic (resolve from registry, install, activate)
        println!(
            "{} Extension {} added (stub - not yet implemented)",
            "OK".green().bold(),
            name.cyan()
        );
    } else {
        println!(
            "{} Extension {} is not in the known registry.",
            "warn:".yellow().bold(),
            name.cyan()
        );
        println!(
            "   Use {} to install from a URL or local path.",
            "repo extension install <source>".dimmed()
        );
    }

    Ok(())
}

/// Handle `repo extension init <name>`
pub fn handle_extension_init(name: &str) -> Result<()> {
    println!(
        "{} Initializing new extension scaffold {}...",
        "=>".blue().bold(),
        name.cyan()
    );

    // TODO: Implement actual init logic (create repo_extension.toml, directory structure)
    println!(
        "{} Extension {} initialized (stub - not yet implemented)",
        "OK".green().bold(),
        name.cyan()
    );

    Ok(())
}

/// Handle `repo extension remove <name>`
pub fn handle_extension_remove(name: &str) -> Result<()> {
    println!(
        "{} Removing extension {}...",
        "=>".blue().bold(),
        name.cyan()
    );

    // TODO: Implement actual remove logic (deactivate, remove files, update config)
    println!(
        "{} Extension {} removed (stub - not yet implemented)",
        "OK".green().bold(),
        name.cyan()
    );

    Ok(())
}

/// Handle `repo extension list [--json]`
pub fn handle_extension_list(json: bool) -> Result<()> {
    let registry = ExtensionRegistry::with_known();
    let names = registry.known_extensions();

    if json {
        let entries: Vec<serde_json::Value> = names
            .iter()
            .filter_map(|name| {
                registry.get(name).map(|entry| {
                    serde_json::json!({
                        "name": entry.name,
                        "description": entry.description,
                        "source": entry.source,
                    })
                })
            })
            .collect();

        println!(
            "{}",
            serde_json::to_string_pretty(&entries).unwrap_or_default()
        );
    } else {
        println!("{} Known extensions:", "=>".blue().bold());
        if names.is_empty() {
            println!("   No extensions registered.");
        } else {
            for name in &names {
                if let Some(entry) = registry.get(name) {
                    println!("   {} - {}", entry.name.cyan(), entry.description.dimmed());
                }
            }
        }
        // TODO: Also list installed extensions from config
        println!();
        println!(
            "   Use {} to add one.",
            "repo extension add <name>".dimmed()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_extension_install_stub() {
        let result = handle_extension_install("https://example.com/ext.git", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_extension_install_no_activate() {
        let result = handle_extension_install("https://example.com/ext.git", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_extension_add_known() {
        let result = handle_extension_add("vaultspec");
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_extension_add_unknown() {
        let result = handle_extension_add("nonexistent");
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_extension_init_stub() {
        let result = handle_extension_init("my-extension");
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_extension_remove_stub() {
        let result = handle_extension_remove("my-extension");
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_extension_list_text() {
        let result = handle_extension_list(false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_extension_list_json() {
        let result = handle_extension_list(true);
        assert!(result.is_ok());
    }
}
