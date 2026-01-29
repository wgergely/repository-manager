# CLI/UX Improvements Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add discoverability commands (list-tools, list-presets, status) and shell completions to improve CLI user experience from 6.5/10 to 8+/10.

**Architecture:** Add new CLI commands that expose the existing `ToolRegistry` and preset `Registry` through user-facing subcommands. Use clap's built-in shell completion generation. Keep commands simple - read-only display of registry state.

**Tech Stack:** Rust, clap (with `derive` feature), clap_complete for shell completions, colored for terminal output.

---

## Task 1: Add `list-tools` Command

**Files:**
- Modify: `crates/repo-cli/src/cli.rs:20-145` (add command enum variant)
- Create: `crates/repo-cli/src/commands/list.rs` (new module)
- Modify: `crates/repo-cli/src/commands/mod.rs` (export new module)
- Modify: `crates/repo-cli/src/main.rs:57-86` (wire up command)
- Modify: `crates/repo-cli/Cargo.toml` (add repo-tools dependency)

**Step 1: Write the failing test for list-tools command parsing**

Add to `crates/repo-cli/src/cli.rs` in the tests module:

```rust
#[test]
fn parse_list_tools_command() {
    let cli = Cli::parse_from(["repo", "list-tools"]);
    assert!(matches!(cli.command, Some(Commands::ListTools { category: None })));
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p repo-cli parse_list_tools`
Expected: FAIL with "no variant named `ListTools`"

**Step 3: Add ListTools command variant to cli.rs**

In `crates/repo-cli/src/cli.rs`, add to the `Commands` enum after line 108 (after `ListRules`):

```rust
    /// List available tools
    ListTools {
        /// Filter by category (ide, cli-agent, autonomous, copilot)
        #[arg(short, long)]
        category: Option<String>,
    },

    /// List available presets
    ListPresets,
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p repo-cli parse_list_tools`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/repo-cli/src/cli.rs
git commit -m "feat(cli): add ListTools and ListPresets command variants"
```

---

## Task 2: Implement list-tools Command Logic

**Files:**
- Modify: `crates/repo-cli/Cargo.toml` (add repo-tools dependency)
- Create: `crates/repo-cli/src/commands/list.rs`
- Modify: `crates/repo-cli/src/commands/mod.rs`

**Step 1: Add repo-tools dependency**

In `crates/repo-cli/Cargo.toml`, add after line 17:

```toml
repo-tools = { path = "../repo-tools" }
```

**Step 2: Create list.rs with run_list_tools function**

Create `crates/repo-cli/src/commands/list.rs`:

```rust
//! List commands for tools and presets

use colored::Colorize;
use repo_meta::Registry;
use repo_tools::{ToolCategory, ToolRegistry};

use crate::error::Result;

/// Run the list-tools command
pub fn run_list_tools(category_filter: Option<&str>) -> Result<()> {
    let registry = ToolRegistry::with_builtins();

    // Parse category filter if provided
    let filter: Option<ToolCategory> = match category_filter {
        Some("ide") => Some(ToolCategory::Ide),
        Some("cli-agent") => Some(ToolCategory::CliAgent),
        Some("autonomous") => Some(ToolCategory::Autonomous),
        Some("copilot") => Some(ToolCategory::Copilot),
        Some(other) => {
            eprintln!(
                "{} Unknown category '{}'. Valid: ide, cli-agent, autonomous, copilot",
                "warning:".yellow().bold(),
                other
            );
            None
        }
        None => None,
    };

    println!("{}", "Available Tools".bold());
    println!();

    // Group by category
    let categories = [
        (ToolCategory::Ide, "IDE Tools"),
        (ToolCategory::CliAgent, "CLI Agents"),
        (ToolCategory::Autonomous, "Autonomous Agents"),
        (ToolCategory::Copilot, "Copilots"),
    ];

    for (cat, label) in categories {
        // Skip if filtering and this isn't the category
        if let Some(f) = filter {
            if f != cat {
                continue;
            }
        }

        let tools = registry.by_category(cat);
        if tools.is_empty() {
            continue;
        }

        println!("{}:", label.cyan().bold());
        for slug in tools {
            if let Some(reg) = registry.get(slug) {
                let config = &reg.definition.integration.config_path;
                println!(
                    "  {:<14} {} ({})",
                    slug.green(),
                    reg.name,
                    config.dimmed()
                );
            }
        }
        println!();
    }

    let total = registry.len();
    println!(
        "{} {} tools available. Use {} to add one.",
        "Total:".dimmed(),
        total,
        "repo add-tool <name>".cyan()
    );

    Ok(())
}

