//! Git repository fixtures at three realism levels.
//!
//! Choose the lowest-realism fixture that satisfies your test's needs —
//! fakes are faster and have fewer external dependencies.

use std::fs;
use std::path::Path;
use std::process::Command;

/// Creates a minimal `.git` directory structure **without** initialising a real
/// git repository.
///
/// Realism level: **FAKE** — directory structure only, no git object store.
///
/// Use for: tests that need a `.git` marker to satisfy path detection logic but
/// do not perform any real git operations (branch listing, commits, etc.).
///
/// # Panics
/// Panics if the filesystem operations fail.
pub fn fake_git_dir(path: &Path) {
    fs::create_dir(path.join(".git"))
        .unwrap_or_else(|e| panic!("fake_git_dir: failed to create .git: {e}"));
    fs::write(path.join(".git/HEAD"), "ref: refs/heads/main\n")
        .unwrap_or_else(|e| panic!("fake_git_dir: failed to write HEAD: {e}"));
    fs::create_dir_all(path.join(".git/refs/heads"))
        .unwrap_or_else(|e| panic!("fake_git_dir: failed to create refs/heads: {e}"));
    fs::write(path.join(".git/refs/heads/main"), "")
        .unwrap_or_else(|e| panic!("fake_git_dir: failed to write refs/heads/main: {e}"));
}

/// Initialises a real git repository using `git2` (no initial commit, no config).
///
/// Realism level: **REAL** — valid git object store, empty history.
///
/// Use for: tests that need `git2::Repository` state but do not need a commit
/// history or branch configuration.
///
/// # Panics
/// Panics if `git2::Repository::init` fails.
pub fn real_git_repo(path: &Path) -> git2::Repository {
    git2::Repository::init(path).unwrap_or_else(|e| {
        panic!(
            "real_git_repo: failed to init repository at {}: {e}",
            path.display()
        )
    })
}

/// Initialises a real git repository with an initial commit using the `git` CLI.
///
/// Realism level: **REAL WITH HISTORY** — valid git state, `main` branch, one
/// commit in history.
///
/// Specifically:
/// - Runs `git init`
/// - Configures `user.email`, `user.name`, and `commit.gpgsign = false`
/// - Creates `README.md` and makes an initial commit
/// - Renames the default branch to `main`
///
/// Use for: CLI tests that need a real branch history (branch listing, checkout,
/// rename, etc.).
///
/// # Panics
/// Panics if any git operation fails.
pub fn real_git_repo_with_commit(path: &Path) {
    let run = |args: &[&str]| {
        let output = Command::new("git")
            .args(args)
            .current_dir(path)
            .output()
            .unwrap_or_else(|e| {
                panic!("real_git_repo_with_commit: failed to run `git {args:?}`: {e}")
            });
        if !output.status.success() {
            panic!(
                "real_git_repo_with_commit: `git {args:?}` failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    };

    run(&["init"]);
    run(&["config", "user.email", "test@test.com"]);
    run(&["config", "user.name", "Test User"]);
    run(&["config", "commit.gpgsign", "false"]);

    fs::write(path.join("README.md"), "# Test")
        .unwrap_or_else(|e| panic!("real_git_repo_with_commit: failed to write README.md: {e}"));

    run(&["add", "."]);
    run(&["commit", "-m", "Initial commit"]);
    // Best-effort: older git versions may not support this flag
    let _ = Command::new("git")
        .args(["branch", "-m", "main"])
        .current_dir(path)
        .output();
}
