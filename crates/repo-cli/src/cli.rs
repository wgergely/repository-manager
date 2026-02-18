//! CLI argument parsing using clap derive

use clap::{Parser, Subcommand};
use clap_complete::Shell;

/// Repository Manager - Manage tool configurations for your repository
#[derive(Parser, Debug)]
#[command(name = "repo")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// The command to run
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available commands
#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum Commands {
    /// Show repository status overview
    Status {
        /// Output as JSON for scripting
        #[arg(long)]
        json: bool,
    },

    /// Preview what sync would change
    Diff {
        /// Output as JSON for scripting
        #[arg(long)]
        json: bool,
    },

    /// Initialize a new repository configuration
    ///
    /// Creates a .repository/ directory with config.toml.
    ///
    /// Examples:
    ///   repo init                    # Initialize in current directory
    ///   repo init my-project         # Create and initialize my-project/
    ///   repo init --interactive      # Guided setup
    ///   repo init -t claude -t cursor # With specific tools
    Init {
        /// Project name (creates folder if not ".")
        #[arg(default_value = ".")]
        name: String,

        /// Repository mode (standard or worktrees)
        #[arg(short, long, default_value = "worktrees")]
        mode: String,

        /// Tools to enable
        #[arg(short, long)]
        tools: Vec<String>,

        /// Presets to apply
        #[arg(short, long)]
        presets: Vec<String>,

        /// Remote repository URL
        #[arg(short, long)]
        remote: Option<String>,

        /// Interactive mode for guided setup
        #[arg(short, long)]
        interactive: bool,
    },

    /// Check repository configuration for drift
    Check,

    /// Synchronize tool configurations
    Sync {
        /// Preview changes without applying them
        #[arg(long)]
        dry_run: bool,

        /// Output as JSON for CI/CD integration
        #[arg(long)]
        json: bool,
    },

    /// Fix configuration drift automatically
    Fix {
        /// Preview fixes without applying them
        #[arg(long)]
        dry_run: bool,
    },

    /// Add a tool to the repository
    ///
    /// Adds the tool to config.toml and runs sync.
    /// Use 'repo list-tools' to see available tools.
    ///
    /// Examples:
    ///   repo add-tool claude    # Add Claude Code support
    ///   repo add-tool cursor    # Add Cursor IDE support
    ///   repo add-tool cursor --dry-run  # Preview without changing
    AddTool {
        /// Name of the tool (use 'repo list-tools' to see options)
        name: String,

        /// Preview changes without applying them
        #[arg(long)]
        dry_run: bool,
    },

    /// Remove a tool from the repository
    RemoveTool {
        /// Name of the tool to remove
        name: String,

        /// Preview changes without applying them
        #[arg(long)]
        dry_run: bool,
    },

    /// Add a preset to the repository
    AddPreset {
        /// Name of the preset to add
        name: String,

        /// Preview changes without applying them
        #[arg(long)]
        dry_run: bool,
    },

    /// Remove a preset from the repository
    RemovePreset {
        /// Name of the preset to remove
        name: String,

        /// Preview changes without applying them
        #[arg(long)]
        dry_run: bool,
    },

    /// Add a rule to the repository
    AddRule {
        /// Rule identifier (e.g., "python-style")
        id: String,
        /// Rule instruction text
        #[arg(short, long)]
        instruction: String,
        /// Optional tags
        #[arg(short, long)]
        tags: Vec<String>,
    },

    /// Remove a rule from the repository
    RemoveRule {
        /// Rule ID to remove
        id: String,
    },

    /// List all active rules
    ListRules,

    /// Lint configuration for consistency issues
    RulesLint {
        /// Output as JSON for scripting
        #[arg(long)]
        json: bool,
    },

    /// Show config drift between expected and actual state
    RulesDiff {
        /// Output as JSON for scripting
        #[arg(long)]
        json: bool,
    },

    /// Export rules to AGENTS.md format
    RulesExport {
        /// Output format (agents)
        #[arg(long, default_value = "agents")]
        format: String,
    },

    /// Import rules from AGENTS.md file
    RulesImport {
        /// Path to the file to import
        file: String,
    },

    /// List available tools
    ///
    /// Shows all tools that can be added to your repository.
    ///
    /// Examples:
    ///   repo list-tools                # Show all tools
    ///   repo list-tools --category ide # Show only IDE tools
    ListTools {
        /// Filter by category (ide, cli-agent, autonomous, copilot)
        #[arg(short, long)]
        category: Option<String>,
    },

    /// List available presets
    ListPresets,

    /// Manage branches (worktree mode)
    Branch {
        /// Branch action to perform
        #[command(subcommand)]
        action: BranchAction,
    },

    /// Push current branch to remote
    Push {
        /// Remote name (defaults to origin)
        #[arg(short, long)]
        remote: Option<String>,

        /// Branch to push (defaults to current branch)
        #[arg(short, long)]
        branch: Option<String>,
    },

    /// Pull changes from remote
    Pull {
        /// Remote name (defaults to origin)
        #[arg(short, long)]
        remote: Option<String>,

        /// Branch to pull (defaults to current branch)
        #[arg(short, long)]
        branch: Option<String>,
    },

    /// Merge a branch into current branch
    Merge {
        /// Branch to merge from
        source: String,
    },

    /// Generate shell completions
    ///
    /// Outputs completion script for your shell.
    ///
    /// Examples:
    ///   repo completions bash > ~/.local/share/bash-completion/completions/repo
    ///   repo completions zsh > ~/.zfunc/_repo
    ///   repo completions fish > ~/.config/fish/completions/repo.fish
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Manage repository configuration
    Config {
        /// Config action to perform
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Show detailed information about a tool
    ///
    /// Displays metadata, config paths, capabilities, and whether
    /// the tool is active in the current project.
    ToolInfo {
        /// Tool name (e.g., "claude", "cursor", "vscode")
        name: String,
    },

    /// Manage plugins
    Plugins {
        #[command(subcommand)]
        action: PluginsAction,
    },

    /// Manage AI agents (requires vaultspec)
    Agent {
        /// Agent action to perform
        #[command(subcommand)]
        action: AgentAction,
    },

    /// Manage lifecycle hooks
    ///
    /// Configure hooks that run before/after branch creation, deletion,
    /// agent completion, and sync operations.
    ///
    /// Examples:
    ///   repo hooks list
    ///   repo hooks add post-branch-create npm -- install
    ///   repo hooks remove post-branch-create
    Hooks {
        #[command(subcommand)]
        action: HooksAction,
    },
}

