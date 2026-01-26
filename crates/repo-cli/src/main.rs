//! Repository Manager CLI
//!
//! The command-line interface for managing repository tool configurations.

mod cli;
mod commands;
mod error;

use clap::Parser;
use colored::Colorize;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use cli::{BranchAction, Cli, Commands};
use error::Result;

fn main() {
    if let Err(e) = run() {
        eprintln!("{}: {}", "error".red().bold(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // Setup tracing if verbose
    if cli.verbose {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::DEBUG)
            .with_target(true)
            .finish();
        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set tracing subscriber");
        tracing::debug!("Verbose mode enabled");
    }

    // Execute command
    match cli.command {
        Some(cmd) => execute_command(cmd),
        None => {
            // No command provided - show help hint
            println!(
                "{} Repository Manager CLI",
                "repo".green().bold()
            );
            println!();
            println!("Run {} for available commands.", "repo --help".cyan());
            Ok(())
        }
    }
}

fn execute_command(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Init {
            mode,
            tools,
            presets,
        } => cmd_init(&mode, &tools, &presets),
        Commands::Check => cmd_check(),
        Commands::Sync { dry_run } => cmd_sync(dry_run),
        Commands::Fix { dry_run } => cmd_fix(dry_run),
        Commands::AddTool { name } => cmd_add_tool(&name),
        Commands::RemoveTool { name } => cmd_remove_tool(&name),
        Commands::AddPreset { name } => cmd_add_preset(&name),
        Commands::RemovePreset { name } => cmd_remove_preset(&name),
        Commands::Branch { action } => cmd_branch(action),
    }
}

// Stub implementations - these will be replaced with actual logic

fn cmd_init(mode: &str, tools: &[String], presets: &[String]) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_init(&cwd, mode, tools, presets)
}

fn cmd_check() -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_check(&cwd)
}

fn cmd_sync(dry_run: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_sync(&cwd, dry_run)
}

fn cmd_fix(dry_run: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_fix(&cwd, dry_run)
}

fn cmd_add_tool(name: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_add_tool(&cwd, name)
}

fn cmd_remove_tool(name: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_remove_tool(&cwd, name)
}

fn cmd_add_preset(name: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_add_preset(&cwd, name)
}

fn cmd_remove_preset(name: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_remove_preset(&cwd, name)
}

fn cmd_branch(action: BranchAction) -> Result<()> {
    let cwd = std::env::current_dir()?;
    match action {
        BranchAction::Add { name, base } => {
            commands::run_branch_add(&cwd, &name, Some(&base))
        }
        BranchAction::Remove { name } => {
            commands::run_branch_remove(&cwd, &name)
        }
        BranchAction::List => {
            commands::run_branch_list(&cwd)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_minimal_repo(dir: &std::path::Path, mode: &str) {
        // Create .git directory to simulate git repo
        let git_dir = dir.join(".git");
        fs::create_dir_all(&git_dir).unwrap();

        let repo_dir = dir.join(".repository");
        fs::create_dir_all(&repo_dir).unwrap();
        let config_content = format!("[core]\nmode = \"{}\"\n", mode);
        fs::write(repo_dir.join("config.toml"), config_content).unwrap();
    }

    #[test]
    fn test_add_tool_with_temp_repo() {
        let temp_dir = TempDir::new().unwrap();
        create_minimal_repo(temp_dir.path(), "standard");

        let result = commands::run_add_tool(temp_dir.path(), "eslint");
        assert!(result.is_ok());
    }

    #[test]
    fn test_remove_tool_with_temp_repo() {
        let temp_dir = TempDir::new().unwrap();
        create_minimal_repo(temp_dir.path(), "standard");

        // First add the tool
        commands::run_add_tool(temp_dir.path(), "eslint").unwrap();
        // Then remove it
        let result = commands::run_remove_tool(temp_dir.path(), "eslint");
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_preset_with_temp_repo() {
        let temp_dir = TempDir::new().unwrap();
        create_minimal_repo(temp_dir.path(), "standard");

        let result = commands::run_add_preset(temp_dir.path(), "typescript");
        assert!(result.is_ok());
    }

    #[test]
    fn test_remove_preset_with_temp_repo() {
        let temp_dir = TempDir::new().unwrap();
        create_minimal_repo(temp_dir.path(), "standard");

        // First add the preset
        commands::run_add_preset(temp_dir.path(), "typescript").unwrap();
        // Then remove it
        let result = commands::run_remove_preset(temp_dir.path(), "typescript");
        assert!(result.is_ok());
    }

    // Branch command tests are in commands/branch.rs
    // because they require a real git repo setup

    #[test]
    fn test_cli_error_user() {
        let error = crate::error::CliError::user("test error");
        assert_eq!(format!("{}", error), "test error");
    }

    // Tests for check, sync, fix are in commands/sync.rs
    // because they require temp directory setup

    #[test]
    fn test_check_with_temp_repo() {
        let temp_dir = TempDir::new().unwrap();
        create_minimal_repo(temp_dir.path(), "standard");

        let result = commands::run_check(temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_sync_with_temp_repo() {
        let temp_dir = TempDir::new().unwrap();
        create_minimal_repo(temp_dir.path(), "standard");

        let result = commands::run_sync(temp_dir.path(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fix_with_temp_repo() {
        let temp_dir = TempDir::new().unwrap();
        create_minimal_repo(temp_dir.path(), "standard");

        let result = commands::run_fix(temp_dir.path(), false);
        assert!(result.is_ok());
    }
}