/// Run the list-presets command
pub fn run_list_presets() -> Result<()> {
    let registry = Registry::with_builtins();

    println!("{}", "Available Presets".bold());
    println!();

    for preset in registry.list_presets() {
        if let Some(provider) = registry.get_provider(&preset) {
            println!("  {:<16} (provider: {})", preset.green(), provider.dimmed());
        }
    }

    println!();
    println!(
        "{} {} presets available. Use {} to add one.",
        "Total:".dimmed(),
        registry.len(),
        "repo add-preset <name>".cyan()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_tools_runs() {
        let result = run_list_tools(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_tools_with_category() {
        let result = run_list_tools(Some("ide"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_presets_runs() {
        let result = run_list_presets();
        assert!(result.is_ok());
    }
}
```

**Step 3: Update commands/mod.rs to export list module**

In `crates/repo-cli/src/commands/mod.rs`, add after line 7:

```rust
pub mod list;
```

And add to exports after line 15:

```rust
pub use list::{run_list_presets, run_list_tools};
```

**Step 4: Run tests to verify**

Run: `cargo test -p repo-cli list`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/repo-cli/Cargo.toml crates/repo-cli/src/commands/list.rs crates/repo-cli/src/commands/mod.rs
git commit -m "feat(cli): implement list-tools and list-presets logic"
```

---

## Task 3: Wire Up list-tools and list-presets in main.rs

**Files:**
- Modify: `crates/repo-cli/src/main.rs:57-86`

**Step 1: Write integration test for list-tools**

Create `crates/repo-cli/tests/cli_list_tools.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_list_tools_shows_output() {
    let mut cmd = Command::cargo_bin("repo").unwrap();
    cmd.arg("list-tools")
        .assert()
        .success()
        .stdout(predicate::str::contains("Available Tools"))
        .stdout(predicate::str::contains("claude"))
        .stdout(predicate::str::contains("cursor"));
}

#[test]
fn test_list_tools_with_ide_filter() {
    let mut cmd = Command::cargo_bin("repo").unwrap();
    cmd.args(["list-tools", "--category", "ide"])
        .assert()
        .success()
        .stdout(predicate::str::contains("IDE Tools"))
        .stdout(predicate::str::contains("vscode"));
}

#[test]
fn test_list_presets_shows_output() {
    let mut cmd = Command::cargo_bin("repo").unwrap();
    cmd.arg("list-presets")
        .assert()
        .success()
        .stdout(predicate::str::contains("Available Presets"))
        .stdout(predicate::str::contains("env:python"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p repo-cli --test cli_list_tools`
Expected: FAIL (commands not wired up yet)

**Step 3: Wire up commands in main.rs**

In `crates/repo-cli/src/main.rs`, add to the `execute_command` match after line 80:

```rust
        Commands::ListTools { category } => cmd_list_tools(category.as_deref()),
        Commands::ListPresets => cmd_list_presets(),
```

Add command functions after line 166 (before `#[cfg(test)]`):

```rust
fn cmd_list_tools(category: Option<&str>) -> Result<()> {
    commands::run_list_tools(category)
}

fn cmd_list_presets() -> Result<()> {
    commands::run_list_presets()
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p repo-cli --test cli_list_tools`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/repo-cli/src/main.rs crates/repo-cli/tests/cli_list_tools.rs
git commit -m "feat(cli): wire up list-tools and list-presets commands"
```

---

## Task 4: Add `status` Command

**Files:**
- Modify: `crates/repo-cli/src/cli.rs` (add command)
- Create: `crates/repo-cli/src/commands/status.rs`
- Modify: `crates/repo-cli/src/commands/mod.rs`
- Modify: `crates/repo-cli/src/main.rs`

**Step 1: Write failing test for status command parsing**

Add to `crates/repo-cli/src/cli.rs` tests:

```rust
#[test]
fn parse_status_command() {
    let cli = Cli::parse_from(["repo", "status"]);
    assert!(matches!(cli.command, Some(Commands::Status)));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p repo-cli parse_status`
Expected: FAIL

**Step 3: Add Status command variant**

In `crates/repo-cli/src/cli.rs`, add to Commands enum after ListPresets:

```rust
    /// Show repository status
    Status,
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p repo-cli parse_status`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/repo-cli/src/cli.rs
git commit -m "feat(cli): add Status command variant"
```

---

## Task 5: Implement status Command Logic

**Files:**
- Create: `crates/repo-cli/src/commands/status.rs`
- Modify: `crates/repo-cli/src/commands/mod.rs`
- Modify: `crates/repo-cli/src/main.rs`

**Step 1: Create status.rs**

Create `crates/repo-cli/src/commands/status.rs`:

```rust
//! Status command implementation

use std::path::Path;

use colored::Colorize;
use repo_core::Manifest;
use repo_fs::NormalizedPath;

use crate::error::{CliError, Result};

const CONFIG_PATH: &str = ".repository/config.toml";

/// Run the status command
pub fn run_status(path: &Path) -> Result<()> {
    let config_path = NormalizedPath::new(path.join(CONFIG_PATH));
    let native_path = config_path.to_native();

    // Check if repo is initialized
    if !native_path.exists() {
        println!("{}", "Not a repository".red().bold());
        println!();
        println!("Run {} to initialize.", "repo init".cyan());
        return Ok(());
    }

    // Load manifest
    let content = std::fs::read_to_string(&native_path)?;
    let manifest = Manifest::parse(&content).map_err(|e| CliError::user(e.to_string()))?;

    // Display status
    println!("{}", "Repository Status".bold());
    println!();

    println!("{}:   {}", "Path".dimmed(), path.display());
    println!("{}:   {}", "Mode".dimmed(), manifest.core.mode.cyan());
    println!("{}:   {}", "Config".dimmed(), CONFIG_PATH);
    println!();

    // Tools
    println!("{}:", "Enabled Tools".bold());
    if manifest.tools.is_empty() {
        println!("  {} (use {} to add)", "None".dimmed(), "repo add-tool".cyan());
    } else {
        for tool in &manifest.tools {
            let config_exists = check_tool_config_exists(path, tool);
            let status = if config_exists {
                "in sync".green()
            } else {
                "missing config".yellow()
            };
            println!("  {} {} ({})", "+".green(), tool.cyan(), status);
        }
    }
    println!();

    // Presets
    println!("{}:", "Presets".bold());
    if manifest.presets.is_empty() {
        println!("  {} (use {} to add)", "None".dimmed(), "repo add-preset".cyan());
    } else {
        for (name, _value) in &manifest.presets {
            println!("  {} {}", "+".green(), name.cyan());
        }
    }
    println!();

    // Rules
    println!("{}:", "Rules".bold());
    if manifest.rules.is_empty() {
        println!("  {} (use {} to add)", "None".dimmed(), "repo add-rule".cyan());
    } else {
        println!("  {} active rules", manifest.rules.len());
        for rule in &manifest.rules {
            println!("  {} {}", "+".green(), rule);
        }
    }

    Ok(())
}

/// Check if a tool's config file exists
fn check_tool_config_exists(path: &Path, tool: &str) -> bool {
    let config_file = match tool {
        "claude" => "CLAUDE.md",
        "cursor" => ".cursorrules",
        "aider" => ".aider.conf.yml",
        "gemini" => "GEMINI.md",
        "cline" => ".clinerules",
        "roo" => ".roorules",
        "copilot" => ".github/copilot-instructions.md",
        "vscode" => ".vscode/settings.json",
        "zed" => ".zed/settings.json",
        "jetbrains" => ".idea/.junie/guidelines.md",
        "windsurf" => ".windsurfrules",
        "antigravity" => ".antigravity/rules.md",
        "amazonq" => ".amazonq/rules.md",
        _ => return false,
    };
    path.join(config_file).exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo(dir: &Path) {
        std::fs::create_dir_all(dir.join(".repository")).unwrap();
        std::fs::write(
            dir.join(".repository/config.toml"),
            r#"tools = ["claude"]

[core]
mode = "standard"
"#,
        )
        .unwrap();
    }

    #[test]
    fn test_status_not_initialized() {
        let temp = TempDir::new().unwrap();
        let result = run_status(temp.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_status_initialized() {
        let temp = TempDir::new().unwrap();
        create_test_repo(temp.path());
        let result = run_status(temp.path());
        assert!(result.is_ok());
    }
}
```

**Step 2: Update commands/mod.rs**

Add after `pub mod list;`:

```rust
pub mod status;
```

Add to exports:

```rust
pub use status::run_status;
```

**Step 3: Wire up in main.rs**

Add to execute_command match:

```rust
        Commands::Status => cmd_status(),
```

Add command function:

```rust
fn cmd_status() -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_status(&cwd)
}
```

**Step 4: Run tests**

Run: `cargo test -p repo-cli status`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/repo-cli/src/commands/status.rs crates/repo-cli/src/commands/mod.rs crates/repo-cli/src/main.rs
git commit -m "feat(cli): implement status command"
```

---

## Task 6: Add Shell Completions

**Files:**
- Modify: `crates/repo-cli/Cargo.toml`
- Modify: `crates/repo-cli/src/cli.rs`
- Modify: `crates/repo-cli/src/main.rs`

**Step 1: Add clap_complete dependency**

In `crates/repo-cli/Cargo.toml`, add:

```toml
clap_complete = "4"
```

**Step 2: Add Completions command variant**

In `crates/repo-cli/src/cli.rs`, add to Commands enum:

```rust
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
```

Add test:

```rust
#[test]
fn parse_completions_command() {
    let cli = Cli::parse_from(["repo", "completions", "bash"]);
    assert!(matches!(cli.command, Some(Commands::Completions { .. })));
}
```

**Step 3: Wire up completions in main.rs**

Add import at top:

```rust
use clap::CommandFactory;
use clap_complete::generate;
```

Add to execute_command match:

```rust
        Commands::Completions { shell } => cmd_completions(shell),
```

Add command function:

```rust
fn cmd_completions(shell: clap_complete::Shell) -> Result<()> {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    generate(shell, &mut cmd, name, &mut std::io::stdout());
    Ok(())
}
```

**Step 4: Run test**

Run: `cargo test -p repo-cli parse_completions`
Expected: PASS

**Step 5: Manual verification**

Run: `cargo run -p repo-cli -- completions bash | head -20`
Expected: Shell completion script output

**Step 6: Commit**

```bash
git add crates/repo-cli/Cargo.toml crates/repo-cli/src/cli.rs crates/repo-cli/src/main.rs
git commit -m "feat(cli): add shell completions command"
```

---

## Task 7: Improve --help Text with Examples

**Files:**
- Modify: `crates/repo-cli/src/cli.rs`

**Step 1: Update command documentation with examples**

In `crates/repo-cli/src/cli.rs`, update the `Commands` enum doc comments:

```rust
/// Available commands
#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum Commands {
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
        // ... existing fields
    },

    /// Add a tool to the repository
    ///
    /// Adds the tool to config.toml and runs sync.
    /// Use 'repo list-tools' to see available tools.
    ///
    /// Examples:
    ///   repo add-tool claude    # Add Claude Code support
    ///   repo add-tool cursor    # Add Cursor IDE support
    AddTool {
        /// Name of the tool (use 'repo list-tools' to see options)
        name: String,
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

    /// Show repository status
    ///
    /// Displays current configuration and tool sync status.
    ///
    /// Example:
    ///   repo status
    Status,

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
        shell: clap_complete::Shell,
    },
}
```

**Step 2: Verify help output**

Run: `cargo run -p repo-cli -- add-tool --help`
Expected: Shows "Examples:" section

Run: `cargo run -p repo-cli -- list-tools --help`
Expected: Shows "Examples:" section

**Step 3: Commit**

```bash
git add crates/repo-cli/src/cli.rs
git commit -m "docs(cli): add examples to --help documentation"
```

---

## Task 8: Create Integration Test for New Commands

**Files:**
- Modify: `docker/scripts/test-all.sh`
- Create: `docker/scripts/test-cli-discovery.sh`

**Step 1: Create CLI discovery test script**

Create `docker/scripts/test-cli-discovery.sh`:

```bash
#!/bin/bash
# CLI Discovery Tests
# Tests the new list-tools, list-presets, status, and completions commands

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
RESULTS_DIR="$PROJECT_ROOT/test-results/cli-discovery"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

mkdir -p "$RESULTS_DIR"

TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

log_test() {
    local name="$1"
    local status="$2"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    if [ "$status" = "PASS" ]; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
        echo -e "  ${GREEN}✓${NC} $name"
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
        echo -e "  ${RED}✗${NC} $name"
    fi
}

WORK_DIR=$(mktemp -d)
trap "rm -rf $WORK_DIR" EXIT

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}CLI Discovery Command Tests${NC}"
echo "Testing list-tools, list-presets, status, and completions"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

REPO_BIN="$PROJECT_ROOT/target/debug/repo"

# Build if needed
if [ ! -f "$REPO_BIN" ]; then
    echo "Building repo CLI..."
    cd "$PROJECT_ROOT" && cargo build -p repo-cli --quiet
fi

# ============================================
# list-tools Tests
# ============================================
echo ""
echo -e "${YELLOW}Testing: list-tools${NC}"

# Test: list-tools shows output
if $REPO_BIN list-tools 2>&1 | grep -q "Available Tools"; then
    log_test "list-tools shows Available Tools header" "PASS"
else
    log_test "list-tools shows Available Tools header" "FAIL"
fi

# Test: list-tools shows claude
if $REPO_BIN list-tools 2>&1 | grep -q "claude"; then
    log_test "list-tools shows claude" "PASS"
else
    log_test "list-tools shows claude" "FAIL"
fi

# Test: list-tools shows cursor
if $REPO_BIN list-tools 2>&1 | grep -q "cursor"; then
    log_test "list-tools shows cursor" "PASS"
else
    log_test "list-tools shows cursor" "FAIL"
fi

# Test: list-tools with category filter
if $REPO_BIN list-tools --category ide 2>&1 | grep -q "IDE Tools"; then
    log_test "list-tools --category ide works" "PASS"
else
    log_test "list-tools --category ide works" "FAIL"
fi

# Test: list-tools shows tool count
if $REPO_BIN list-tools 2>&1 | grep -q "Total:"; then
    log_test "list-tools shows total count" "PASS"
else
    log_test "list-tools shows total count" "FAIL"
fi

# ============================================
# list-presets Tests
# ============================================
echo ""
echo -e "${YELLOW}Testing: list-presets${NC}"

# Test: list-presets shows output
if $REPO_BIN list-presets 2>&1 | grep -q "Available Presets"; then
    log_test "list-presets shows Available Presets header" "PASS"
else
    log_test "list-presets shows Available Presets header" "FAIL"
fi

# Test: list-presets shows env:python
if $REPO_BIN list-presets 2>&1 | grep -q "env:python"; then
    log_test "list-presets shows env:python" "PASS"
else
    log_test "list-presets shows env:python" "FAIL"
fi

# ============================================
# status Tests
# ============================================
echo ""
echo -e "${YELLOW}Testing: status${NC}"

# Test: status in non-repo shows not initialized
cd "$WORK_DIR"
if $REPO_BIN status 2>&1 | grep -q "Not a repository"; then
    log_test "status in non-repo shows not initialized" "PASS"
else
    log_test "status in non-repo shows not initialized" "FAIL"
fi

# Test: status in initialized repo
mkdir -p .repository
cat > .repository/config.toml << 'EOF'
tools = ["claude", "cursor"]

[core]
mode = "standard"
EOF
if $REPO_BIN status 2>&1 | grep -q "Repository Status"; then
    log_test "status in repo shows Repository Status" "PASS"
else
    log_test "status in repo shows Repository Status" "FAIL"
fi

# Test: status shows enabled tools
if $REPO_BIN status 2>&1 | grep -q "Enabled Tools"; then
    log_test "status shows Enabled Tools section" "PASS"
else
    log_test "status shows Enabled Tools section" "FAIL"
fi

# Test: status shows mode
if $REPO_BIN status 2>&1 | grep -q "standard"; then
    log_test "status shows mode" "PASS"
else
    log_test "status shows mode" "FAIL"
fi

# ============================================
# completions Tests
# ============================================
echo ""
echo -e "${YELLOW}Testing: completions${NC}"

# Test: completions bash produces output
if $REPO_BIN completions bash 2>&1 | grep -q "complete"; then
    log_test "completions bash produces script" "PASS"
else
    log_test "completions bash produces script" "FAIL"
fi

# Test: completions zsh produces output
if $REPO_BIN completions zsh 2>&1 | grep -q "#compdef"; then
    log_test "completions zsh produces script" "PASS"
else
    log_test "completions zsh produces script" "FAIL"
fi

# ============================================
# --help Tests
# ============================================
echo ""
echo -e "${YELLOW}Testing: --help improvements${NC}"

# Test: add-tool --help mentions list-tools
if $REPO_BIN add-tool --help 2>&1 | grep -q "list-tools"; then
    log_test "add-tool --help references list-tools" "PASS"
else
    log_test "add-tool --help references list-tools" "FAIL"
fi

# Test: list-tools --help shows examples
if $REPO_BIN list-tools --help 2>&1 | grep -qi "example"; then
    log_test "list-tools --help shows examples" "PASS"
else
    log_test "list-tools --help shows examples" "FAIL"
fi

# ============================================
# Summary
# ============================================
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BOLD}CLI Discovery Test Summary${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "  Total Tests:  $TOTAL_TESTS"
echo -e "  Passed:       ${GREEN}$PASSED_TESTS${NC}"
echo -e "  Failed:       ${RED}$FAILED_TESTS${NC}"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}${BOLD}ALL CLI DISCOVERY TESTS PASSED${NC}"
    exit 0
else
    echo -e "${RED}${BOLD}SOME TESTS FAILED${NC}"
    exit 1
fi
```

**Step 2: Make executable and add to test-all.sh**

```bash
chmod +x docker/scripts/test-cli-discovery.sh
```

In `docker/scripts/test-all.sh`, add after expert-workflows:

```bash
run_test_suite "cli-discovery" "test-cli-discovery.sh" false || true
```

**Step 3: Run tests**

Run: `./docker/scripts/test-cli-discovery.sh`
Expected: All tests PASS

**Step 4: Commit**

```bash
git add docker/scripts/test-cli-discovery.sh docker/scripts/test-all.sh
git commit -m "test(cli): add CLI discovery command integration tests"
```

---

## Task 9: Final Integration and Verification

**Step 1: Run full test suite**

Run: `cargo test --workspace`
Expected: All tests pass

**Step 2: Run shell integration tests**

Run: `./docker/scripts/test-all.sh`
Expected: All suites pass including cli-discovery

**Step 3: Manual verification of UX improvements**

```bash
# List tools (should show categorized list)
cargo run -p repo-cli -- list-tools

# List with filter
cargo run -p repo-cli -- list-tools --category cli-agent

# List presets
cargo run -p repo-cli -- list-presets

# Status (in test repo)
cargo run -p repo-cli -- status

# Completions
cargo run -p repo-cli -- completions bash | head -10

# Help with examples
cargo run -p repo-cli -- add-tool --help
```

**Step 4: Final commit**

```bash
git add -A
git commit -m "feat(cli): complete CLI/UX improvements

- Add list-tools command with category filter
- Add list-presets command
- Add status command showing repo state
- Add shell completions (bash, zsh, fish, powershell)
- Improve --help with examples
- Add integration tests for discovery commands

Closes #cli-ux-improvements"
```

---

## Summary

| Task | Command | Description |
|------|---------|-------------|
| 1-3 | `list-tools` | Show available tools by category |
| 4-5 | `status` | Show repository configuration state |
| 3 | `list-presets` | Show available presets |
| 6 | `completions` | Generate shell completions |
| 7 | `--help` | Add examples to help text |
| 8-9 | Tests | Integration tests for new commands |

**Expected UX Score Improvement:** 6.5/10 → 8.5/10
