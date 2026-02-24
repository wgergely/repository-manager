//! Extension command implementations.
//!
//! Provides CLI handlers for the `repo extension` subcommands: install, add,
//! init, remove, and list. Install and add validate extension manifests and
//! report dependency requirements. Init scaffolds a new `repo_extension.toml`.
//! Remove marks an extension for cleanup on the next sync.

use std::path::Path;

use colored::Colorize;
use repo_core::config::Manifest;
use repo_core::{HookConfig, HookContext, HookEvent, run_hooks};
use repo_extensions::{
    ExtensionManifest, ExtensionRegistry, LockFile, LockedExtension, MANIFEST_FILENAME,
    LOCK_FILENAME, check_binary_on_path, query_python_version, run_install,
    synthesize_install_command,
};

use crate::error::{CliError, Result};

/// Load hooks from `.repository/config.toml` at `repo_root`.
fn load_hooks(repo_root: &Path) -> Vec<HookConfig> {
    let config_path = repo_root.join(".repository").join("config.toml");
    if !config_path.exists() {
        return Vec::new();
    }
    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    match Manifest::parse(&content) {
        Ok(m) => m.hooks,
        Err(_) => Vec::new(),
    }
}

/// Handle `repo extension install <source> [--no-activate]`
///
/// Validates the extension manifest, checks runtime constraints, runs the
/// install command, writes the lock file, and fires post-extension-install hooks.
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

    let name = &manifest.extension.name;
    let version = &manifest.extension.version;

    // Step 1: Check Python version constraint (if declared)
    let mut python_version: Option<String> = None;
    if manifest.requires.as_ref().and_then(|r| r.python.as_ref()).is_some() {
        let pv = query_python_version(None)
            .map_err(|e| CliError::user(format!("Cannot determine Python version: {e}")))?;
        if !manifest.python_version_satisfied(&pv)
            .map_err(|e| CliError::user(format!("Version constraint error: {e}")))? {
            let constraint = &manifest.requires.as_ref().unwrap().python.as_ref().unwrap().version;
            return Err(CliError::user(format!(
                "Python {} does not satisfy constraint '{}' for extension '{}'",
                pv, constraint, name
            )));
        }
        python_version = Some(pv);
    }

    // Step 2: Check package_manager binary (if declared)
    let package_manager = manifest.runtime.as_ref().and_then(|r| r.package_manager.as_deref());
    if let Some(tool) = package_manager {
        check_binary_on_path(tool)
            .map_err(|e| CliError::user(e.to_string()))?;
    }

    // Step 3: Determine effective install command
    let explicit_install = manifest.runtime.as_ref().and_then(|r| r.install.as_deref());
    let packages = manifest.requires.as_ref()
        .and_then(|r| r.python.as_ref())
        .map(|p| p.packages.as_slice())
        .unwrap_or(&[]);
    let synthesized = synthesize_install_command(packages, package_manager);

    let effective_install = match (explicit_install, &synthesized) {
        (Some(cmd), Some(_)) => {
            // ADR-009 §9.2: explicit install wins; packages treated as documentation only
            eprintln!(
                "{} Both [runtime].install and [requires.python].packages are set; \
                 using explicit install command, packages list is documentation only.",
                "warning:".yellow().bold()
            );
            Some(cmd.to_string())
        }
        (Some(cmd), None) => Some(cmd.to_string()),
        (None, Some(cmd)) => Some(cmd.clone()),
        (None, None) => None,
    };

    // Step 4: Run install command
    if let Some(ref install_cmd) = effective_install {
        println!(
            "{} Running install for '{}': {}",
            "=>".blue().bold(),
            name.cyan(),
            install_cmd.dimmed()
        );
        // repo_root resolved below; for the env var we use cwd or source_path as fallback
        let rr = std::env::current_dir().unwrap_or_else(|_| source_path.to_path_buf());
        run_install(name, version, install_cmd, source_path, &rr)?;
    }

    // Step 5: Load or create lock file
    let repo_root = std::env::current_dir().unwrap_or_else(|_| source_path.to_path_buf());
    let lock_path = repo_root.join(".repository").join(LOCK_FILENAME);
    let mut lock_file = LockFile::load(&lock_path)?;

    // Step 6: Upsert lock entry
    let venv_path = manifest.runtime.as_ref().and_then(|r| r.venv_path.clone());
    lock_file.upsert(LockedExtension {
        name: name.clone(),
        version: version.clone(),
        source: source.to_string(),
        resolved_ref: None,
        runtime_type: manifest.runtime.as_ref().map(|r| r.runtime_type.clone()),
        python_version,
        package_manager: package_manager.map(|s| s.to_string()),
        packages: packages.to_vec(),
        venv_path: venv_path.clone(),
    });

    // Step 7: Save lock file
    lock_file.save(&lock_path)?;

    // Step 8: Fire post-extension-install hooks
    let venv_abs = venv_path.as_ref().map(|vp| source_path.join(vp));
    let hook_ctx = HookContext::for_extension_install(
        name,
        version,
        source,
        source_path,
        venv_abs.as_deref(),
    );
    let hooks = load_hooks(&repo_root);
    run_hooks(&hooks, HookEvent::PostExtensionInstall, &hook_ctx, &repo_root)?;

    // Step 9: Print success
    println!(
        "{} Installed '{}' v{}",
        "=>".green().bold(),
        name.cyan(),
        version
    );
    if effective_install.is_some() {
        println!("   {} Lock file updated at {}", "=>".dimmed(), lock_path.display());
    }

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

