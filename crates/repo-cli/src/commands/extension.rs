//! Extension command implementations
//!
//! Extension lifecycle operations are not yet implemented. These handlers
//! return errors to prevent callers from mistakenly believing an operation
//! succeeded. The `list` command is the exception: it returns known extension
//! types from the registry, which is a valid read-only operation.

use colored::Colorize;
use repo_extensions::ExtensionRegistry;

use crate::error::{CliError, Result};

/// Handle `repo extension install <source> [--no-activate]`
pub fn handle_extension_install(source: &str, _no_activate: bool) -> Result<()> {
    Err(CliError::user(format!(
        "Extension install is not yet implemented. Source: {source}"
    )))
}

/// Handle `repo extension add <name>`
pub fn handle_extension_add(name: &str) -> Result<()> {
    Err(CliError::user(format!(
        "Extension add is not yet implemented. Extension: {name}"
    )))
}

/// Handle `repo extension init <name>`
pub fn handle_extension_init(name: &str) -> Result<()> {
    Err(CliError::user(format!(
        "Extension init is not yet implemented. Extension: {name}"
    )))
}

/// Handle `repo extension remove <name>`
pub fn handle_extension_remove(name: &str) -> Result<()> {
    Err(CliError::user(format!(
        "Extension remove is not yet implemented. Extension: {name}"
    )))
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
    fn test_extension_install_returns_error() {
        let result = handle_extension_install("test-source", false);
        assert!(result.is_err(), "extension install must return an error");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not yet implemented"),
            "error message should indicate not implemented, got: {err_msg}"
        );
        assert!(
            err_msg.contains("test-source"),
            "error message should include the source, got: {err_msg}"
        );
    }

    #[test]
    fn test_extension_install_no_activate_returns_error() {
        let result = handle_extension_install("https://example.com/ext.git", true);
        assert!(
            result.is_err(),
            "extension install with no_activate must return an error"
        );
    }

    #[test]
    fn test_extension_add_returns_error() {
        let result = handle_extension_add("test-ext");
        assert!(result.is_err(), "extension add must return an error");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not yet implemented"),
            "error message should indicate not implemented, got: {err_msg}"
        );
        assert!(
            err_msg.contains("test-ext"),
            "error message should include the extension name, got: {err_msg}"
        );
    }

    #[test]
    fn test_extension_init_returns_error() {
        let result = handle_extension_init("new-ext");
        assert!(result.is_err(), "extension init must return an error");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not yet implemented"),
            "error message should indicate not implemented, got: {err_msg}"
        );
        assert!(
            err_msg.contains("new-ext"),
            "error message should include the extension name, got: {err_msg}"
        );
    }

    #[test]
    fn test_extension_remove_returns_error() {
        let result = handle_extension_remove("test-ext");
        assert!(result.is_err(), "extension remove must return an error");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not yet implemented"),
            "error message should indicate not implemented, got: {err_msg}"
        );
        assert!(
            err_msg.contains("test-ext"),
            "error message should include the extension name, got: {err_msg}"
        );
    }

    #[test]
    fn test_extension_list_succeeds() {
        // list is a valid operation that shows known extension types
        let result = handle_extension_list(false);
        assert!(result.is_ok(), "extension list should succeed");
    }

    #[test]
    fn test_extension_list_json_succeeds() {
        let result = handle_extension_list(true);
        assert!(result.is_ok(), "extension list --json should succeed");
    }
}
