# CLI Improvements Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add missing CLI commands (`status`, `diff`, `checkout`) and improve dry-run output for professional workflows and CI/CD integration.

**Architecture:** Extend existing clap-based CLI with new commands that delegate to repo-core. Use colored output for better UX. Ensure all commands are scriptable with machine-readable output option.

**Tech Stack:** Rust, clap (derive), colored, repo-core, serde_json (for --json flag)

---

## Prerequisites

Review existing CLI structure:
- `crates/repo-cli/src/cli.rs` - Command definitions
- `crates/repo-cli/src/commands/` - Command implementations
- `crates/repo-cli/src/main.rs` - Entry point

---

## Task 1: Add `repo status` Command

**Files:**
- Modify: `crates/repo-cli/src/cli.rs`
- Create: `crates/repo-cli/src/commands/status.rs`
- Modify: `crates/repo-cli/src/commands/mod.rs`
- Modify: `crates/repo-cli/src/main.rs`

**Step 1: Add Status to Commands enum**

In `crates/repo-cli/src/cli.rs`, add to `Commands` enum:

```rust
/// Show repository status overview
Status {
    /// Output as JSON for scripting
    #[arg(long)]
    json: bool,
},
```

**Step 2: Write test for status command**

Create test in `crates/repo-cli/tests/status_test.rs`:

```rust
use assert_cmd::Command;
use tempfile::TempDir;
use std::fs;

fn create_test_repo(dir: &std::path::Path) {
    fs::create_dir_all(dir.join(".git")).unwrap();
    fs::create_dir_all(dir.join(".repository")).unwrap();
    fs::write(
        dir.join(".repository/config.toml"),
        "tools = [\"cursor\", \"claude\"]\n\n[core]\nmode = \"standard\"\n",
    ).unwrap();
}

#[test]
fn test_status_shows_mode() {
    let temp = TempDir::new().unwrap();
    create_test_repo(temp.path());

    let mut cmd = Command::cargo_bin("repo").unwrap();
    let output = cmd
        .current_dir(temp.path())
        .arg("status")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Mode") || stdout.contains("mode"));
    assert!(stdout.contains("standard") || stdout.contains("Standard"));
}

#[test]
fn test_status_shows_tools() {
    let temp = TempDir::new().unwrap();
    create_test_repo(temp.path());

    let mut cmd = Command::cargo_bin("repo").unwrap();
    let output = cmd
        .current_dir(temp.path())
        .arg("status")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cursor"));
    assert!(stdout.contains("claude"));
}

#[test]
fn test_status_json_output() {
    let temp = TempDir::new().unwrap();
    create_test_repo(temp.path());

    let mut cmd = Command::cargo_bin("repo").unwrap();
    let output = cmd
        .current_dir(temp.path())
        .args(["status", "--json"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Output should be valid JSON");

    assert!(parsed.get("mode").is_some());
    assert!(parsed.get("tools").is_some());
}
```

**Step 3: Run test to verify it fails**

Run: `cargo test -p repo-cli test_status_shows_mode`
Expected: FAIL - command not found

**Step 4: Implement status command**

Create `crates/repo-cli/src/commands/status.rs`:

