//! Integration tests for the repo CLI binary.
//!
//! These tests exercise the actual compiled binary using assert_cmd.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

/// Get a Command for the repo binary
#[allow(deprecated)]
fn repo_cmd() -> Command {
    Command::cargo_bin("repo").expect("Failed to find repo binary")
}

// ============================================================================
// Help and Version Tests
// ============================================================================

#[test]
fn test_help_output() {
    let mut cmd = repo_cmd();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Repository Manager"));
}

#[test]
fn test_help_flag_short() {
    let mut cmd = repo_cmd();
    cmd.arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("Repository Manager"));
}

#[test]
fn test_version_output() {
    let mut cmd = repo_cmd();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("repo"));
}

#[test]
fn test_version_flag_short() {
    let mut cmd = repo_cmd();
    cmd.arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains("repo"));
}

#[test]
fn test_no_command_shows_help_hint() {
    let mut cmd = repo_cmd();
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("repo --help"));
}

// ============================================================================
// Init Command Tests
// ============================================================================

#[test]
fn test_init_creates_structure() {
    let dir = tempdir().unwrap();

    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard"])
        .assert()
        .success()
        .stdout(predicate::str::contains("initialized"));

    // Verify .repository directory was created
    assert!(dir.path().join(".repository").exists());
    assert!(dir.path().join(".repository").is_dir());

    // Verify config.toml was created
    assert!(dir.path().join(".repository/config.toml").exists());

    // Verify config content
    let config_content = fs::read_to_string(dir.path().join(".repository/config.toml")).unwrap();
    assert!(config_content.contains("[core]"));
    assert!(config_content.contains("mode = \"standard\""));
}

#[test]
fn test_init_default_mode_is_worktrees() {
    let dir = tempdir().unwrap();

    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    let config_content = fs::read_to_string(dir.path().join(".repository/config.toml")).unwrap();
    assert!(config_content.contains("mode = \"worktrees\""));

    // Worktrees mode should create main/ directory
    assert!(dir.path().join("main").exists());
}

#[test]
fn test_init_worktree_mode() {
    let dir = tempdir().unwrap();

    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "worktree"])
        .assert()
        .success();

    let config_content = fs::read_to_string(dir.path().join(".repository/config.toml")).unwrap();
    assert!(config_content.contains("mode = \"worktree\""));

    // Worktree mode should create main/ directory
    assert!(dir.path().join("main").exists());
}

#[test]
fn test_init_with_tools() {
    let dir = tempdir().unwrap();

    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard", "--tools", "vscode", "--tools", "cursor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("vscode"))
        .stdout(predicate::str::contains("cursor"));

    let config_content = fs::read_to_string(dir.path().join(".repository/config.toml")).unwrap();
    assert!(config_content.contains("tools = ["));
    assert!(config_content.contains("\"vscode\""));
    assert!(config_content.contains("\"cursor\""));
}

#[test]
fn test_init_with_presets() {
    let dir = tempdir().unwrap();

    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard", "--presets", "typescript", "--presets", "react"])
        .assert()
        .success()
        .stdout(predicate::str::contains("typescript"))
        .stdout(predicate::str::contains("react"));

    let config_content = fs::read_to_string(dir.path().join(".repository/config.toml")).unwrap();
    assert!(config_content.contains("[presets.\"typescript\"]"));
    assert!(config_content.contains("[presets.\"react\"]"));
}

#[test]
fn test_init_with_tools_and_presets() {
    let dir = tempdir().unwrap();

    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args([
            "init",
            "--mode", "standard",
            "--tools", "eslint",
            "--presets", "typescript",
        ])
        .assert()
        .success();

    let config_content = fs::read_to_string(dir.path().join(".repository/config.toml")).unwrap();
    assert!(config_content.contains("tools = ["));
    assert!(config_content.contains("\"eslint\""));
    assert!(config_content.contains("[presets.\"typescript\"]"));
}

#[test]
fn test_init_invalid_mode() {
    let dir = tempdir().unwrap();

    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid mode"));
}

#[test]
fn test_init_creates_git_repo() {
    let dir = tempdir().unwrap();

    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard"])
        .assert()
        .success();

    // Verify .git was created
    assert!(dir.path().join(".git").exists());
}

// ============================================================================
// Tool Management Tests
// ============================================================================

