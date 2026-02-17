//! Integration tests for the repo CLI binary.
//!
//! These tests exercise the actual compiled binary using assert_cmd.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

/// Get a Command for the repo binary
fn repo_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("repo"))
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
    cmd.current_dir(dir.path()).arg("init").assert().success();

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
        .args([
            "init", "--mode", "standard", "--tools", "vscode", "--tools", "cursor",
        ])
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
        .args([
            "init",
            "--mode",
            "standard",
            "--presets",
            "typescript",
            "--presets",
            "react",
        ])
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
            "--mode",
            "standard",
            "--tools",
            "eslint",
            "--presets",
            "typescript",
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
fn test_sync_creates_ledger_with_valid_content() {
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
    cmd.current_dir(dir.path()).arg("sync").assert().success();

    // Ledger should now exist with valid TOML content
    let ledger_path = dir.path().join(".repository/ledger.toml");
    assert!(ledger_path.exists());

    let ledger_content = fs::read_to_string(&ledger_path).unwrap();
    // Ledger must be valid TOML
    let parsed: toml::Value = toml::from_str(&ledger_content)
        .expect("Ledger should be valid TOML");

    // Must contain version field
    let version = parsed.get("version").expect("Ledger must have 'version' field");
    assert_eq!(
        version.as_str().unwrap(), "1.0",
        "Ledger version should be 1.0"
    );

    // Must contain intents array (even if empty)
    let intents = parsed.get("intents").expect("Ledger must have 'intents' field");
    assert!(intents.is_array(), "Ledger 'intents' should be an array");
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
    cmd.current_dir(dir.path()).arg("sync").assert().success();

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
    let _ = cmd
        .current_dir(dir.path())
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
fn test_full_workflow_init_add_check_sync_verifies_content() {
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
        .args(["add-tool", "cursor"])
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
    cmd.current_dir(dir.path()).arg("check").assert().success();

    // 5. Sync
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path()).arg("sync").assert().success();

    // Verify config.toml content
    let config_content = fs::read_to_string(dir.path().join(".repository/config.toml")).unwrap();
    assert!(config_content.contains("cursor"), "Config should contain cursor tool");
    assert!(config_content.contains("typescript"), "Config should contain typescript preset");
    assert!(config_content.contains("[core]"), "Config should have [core] section");
    assert!(config_content.contains("mode = \"standard\""), "Config should have standard mode");

    // Verify ledger exists AND has valid TOML structure
    let ledger_path = dir.path().join(".repository/ledger.toml");
    assert!(ledger_path.exists(), "Ledger file must be created by sync");

    let ledger_content = fs::read_to_string(&ledger_path).unwrap();
    let ledger: toml::Value = toml::from_str(&ledger_content)
        .expect("Ledger must be valid TOML");
    assert_eq!(
        ledger.get("version").and_then(|v| v.as_str()),
        Some("1.0"),
        "Ledger version must be 1.0"
    );
    assert!(
        ledger.get("intents").unwrap().is_array(),
        "Ledger must have intents array"
    );
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
        .args([
            "add-rule",
            "test-rule",
            "--instruction",
            "Test instruction for testing",
        ])
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
            "--instruction",
            "Use snake_case for variables",
            "-t",
            "python",
            "-t",
            "style",
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
    assert!(
        initial_config.contains("\"cursor\""),
        "Initial config should contain cursor"
    );

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
    cmd.current_dir(&subdir).arg("check").assert().success();
}

