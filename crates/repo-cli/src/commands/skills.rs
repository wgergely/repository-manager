//! Skills marketplace command implementations
//!
//! Provides list/search/install operations for discovering and installing
//! extensions that provide `skills` content types (see `[provides].content_types`
//! in extension manifests). The marketplace reads from the local
//! `ExtensionRegistry` (pluggable, local-first) — no external API dependency.
//!
//! The registry protocol (Tessl, OCI, custom) is TBD per the Phase 4 status
//! document. This implementation uses `ExtensionRegistry::with_known()` as the
//! on-ramp, which can be extended to remote sources without breaking changes.

use colored::Colorize;
use repo_extensions::{ExtensionRegistry, LockFile, LOCK_FILENAME};

use crate::error::{CliError, Result};

/// List all available skills in the registry, enriched with lock file status.
///
/// Shows each registered extension with its installed/available status,
/// version (if installed), and source URL. Follows the same lock-file
/// enrichment pattern as `repo extension list` (ADR-008 §8.4).
pub fn run_skills_list(registry: &ExtensionRegistry) -> Result<()> {
    let names = registry.known_extensions();

    if names.is_empty() {
        println!("No skills available in the registry.");
        return Ok(());
    }

    // Load lock file to determine installed status
    let repo_root = std::env::current_dir().unwrap_or_default();
    let lock_path = repo_root.join(".repository").join(LOCK_FILENAME);
    let lock_file = LockFile::load(&lock_path).unwrap_or_default();

    println!("{} Skills marketplace:", "=>".blue().bold());
    println!();

    for name in &names {
        if let Some(entry) = registry.get(name) {
            let locked = lock_file.get(name);
            let status = if locked.is_some() {
                "installed".green().to_string()
            } else {
                "available".yellow().to_string()
            };
            let version = locked
                .map(|l| l.version.as_str())
                .unwrap_or("-");

            println!(
                "  {} {} [{}]",
                entry.name.cyan().bold(),
                version.dimmed(),
                status
            );
            println!(
                "    {}",
                entry.description.dimmed()
            );
            println!(
                "    {}",
                entry.source.dimmed()
            );
        }
    }

    println!();
    println!(
        "{} {} skill(s) registered. Install with {}.",
        "OK".green().bold(),
        names.len(),
        "repo skills install <name>".cyan()
    );

    Ok(())
}

/// Search skills by name or description substring (case-insensitive).
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
        "{} {} result(s). Install with {}.",
        "OK".green().bold(),
        matches.len(),
        "repo skills install <name>".cyan()
    );

    Ok(())
}

/// Install a skill by name from the registry.
///
/// Delegates to `handle_extension_add` for registry lookup, then to
/// `handle_extension_install` if the extension source is a local path.
/// Per ADR-007 §7.4, `handle_extension_install` is the entry point for
/// install execution. Since git clone is not yet supported, this function
/// checks for a local source path and falls back to printing install
/// guidance when only a remote URL is available.
pub fn run_skills_install(registry: &ExtensionRegistry, name: &str) -> Result<()> {
    let entry = registry.get(name).ok_or_else(|| {
        CliError::user(format!(
            "Skill '{}' not found in the registry. Known skills: {}",
            name,
            registry.known_extensions().join(", ")
        ))
    })?;

    println!(
        "{} Installing skill: {} ({})",
        "=>".blue().bold(),
        entry.name.cyan().bold(),
        entry.description
    );

    // Check if the source is a local path (handle_extension_install requires
    // a local path; git clone is not yet supported per ADR-007).
    let source_path = std::path::Path::new(&entry.source);
    if source_path.exists() {
        crate::commands::extension::handle_extension_install(&entry.source, false)?;
        println!(
            "{} Skill '{}' installed successfully.",
            "OK".green().bold(),
            name
        );
    } else {
        // Remote URL — git clone not yet implemented. Print guidance.
        println!(
            "   {} {}", "Source:".dimmed(), entry.source
        );
        println!(
            "   {} Clone the repository locally, then run:",
            "Next:".dimmed()
        );
        println!(
            "   {}",
            format!("repo extension install <local-path>").cyan()
        );
        println!();
        println!(
            "{} Remote git clone is not yet supported. \
             Clone manually and use {} to install from the local path.",
            "NOTE".yellow().bold(),
            "repo extension install".cyan()
        );
    }

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
    fn test_skills_install_not_found_returns_error() {
        let registry = test_registry();
        let result = run_skills_install(&registry, "nonexistent");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("not found"));
    }

    #[test]
    fn test_skills_install_remote_source_prints_guidance() {
        // Registry entries have remote URLs, so install should print guidance
        let registry = test_registry();
        let result = run_skills_install(&registry, "alpha");
        assert!(result.is_ok());
    }

    #[test]
    fn test_skills_install_local_source() {
        // Create a local extension with a minimal manifest
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("repo_extension.toml"),
            r#"
[extension]
name = "local-skill"
version = "1.0.0"
description = "A local test skill"

[provides]
content_types = ["skills"]
"#,
        )
        .unwrap();

        let mut registry = ExtensionRegistry::new();
        registry.register(ExtensionEntry {
            name: "local-skill".to_string(),
            description: "A local test skill".to_string(),
            source: temp.path().to_str().unwrap().to_string(),
        });

        let result = run_skills_install(&registry, "local-skill");
        assert!(result.is_ok());
    }
}