/// Handle `repo extension reinit <name>`
///
/// Re-fires `post-extension-install` hooks for an already-installed extension.
/// Reads the extension's lock entry to reconstruct the hook context.
pub fn handle_extension_reinit(name: &str) -> Result<()> {
    let repo_root = std::env::current_dir()
        .map_err(|e| CliError::user(format!("Cannot determine current directory: {e}")))?;
    let lock_path = repo_root.join(".repository").join(LOCK_FILENAME);
    let lock_file = LockFile::load(&lock_path)?;

    let entry = lock_file.get(name).ok_or_else(|| {
        CliError::Extensions(repo_extensions::Error::ExtensionNotInstalled(name.to_string()))
    })?;

    let extension_dir = std::path::PathBuf::from(&entry.source);
    let venv_abs = entry.venv_path.as_ref().map(|vp| extension_dir.join(vp));
    let hook_ctx = HookContext::for_extension_install(
        &entry.name,
        &entry.version,
        &entry.source,
        &extension_dir,
        venv_abs.as_deref(),
    );

    let hooks = load_hooks(&repo_root);
    run_hooks(&hooks, HookEvent::PostExtensionInstall, &hook_ctx, &repo_root)?;

    println!(
        "{} Re-fired install hooks for '{}' v{}",
        "=>".green().bold(),
        entry.name.cyan(),
        entry.version
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
/// Lists known extensions from the built-in registry, enriched with lock file
/// data (version, runtime, package_manager, installed status) per ADR-008 §8.4.
pub fn handle_extension_list(json: bool) -> Result<()> {
    let registry = ExtensionRegistry::with_known();
    let names = registry.known_extensions();

    // Load lock file to determine installed status and runtime details
    let repo_root = std::env::current_dir().unwrap_or_default();
    let lock_path = repo_root.join(".repository").join(LOCK_FILENAME);
    let lock_file = LockFile::load(&lock_path).unwrap_or_default();

    if json {
        let entries: Vec<serde_json::Value> = names
            .iter()
            .filter_map(|name| {
                registry.get(name).map(|entry| {
                    let locked = lock_file.get(name);
                    let mut obj = serde_json::json!({
                        "name": entry.name,
                        "description": entry.description,
                        "source": entry.source,
                        "installed": locked.is_some(),
                    });
                    if let Some(le) = locked {
                        obj["version"] = serde_json::json!(le.version);
                        obj["runtime_type"] = serde_json::json!(le.runtime_type);
                        obj["package_manager"] = serde_json::json!(le.package_manager);
                    }
                    obj
                })
            })
            .collect();

        println!(
            "{}",
            serde_json::to_string_pretty(&entries).unwrap_or_default()
        );
    } else {
        println!(
            "{} Extensions:",
            "=>".blue().bold()
        );
        if names.is_empty() {
            println!("   No extensions registered.");
        } else {
            for name in &names {
                if let Some(entry) = registry.get(name) {
                    let locked = lock_file.get(name);
                    let status = if locked.is_some() { "installed" } else { "available" };
                    let version = locked.map(|l| l.version.as_str()).unwrap_or("-");
                    let manager = locked
                        .and_then(|l| l.package_manager.as_deref())
                        .unwrap_or("\u{2014}");
                    println!(
                        "   {} {} {} [{}]  {}",
                        entry.name.cyan(),
                        version.dimmed(),
                        manager.dimmed(),
                        status,
                        entry.description.dimmed()
                    );
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
