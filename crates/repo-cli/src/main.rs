//! Repository Manager CLI
//!
//! The command-line interface for managing repository tool configurations.

mod cli;
mod commands;
mod context;
mod error;
mod interactive;

use std::io;

use clap::{CommandFactory, Parser};
use clap_complete::{Shell, generate};
use colored::Colorize;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use cli::{
    BranchAction, Cli, Commands, ConfigAction, ExtensionAction, HooksAction, PresetAction,
    RuleAction, ToolAction,
};
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
        if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
            eprintln!("Warning: Could not set tracing subscriber: {}", e);
        }
        tracing::debug!("Verbose mode enabled");
    }

    // Execute command
    match cli.command {
        Some(cmd) => execute_command(cmd),
        None => {
            // No command provided - show help hint
            println!("{} Repository Manager CLI", "repo".green().bold());
            println!();
            println!("Run {} for available commands.", "repo --help".cyan());
            Ok(())
        }
    }
}

fn execute_command(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Status { json } => cmd_status(json),
        Commands::Diff { json } => cmd_diff(json),
        Commands::Init {
            name,
            mode,
            tools,
            presets,
            extensions,
            remote,
            interactive,
        } => cmd_init(name, mode, tools, presets, extensions, remote, interactive),
        Commands::Check => cmd_check(),
        Commands::Sync { dry_run, json } => cmd_sync(dry_run, json),
        Commands::Fix { dry_run } => cmd_fix(dry_run),
        Commands::AddTool { name, dry_run } => cmd_add_tool(&name, dry_run),
        Commands::RemoveTool { name, dry_run } => cmd_remove_tool(&name, dry_run),
        Commands::AddPreset { name, dry_run } => cmd_add_preset(&name, dry_run),
        Commands::RemovePreset { name, dry_run } => cmd_remove_preset(&name, dry_run),
        Commands::AddRule {
            id,
            instruction,
            tags,
        } => cmd_add_rule(&id, &instruction, tags),
        Commands::RemoveRule { id } => cmd_remove_rule(&id),
        Commands::ListRules => cmd_list_rules(),
        Commands::RulesLint { json } => cmd_rules_lint(json),
        Commands::RulesDiff { json } => cmd_rules_diff(json),
        Commands::RulesExport { format } => cmd_rules_export(&format),
        Commands::RulesImport { file } => cmd_rules_import(&file),
        Commands::ListTools { category } => cmd_list_tools(category.as_deref()),
        Commands::ListPresets => cmd_list_presets(),
        Commands::Completions { shell } => cmd_completions(shell),
        Commands::Branch { action } => cmd_branch(action),
        Commands::Push { remote, branch } => cmd_push(remote, branch),
        Commands::Pull { remote, branch } => cmd_pull(remote, branch),
        Commands::Merge { source } => cmd_merge(&source),
        Commands::Config { action } => cmd_config(action),
        Commands::ToolInfo { name } => cmd_tool_info(&name),
        Commands::Hooks { action } => cmd_hooks(action),
        Commands::Extension { action } => cmd_extension(action),
        Commands::Open { worktree, tool } => cmd_open(&worktree, tool.as_deref()),
        Commands::Tool { action } => cmd_tool(action),
        Commands::Preset { action } => cmd_preset(action),
        Commands::Rule { action } => cmd_rule(action),
    }
}

// Command implementations

fn cmd_completions(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    generate(shell, &mut cmd, name, &mut io::stdout());
    Ok(())
}

fn cmd_status(json: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_status(&cwd, json)
}

fn cmd_diff(json: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_diff(&cwd, json)
}

fn cmd_init(
    name: String,
    mode: String,
    tools: Vec<String>,
    presets: Vec<String>,
    extensions: Vec<String>,
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
            extensions,
            remote,
        }
    };

    commands::run_init(&cwd, config)?;
    Ok(())
}

fn cmd_check() -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_check(&cwd)
}

fn cmd_sync(dry_run: bool, json: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_sync(&cwd, dry_run, json)
}

fn cmd_fix(dry_run: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_fix(&cwd, dry_run)
}

fn cmd_add_tool(name: &str, dry_run: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_add_tool(&cwd, name, dry_run)
}

fn cmd_remove_tool(name: &str, dry_run: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_remove_tool(&cwd, name, dry_run)
}

fn cmd_add_preset(name: &str, dry_run: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_add_preset(&cwd, name, dry_run)
}