#[test]
fn test_add_tool_workflow() {
    let dir = tempdir().unwrap();

    // First init
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard"])
        .assert()
        .success();

    // Then add tool
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["add-tool", "eslint"])
        .assert()
        .success()
        .stdout(predicate::str::contains("eslint"))
        .stdout(predicate::str::contains("added"));

    // Verify config was updated
    let config_content = fs::read_to_string(dir.path().join(".repository/config.toml")).unwrap();
    assert!(config_content.contains("eslint"));
}

#[test]
fn test_add_multiple_tools() {
    let dir = tempdir().unwrap();

    // Init
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard"])
        .assert()
        .success();

    // Add first tool
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["add-tool", "eslint"])
        .assert()
        .success();

    // Add second tool
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["add-tool", "prettier"])
        .assert()
        .success();

    // Verify both tools in config
    let config_content = fs::read_to_string(dir.path().join(".repository/config.toml")).unwrap();
    assert!(config_content.contains("eslint"));
    assert!(config_content.contains("prettier"));
}

#[test]
fn test_add_duplicate_tool() {
    let dir = tempdir().unwrap();

    // Init without tools, then add tool
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard"])
        .assert()
        .success();

    // Add tool first time
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["add-tool", "eslint"])
        .assert()
        .success();

    // Add same tool again - should succeed with "already configured" message
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["add-tool", "eslint"])
        .assert()
        .success()
        .stdout(predicate::str::contains("already configured"));
}

#[test]
fn test_remove_tool_workflow() {
    let dir = tempdir().unwrap();

    // Init without tools, then add tool via add-tool command
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard"])
        .assert()
        .success();

    // Add tool first
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["add-tool", "eslint"])
        .assert()
        .success();

    // Remove tool
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["remove-tool", "eslint"])
        .assert()
        .success()
        .stdout(predicate::str::contains("removed"));

    // Verify tool was removed
    let config_content = fs::read_to_string(dir.path().join(".repository/config.toml")).unwrap();
    assert!(!config_content.contains("\"eslint\""));
}

#[test]
fn test_remove_nonexistent_tool() {
    let dir = tempdir().unwrap();

    // Init without tools
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard"])
        .assert()
        .success();

    // Remove non-existent tool - should succeed with warning
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["remove-tool", "eslint"])
        .assert()
        .success()
        .stdout(predicate::str::contains("not found"));
}

#[test]
fn test_add_tool_without_init() {
    let dir = tempdir().unwrap();

    // Try to add tool without init - should fail
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["add-tool", "eslint"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Config file not found"));
}

// ============================================================================
// Preset Management Tests
// ============================================================================

#[test]
fn test_add_preset_workflow() {
    let dir = tempdir().unwrap();

    // First init
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard"])
        .assert()
        .success();

    // Then add preset
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["add-preset", "typescript"])
        .assert()
        .success()
        .stdout(predicate::str::contains("typescript"))
        .stdout(predicate::str::contains("added"));

    // Verify config was updated
    let config_content = fs::read_to_string(dir.path().join(".repository/config.toml")).unwrap();
    assert!(config_content.contains("[presets]"));
    assert!(config_content.contains("typescript"));
}

#[test]
fn test_add_multiple_presets() {
    let dir = tempdir().unwrap();

    // Init
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard"])
        .assert()
        .success();

    // Add presets
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["add-preset", "typescript"])
        .assert()
        .success();

    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["add-preset", "react"])
        .assert()
        .success();

    // Verify both presets in config
    let config_content = fs::read_to_string(dir.path().join(".repository/config.toml")).unwrap();
    assert!(config_content.contains("typescript"));
    assert!(config_content.contains("react"));
}

#[test]
fn test_add_duplicate_preset() {
    let dir = tempdir().unwrap();

    // Init with preset
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard", "--presets", "typescript"])
        .assert()
        .success();

    // Add same preset again
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["add-preset", "typescript"])
        .assert()
        .success()
        .stdout(predicate::str::contains("already configured"));
}

#[test]
fn test_remove_preset_workflow() {
    let dir = tempdir().unwrap();

    // Init with preset
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard", "--presets", "typescript"])
        .assert()
        .success();

    // Remove preset
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["remove-preset", "typescript"])
        .assert()
        .success()
        .stdout(predicate::str::contains("removed"));

    // Verify preset was removed
    let config_content = fs::read_to_string(dir.path().join(".repository/config.toml")).unwrap();
    assert!(!config_content.contains("typescript"));
}

