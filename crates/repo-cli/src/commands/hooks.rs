//! Hooks command implementations
//!
//! Provides CLI handlers for listing, adding, and removing lifecycle hooks.

use std::path::Path;

use colored::Colorize;

use repo_core::config::Manifest;
use repo_core::hooks::{HookConfig, HookEvent};

use crate::error::Result;

/// List all configured hooks
pub fn run_hooks_list(path: &Path) -> Result<()> {
    let config_path = path.join(".repository").join("config.toml");
    if !config_path.exists() {
        println!(
            "{} No .repository/config.toml found. Run {} first.",
            "note:".yellow().bold(),
            "repo init".cyan()
        );
        return Ok(());
    }

    let content = std::fs::read_to_string(&config_path)?;
    let manifest = Manifest::parse(&content)?;

    if manifest.hooks.is_empty() {
        println!("{} No hooks configured.", "note:".yellow().bold());
        println!(
            "\n{} Add a hook with: {}",
            "hint:".cyan().bold(),
            "repo hooks add <event> <command> [args...]".cyan()
        );
        println!(
            "  Events: {}",
            HookEvent::all_names().join(", ").dimmed()
        );
        return Ok(());
    }

    println!(
        "{} {} hook(s) configured:\n",
        "=>".blue().bold(),
        manifest.hooks.len()
    );
    println!(
        "  {:<25} {:<15} {}",
        "EVENT".bold(),
        "COMMAND".bold(),
        "ARGS".bold()
    );
    println!("  {}", "\u{2500}".repeat(55).dimmed());

    for hook in &manifest.hooks {
        println!(
            "  {:<25} {:<15} {}",
            hook.event.to_string().cyan(),
            hook.command.clone(),
            hook.args.join(" ").dimmed()
        );
    }

    Ok(())
}

/// Add a new hook to the configuration
pub fn run_hooks_add(path: &Path, event_str: &str, command: &str, args: Vec<String>) -> Result<()> {
    let event = match HookEvent::parse(event_str) {
        Some(e) => e,
        None => {
            println!(
                "{} Unknown event '{}'. Valid events:",
                "error:".red().bold(),
                event_str
            );
            for name in HookEvent::all_names() {
                println!("  - {}", name.cyan());
            }
            return Ok(());
        }
    };

    let config_path = path.join(".repository").join("config.toml");
    if !config_path.exists() {
        println!(
            "{} No .repository/config.toml found. Run {} first.",
            "note:".yellow().bold(),
            "repo init".cyan()
        );
        return Ok(());
    }

    let content = std::fs::read_to_string(&config_path)?;
    let mut manifest = Manifest::parse(&content)?;

    let hook = HookConfig {
        event,
        command: command.to_string(),
        args,
        working_dir: None,
    };

    manifest.hooks.push(hook);

    let toml_content = manifest.to_toml();
    std::fs::write(&config_path, toml_content)?;

    println!(
        "{} Hook added: {} -> {}",
        "\u{2713}".green().bold(),
        event.to_string().cyan(),
        command
    );

    Ok(())
}

/// Remove all hooks for a given event
pub fn run_hooks_remove(path: &Path, event_str: &str) -> Result<()> {
    let event = match HookEvent::parse(event_str) {
        Some(e) => e,
        None => {
            println!(
                "{} Unknown event '{}'. Valid events:",
                "error:".red().bold(),
                event_str
            );
            for name in HookEvent::all_names() {
                println!("  - {}", name.cyan());
            }
            return Ok(());
        }
    };

    let config_path = path.join(".repository").join("config.toml");
    if !config_path.exists() {
        println!(
            "{} No .repository/config.toml found. Run {} first.",
            "note:".yellow().bold(),
            "repo init".cyan()
        );
        return Ok(());
    }

    let content = std::fs::read_to_string(&config_path)?;
    let mut manifest = Manifest::parse(&content)?;

    let before = manifest.hooks.len();
    manifest.hooks.retain(|h| h.event != event);
    let removed = before - manifest.hooks.len();

    if removed == 0 {
        println!(
            "{} No hooks found for event '{}'.",
            "note:".yellow().bold(),
            event
        );
        return Ok(());
    }

    let toml_content = manifest.to_toml();
    std::fs::write(&config_path, toml_content)?;

    println!(
        "{} Removed {} hook(s) for event '{}'.",
        "\u{2713}".green().bold(),
        removed,
        event.to_string().cyan()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_repo(dir: &Path) {
        let repo_dir = dir.join(".repository");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(
            repo_dir.join("config.toml"),
            "[core]\nmode = \"worktrees\"\n",
        )
        .unwrap();
    }

    #[test]
    fn test_hooks_list_empty() {
        let temp = TempDir::new().unwrap();
        setup_repo(temp.path());
        let result = run_hooks_list(temp.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_hooks_add_and_list() {
        let temp = TempDir::new().unwrap();
        setup_repo(temp.path());

        let result = run_hooks_add(
            temp.path(),
            "post-branch-create",
            "npm",
            vec!["install".to_string()],
        );
        assert!(result.is_ok());

        // Verify hook was written
        let content = fs::read_to_string(temp.path().join(".repository/config.toml")).unwrap();
        let manifest = Manifest::parse(&content).unwrap();
        assert_eq!(manifest.hooks.len(), 1);
        assert_eq!(manifest.hooks[0].event, HookEvent::PostBranchCreate);
        assert_eq!(manifest.hooks[0].command, "npm");
    }

    #[test]
    fn test_hooks_add_invalid_event() {
        let temp = TempDir::new().unwrap();
        setup_repo(temp.path());

        let result = run_hooks_add(temp.path(), "invalid-event", "echo", vec![]);
        assert!(result.is_ok()); // Prints error but doesn't fail
    }

    #[test]
    fn test_hooks_remove() {
        let temp = TempDir::new().unwrap();
        setup_repo(temp.path());

        // Add a hook
        run_hooks_add(
            temp.path(),
            "pre-sync",
            "cargo",
            vec!["check".to_string()],
        )
        .unwrap();

        // Remove it
        let result = run_hooks_remove(temp.path(), "pre-sync");
        assert!(result.is_ok());

        // Verify removal
        let content = fs::read_to_string(temp.path().join(".repository/config.toml")).unwrap();
        let manifest = Manifest::parse(&content).unwrap();
        assert!(manifest.hooks.is_empty());
    }

    #[test]
    fn test_hooks_remove_nonexistent() {
        let temp = TempDir::new().unwrap();
        setup_repo(temp.path());

        let result = run_hooks_remove(temp.path(), "post-sync");
        assert!(result.is_ok()); // No-op, prints note
    }

    #[test]
    fn test_hooks_roundtrip_toml() {
        let temp = TempDir::new().unwrap();
        setup_repo(temp.path());

        // Add multiple hooks
        run_hooks_add(
            temp.path(),
            "post-branch-create",
            "npm",
            vec!["install".to_string()],
        )
        .unwrap();
        run_hooks_add(
            temp.path(),
            "pre-sync",
            "cargo",
            vec!["check".to_string()],
        )
        .unwrap();

        // Read and verify
        let content = fs::read_to_string(temp.path().join(".repository/config.toml")).unwrap();
        let manifest = Manifest::parse(&content).unwrap();
        assert_eq!(manifest.hooks.len(), 2);
        assert_eq!(manifest.hooks[0].event, HookEvent::PostBranchCreate);
        assert_eq!(manifest.hooks[1].event, HookEvent::PreSync);
    }
}