fn cmd_remove_preset(name: &str, dry_run: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_remove_preset(&cwd, name, dry_run)
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

fn cmd_rules_lint(json: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_rules_lint(&cwd, json)
}

fn cmd_rules_diff(json: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_rules_diff(&cwd, json)
}

fn cmd_rules_export(format: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_rules_export(&cwd, format)
}

fn cmd_rules_import(file: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_rules_import(&cwd, file)
}

fn cmd_list_tools(category: Option<&str>) -> Result<()> {
    commands::run_list_tools(category)
}

fn cmd_list_presets() -> Result<()> {
    commands::run_list_presets()
}

fn cmd_branch(action: BranchAction) -> Result<()> {
    let cwd = std::env::current_dir()?;
    match action {
        BranchAction::Add { name, base } => commands::run_branch_add(&cwd, &name, Some(&base)),
        BranchAction::Remove { name } => commands::run_branch_remove(&cwd, &name),
        BranchAction::List => commands::run_branch_list(&cwd),
        BranchAction::Checkout { name } => commands::run_branch_checkout(&cwd, &name),
        BranchAction::Rename { old, new } => commands::run_branch_rename(&cwd, &old, &new),
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

fn cmd_config(action: ConfigAction) -> Result<()> {
    let cwd = std::env::current_dir()?;
    match action {
        ConfigAction::Show { json } => commands::config::run_config_show(&cwd, json),
    }
}

fn cmd_tool_info(name: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::config::run_tool_info(&cwd, name)
}

fn cmd_hooks(action: HooksAction) -> Result<()> {
    let cwd = std::env::current_dir()?;
    match action {
        HooksAction::List => commands::hooks::run_hooks_list(&cwd),
        HooksAction::Add {
            event,
            command,
            args,
        } => commands::hooks::run_hooks_add(&cwd, &event, &command, args),
        HooksAction::Remove { event } => commands::hooks::run_hooks_remove(&cwd, &event),
    }
}

fn cmd_extension(action: ExtensionAction) -> Result<()> {
    match action {
        ExtensionAction::Install {
            source,
            no_activate,
        } => commands::extension::handle_extension_install(&source, no_activate),
        ExtensionAction::Add { name } => commands::extension::handle_extension_add(&name),
        ExtensionAction::Init { name } => commands::extension::handle_extension_init(&name),
        ExtensionAction::Reinit { name } => commands::extension::handle_extension_reinit(&name),
        ExtensionAction::Remove { name } => commands::extension::handle_extension_remove(&name),
        ExtensionAction::List { json } => commands::extension::handle_extension_list(json),
    }
}

fn cmd_open(worktree: &str, tool: Option<&str>) -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::open::run_open(&cwd, worktree, tool)
}

fn cmd_tool(action: ToolAction) -> Result<()> {
    match action {
        ToolAction::Add { name, dry_run } => cmd_add_tool(&name, dry_run),
        ToolAction::Remove { name, dry_run } => cmd_remove_tool(&name, dry_run),
        ToolAction::List { category } => cmd_list_tools(category.as_deref()),
        ToolAction::Info { name } => cmd_tool_info(&name),
    }
}

fn cmd_preset(action: PresetAction) -> Result<()> {
    match action {
        PresetAction::Add { name, dry_run } => cmd_add_preset(&name, dry_run),
        PresetAction::Remove { name, dry_run } => cmd_remove_preset(&name, dry_run),
        PresetAction::List => cmd_list_presets(),
    }
}

fn cmd_rule(action: RuleAction) -> Result<()> {
    match action {
        RuleAction::Add {
            id,
            instruction,
            tags,
        } => cmd_add_rule(&id, &instruction, tags),
        RuleAction::Remove { id } => cmd_remove_rule(&id),
        RuleAction::List => cmd_list_rules(),
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

        let result = commands::run_add_tool(temp_dir.path(), "eslint", false);
        assert!(result.is_ok());

        // Verify the tool was added to config.toml
        let config_path = temp_dir.path().join(".repository/config.toml");
        let config_content = fs::read_to_string(&config_path).unwrap();
        assert!(
            config_content.contains("eslint"),
            "Config should contain the added tool 'eslint'"
        );
    }

    #[test]
    fn test_remove_tool_with_temp_repo() {
        let temp_dir = TempDir::new().unwrap();
        create_minimal_repo(temp_dir.path(), "standard");

        // First add the tool
        commands::run_add_tool(temp_dir.path(), "eslint", false).unwrap();
        // Then remove it
        let result = commands::run_remove_tool(temp_dir.path(), "eslint", false);
        assert!(result.is_ok());

        // Verify the tool was removed from config.toml
        let config_path = temp_dir.path().join(".repository/config.toml");
        let config_content = fs::read_to_string(&config_path).unwrap();
        assert!(
            !config_content.contains("eslint"),
            "Config should no longer contain the removed tool 'eslint'"
        );
    }

    #[test]
    fn test_add_preset_with_temp_repo() {
        let temp_dir = TempDir::new().unwrap();
        create_minimal_repo(temp_dir.path(), "standard");

        let result = commands::run_add_preset(temp_dir.path(), "typescript", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_remove_preset_with_temp_repo() {
        let temp_dir = TempDir::new().unwrap();
        create_minimal_repo(temp_dir.path(), "standard");

        // First add the preset
        commands::run_add_preset(temp_dir.path(), "typescript", false).unwrap();
        // Then remove it
        let result = commands::run_remove_preset(temp_dir.path(), "typescript", false);
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

        // Verify rule file was created with correct content
        let rule_path = temp_dir.path().join(".repository/rules/python-style.md");
        assert!(rule_path.exists());
        let content = fs::read_to_string(&rule_path).unwrap();
        assert!(
            content.contains("snake_case"),
            "Rule file should contain the instruction text"
        );
    }

    #[test]
    fn test_remove_rule_with_temp_repo() {
        let temp_dir = TempDir::new().unwrap();
        create_minimal_repo(temp_dir.path(), "standard");

        // First add the rule
        commands::run_add_rule(temp_dir.path(), "test-rule", "Test instruction.", vec![]).unwrap();
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

        let result = commands::run_sync(temp_dir.path(), false, false);
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
