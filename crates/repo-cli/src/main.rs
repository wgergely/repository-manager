//! Repository Manager CLI
//!
//! The command-line interface for managing repository tool configurations.

mod cli;
mod commands;
mod context;
mod error;
mod interactive;

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
            name,
            mode,
            tools,
            presets,
            remote,
            interactive,
        } => cmd_init(name, mode, tools, presets, remote, interactive),
        Commands::Check => cmd_check(),
        Commands::Sync { dry_run } => cmd_sync(dry_run),
        Commands::Fix { dry_run } => cmd_fix(dry_run),
        Commands::AddTool { name } => cmd_add_tool(&name),
        Commands::RemoveTool { name } => cmd_remove_tool(&name),
        Commands::AddPreset { name } => cmd_add_preset(&name),
        Commands::RemovePreset { name } => cmd_remove_preset(&name),
        Commands::AddRule {
            id,
            instruction,
            tags,
        } => cmd_add_rule(&id, &instruction, tags),
        Commands::RemoveRule { id } => cmd_remove_rule(&id),
        Commands::ListRules => cmd_list_rules(),
        Commands::Branch { action } => cmd_branch(action),
        Commands::Push { remote, branch } => cmd_push(remote, branch),
        Commands::Pull { remote, branch } => cmd_pull(remote, branch),
        Commands::Merge { source } => cmd_merge(&source),
    }
}

// Command implementations

fn cmd_init(
    name: String,
    mode: String,
    tools: Vec<String>,
    presets: Vec<String>,
    remote: Option<String>,
    interactive_flag: bool,
) -> Result<()> {
    let cwd = std::env::current_dir()?;

    // Use interactive mode if requested
    let config = if interactive_flag {
        interactive::interactive_init(&name)?
    } else {
        commands::init::InitConfig {
            name,
            mode,
            tools,
            presets,
            remote,
            interactive: false,
        }
    };

    commands::run_init(&cwd, config)?;
    Ok(())
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

fn cmd_add_rule(id: &str, instruction: &str, tags: Vec<String>) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_add_rule(&cwd, id, instruction, tags)
}

fn cmd_remove_rule(id: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_remove_rule(&cwd, id)
}

fn cmd_list_rules() -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_list_rules(&cwd)
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

fn cmd_push(remote: Option<String>, branch: Option<String>) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_push(&cwd, remote.as_deref(), branch.as_deref())
}

fn cmd_pull(remote: Option<String>, branch: Option<String>) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_pull(&cwd, remote.as_deref(), branch.as_deref())
}

fn cmd_merge(source: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_merge(&cwd, source)
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

    #[test]
    fn test_add_rule_with_temp_repo() {
        let temp_dir = TempDir::new().unwrap();
        create_minimal_repo(temp_dir.path(), "standard");

        let result = commands::run_add_rule(
            temp_dir.path(),
            "python-style",
            "Use snake_case for variables.",
            vec![],
        );
        assert!(result.is_ok());

        // Verify rule file was created
        let rule_path = temp_dir.path().join(".repository/rules/python-style.md");
        assert!(rule_path.exists());
    }

    #[test]
    fn test_remove_rule_with_temp_repo() {
        let temp_dir = TempDir::new().unwrap();
        create_minimal_repo(temp_dir.path(), "standard");

        // First add the rule
        commands::run_add_rule(
            temp_dir.path(),
            "test-rule",
            "Test instruction.",
            vec![],
        )
        .unwrap();
        // Then remove it
        let result = commands::run_remove_rule(temp_dir.path(), "test-rule");
        assert!(result.is_ok());

        // Verify rule file was removed
        let rule_path = temp_dir.path().join(".repository/rules/test-rule.md");
        assert!(!rule_path.exists());
    }

    #[test]
    fn test_list_rules_with_temp_repo() {
        let temp_dir = TempDir::new().unwrap();
        create_minimal_repo(temp_dir.path(), "standard");

        // List rules when none exist
        let result = commands::run_list_rules(temp_dir.path());
        assert!(result.is_ok());

        // Add a rule
        commands::run_add_rule(temp_dir.path(), "my-rule", "A rule.", vec![]).unwrap();

        // List rules again
        let result = commands::run_list_rules(temp_dir.path());
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
