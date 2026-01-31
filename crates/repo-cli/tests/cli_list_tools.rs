//! Integration tests for list-tools and list-presets commands

use assert_cmd::Command;
use predicates::prelude::*;

/// Get a Command for the repo binary
fn repo_cmd() -> Command {
    Command::cargo_bin("repo").expect("Failed to find repo binary")
}

// ============================================================================
// list-tools Command Tests
// ============================================================================

#[test]
fn test_list_tools_shows_output() {
    let mut cmd = repo_cmd();
    cmd.arg("list-tools")
        .assert()
        .success()
        .stdout(predicate::str::contains("Available Tools"))
        .stdout(predicate::str::contains("claude"))
        .stdout(predicate::str::contains("cursor"));
}

#[test]
fn test_list_tools_with_ide_filter() {
    let mut cmd = repo_cmd();
    cmd.args(["list-tools", "--category", "ide"])
        .assert()
        .success()
        .stdout(predicate::str::contains("IDE Tools"))
        .stdout(predicate::str::contains("vscode"));
}

#[test]
fn test_list_tools_with_cli_agent_filter() {
    let mut cmd = repo_cmd();
    cmd.args(["list-tools", "--category", "cli-agent"])
        .assert()
        .success()
        .stdout(predicate::str::contains("CLI Agents"))
        .stdout(predicate::str::contains("claude"));
}

#[test]
fn test_list_tools_with_copilot_filter() {
    let mut cmd = repo_cmd();
    cmd.args(["list-tools", "--category", "copilot"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Copilots"));
}

#[test]
fn test_list_tools_shows_total_count() {
    let mut cmd = repo_cmd();
    cmd.arg("list-tools")
        .assert()
        .success()
        .stdout(predicate::str::contains("Total:"))
        .stdout(predicate::str::contains("tools available"));
}

#[test]
fn test_list_tools_with_unknown_category_warns() {
    let mut cmd = repo_cmd();
    cmd.args(["list-tools", "--category", "unknown"])
        .assert()
        .success()
        .stderr(predicate::str::contains("warning"))
        .stderr(predicate::str::contains("unknown"));
}

#[test]
fn test_list_tools_help() {
    let mut cmd = repo_cmd();
    cmd.args(["list-tools", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("List"))
        .stdout(predicate::str::contains("tool"));
}

// ============================================================================
// list-presets Command Tests
// ============================================================================

#[test]
fn test_list_presets_shows_output() {
    let mut cmd = repo_cmd();
    cmd.arg("list-presets")
        .assert()
        .success()
        .stdout(predicate::str::contains("Available Presets"))
        .stdout(predicate::str::contains("env:python"));
}

#[test]
fn test_list_presets_shows_multiple_presets() {
    let mut cmd = repo_cmd();
    cmd.arg("list-presets")
        .assert()
        .success()
        .stdout(predicate::str::contains("env:node"))
        .stdout(predicate::str::contains("env:rust"));
}

#[test]
fn test_list_presets_shows_total_count() {
    let mut cmd = repo_cmd();
    cmd.arg("list-presets")
        .assert()
        .success()
        .stdout(predicate::str::contains("Total:"))
        .stdout(predicate::str::contains("presets available"));
}

#[test]
fn test_list_presets_shows_provider() {
    let mut cmd = repo_cmd();
    cmd.arg("list-presets")
        .assert()
        .success()
        .stdout(predicate::str::contains("provider:"));
}

#[test]
fn test_list_presets_help() {
    let mut cmd = repo_cmd();
    cmd.args(["list-presets", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("List"))
        .stdout(predicate::str::contains("preset"));
}