#[test]
fn test_e2e_worktree_mode_init() {
    let dir = tempdir().unwrap();

    // 1. Init in worktree mode
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args([
            "init", "--mode", "worktree", "--tools", "cursor", "--tools", "claude",
        ])
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
            "init", "--mode", "standard", "--tools", "cursor", "--tools", "claude", "--tools",
            "vscode",
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
    cmd.current_dir(dir.path()).arg("check").assert().success();

    // 4. Sync
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path()).arg("sync").assert().success();

    // 5. Verify ledger has valid structure and tool intents
    let ledger_path = dir.path().join(".repository/ledger.toml");
    assert!(ledger_path.exists(), "Ledger must be created by sync");
    let ledger_content = fs::read_to_string(&ledger_path).unwrap();
    let ledger: toml::Value = toml::from_str(&ledger_content)
        .expect("Ledger must be valid TOML");
    assert_eq!(ledger.get("version").and_then(|v| v.as_str()), Some("1.0"));
    let _intents = ledger.get("intents").unwrap().as_array().unwrap();
    // With 3 tools configured, we expect tool intents in the ledger
    // (exact count depends on which tools have integrations)
    // At minimum the ledger structure should be parseable
    assert!(
        !ledger_content.is_empty(),
        "Ledger content should not be empty after sync with tools"
    );

    // 6. Verify rules exist with correct CONTENT
    let api_rule = fs::read_to_string(dir.path().join(".repository/rules/api-design.md")).unwrap();
    assert!(
        api_rule.contains("Return JSON with data, error, meta fields"),
        "Rule file should contain the instruction text. Got: {}",
        api_rule
    );
    let code_rule = fs::read_to_string(dir.path().join(".repository/rules/code-style.md")).unwrap();
    assert!(
        code_rule.contains("Use consistent naming conventions"),
        "Rule file should contain the instruction text. Got: {}",
        code_rule
    );
    let test_rule = fs::read_to_string(dir.path().join(".repository/rules/testing.md")).unwrap();
    assert!(
        test_rule.contains("Write unit tests for all public functions"),
        "Rule file should contain the instruction text. Got: {}",
        test_rule
    );
}

// =============================================================================
// Content verification tests (C9, S5, C4-CLI)
// =============================================================================

#[test]
fn test_sync_with_cursor_tool_creates_config_with_content() {
    // S5: init -> add-tool -> sync -> verify config file CONTENT
    let dir = tempdir().unwrap();

    // Init with cursor tool
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard", "--tools", "cursor"])
        .assert()
        .success();

    // Sync to generate config files
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .arg("sync")
        .assert()
        .success();

    // Verify ledger tracks the tool intent with projections
    let ledger_content = fs::read_to_string(dir.path().join(".repository/ledger.toml")).unwrap();
    let ledger: toml::Value = toml::from_str(&ledger_content)
        .expect("Ledger must be valid TOML");

    let intents = ledger.get("intents").unwrap().as_array().unwrap();

    // Find the cursor tool intent
    let cursor_intent = intents.iter().find(|i| {
        i.get("id")
            .and_then(|id| id.as_str())
            .map(|id| id.contains("cursor"))
            .unwrap_or(false)
    });

    let intent = cursor_intent.expect("Sync should create a cursor tool intent in the ledger");

    // Verify it created projections
    let projections = intent.get("projections");
    assert!(
        projections.is_some(),
        "Cursor intent should have projections field"
    );

    // Each projection should have tool, file, and kind fields
    let projs = projections.unwrap().as_array()
        .expect("Projections should be an array");
    assert!(!projs.is_empty(), "Cursor intent must have at least one projection");
    for proj in projs {
        assert!(
            proj.get("tool").is_some(),
            "Projection must have 'tool' field"
        );
        assert!(
            proj.get("file").is_some(),
            "Projection must have 'file' field"
        );
    }
}

#[test]
fn test_sync_json_output_contains_structured_data() {
    // C4-CLI: Verify JSON output mode produces parseable, structured data
    let dir = tempdir().unwrap();

    // Init with a tool
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard", "--tools", "cursor"])
        .assert()
        .success();

    // Sync with JSON output
    let output = repo_cmd()
        .current_dir(dir.path())
        .args(["sync", "--json"])
        .output()
        .expect("Failed to execute sync --json");

    assert!(output.status.success(), "sync --json should succeed");

    let stdout = String::from_utf8(output.stdout).unwrap();
    // Must be valid JSON
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("sync --json output must be valid JSON: {}. Got: {}", e, stdout));

    // Must have required fields
    assert!(json.get("dry_run").is_some(), "JSON output must have 'dry_run' field");
    assert!(json.get("success").is_some(), "JSON output must have 'success' field");
    assert!(json.get("has_changes").is_some(), "JSON output must have 'has_changes' field");
    assert!(json.get("changes").is_some(), "JSON output must have 'changes' field");
    assert!(json.get("root").is_some(), "JSON output must have 'root' field");
    assert!(json.get("mode").is_some(), "JSON output must have 'mode' field");

    // dry_run should be false
    assert_eq!(json["dry_run"], false);
    // success should be true
    assert_eq!(json["success"], true);
    // mode should be "standard"
    assert_eq!(json["mode"], "standard");
    // changes should be an array
    assert!(json["changes"].is_array(), "'changes' should be an array");
}