```rust
//! Status command implementation
//!
//! Shows an overview of repository configuration state.

use std::path::Path;
use colored::Colorize;
use serde::Serialize;
use repo_core::{ConfigResolver, Mode, SyncEngine, CheckStatus};
use repo_fs::NormalizedPath;
use crate::error::Result;

#[derive(Serialize)]
struct StatusOutput {
    mode: String,
    root: String,
    config_exists: bool,
    tools: Vec<String>,
    rules_count: usize,
    sync_status: String,
    missing_count: usize,
    drifted_count: usize,
}

/// Run the status command
pub fn run_status(path: &Path, json: bool) -> Result<()> {
    let normalized = NormalizedPath::new(path);

    // Find repository root
    let root = find_repo_root(&normalized)?;
    let resolver = ConfigResolver::new(root.clone());

    // Gather status info
    let (mode, tools) = if resolver.has_config() {
        let config = resolver.resolve()?;
        let mode = config.mode.parse().unwrap_or(Mode::Worktrees);
        (mode, config.tools)
    } else {
        (Mode::Worktrees, vec![])
    };

    // Count rules
    let rules_dir = root.join(".repository/rules");
    let rules_count = if rules_dir.as_ref().exists() {
        std::fs::read_dir(rules_dir.as_ref())
            .map(|entries| entries.filter_map(|e| e.ok()).count())
            .unwrap_or(0)
    } else {
        0
    };

    // Get sync status
    let engine = SyncEngine::new(root.clone(), mode)?;
    let report = engine.check()?;

    if json {
        let output = StatusOutput {
            mode: format!("{:?}", mode).to_lowercase(),
            root: root.to_string(),
            config_exists: resolver.has_config(),
            tools,
            rules_count,
            sync_status: format!("{:?}", report.status).to_lowercase(),
            missing_count: report.missing.len(),
            drifted_count: report.drifted.len(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        print_human_status(&root, mode, &tools, rules_count, &report)?;
    }

    Ok(())
}

fn print_human_status(
    root: &NormalizedPath,
    mode: Mode,
    tools: &[String],
    rules_count: usize,
    report: &repo_core::CheckReport,
) -> Result<()> {
    println!("{}", "Repository Status".green().bold());
    println!();

    // Basic info
    println!("  {}: {}", "Root".bold(), root.to_string().cyan());
    println!("  {}: {}", "Mode".bold(), format!("{:?}", mode).cyan());

    // Tools
    if tools.is_empty() {
        println!("  {}: {}", "Tools".bold(), "none".dimmed());
    } else {
        println!("  {}: {}", "Tools".bold(), tools.join(", ").yellow());
    }

    // Rules
    if rules_count == 0 {
        println!("  {}: {}", "Rules".bold(), "none".dimmed());
    } else {
        println!("  {}: {} defined", "Rules".bold(), rules_count.to_string().yellow());
    }

    // Sync status
    println!();
    match report.status {
        CheckStatus::Healthy => {
            println!("  {}: {}", "Sync".bold(), "healthy".green());
        }
        CheckStatus::Missing => {
            println!("  {}: {} ({} missing)",
                "Sync".bold(),
                "incomplete".yellow(),
                report.missing.len()
            );
            for item in report.missing.iter().take(3) {
                println!("    {} {}", "-".dimmed(), item.file.dimmed());
            }
            if report.missing.len() > 3 {
                println!("    {} ...and {} more", "-".dimmed(), report.missing.len() - 3);
            }
        }
        CheckStatus::Drifted => {
            println!("  {}: {} ({} drifted)",
                "Sync".bold(),
                "drifted".red(),
                report.drifted.len()
            );
            for item in report.drifted.iter().take(3) {
                println!("    {} {}", "~".yellow(), item.file);
            }
            if report.drifted.len() > 3 {
                println!("    {} ...and {} more", "~".yellow(), report.drifted.len() - 3);
            }
        }
        CheckStatus::Broken => {
            println!("  {}: {}", "Sync".bold(), "broken".red().bold());
            for msg in &report.messages {
                println!("    {} {}", "!".red(), msg);
            }
        }
    }

    // Hint
    if report.status != CheckStatus::Healthy {
        println!();
        println!("  Run {} to fix issues.", "repo sync".cyan());
    }

    Ok(())
}

fn find_repo_root(start: &NormalizedPath) -> Result<NormalizedPath> {
    let mut current = start.clone();

    loop {
        if current.join(".repository").as_ref().exists() {
            return Ok(current);
        }

        // Check parent
        let path = current.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            if parent == path {
                break;
            }
            current = NormalizedPath::new(parent);
        } else {
            break;
        }
    }

    // Not found, use current directory
    Ok(start.clone())
}
```

**Step 5: Export from mod.rs**

In `crates/repo-cli/src/commands/mod.rs`:

