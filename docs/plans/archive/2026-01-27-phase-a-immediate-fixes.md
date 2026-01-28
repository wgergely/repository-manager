# Phase A: Immediate Fixes Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Date:** 2026-01-27
**Priority:** Critical
**Estimated Tasks:** 6
**Dependencies:** None (can start immediately)

---

## Goal

Address critical security vulnerabilities and robustness issues identified in code audits before proceeding with feature development.

---

## Prerequisites

- All tests passing: `cargo test --workspace`
- No clippy warnings: `cargo clippy --workspace -- -D warnings`

---

## Task A.1: Fix Symlink Vulnerability in repo-fs

**Files:**
- Modify: `crates/repo-fs/src/io.rs`
- Modify: `crates/repo-fs/src/error.rs`
- Modify: `crates/repo-fs/tests/security_tests.rs`

**Step 1: Add error variant**

In `crates/repo-fs/src/error.rs`, add:

```rust
#[error("Refusing to write through symlink: {path}")]
SymlinkInPath { path: PathBuf },
```

**Step 2: Implement symlink detection**

In `crates/repo-fs/src/io.rs`, add function:

```rust
/// Check if any component of the path is a symlink
fn contains_symlink(path: &Path) -> std::io::Result<bool> {
    let mut current = path.to_path_buf();
    while current.parent().is_some() {
        if current.exists() {
            let metadata = std::fs::symlink_metadata(&current)?;
            if metadata.file_type().is_symlink() {
                return Ok(true);
            }
        }
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }
    Ok(false)
}
```

**Step 3: Add check to write_atomic**

At the start of `write_atomic` function, add:

```rust
if contains_symlink(path)? {
    return Err(Error::SymlinkInPath { path: path.to_path_buf() });
}
```

**Step 4: Update tests**

Update `security_tests.rs` to verify symlink writes are rejected.

**Step 5: Run tests**

```bash
cargo test -p repo-fs security
```

**Step 6: Commit**

```bash
git add crates/repo-fs/
git commit -m "fix(repo-fs): reject writes through symlinks

Adds contains_symlink() check before write_atomic to prevent
symlink-based path traversal attacks.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task A.2: Add Error Injection Tests

**Files:**
- Create: `crates/repo-fs/tests/error_condition_tests.rs`

**Step 1: Create test file**

```rust
//! Tests for error handling under adverse conditions

use repo_fs::{io, Error};
use std::fs;
use tempfile::tempdir;

#[test]
#[cfg(unix)]
fn test_write_atomic_permission_denied_directory() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir().unwrap();
    let readonly_dir = dir.path().join("readonly");
    fs::create_dir(&readonly_dir).unwrap();

    // Make directory read-only
    fs::set_permissions(&readonly_dir, fs::Permissions::from_mode(0o444)).unwrap();

    let file_path = readonly_dir.join("test.txt");
    let result = io::write_atomic(&file_path, "content");

    // Restore permissions for cleanup
    fs::set_permissions(&readonly_dir, fs::Permissions::from_mode(0o755)).unwrap();

    assert!(result.is_err());
}

#[test]
fn test_read_text_nonexistent_file() {
    let result = io::read_text(std::path::Path::new("/nonexistent/path/file.txt"));
    assert!(result.is_err());
}

#[test]
#[cfg(unix)]
fn test_read_text_permission_denied() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("unreadable.txt");
    fs::write(&file_path, "secret").unwrap();

    // Make file unreadable
    fs::set_permissions(&file_path, fs::Permissions::from_mode(0o000)).unwrap();

    let result = io::read_text(&file_path);

    // Restore permissions for cleanup
    fs::set_permissions(&file_path, fs::Permissions::from_mode(0o644)).unwrap();

    assert!(result.is_err());
}
```

**Step 2: Run tests**

```bash
cargo test -p repo-fs error_condition
```

**Step 3: Commit**

```bash
git add crates/repo-fs/tests/
git commit -m "test(repo-fs): add error injection tests

Tests write_atomic and read_text under permission denied
and nonexistent file conditions.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task A.3: Add Concurrent Access Tests

**Files:**
- Create: `crates/repo-fs/tests/concurrency_tests.rs` (if not exists, extend)

**Step 1: Add concurrent write test**

```rust
use repo_fs::io;
use std::sync::{Arc, Barrier};
use std::thread;
use tempfile::tempdir;

#[test]
fn test_concurrent_writes_no_corruption() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("concurrent.txt");
    let barrier = Arc::new(Barrier::new(10));

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let path = file_path.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                for j in 0..20 {
                    let content = format!("Thread {} write {}\n", i, j);
                    let _ = io::write_atomic(&path, &content);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // File should exist and be valid (one of the writes should have won)
    let content = std::fs::read_to_string(&file_path).unwrap();
    assert!(content.starts_with("Thread "));
    assert!(content.contains("write"));
}

#[test]
fn test_concurrent_writes_different_files_all_succeed() {
    let dir = tempdir().unwrap();
    let barrier = Arc::new(Barrier::new(10));

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let dir_path = dir.path().to_path_buf();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                let file_path = dir_path.join(format!("file_{}.txt", i));
                io::write_atomic(&file_path, &format!("Content {}", i)).unwrap();
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // All files should exist
    for i in 0..10 {
        let file_path = dir.path().join(format!("file_{}.txt", i));
        assert!(file_path.exists(), "File {} should exist", i);
    }
}
```