#[test]
fn test_sync_dry_run_json_does_not_modify_filesystem() {
    let dir = tempdir().unwrap();

    // Init with tool
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard", "--tools", "cursor"])
        .assert()
        .success();

    // Sync with dry-run + JSON
    let output = repo_cmd()
        .current_dir(dir.path())
        .args(["sync", "--dry-run", "--json"])
        .output()
        .expect("Failed to execute sync --dry-run --json");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("dry-run JSON output must be valid JSON");

    assert_eq!(json["dry_run"], true, "dry_run flag should be true");

    // In dry-run mode, the ledger should NOT be created
    // (no files modified on disk)
    let ledger_path = dir.path().join(".repository/ledger.toml");
    assert!(
        !ledger_path.exists(),
        "Dry-run sync should not create the ledger file"
    );
}

#[test]
fn test_sync_idempotent_ledger_content_unchanged() {
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

    let ledger_after_first = fs::read_to_string(dir.path().join(".repository/ledger.toml")).unwrap();

    // Second sync
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .arg("sync")
        .assert()
        .success();

    let ledger_after_second = fs::read_to_string(dir.path().join(".repository/ledger.toml")).unwrap();

    // Ledger content should be identical after idempotent sync
    assert_eq!(
        ledger_after_first, ledger_after_second,
        "Ledger content should not change on idempotent sync"
    );
}

#[test]
fn test_check_without_init_gives_meaningful_error() {
    let dir = tempdir().unwrap();

    // Running check without any init should give a clear error message
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .arg("check")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not in a repository"));
}

#[test]
fn test_sync_without_init_gives_meaningful_error() {
    let dir = tempdir().unwrap();

    // Running sync without any init should give a clear error message
    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .arg("sync")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not in a repository"));
}

#[test]
fn test_init_config_toml_is_valid_toml() {
    // C4-CLI: Verify init produces valid, parseable config - not just string contains
    let dir = tempdir().unwrap();

    let mut cmd = repo_cmd();
    cmd.current_dir(dir.path())
        .args(["init", "--mode", "standard", "--tools", "cursor", "--tools", "claude", "--presets", "typescript"])
        .assert()
        .success();

    let config_content = fs::read_to_string(dir.path().join(".repository/config.toml")).unwrap();

    // Must be valid TOML
    let parsed: toml::Value = toml::from_str(&config_content)
        .unwrap_or_else(|e| panic!("Config must be valid TOML: {}. Got:\n{}", e, config_content));

    // Verify tools array contains exact tool names
    let tools = parsed.get("tools").expect("Config must have 'tools' field");
    let tools_arr = tools.as_array().expect("'tools' should be an array");
    let tool_names: Vec<&str> = tools_arr.iter().map(|t| t.as_str().unwrap()).collect();
    assert!(tool_names.contains(&"cursor"), "Tools should contain 'cursor'");
    assert!(tool_names.contains(&"claude"), "Tools should contain 'claude'");

    // Verify core.mode
    let core = parsed.get("core").expect("Config must have 'core' section");
    assert_eq!(
        core.get("mode").and_then(|m| m.as_str()),
        Some("standard"),
        "core.mode should be 'standard'"
    );

    // Verify presets section exists
    let presets = parsed.get("presets").expect("Config must have 'presets' section");
    assert!(
        presets.get("typescript").is_some(),
        "Presets should contain 'typescript'"
    );
}