```rust
pub mod status;
pub use status::run_status;
```

**Step 6: Wire up in main.rs**

In `crates/repo-cli/src/main.rs`, add to match block:

```rust
Commands::Status { json } => {
    let cwd = std::env::current_dir()?;
    commands::run_status(&cwd, json)?;
}
```

**Step 7: Run tests**

Run: `cargo test -p repo-cli status`
Expected: All tests pass

**Step 8: Commit**

```bash
git add crates/repo-cli/src/cli.rs crates/repo-cli/src/commands/status.rs crates/repo-cli/src/commands/mod.rs crates/repo-cli/src/main.rs
git commit -m "feat(repo-cli): add status command for repository overview"
```

---

## Task 2: Add `repo diff` Command

**Files:**
- Modify: `crates/repo-cli/src/cli.rs`
- Create: `crates/repo-cli/src/commands/diff.rs`
- Modify: `crates/repo-cli/src/commands/mod.rs`
- Modify: `crates/repo-cli/src/main.rs`

**Step 1: Add Diff to Commands enum**

```rust
/// Preview what sync would change
Diff {
    /// Output as JSON for scripting
    #[arg(long)]
    json: bool,
},
```

**Step 2: Implement diff command**

Create `crates/repo-cli/src/commands/diff.rs`:

```rust
//! Diff command implementation
//!
//! Shows what changes sync would make without applying them.

use std::path::Path;
use colored::Colorize;
use serde::Serialize;
use repo_core::{ConfigResolver, Mode, SyncEngine, SyncOptions};
use repo_fs::NormalizedPath;
use crate::error::Result;

#[derive(Serialize)]
struct DiffOutput {
    has_changes: bool,
    actions: Vec<DiffAction>,
}

#[derive(Serialize)]
struct DiffAction {
    action_type: String, // "create", "update", "delete"
    path: String,
    description: String,
}

/// Run the diff command
pub fn run_diff(path: &Path, json: bool) -> Result<()> {
    let normalized = NormalizedPath::new(path);

    // Find repository root
    let root = crate::commands::status::find_repo_root_internal(&normalized)?;
    let resolver = ConfigResolver::new(root.clone());

    let mode = if resolver.has_config() {
        let config = resolver.resolve()?;
        config.mode.parse().unwrap_or(Mode::Worktrees)
    } else {
        Mode::Worktrees
    };

    // Run sync with dry_run=true
    let engine = SyncEngine::new(root, mode)?;
    let options = SyncOptions { dry_run: true };
    let report = engine.sync_with_options(options)?;

    if json {
        let actions: Vec<DiffAction> = report.actions.iter().map(|a| {
            let (action_type, path, description) = parse_action(a);
            DiffAction { action_type, path, description }
        }).collect();

        let output = DiffOutput {
            has_changes: !report.actions.is_empty(),
            actions,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        print_human_diff(&report)?;
    }

    Ok(())
}

fn print_human_diff(report: &repo_core::SyncReport) -> Result<()> {
    if report.actions.is_empty() {
        println!("{} No changes needed", "OK".green().bold());
        return Ok(());
    }

    println!("{}", "Changes that would be made:".bold());
    println!();

    for action in &report.actions {
        let (action_type, path, description) = parse_action(action);

        let prefix = match action_type.as_str() {
            "create" => "+".green(),
            "update" => "~".yellow(),
            "delete" => "-".red(),
            _ => "*".cyan(),
        };

        println!("  {} {} {}", prefix, path.bold(), description.dimmed());
    }

    println!();
    println!("Run {} to apply these changes.", "repo sync".cyan());

    Ok(())
}

fn parse_action(action: &str) -> (String, String, String) {
    // Parse action strings like "Create .cursor/rules/foo.mdc"
    let lower = action.to_lowercase();

    let action_type = if lower.contains("create") || lower.contains("add") {
        "create"
    } else if lower.contains("update") || lower.contains("modify") || lower.contains("change") {
        "update"
    } else if lower.contains("delete") || lower.contains("remove") {
        "delete"
    } else {
        "other"
    };

    // Extract path (first word that looks like a path)
    let path = action.split_whitespace()
        .find(|w| w.contains('/') || w.contains('.'))
        .unwrap_or("")
        .to_string();

    (action_type.to_string(), path, action.to_string())
}
```

