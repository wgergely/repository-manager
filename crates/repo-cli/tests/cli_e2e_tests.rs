//! CLI end-to-end tests that invoke the compiled `repo` binary.
//!
//! These tests use `env!("CARGO_BIN_EXE_repo")` to locate the binary and
//! `std::process::Command` to run it against temporary directories.

use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Returns the path to the compiled `repo` binary.
fn repo_bin() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_repo"))
}

/// Run `repo` with the given args in the given directory.
fn run(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(repo_bin())
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to execute repo binary")
}

/// Initialise a real git repo at `path` using the `git` CLI.
fn git_init(path: &std::path::Path) {
    let status = Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(path)
        .output()
        .expect("failed to run git init");
    if !status.status.success() {
        // Older git may not support -b; fall back to plain init + rename
        Command::new("git")
            .args(["init"])
            .current_dir(path)
            .status()
            .expect("git init failed");
    }
    // Configure minimal user identity so git operations inside the test don't fail
    for (k, v) in [
        ("user.email", "test@example.com"),
        ("user.name", "Test"),
        ("commit.gpgsign", "false"),
    ] {
        Command::new("git")
            .args(["config", k, v])
            .current_dir(path)
            .status()
            .ok();
    }
}

// ============================================================================
// 1. test_help_exits_zero
// ============================================================================

#[test]
fn test_help_exits_zero() {
    let out = Command::new(repo_bin())
        .arg("--help")
        .output()
        .expect("failed to run repo --help");

    assert!(out.status.success(), "repo --help should exit 0");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("sync"),
        "help output should mention 'sync', got:\n{}",
        stdout
    );
}

// ============================================================================
// 2. test_version_flag
// ============================================================================

#[test]
fn test_version_flag() {
    let out = Command::new(repo_bin())
        .arg("--version")
        .output()
        .expect("failed to run repo --version");

    assert!(out.status.success(), "repo --version should exit 0");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("repo"),
        "--version output should contain 'repo', got:\n{}",
        stdout
    );
}

// ============================================================================
// 3. test_sync_requires_repository
// ============================================================================

#[test]
fn test_sync_requires_repository() {
    // A plain temp dir with no git repo / .repository dir
    let dir = TempDir::new().unwrap();

    let out = run(dir.path(), &["sync"]);

    assert!(
        !out.status.success(),
        "repo sync in non-repo dir should exit non-zero"
    );

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.is_empty() || !String::from_utf8_lossy(&out.stdout).is_empty(),
        "should produce a useful error message"
    );
}

// ============================================================================
// 4. test_init_creates_repository_dir
// ============================================================================

#[test]
fn test_init_creates_repository_dir() {
    let dir = TempDir::new().unwrap();

    let out = run(dir.path(), &["init", "--mode", "standard"]);
    assert!(
        out.status.success(),
        "repo init should succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    assert!(
        dir.path().join(".repository").exists(),
        ".repository directory should be created"
    );
    assert!(
        dir.path().join(".repository/config.toml").exists(),
        ".repository/config.toml should be created"
    );

    let config = fs::read_to_string(dir.path().join(".repository/config.toml")).unwrap();
    assert!(
        config.contains("[core]"),
        "config.toml should contain [core] section"
    );
    assert!(
        config.contains("mode = \"standard\""),
        "config.toml should contain mode = \"standard\""
    );
}

// ============================================================================
// 5. test_init_sync_creates_tool_configs
// ============================================================================

#[test]
fn test_init_sync_creates_tool_configs() {
    let dir = TempDir::new().unwrap();

    // Init with cursor and claude tools
    let out = run(
        dir.path(),
        &[
            "init", "--mode", "standard", "--tools", "cursor", "--tools", "claude",
        ],
    );
    assert!(
        out.status.success(),
        "repo init should succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Run sync to generate tool config files
    let out = run(dir.path(), &["sync"]);
    assert!(
        out.status.success(),
        "repo sync should succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // .cursorrules must exist after syncing with cursor tool
    assert!(
        dir.path().join(".cursorrules").exists(),
        ".cursorrules should be created by sync with cursor tool"
    );

    // CLAUDE.md must exist after syncing with claude tool
    assert!(
        dir.path().join("CLAUDE.md").exists(),
        "CLAUDE.md should be created by sync with claude tool"
    );
}

// ============================================================================
// 6. test_add_tool_then_sync
// ============================================================================

#[test]
fn test_add_tool_then_sync() {
    let dir = TempDir::new().unwrap();

    // Init without tools
    let out = run(dir.path(), &["init", "--mode", "standard"]);
    assert!(
        out.status.success(),
        "repo init should succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Add cursor tool
    let out = run(dir.path(), &["add-tool", "cursor"]);
    assert!(
        out.status.success(),
        "repo add-tool cursor should succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Sync
    let out = run(dir.path(), &["sync"]);
    assert!(
        out.status.success(),
        "repo sync should succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // .cursorrules must now exist
    assert!(
        dir.path().join(".cursorrules").exists(),
        ".cursorrules should be created after add-tool cursor + sync"
    );
}

// ============================================================================
// 7. test_list_tools_shows_known_tools
// ============================================================================

#[test]
fn test_list_tools_shows_known_tools() {
    let out = Command::new(repo_bin())
        .arg("list-tools")
        .output()
        .expect("failed to run repo list-tools");

    assert!(
        out.status.success(),
        "repo list-tools should exit 0; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("cursor"),
        "list-tools output should contain 'cursor', got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("claude"),
        "list-tools output should contain 'claude', got:\n{}",
        stdout
    );
}

// ============================================================================
// Additional: git-initialised repo
// ============================================================================

#[test]
fn test_init_in_git_repo_succeeds() {
    let dir = TempDir::new().unwrap();
    git_init(dir.path());

    let out = run(dir.path(), &["init", "--mode", "standard"]);
    assert!(
        out.status.success(),
        "repo init in a real git repo should succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    assert!(dir.path().join(".repository/config.toml").exists());
}

#[test]
fn test_sync_in_git_repo_creates_ledger() {
    let dir = TempDir::new().unwrap();
    git_init(dir.path());

    run(dir.path(), &["init", "--mode", "standard"]);

    let out = run(dir.path(), &["sync"]);
    assert!(
        out.status.success(),
        "repo sync in real git repo should succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    assert!(
        dir.path().join(".repository/ledger.toml").exists(),
        "sync should create .repository/ledger.toml"
    );
}
