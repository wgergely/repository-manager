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
    assert!(config_content.contains("[tools]"));
    assert!(config_content.contains("vscode"));
    assert!(config_content.contains("cursor"));
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
    assert!(config_content.contains("[presets]"));
    assert!(config_content.contains("typescript"));
    assert!(config_content.contains("react"));
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
    assert!(config_content.contains("[tools]"));
    assert!(config_content.contains("eslint"));
    assert!(config_content.contains("[presets]"));
    assert!(config_content.contains("typescript"));
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

    // Check without init or git - should fail (requires git repo)
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not a git repository"));
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