**Step 3: Export and wire up**

Follow same pattern as status command.

**Step 4: Commit**

```bash
git add crates/repo-cli/src/cli.rs crates/repo-cli/src/commands/diff.rs crates/repo-cli/src/commands/mod.rs crates/repo-cli/src/main.rs
git commit -m "feat(repo-cli): add diff command for sync preview"
```

---

## Task 3: Add `repo branch checkout` Command

**Files:**
- Modify: `crates/repo-cli/src/cli.rs`
- Modify: `crates/repo-cli/src/commands/branch.rs`

**Step 1: Add Checkout to BranchAction enum**

In `crates/repo-cli/src/cli.rs`, find the `BranchAction` enum and add:

```rust
/// Switch to a branch (or worktree in worktrees mode)
Checkout {
    /// Branch name to checkout
    name: String,
},
```

**Step 2: Implement checkout in branch.rs**

In `crates/repo-cli/src/commands/branch.rs`, add to match:

```rust
BranchAction::Checkout { name } => {
    run_branch_checkout(&cwd, &name, verbose)?;
}
```

Add the function:

```rust
fn run_branch_checkout(path: &Path, name: &str, verbose: bool) -> Result<()> {
    use repo_core::{ConfigResolver, Mode, ModeBackend, StandardBackend, WorktreeBackend};
    use repo_fs::NormalizedPath;

    let normalized = NormalizedPath::new(path);
    let resolver = ConfigResolver::new(normalized.clone());

    let mode = if resolver.has_config() {
        let config = resolver.resolve()?;
        config.mode.parse().unwrap_or(Mode::Worktrees)
    } else {
        Mode::Worktrees
    };

    if verbose {
        println!("Checking out branch: {}", name);
    }

    match mode {
        Mode::Standard => {
            let backend = StandardBackend::new(normalized)?;
            backend.switch_branch(name)?;
            println!("{} Switched to branch '{}'", "OK".green(), name);
        }
        Mode::Worktrees => {
            let backend = WorktreeBackend::new(normalized)?;

            // In worktrees mode, "checkout" opens the worktree directory
            let branches = backend.list_branches()?;
            if let Some(branch) = branches.iter().find(|b| b.name == name) {
                if let Some(worktree_path) = &branch.path {
                    println!("{} Worktree for '{}' is at:", "OK".green(), name);
                    println!("  {}", worktree_path.display());
                    println!();
                    println!("To switch to it, run:");
                    println!("  cd {}", worktree_path.display());
                } else {
                    // Branch exists but no worktree
                    println!("{} Branch '{}' exists but has no worktree.", "!".yellow(), name);
                    println!("Create one with: repo branch add {}", name);
                }
            } else {
                return Err(crate::error::CliError::BranchNotFound(name.to_string()));
            }
        }
    }

    Ok(())
}
```

**Step 3: Add BranchNotFound error**

In `crates/repo-cli/src/error.rs`:

```rust
#[error("Branch not found: {0}")]
BranchNotFound(String),
```

**Step 4: Run tests**

Run: `cargo test -p repo-cli branch`
Expected: All tests pass

**Step 5: Commit**

```bash
git add crates/repo-cli/src/cli.rs crates/repo-cli/src/commands/branch.rs crates/repo-cli/src/error.rs
git commit -m "feat(repo-cli): add branch checkout command"
```

---

## Task 4: Improve Dry-Run Output Verbosity

**Files:**
- Modify: `crates/repo-cli/src/commands/sync.rs`
- Modify: `crates/repo-core/src/sync/engine.rs`

**Step 1: Enhance SyncReport actions**

In `crates/repo-core/src/sync/engine.rs`, ensure actions include:
- File path
- Action type (create/update/delete)
- Old vs new content preview (optional)
- Tool name