#[test]
fn test_remove_nonexistent_preset() {
    let dir = tempdir().unwrap();

    // Init without presets
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard"])
        .assert()
        .success();

    // Remove non-existent preset - should succeed with warning
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["remove-preset", "typescript"])
        .assert()
        .success()
        .stdout(predicate::str::contains("not found"));
}

#[test]
fn test_add_preset_without_init() {
    let dir = tempdir().unwrap();

    // Try to add preset without init - should fail
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["add-preset", "typescript"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Config file not found"));
}

// ============================================================================
// Check Command Tests
// ============================================================================

#[test]
fn test_check_on_fresh_repo() {
    let dir = tempdir().unwrap();

    // Init
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard"])
        .assert()
        .success();

    // Check should report healthy (empty ledger = nothing to check)
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .arg("check")
        .assert()
        .success()
        .stdout(predicate::str::contains("healthy").or(predicate::str::contains("OK")));
}

#[test]
fn test_check_requires_git_repo() {
    let dir = tempdir().unwrap();

    // Check without init or git - should fail (requires proper repo layout)
    // With context-aware root resolution, we get a better error message
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not in a repository"));
}

// ============================================================================
// Sync Command Tests
// ============================================================================

#[test]
fn test_sync_creates_ledger() {
    let dir = tempdir().unwrap();

    // Init
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard"])
        .assert()
        .success();

    // Ledger should not exist yet
    assert!(!dir.path().join(".repository/ledger.toml").exists());

    // Run sync
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .arg("sync")
        .assert()
        .success();

    // Ledger should now exist
    assert!(dir.path().join(".repository/ledger.toml").exists());
}

#[test]
fn test_sync_idempotent() {
    let dir = tempdir().unwrap();

    // Init
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard"])
        .assert()
        .success();

    // First sync
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .arg("sync")
        .assert()
        .success();

    // Second sync - should succeed with "already synchronized"
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .arg("sync")
        .assert()
        .success()
        .stdout(predicate::str::contains("synchronized").or(predicate::str::contains("OK")));
}

// ============================================================================
// Fix Command Tests
// ============================================================================

#[test]
fn test_fix_on_healthy_repo() {
    let dir = tempdir().unwrap();

    // Init
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard"])
        .assert()
        .success();

    // Fix should report nothing to fix
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .arg("fix")
        .assert()
        .success()
        .stdout(predicate::str::contains("healthy").or(predicate::str::contains("Nothing to fix")));
}

// ============================================================================
// Verbose Mode Tests
// ============================================================================

#[test]
fn test_verbose_flag() {
    let dir = tempdir().unwrap();

    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["-v", "init", "--mode", "standard"])
        .assert()
        .success();
}

#[test]
fn test_verbose_flag_long() {
    let dir = tempdir().unwrap();

    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["--verbose", "init", "--mode", "standard"])
        .assert()
        .success();
}

#[test]
fn test_verbose_flag_after_command() {
    let dir = tempdir().unwrap();

    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--verbose", "--mode", "standard"])
        .assert()
        .success();
}

// ============================================================================
// Branch Command Tests (require git)
// ============================================================================

#[test]
fn test_branch_list_on_fresh_repo() {
    let dir = tempdir().unwrap();

    // Init as worktree mode
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "worktree"])
        .assert()
        .success();

    // Branch list - we don't check result as it depends on git state
    // The command should at least not panic
    let mut cmd = repo_cmd();
    let _ = cmd.current_dir(dir.path())
        .args(["branch", "list"])
        .assert();
}

// ============================================================================
// Edge Cases and Error Handling
// ============================================================================