/// Branch management actions
#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum BranchAction {
    /// Add a new branch worktree
    Add {
        /// Name of the branch
        name: String,

        /// Base branch to create from
        #[arg(short, long, default_value = "main")]
        base: String,
    },

    /// Remove a branch worktree
    Remove {
        /// Name of the branch to remove
        name: String,
    },

    /// List all branch worktrees
    List,

    /// Switch to a branch (or worktree in worktrees mode)
    Checkout {
        /// Branch name to checkout
        name: String,
    },

    /// Rename a branch (and its worktree in worktrees mode)
    Rename {
        /// Current branch name
        old: String,

        /// New branch name
        new: String,
    },
}

/// Configuration management actions
#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum ConfigAction {
    /// Display the current configuration
    Show {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

/// Hook management actions
#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum HooksAction {
    /// List all configured hooks
    List,

    /// Add a new hook
    ///
    /// The event must be one of: pre-branch-create, post-branch-create,
    /// pre-branch-delete, post-branch-delete, pre-agent-complete,
    /// post-agent-complete, pre-sync, post-sync
    Add {
        /// Event that triggers the hook
        event: String,
        /// Command to execute
        command: String,
        /// Arguments to pass to the command
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Remove all hooks for an event
    Remove {
        /// Event to remove hooks for
        event: String,
    },
}

/// Plugin management actions
#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum PluginsAction {
    /// Install a plugin
    Install {
        /// Version tag to install (e.g., v4.1.1)
        #[arg(long, default_value = "v4.1.1")]
        version: String,
    },
    /// Check plugin installation status
    Status,
    /// Uninstall a plugin
    Uninstall {
        /// Version to uninstall
        #[arg(long, default_value = "v4.1.1")]
        version: String,
    },
}

/// Agent management actions
#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum AgentAction {
    /// Verify vaultspec installation and agent prerequisites
    Check,

    /// List available agents
    List,

    /// Spawn an agent to work on a task
    Spawn {
        /// Agent name or type
        name: String,

        /// Goal or task description for the agent
        #[arg(short, long)]
        goal: Option<String>,

        /// Worktree to run the agent in
        #[arg(short, long)]
        worktree: Option<String>,
    },

    /// Check status of running agents
    Status {
        /// Specific task ID to check (shows all if omitted)
        #[arg(short, long)]
        task_id: Option<String>,
    },

    /// Stop a running agent
    Stop {
        /// Task ID of the agent to stop
        task_id: String,
    },

    /// Sync agent configuration
    Sync,

    /// Show or set agent configuration
    Config {
        /// Show current configuration
        #[arg(long)]
        show: bool,
    },

    /// Manage agent rules
    Rules {
        /// Rules action to perform
        #[command(subcommand)]
        action: RulesSubAction,
    },
}

