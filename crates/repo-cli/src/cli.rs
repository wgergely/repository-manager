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
    ///   repo init -e vaultspec        # With extensions
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

        /// Extensions to enable (by name or URL)
        #[arg(short, long)]
        extensions: Vec<String>,

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

    /// Manage extensions
    ///
    /// Install, add, initialize, remove, and list extensions.
    ///
    /// Examples:
    ///   repo extension list
    ///   repo extension install https://github.com/example/ext.git
    ///   repo ext add my-extension
    #[command(alias = "ext")]
    Extension {
        #[command(subcommand)]
        action: ExtensionAction,
    },

    /// Open a worktree in an editor/IDE
    ///
    /// Launches the specified editor in the target worktree directory.
    /// Runs sync before opening to ensure configs are up to date.
    ///
    /// Examples:
    ///   repo open feature-x                # Open with auto-detected editor
    ///   repo open feature-x --tool cursor  # Open with Cursor
    ///   repo open feature-x --tool vscode  # Open with VS Code
    Open {
        /// Name of the worktree to open
        worktree: String,

        /// Editor to use (cursor, vscode, zed). Auto-detected if not specified.
        #[arg(short, long)]
        tool: Option<String>,
    },

    /// Manage tools (nested form of add-tool, remove-tool, list-tools, tool-info)
    ///
    /// Examples:
    ///   repo tool add claude
    ///   repo tool remove cursor
    ///   repo tool list
    ///   repo tool info claude
    #[command(alias = "t")]
    Tool {
        #[command(subcommand)]
        action: ToolAction,
    },

    /// Manage presets (nested form of add-preset, remove-preset, list-presets)
    ///
    /// Examples:
    ///   repo preset add typescript
    ///   repo preset remove typescript
    ///   repo preset list
    #[command(alias = "p")]
    Preset {
        #[command(subcommand)]
        action: PresetAction,
    },

    /// Manage rules (nested form of add-rule, remove-rule, list-rules)
    ///
    /// Examples:
    ///   repo rule add python-style --instruction "Use snake_case"
    ///   repo rule remove python-style
    ///   repo rule list
    #[command(alias = "r")]
    Rule {
        #[command(subcommand)]
        action: RuleAction,
    },

    /// Assign yourself to an issue and create a branch/worktree
    ///
    /// Parses a GitHub or GitLab issue URL, generates a branch name,
    /// and creates a branch (standard) or worktree (worktrees mode).
    ///
    /// Examples:
    ///   repo issue assign https://github.com/org/repo/issues/42
    ///   repo i assign https://gitlab.com/org/repo/-/issues/99
    #[command(alias = "i")]
    Issue {
        #[command(subcommand)]
        action: IssueAction,
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
    /// pre-branch-delete, post-branch-delete, pre-sync, post-sync
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

/// Extension management actions
#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum ExtensionAction {
    /// Install an extension from a URL or local path
    Install {
        /// Source URL or local path for the extension
        source: String,

        /// Don't activate the extension after installing
        #[arg(long)]
        no_activate: bool,
    },

    /// Add a known extension by name
    Add {
        /// Name of the extension to add
        name: String,
    },

    /// Initialize a new extension scaffold
    Init {
        /// Name for the new extension
        name: String,
    },

    /// Re-fire post-install hooks for an already-installed extension
    Reinit {
        /// Name of the installed extension
        name: String,
    },

    /// Remove an installed extension
    Remove {
        /// Name of the extension to remove
        name: String,
    },

    /// List installed and known extensions
    List {
        /// Output as JSON for scripting
        #[arg(long)]
        json: bool,
    },
}

/// Tool management actions
#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum ToolAction {
    /// Add a tool to the repository
    Add {
        /// Name of the tool (use 'repo tool list' to see options)
        name: String,

        /// Preview changes without applying them
        #[arg(long)]
        dry_run: bool,
    },

    /// Remove a tool from the repository
    Remove {
        /// Name of the tool to remove
        name: String,

        /// Preview changes without applying them
        #[arg(long)]
        dry_run: bool,
    },

    /// List available tools
    List {
        /// Filter by category (ide, cli-agent, autonomous, copilot)
        #[arg(short, long)]
        category: Option<String>,
    },

    /// Show detailed information about a tool
    Info {
        /// Tool name (e.g., "claude", "cursor", "vscode")
        name: String,
    },
}

/// Preset management actions
#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum PresetAction {
    /// Add a preset to the repository
    Add {
        /// Name of the preset to add
        name: String,

        /// Preview changes without applying them
        #[arg(long)]
        dry_run: bool,
    },

    /// Remove a preset from the repository
    Remove {
        /// Name of the preset to remove
        name: String,

        /// Preview changes without applying them
        #[arg(long)]
        dry_run: bool,
    },

    /// List available presets
    List,
}

/// Rule management actions
#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum RuleAction {
    /// Add a rule to the repository
    Add {
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
    Remove {
        /// Rule ID to remove
        id: String,
    },

    /// List all active rules
    List,
}