**Step 2: Run tests**

```bash
cargo test -p repo-fs concurrency
```

**Step 3: Commit**

```bash
git add crates/repo-fs/tests/
git commit -m "test(repo-fs): add concurrent access tests

Verifies write_atomic handles concurrent writes without
file corruption and multiple simultaneous file operations.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task A.4: Add Tool/Preset Validation Registry

**Files:**
- Create: `crates/repo-meta/src/validation.rs`
- Modify: `crates/repo-meta/src/lib.rs`

**Step 1: Create validation module**

```rust
//! Validation for tool and preset names

use std::collections::HashSet;

/// Registry of known tools for validation
pub struct ToolRegistry {
    known: HashSet<&'static str>,
}

impl ToolRegistry {
    pub fn with_builtins() -> Self {
        let known = [
            "claude", "claude-desktop", "cursor", "vscode",
            "windsurf", "gemini-cli", "antigravity", "zed",
        ].into_iter().collect();
        Self { known }
    }

    pub fn is_known(&self, name: &str) -> bool {
        self.known.contains(name)
    }

    pub fn list_known(&self) -> Vec<&'static str> {
        self.known.iter().copied().collect()
    }
}

/// Registry of known presets for validation
pub struct PresetRegistry {
    known: HashSet<&'static str>,
}

impl PresetRegistry {
    pub fn with_builtins() -> Self {
        let known = [
            "python", "python-uv", "python-conda",
            "node", "rust", "web",
        ].into_iter().collect();
        Self { known }
    }

    pub fn is_known(&self, name: &str) -> bool {
        self.known.contains(name)
    }

    pub fn list_known(&self) -> Vec<&'static str> {
        self.known.iter().copied().collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::with_builtins()
    }
}

impl Default for PresetRegistry {
    fn default() -> Self {
        Self::with_builtins()
    }
}
```

**Step 2: Export from lib.rs**

Add to `crates/repo-meta/src/lib.rs`:

```rust
pub mod validation;
pub use validation::{ToolRegistry, PresetRegistry};
```

**Step 3: Run tests**

```bash
cargo test -p repo-meta
```

**Step 4: Commit**

```bash
git add crates/repo-meta/
git commit -m "feat(repo-meta): add tool/preset validation registry

Adds ToolRegistry and PresetRegistry with built-in known
values for validation warnings in CLI.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task A.5: Add CLI Validation Warnings

**Files:**
- Modify: `crates/repo-cli/src/commands/tool.rs`

**Step 1: Add validation warning**

In the tool add command handler, add:

```rust
use repo_meta::ToolRegistry;
use colored::Colorize;

// At start of add function:
let registry = ToolRegistry::with_builtins();
if !registry.is_known(&name) {
    eprintln!(
        "{}: '{}' is not a recognized tool. Known tools: {}",
        "Warning".yellow(),
        name,
        registry.list_known().join(", ")
    );
    // Continue execution - don't block
}
```

**Step 2: Run CLI tests**

```bash
cargo test -p repo-cli
```

**Step 3: Commit**

```bash
git add crates/repo-cli/
git commit -m "feat(repo-cli): add validation warnings for unknown tools

Shows yellow warning when adding unrecognized tool names
but continues execution.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task A.6: Fix WorkspaceLayout Validation Bug

**Files:**
- Modify: `crates/repo-fs/src/layout.rs`
- Modify: `crates/repo-fs/tests/layout_tests.rs`

**Step 1: Fix validation**

In `WorkspaceLayout::validate()`, change `exists()` to `is_dir()`:

```rust
// Before:
if !self.root.join(".git").exists() { ... }

// After:
if !self.root.join(".git").is_dir() { ... }
```

Apply same change for `.gt`, `main/`, `.worktrees/` checks.

**Step 2: Add test for file vs directory**

```rust
#[test]
fn test_validate_rejects_file_as_git_dir() {
    let dir = tempdir().unwrap();
    // Create .git as a FILE, not directory
    std::fs::write(dir.path().join(".git"), "not a directory").unwrap();

    let layout = WorkspaceLayout::detect_at(dir.path());
    // Should fail or detect as unknown
    assert!(layout.is_err() || matches!(layout, Ok(WorkspaceLayout::Unknown)));
}
```

**Step 3: Run tests**

```bash
cargo test -p repo-fs layout
```

**Step 4: Commit**

```bash
git add crates/repo-fs/
git commit -m "fix(repo-fs): validate layout dirs are directories

Changes exists() to is_dir() in WorkspaceLayout::validate()
to reject files masquerading as required directories.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Verification

After completing all tasks:

```bash
# Run full test suite
cargo test --workspace

# Run clippy
cargo clippy --workspace -- -D warnings

# Verify documentation
cargo doc --workspace --no-deps
```

All should pass with no warnings.

---

## Summary

| Task | Description | Risk | Effort |
|------|-------------|------|--------|
| A.1 | Symlink vulnerability fix | High | Low |
| A.2 | Error injection tests | Low | Low |
| A.3 | Concurrency tests | Low | Low |
| A.4 | Validation registry | Low | Low |
| A.5 | CLI validation warnings | Low | Low |
| A.6 | Layout validation bug | Medium | Low |

**Total Effort:** ~2-3 hours of focused work