/// Rules sub-actions for agent management
#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum RulesSubAction {
    /// List agent rules
    List,

    /// Add a rule for agents
    Add {
        /// Rule identifier
        id: String,

        /// Rule instruction text
        #[arg(short, long)]
        instruction: String,
    },

    /// Remove an agent rule
    Remove {
        /// Rule ID to remove
        id: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        // Verify the CLI is valid
        Cli::command().debug_assert();
    }

    #[test]
    fn parse_no_args() {
        let cli = Cli::parse_from::<[&str; 0], &str>([]);
        assert!(!cli.verbose);
        assert!(cli.command.is_none());
    }

    #[test]
    fn parse_verbose_flag() {
        let cli = Cli::parse_from(["repo", "--verbose"]);
        assert!(cli.verbose);
        assert!(cli.command.is_none());
    }

    #[test]
    fn parse_short_verbose_flag() {
        let cli = Cli::parse_from(["repo", "-v"]);
        assert!(cli.verbose);
    }

    #[test]
    fn parse_init_command_defaults() {
        let cli = Cli::parse_from(["repo", "init"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Init {
                name,
                mode,
                tools,
                presets,
                ..
            }) if name == "." && mode == "worktrees" && tools.is_empty() && presets.is_empty()
        ));
    }

    #[test]
    fn parse_init_command_with_name() {
        let cli = Cli::parse_from(["repo", "init", "my-project"]);
        match cli.command {
            Some(Commands::Init { name, .. }) => {
                assert_eq!(name, "my-project");
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn parse_init_command_with_options() {
        let cli = Cli::parse_from([
            "repo",
            "init",
            "project",
            "--mode",
            "worktree",
            "--tools",
            "eslint",
            "--tools",
            "prettier",
            "--presets",
            "typescript",
            "--remote",
            "https://github.com/user/repo.git",
        ]);
        match cli.command {
            Some(Commands::Init {
                name,
                mode,
                tools,
                presets,
                remote,
                interactive,
            }) => {
                assert_eq!(name, "project");
                assert_eq!(mode, "worktree");
                assert_eq!(tools, vec!["eslint", "prettier"]);
                assert_eq!(presets, vec!["typescript"]);
                assert_eq!(remote, Some("https://github.com/user/repo.git".to_string()));
                assert!(!interactive);
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn parse_init_command_interactive() {
        let cli = Cli::parse_from(["repo", "init", "--interactive"]);
        match cli.command {
            Some(Commands::Init { interactive, .. }) => {
                assert!(interactive);
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn parse_check_command() {
        let cli = Cli::parse_from(["repo", "check"]);
        assert!(matches!(cli.command, Some(Commands::Check)));
    }

    #[test]
    fn parse_sync_command() {
        let cli = Cli::parse_from(["repo", "sync"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Sync {
                dry_run: false,
                json: false
            })
        ));
    }

    #[test]
    fn parse_sync_command_dry_run() {
        let cli = Cli::parse_from(["repo", "sync", "--dry-run"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Sync {
                dry_run: true,
                json: false
            })
        ));
    }

    #[test]
    fn parse_sync_command_json() {
        let cli = Cli::parse_from(["repo", "sync", "--json"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Sync {
                dry_run: false,
                json: true
            })
        ));
    }

    #[test]
    fn parse_fix_command() {
        let cli = Cli::parse_from(["repo", "fix"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Fix { dry_run: false })
        ));
    }

    #[test]
    fn parse_fix_command_dry_run() {
        let cli = Cli::parse_from(["repo", "fix", "--dry-run"]);
        assert!(matches!(cli.command, Some(Commands::Fix { dry_run: true })));
    }

    #[test]
    fn parse_add_tool_command() {
        let cli = Cli::parse_from(["repo", "add-tool", "eslint"]);
        match cli.command {
            Some(Commands::AddTool { name, dry_run }) => {
                assert_eq!(name, "eslint");
                assert!(!dry_run);
            }
            _ => panic!("Expected AddTool command"),
        }
    }

    #[test]
    fn parse_add_tool_command_dry_run() {
        let cli = Cli::parse_from(["repo", "add-tool", "eslint", "--dry-run"]);
        match cli.command {
            Some(Commands::AddTool { name, dry_run }) => {
                assert_eq!(name, "eslint");
                assert!(dry_run);
            }
            _ => panic!("Expected AddTool command"),
        }
    }

    #[test]
    fn parse_remove_tool_command() {
        let cli = Cli::parse_from(["repo", "remove-tool", "eslint"]);
        match cli.command {
            Some(Commands::RemoveTool { name, dry_run }) => {
                assert_eq!(name, "eslint");
                assert!(!dry_run);
            }
            _ => panic!("Expected RemoveTool command"),
        }
    }

    #[test]
    fn parse_remove_tool_command_dry_run() {
        let cli = Cli::parse_from(["repo", "remove-tool", "eslint", "--dry-run"]);
        match cli.command {
            Some(Commands::RemoveTool { name, dry_run }) => {
                assert_eq!(name, "eslint");
                assert!(dry_run);
            }
            _ => panic!("Expected RemoveTool command"),
        }
    }

    #[test]
    fn parse_add_preset_command() {
        let cli = Cli::parse_from(["repo", "add-preset", "typescript"]);
        match cli.command {
            Some(Commands::AddPreset { name, dry_run }) => {
                assert_eq!(name, "typescript");
                assert!(!dry_run);
            }
            _ => panic!("Expected AddPreset command"),
        }
    }

    #[test]
    fn parse_add_preset_command_dry_run() {
        let cli = Cli::parse_from(["repo", "add-preset", "typescript", "--dry-run"]);
        match cli.command {
            Some(Commands::AddPreset { name, dry_run }) => {
                assert_eq!(name, "typescript");
                assert!(dry_run);
            }
            _ => panic!("Expected AddPreset command"),
        }
    }

    #[test]
    fn parse_remove_preset_command() {
        let cli = Cli::parse_from(["repo", "remove-preset", "typescript"]);
        match cli.command {
            Some(Commands::RemovePreset { name, dry_run }) => {
                assert_eq!(name, "typescript");
                assert!(!dry_run);
            }
            _ => panic!("Expected RemovePreset command"),
        }
    }

    #[test]
    fn parse_remove_preset_command_dry_run() {
        let cli = Cli::parse_from(["repo", "remove-preset", "typescript", "--dry-run"]);
        match cli.command {
            Some(Commands::RemovePreset { name, dry_run }) => {
                assert_eq!(name, "typescript");
                assert!(dry_run);
            }
            _ => panic!("Expected RemovePreset command"),
        }
    }

    #[test]
    fn parse_add_rule_command() {
        let cli = Cli::parse_from([
            "repo",
            "add-rule",
            "python-style",
            "--instruction",
            "Use snake_case for variables.",
        ]);
        match cli.command {
            Some(Commands::AddRule {
                id,
                instruction,
                tags,
            }) => {
                assert_eq!(id, "python-style");
                assert_eq!(instruction, "Use snake_case for variables.");
                assert!(tags.is_empty());
            }
            _ => panic!("Expected AddRule command"),
        }
    }

    #[test]
    fn parse_add_rule_command_with_tags() {
        let cli = Cli::parse_from([
            "repo",
            "add-rule",
            "naming-conventions",
            "-i",
            "Follow consistent naming.",
            "-t",
            "style",
            "-t",
            "python",
        ]);
        match cli.command {
            Some(Commands::AddRule {
                id,
                instruction,
                tags,
            }) => {
                assert_eq!(id, "naming-conventions");
                assert_eq!(instruction, "Follow consistent naming.");
                assert_eq!(tags, vec!["style", "python"]);
            }
            _ => panic!("Expected AddRule command"),
        }
    }

    #[test]
    fn parse_remove_rule_command() {
        let cli = Cli::parse_from(["repo", "remove-rule", "python-style"]);
        match cli.command {
            Some(Commands::RemoveRule { id }) => assert_eq!(id, "python-style"),
            _ => panic!("Expected RemoveRule command"),
        }
    }

    #[test]
    fn parse_list_rules_command() {
        let cli = Cli::parse_from(["repo", "list-rules"]);
        assert!(matches!(cli.command, Some(Commands::ListRules)));
    }

    #[test]
    fn parse_branch_add_command() {
        let cli = Cli::parse_from(["repo", "branch", "add", "feature-x"]);
        match cli.command {
            Some(Commands::Branch {
                action: BranchAction::Add { name, base },
            }) => {
                assert_eq!(name, "feature-x");
                assert_eq!(base, "main");
            }
            _ => panic!("Expected Branch Add command"),
        }
    }

    #[test]
    fn parse_branch_add_with_base() {
        let cli = Cli::parse_from(["repo", "branch", "add", "feature-x", "--base", "develop"]);
        match cli.command {
            Some(Commands::Branch {
                action: BranchAction::Add { name, base },
            }) => {
                assert_eq!(name, "feature-x");
                assert_eq!(base, "develop");
            }
            _ => panic!("Expected Branch Add command"),
        }
    }

    #[test]
    fn parse_branch_remove_command() {
        let cli = Cli::parse_from(["repo", "branch", "remove", "feature-x"]);
        match cli.command {
            Some(Commands::Branch {
                action: BranchAction::Remove { name },
            }) => {
                assert_eq!(name, "feature-x");
            }
            _ => panic!("Expected Branch Remove command"),
        }
    }

    #[test]
    fn parse_branch_list_command() {
        let cli = Cli::parse_from(["repo", "branch", "list"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Branch {
                action: BranchAction::List
            })
        ));
    }

    #[test]
    fn parse_branch_rename_command() {
        let cli = Cli::parse_from(["repo", "branch", "rename", "old-name", "new-name"]);
        match cli.command {
            Some(Commands::Branch {
                action: BranchAction::Rename { old, new },
            }) => {
                assert_eq!(old, "old-name");
                assert_eq!(new, "new-name");
            }
            _ => panic!("Expected Branch Rename command"),
        }
    }

    #[test]
    fn verbose_flag_works_with_commands() {
        let cli = Cli::parse_from(["repo", "-v", "check"]);
        assert!(cli.verbose);
        assert!(matches!(cli.command, Some(Commands::Check)));

        let cli = Cli::parse_from(["repo", "check", "--verbose"]);
        assert!(cli.verbose);
        assert!(matches!(cli.command, Some(Commands::Check)));
    }

    #[test]
    fn parse_push_command_defaults() {
        let cli = Cli::parse_from(["repo", "push"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Push {
                remote: None,
                branch: None
            })
        ));
    }

    #[test]
    fn parse_push_command_with_remote() {
        let cli = Cli::parse_from(["repo", "push", "--remote", "upstream"]);
        match cli.command {
            Some(Commands::Push { remote, branch }) => {
                assert_eq!(remote, Some("upstream".to_string()));
                assert_eq!(branch, None);
            }
            _ => panic!("Expected Push command"),
        }
    }

    #[test]
    fn parse_pull_command_defaults() {
        let cli = Cli::parse_from(["repo", "pull"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Pull {
                remote: None,
                branch: None
            })
        ));
    }

    #[test]
    fn parse_merge_command() {
        let cli = Cli::parse_from(["repo", "merge", "feature-x"]);
        match cli.command {
            Some(Commands::Merge { source }) => {
                assert_eq!(source, "feature-x");
            }
            _ => panic!("Expected Merge command"),
        }
    }

    #[test]
    fn parse_list_tools_command() {
        let cli = Cli::parse_from(["repo", "list-tools"]);
        assert!(matches!(
            cli.command,
            Some(Commands::ListTools { category: None })
        ));
    }

    #[test]
    fn parse_list_tools_with_category() {
        let cli = Cli::parse_from(["repo", "list-tools", "--category", "ide"]);
        match cli.command {
            Some(Commands::ListTools { category }) => {
                assert_eq!(category, Some("ide".to_string()));
            }
            _ => panic!("Expected ListTools command"),
        }
    }

    #[test]
    fn parse_list_presets_command() {
        let cli = Cli::parse_from(["repo", "list-presets"]);
        assert!(matches!(cli.command, Some(Commands::ListPresets)));
    }

    #[test]
    fn parse_status_command() {
        let cli = Cli::parse_from(["repo", "status"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Status { json: false })
        ));
    }

    #[test]
    fn parse_completions_command() {
        let cli = Cli::parse_from(["repo", "completions", "bash"]);
        assert!(matches!(cli.command, Some(Commands::Completions { .. })));
    }

    #[test]
    fn parse_agent_check_command() {
        let cli = Cli::parse_from(["repo", "agent", "check"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Agent {
                action: AgentAction::Check
            })
        ));
    }

    #[test]
    fn parse_agent_list_command() {
        let cli = Cli::parse_from(["repo", "agent", "list"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Agent {
                action: AgentAction::List
            })
        ));
    }

    #[test]
    fn parse_agent_spawn_command() {
        let cli = Cli::parse_from(["repo", "agent", "spawn", "claude", "--goal", "fix bug"]);
        match cli.command {
            Some(Commands::Agent {
                action: AgentAction::Spawn { name, goal, worktree },
            }) => {
                assert_eq!(name, "claude");
                assert_eq!(goal, Some("fix bug".to_string()));
                assert_eq!(worktree, None);
            }
            _ => panic!("Expected Agent Spawn command"),
        }
    }

    #[test]
    fn parse_agent_spawn_with_worktree() {
        let cli = Cli::parse_from([
            "repo", "agent", "spawn", "claude",
            "--goal", "refactor", "--worktree", "feature-x",
        ]);
        match cli.command {
            Some(Commands::Agent {
                action: AgentAction::Spawn { name, goal, worktree },
            }) => {
                assert_eq!(name, "claude");
                assert_eq!(goal, Some("refactor".to_string()));
                assert_eq!(worktree, Some("feature-x".to_string()));
            }
            _ => panic!("Expected Agent Spawn command"),
        }
    }

    #[test]
    fn parse_agent_status_command() {
        let cli = Cli::parse_from(["repo", "agent", "status"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Agent {
                action: AgentAction::Status { task_id: None }
            })
        ));
    }

    #[test]
    fn parse_agent_stop_command() {
        let cli = Cli::parse_from(["repo", "agent", "stop", "task-123"]);
        match cli.command {
            Some(Commands::Agent {
                action: AgentAction::Stop { task_id },
            }) => {
                assert_eq!(task_id, "task-123");
            }
            _ => panic!("Expected Agent Stop command"),
        }
    }

    #[test]
    fn parse_agent_sync_command() {
        let cli = Cli::parse_from(["repo", "agent", "sync"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Agent {
                action: AgentAction::Sync
            })
        ));
    }

    #[test]
    fn parse_agent_rules_list_command() {
        let cli = Cli::parse_from(["repo", "agent", "rules", "list"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Agent {
                action: AgentAction::Rules {
                    action: RulesSubAction::List
                }
            })
        ));
    }

    #[test]
    fn parse_agent_rules_add_command() {
        let cli = Cli::parse_from([
            "repo", "agent", "rules", "add", "no-secrets",
            "--instruction", "Never expose API keys",
        ]);
        match cli.command {
            Some(Commands::Agent {
                action: AgentAction::Rules {
                    action: RulesSubAction::Add { id, instruction },
                },
            }) => {
                assert_eq!(id, "no-secrets");
                assert_eq!(instruction, "Never expose API keys");
            }
            _ => panic!("Expected Agent Rules Add command"),
        }
    }

    #[test]
    fn parse_agent_rules_remove_command() {
        let cli = Cli::parse_from(["repo", "agent", "rules", "remove", "no-secrets"]);
        match cli.command {
            Some(Commands::Agent {
                action: AgentAction::Rules {
                    action: RulesSubAction::Remove { id },
                },
            }) => {
                assert_eq!(id, "no-secrets");
            }
            _ => panic!("Expected Agent Rules Remove command"),
        }
    }

    #[test]
    fn parse_hooks_list_command() {
        let cli = Cli::parse_from(["repo", "hooks", "list"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Hooks {
                action: HooksAction::List
            })
        ));
    }

    #[test]
    fn parse_hooks_add_command() {
        let cli = Cli::parse_from(["repo", "hooks", "add", "post-branch-create", "npm", "install"]);
        match cli.command {
            Some(Commands::Hooks {
                action: HooksAction::Add { event, command, args },
            }) => {
                assert_eq!(event, "post-branch-create");
                assert_eq!(command, "npm");
                assert_eq!(args, vec!["install"]);
            }
            _ => panic!("Expected Hooks Add command"),
        }
    }

    #[test]
    fn parse_hooks_remove_command() {
        let cli = Cli::parse_from(["repo", "hooks", "remove", "pre-sync"]);
        match cli.command {
            Some(Commands::Hooks {
                action: HooksAction::Remove { event },
            }) => {
                assert_eq!(event, "pre-sync");
            }
            _ => panic!("Expected Hooks Remove command"),
        }
    }
}