**Step 2: Update sync command output**

In `crates/repo-cli/src/commands/sync.rs`:

```rust
fn print_dry_run_report(report: &SyncReport) {
    println!("{} Dry run - no changes made", "INFO".blue().bold());
    println!();

    if report.actions.is_empty() {
        println!("{} No changes would be made", "OK".green());
        return;
    }

    println!("{} changes would be made:", report.actions.len());
    println!();

    for action in &report.actions {
        // Detailed action output
        if action.contains("create") || action.contains("Create") {
            println!("  {} {}", "+".green().bold(), action);
        } else if action.contains("update") || action.contains("Update") {
            println!("  {} {}", "~".yellow().bold(), action);
        } else if action.contains("delete") || action.contains("Delete") {
            println!("  {} {}", "-".red().bold(), action);
        } else {
            println!("  {} {}", "*".cyan(), action);
        }
    }

    if !report.errors.is_empty() {
        println!();
        println!("{} errors:", report.errors.len());
        for error in &report.errors {
            println!("  {} {}", "!".red(), error);
        }
    }

    println!();
    println!("Run {} without --dry-run to apply.", "repo sync".cyan());
}
```

**Step 3: Add --json flag to sync**

In `crates/repo-cli/src/cli.rs`, update Sync command:

```rust
/// Synchronize tool configurations
Sync {
    /// Preview changes without applying them
    #[arg(long)]
    dry_run: bool,

    /// Output as JSON for CI/CD integration
    #[arg(long)]
    json: bool,
},
```

**Step 4: Commit**

```bash
git add crates/repo-cli/src/commands/sync.rs crates/repo-cli/src/cli.rs
git commit -m "feat(repo-cli): improve dry-run output verbosity and add --json flag"
```

---

## Task 5: Add Bash Completion Generation

**Files:**
- Modify: `crates/repo-cli/src/cli.rs`
- Modify: `crates/repo-cli/src/main.rs`

**Step 1: Add completions command**

In `crates/repo-cli/src/cli.rs`:

```rust
/// Generate shell completions
Completions {
    /// Shell to generate completions for
    #[arg(value_enum)]
    shell: clap_complete::Shell,
},
```

**Step 2: Add clap_complete dependency**

In `crates/repo-cli/Cargo.toml`:

```toml
clap_complete = "4.4"
```

**Step 3: Implement completions generation**

In `crates/repo-cli/src/main.rs`:

```rust
use clap_complete::{generate, Shell};

// In match block:
Commands::Completions { shell } => {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    generate(shell, &mut cmd, name, &mut std::io::stdout());
}
```

**Step 4: Test**

Run: `cargo run -p repo-cli -- completions bash > /tmp/repo.bash`
Run: `source /tmp/repo.bash`

**Step 5: Commit**

```bash
git add crates/repo-cli/src/cli.rs crates/repo-cli/src/main.rs crates/repo-cli/Cargo.toml
git commit -m "feat(repo-cli): add shell completion generation"
```

---

## Completion Checklist

- [ ] `repo status` command implemented with --json flag
- [ ] `repo diff` command implemented with --json flag
- [ ] `repo branch checkout` command implemented
- [ ] Dry-run output improved with detailed changes
- [ ] `repo sync --json` flag added for CI/CD
- [ ] Shell completion generation added
- [ ] All tests pass

---

## Verification

After completing all tasks:

```bash
# Test new commands
cargo build -p repo-cli
./target/debug/repo status
./target/debug/repo status --json
./target/debug/repo diff
./target/debug/repo diff --json
./target/debug/repo branch checkout main
./target/debug/repo sync --dry-run

# Generate completions
./target/debug/repo completions bash > repo.bash
./target/debug/repo completions zsh > _repo
./target/debug/repo completions fish > repo.fish
```

---

*Plan created: 2026-01-29*
*Addresses: DX-004 (no status), DX-005 (no diff), DX-006 (no checkout), DX-007 (sparse dry-run)*