/// Issue management actions
#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum IssueAction {
    /// Assign yourself to an issue and create a branch/worktree
    ///
    /// Parses a GitHub or GitLab issue URL, generates a branch name,
    /// and creates a branch (standard mode) or worktree (worktrees mode).
    Assign {
        /// GitHub or GitLab issue URL
        url: String,
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
                extensions,
                ..
            }) if name == "." && mode == "worktrees" && tools.is_empty() && presets.is_empty() && extensions.is_empty()
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
            "--extensions",
            "vaultspec",
            "--remote",
            "https://github.com/user/repo.git",
        ]);
        match cli.command {
            Some(Commands::Init {
                name,
                mode,
                tools,
                presets,
                extensions,
                remote,
                interactive,
            }) => {
                assert_eq!(name, "project");
                assert_eq!(mode, "worktree");
                assert_eq!(tools, vec!["eslint", "prettier"]);
                assert_eq!(presets, vec!["typescript"]);
                assert_eq!(extensions, vec!["vaultspec"]);
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
        let cli = Cli::parse_from([
            "repo",
            "hooks",
            "add",
            "post-branch-create",
            "npm",
            "install",
        ]);
        match cli.command {
            Some(Commands::Hooks {
                action:
                    HooksAction::Add {
                        event,
                        command,
                        args,
                    },
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

    #[test]
    fn parse_open_command() {
        let cli = Cli::parse_from(["repo", "open", "feature-x"]);
        match cli.command {
            Some(Commands::Open { worktree, tool }) => {
                assert_eq!(worktree, "feature-x");
                assert!(tool.is_none());
            }
            _ => panic!("Expected Open command"),
        }
    }

    #[test]
    fn parse_open_command_with_tool() {
        let cli = Cli::parse_from(["repo", "open", "feature-x", "--tool", "cursor"]);
        match cli.command {
            Some(Commands::Open { worktree, tool }) => {
                assert_eq!(worktree, "feature-x");
                assert_eq!(tool, Some("cursor".to_string()));
            }
            _ => panic!("Expected Open command"),
        }
    }

    #[test]
    fn parse_extension_install_command() {
        let cli = Cli::parse_from([
            "repo",
            "extension",
            "install",
            "https://github.com/example/ext.git",
        ]);
        match cli.command {
            Some(Commands::Extension {
                action:
                    ExtensionAction::Install {
                        source,
                        no_activate,
                    },
            }) => {
                assert_eq!(source, "https://github.com/example/ext.git");
                assert!(!no_activate);
            }
            _ => panic!("Expected Extension Install command"),
        }
    }

    #[test]
    fn parse_extension_install_no_activate() {
        let cli = Cli::parse_from([
            "repo",
            "extension",
            "install",
            "https://github.com/example/ext.git",
            "--no-activate",
        ]);
        match cli.command {
            Some(Commands::Extension {
                action:
                    ExtensionAction::Install {
                        source,
                        no_activate,
                    },
            }) => {
                assert_eq!(source, "https://github.com/example/ext.git");
                assert!(no_activate);
            }
            _ => panic!("Expected Extension Install command"),
        }
    }

    #[test]
    fn parse_extension_add_command() {
        let cli = Cli::parse_from(["repo", "extension", "add", "vaultspec"]);
        match cli.command {
            Some(Commands::Extension {
                action: ExtensionAction::Add { name },
            }) => {
                assert_eq!(name, "vaultspec");
            }
            _ => panic!("Expected Extension Add command"),
        }
    }

    #[test]
    fn parse_extension_init_command() {
        let cli = Cli::parse_from(["repo", "extension", "init", "my-ext"]);
        match cli.command {
            Some(Commands::Extension {
                action: ExtensionAction::Init { name },
            }) => {
                assert_eq!(name, "my-ext");
            }
            _ => panic!("Expected Extension Init command"),
        }
    }

    #[test]
    fn parse_extension_remove_command() {
        let cli = Cli::parse_from(["repo", "extension", "remove", "my-ext"]);
        match cli.command {
            Some(Commands::Extension {
                action: ExtensionAction::Remove { name },
            }) => {
                assert_eq!(name, "my-ext");
            }
            _ => panic!("Expected Extension Remove command"),
        }
    }

    #[test]
    fn parse_extension_list_command() {
        let cli = Cli::parse_from(["repo", "extension", "list"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Extension {
                action: ExtensionAction::List { json: false }
            })
        ));
    }

    #[test]
    fn parse_extension_list_json() {
        let cli = Cli::parse_from(["repo", "extension", "list", "--json"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Extension {
                action: ExtensionAction::List { json: true }
            })
        ));
    }

    #[test]
    fn parse_ext_alias() {
        let cli = Cli::parse_from(["repo", "ext", "list"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Extension {
                action: ExtensionAction::List { json: false }
            })
        ));
    }
}