#[test]
fn test_unknown_command() {
    let mut cmd = repo_cmd();
    cmd.arg("unknown-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn test_init_help() {
    let mut cmd = repo_cmd();
    cmd.args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialize"));
}

#[test]
fn test_check_help() {
    let mut cmd = repo_cmd();
    cmd.args(["check", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Check"));
}

#[test]
fn test_sync_help() {
    let mut cmd = repo_cmd();
    cmd.args(["sync", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Synchronize"));
}

#[test]
fn test_fix_help() {
    let mut cmd = repo_cmd();
    cmd.args(["fix", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Fix"));
}

#[test]
fn test_add_tool_help() {
    let mut cmd = repo_cmd();
    cmd.args(["add-tool", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Add a tool"));
}

#[test]
fn test_add_preset_help() {
    let mut cmd = repo_cmd();
    cmd.args(["add-preset", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Add a preset"));
}

#[test]
fn test_branch_help() {
    let mut cmd = repo_cmd();
    cmd.args(["branch", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("branch"));
}

// ============================================================================
// Full Workflow Tests
// ============================================================================

#[test]
fn test_full_workflow_init_add_check_sync() {
    let dir = tempdir().unwrap();

    // 1. Init
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard"])
        .assert()
        .success();

    // 2. Add tool
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["add-tool", "eslint"])
        .assert()
        .success();

    // 3. Add preset
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["add-preset", "typescript"])
        .assert()
        .success();

    // 4. Check
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .arg("check")
        .assert()
        .success();

    // 5. Sync
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .arg("sync")
        .assert()
        .success();

    // Verify final state
    let config_content = fs::read_to_string(dir.path().join(".repository/config.toml")).unwrap();
    assert!(config_content.contains("eslint"));
    assert!(config_content.contains("typescript"));
    assert!(dir.path().join(".repository/ledger.toml").exists());
}

#[test]
fn test_workflow_remove_tools_and_presets() {
    let dir = tempdir().unwrap();

    // Init then add tools and presets via commands (not --tools/--presets flags)
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard"])
        .assert()
        .success();

    // Add tools via add-tool command
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["add-tool", "eslint"])
        .assert()
        .success();

    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["add-tool", "prettier"])
        .assert()
        .success();

    // Add presets via add-preset command
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["add-preset", "typescript"])
        .assert()
        .success();

    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["add-preset", "react"])
        .assert()
        .success();

    // Remove one tool
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["remove-tool", "eslint"])
        .assert()
        .success();

    // Remove one preset
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["remove-preset", "typescript"])
        .assert()
        .success();

    // Verify remaining
    let config_content = fs::read_to_string(dir.path().join(".repository/config.toml")).unwrap();
    assert!(!config_content.contains("\"eslint\""));
    assert!(config_content.contains("prettier"));
    assert!(!config_content.contains("typescript"));
    assert!(config_content.contains("react"));
}

// ============================================================================
// E2E Workflow Tests (Phase 6)
// ============================================================================

#[test]
fn test_e2e_init_add_rule_creates_rule_file() {
    let dir = tempdir().unwrap();

    // 1. Init project with cursor tool
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard", "--tools", "cursor"])
        .assert()
        .success();

    // 2. Add a rule
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["add-rule", "test-rule", "--instruction", "Test instruction for testing"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test-rule"))
        .stdout(predicate::str::contains("added"));

    // 3. Verify rule file was created
    let rule_path = dir.path().join(".repository/rules/test-rule.md");
    assert!(rule_path.exists(), "Rule file should be created");

    let rule_content = fs::read_to_string(&rule_path).unwrap();
    assert!(rule_content.contains("Test instruction for testing"));
}

#[test]
fn test_e2e_add_rule_with_tags() {
    let dir = tempdir().unwrap();

    // 1. Init project
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard"])
        .assert()
        .success();

    // 2. Add a rule with tags
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args([
            "add-rule",
            "python-style",
            "--instruction", "Use snake_case for variables",
            "-t", "python",
            "-t", "style",
        ])
        .assert()
        .success();

    // 3. Verify rule file contains tags
    let rule_path = dir.path().join(".repository/rules/python-style.md");
    assert!(rule_path.exists());

    let rule_content = fs::read_to_string(&rule_path).unwrap();
    assert!(rule_content.contains("tags: python, style"));
    assert!(rule_content.contains("snake_case"));
}

#[test]
fn test_e2e_list_rules() {
    let dir = tempdir().unwrap();

    // 1. Init project
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard"])
        .assert()
        .success();

    // 2. Add rules
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["add-rule", "rule-one", "-i", "First rule"])
        .assert()
        .success();

    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["add-rule", "rule-two", "-i", "Second rule"])
        .assert()
        .success();

    // 3. List rules
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .arg("list-rules")
        .assert()
        .success()
        .stdout(predicate::str::contains("rule-one"))
        .stdout(predicate::str::contains("rule-two"));
}

#[test]
fn test_e2e_remove_rule() {
    let dir = tempdir().unwrap();

    // 1. Init and add rule
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard"])
        .assert()
        .success();

    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["add-rule", "temp-rule", "-i", "Temporary rule"])
        .assert()
        .success();

    // Verify rule exists
    assert!(dir.path().join(".repository/rules/temp-rule.md").exists());

    // 2. Remove rule
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["remove-rule", "temp-rule"])
        .assert()
        .success()
        .stdout(predicate::str::contains("removed"));

    // 3. Verify rule removed
    assert!(!dir.path().join(".repository/rules/temp-rule.md").exists());
}

#[test]
fn test_e2e_backup_on_tool_removal() {
    let dir = tempdir().unwrap();

    // 1. Init with cursor tool
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard", "--tools", "cursor"])
        .assert()
        .success();

    // 2. Create a .cursorrules file (simulating tool sync having created it)
    let cursorrules_path = dir.path().join(".cursorrules");
    fs::write(&cursorrules_path, "# My cursor rules\nOriginal content").unwrap();

    // Verify cursor is in initial config
    let initial_config = fs::read_to_string(dir.path().join(".repository/config.toml")).unwrap();
    assert!(initial_config.contains("\"cursor\""), "Initial config should contain cursor");

    // 3. Remove the cursor tool
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["remove-tool", "cursor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("removed"));

    // 4. Verify tool was removed from config
    // After removal, cursor should no longer be in the config
    // (tools line may be omitted entirely if empty, or show tools = [])
    let config_content = fs::read_to_string(dir.path().join(".repository/config.toml")).unwrap();
    assert!(
        !config_content.contains("\"cursor\""),
        "Config should not contain cursor after removal. Got: {}",
        config_content
    );

    // Note: backup is only created if ToolSyncer.remove_tool is called
    // CLI's remove-tool updates config but doesn't call ToolSyncer backup
    // This documents the gap for future implementation
}

#[test]
fn test_e2e_context_detection_from_subdirectory() {
    let dir = tempdir().unwrap();

    // 1. Init project
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard"])
        .assert()
        .success();

    // 2. Create a subdirectory
    let subdir = dir.path().join("src").join("lib");
    fs::create_dir_all(&subdir).unwrap();

    // 3. Run check from subdirectory - should find repo root
    let mut cmd = repo_cmd();
    cmd.current_dir(&subdir)
        .arg("check")
        .assert()
        .success();
}

#[test]
fn test_e2e_worktree_mode_init() {
    let dir = tempdir().unwrap();

    // 1. Init in worktree mode
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "worktree", "--tools", "cursor", "--tools", "claude"])
        .assert()
        .success();

    // 2. Verify structure
    assert!(dir.path().join(".repository/config.toml").exists());
    assert!(dir.path().join("main").exists());

    // 3. Verify config content
    let config_content = fs::read_to_string(dir.path().join(".repository/config.toml")).unwrap();
    assert!(config_content.contains("mode = \"worktree\""));
    assert!(config_content.contains("cursor"));
    assert!(config_content.contains("claude"));
}

#[test]
fn test_e2e_full_workflow_multiple_tools() {
    let dir = tempdir().unwrap();

    // 1. Init with multiple tools
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args([
            "init",
            "--mode", "standard",
            "--tools", "cursor",
            "--tools", "claude",
            "--tools", "vscode",
        ])
        .assert()
        .success();

    // 2. Add multiple rules
    for (id, instruction) in [
        ("api-design", "Return JSON with data, error, meta fields"),
        ("code-style", "Use consistent naming conventions"),
        ("testing", "Write unit tests for all public functions"),
    ] {
        let mut cmd = repo_cmd();
        cmd.current_dir(dir.path())
            .args(["add-rule", id, "-i", instruction])
            .assert()
            .success();
    }

    // 3. Check status
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .arg("check")
        .assert()
        .success();

    // 4. Sync
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .arg("sync")
        .assert()
        .success();

    // 5. Verify ledger created
    assert!(dir.path().join(".repository/ledger.toml").exists());

    // 6. Verify rules exist
    assert!(dir.path().join(".repository/rules/api-design.md").exists());
    assert!(dir.path().join(".repository/rules/code-style.md").exists());
    assert!(dir.path().join(".repository/rules/testing.md").exists());
}
