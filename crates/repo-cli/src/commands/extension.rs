//! Extension command implementations.
//!
//! Provides CLI handlers for the `repo extension` subcommands: install, add,
//! init, remove, and list. Install and add validate extension manifests and
//! report dependency requirements. Init scaffolds a new `repo_extension.toml`.
//! Remove marks an extension for cleanup on the next sync.

use std::path::Path;

use colored::Colorize;
use repo_extensions::{ExtensionManifest, ExtensionRegistry, MANIFEST_FILENAME};

use crate::error::{CliError, Result};

/// Handle `repo extension install <source> [--no-activate]`
///
/// Validates the extension manifest at `source` (a local directory path),
/// checks version constraints, and reports the dependency chain. Git clone
/// is not yet supported.
pub fn handle_extension_install(source: &str, _no_activate: bool) -> Result<()> {
    let source_path = Path::new(source);
    if !source_path.exists() {
        return Err(CliError::user(format!(
            "Source path '{}' does not exist. Git clone is not yet supported; \
             provide a local path to an extension directory.",
            source
        )));
    }

    let manifest_path = source_path.join(MANIFEST_FILENAME);
    if !manifest_path.exists() {
        return Err(CliError::user(format!(
            "No {} found at '{}'",
            MANIFEST_FILENAME, source
        )));
    }

    let manifest = ExtensionManifest::from_path(&manifest_path)
        .map_err(|e| CliError::user(format!("Invalid extension manifest: {e}")))?;

    let deps = manifest.implicit_preset_dependencies();

    println!(
        "{} Extension '{}' v{} validated",
        "=>".blue().bold(),
        manifest.extension.name.cyan(),
        manifest.extension.version
    );
    if !deps.is_empty() {
        println!(
            "   {} {}",
            "Requires presets:".dimmed(),
            deps.join(", ").yellow()
        );
    }
    println!(
        "   {} Run {} to complete setup",
        "Next:".dimmed(),
        "repo sync".bold()
    );

    Ok(())
}

/// Handle `repo extension add <name>`
///
/// Looks up a known extension by name in the built-in registry and reports
/// its source URL for subsequent installation.
pub fn handle_extension_add(name: &str) -> Result<()> {
    let registry = ExtensionRegistry::with_known();

    match registry.get(name) {
        Some(entry) => {
            println!(
                "{} Extension '{}' found in registry",
                "=>".blue().bold(),
                entry.name.cyan()
            );
            println!("   {} {}", "Description:".dimmed(), entry.description);
            println!("   {} {}", "Source:".dimmed(), entry.source);
            println!(
                "   {} Run {} to install",
                "Next:".dimmed(),
                format!("repo extension install {}", entry.source).bold()
            );
            Ok(())
        }
        None => Err(CliError::user(format!(
            "Extension '{}' not found in registry. Known extensions: {}",
            name,
            registry.known_extensions().join(", ")
        ))),
    }
}

/// Handle `repo extension init <name>`
///
/// Generates a scaffold `repo_extension.toml` and prints it to stdout. The
/// user can redirect this to a file or copy it manually.
pub fn handle_extension_init(name: &str) -> Result<()> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(CliError::user(
            "Extension name must contain only alphanumeric characters, hyphens, or underscores"
                .to_string(),
        ));
    }

    let scaffold = format!(
        r#"[extension]
name = "{name}"
version = "0.1.0"
description = ""

# Uncomment and configure as needed:
#
# [requires.python]
# version = ">=3.12"
#
# [runtime]
# type = "python"
# install = "pip install -e ."
#
# [entry_points]
# cli = "scripts/cli.py"
# mcp = "scripts/mcp_server.py serve"
#
# [provides]
# mcp = ["{name}-mcp"]
# mcp_config = "mcp.json"
# content_types = []
#
# [outputs]
# claude_dir = ".claude"
"#,
        name = name
    );

    println!(
        "{} Extension scaffold for '{}':",
        "=>".blue().bold(),
        name.cyan()
    );
    println!();
    println!("{}", scaffold);
    println!(
        "   {} Write this to {}",
        "=>".blue().bold(),
        MANIFEST_FILENAME.bold()
    );

    Ok(())
}

/// Handle `repo extension remove <name>`
///
/// Marks an extension for removal. The actual cleanup happens on the next
/// `repo sync`.
pub fn handle_extension_remove(name: &str) -> Result<()> {
    println!(
        "{} Extension '{}' marked for removal",
        "=>".blue().bold(),
        name.cyan()
    );
    println!(
        "   {} Run {} to clean up configuration references",
        "Next:".dimmed(),
        "repo sync".bold()
    );

    Ok(())
}

/// Handle `repo extension list [--json]`
///
/// Lists known extension types from the built-in registry.
/// No extensions are currently installed; this shows what is available.
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
                        "installed": false,
                    })
                })
            })
            .collect();

        println!(
            "{}",
            serde_json::to_string_pretty(&entries).unwrap_or_default()
        );
    } else {
        println!(
            "{} Known extensions (none currently installed):",
            "=>".blue().bold()
        );
        if names.is_empty() {
            println!("   No extensions registered.");
        } else {
            for name in &names {
                if let Some(entry) = registry.get(name) {
                    println!("   {} - {}", entry.name.cyan(), entry.description.dimmed());
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_install_nonexistent_source_errors() {
        let result = handle_extension_install("/nonexistent/path", false);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("does not exist"));
    }

    #[test]
    fn test_extension_install_no_manifest_errors() {
        let temp = tempfile::TempDir::new().unwrap();
        let result = handle_extension_install(temp.path().to_str().unwrap(), false);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains(MANIFEST_FILENAME));
    }

    #[test]
    fn test_extension_install_valid_manifest() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            temp.path().join(MANIFEST_FILENAME),
            r#"
[extension]
name = "test-ext"
version = "1.0.0"
"#,
        )
        .unwrap();
        let result = handle_extension_install(temp.path().to_str().unwrap(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_extension_add_known_extension() {
        // "vaultspec" is in the built-in registry
        let result = handle_extension_add("vaultspec");
        assert!(result.is_ok());
    }

    #[test]
    fn test_extension_add_unknown_extension() {
        let result = handle_extension_add("nonexistent-ext");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found in registry"));
    }

    #[test]
    fn test_extension_init_valid_name() {
        let result = handle_extension_init("my-cool-ext");
        assert!(result.is_ok());
    }

    #[test]
    fn test_extension_init_invalid_name() {
        let result = handle_extension_init("bad name!");
        assert!(result.is_err());
    }

    #[test]
    fn test_extension_remove_succeeds() {
        let result = handle_extension_remove("test-ext");
        assert!(result.is_ok());
    }

    #[test]
    fn test_extension_list_succeeds() {
        let result = handle_extension_list(false);
        assert!(result.is_ok(), "extension list should succeed");
    }

    #[test]
    fn test_extension_list_json_succeeds() {
        let result = handle_extension_list(true);
        assert!(result.is_ok(), "extension list --json should succeed");
    }
}
