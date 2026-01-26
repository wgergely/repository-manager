# Immediate & Short-Term Fixes Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Address six audit findings to improve robustness, security, test coverage, and spec compliance.

**Architecture:** Changes span four crates (repo-fs, repo-core, repo-meta, repo-cli). Each task is independent and can be committed separately. Tests use real filesystem operations with no mocking.

**Tech Stack:** Rust 2024 edition, clap, thiserror, tempfile, fs2, colored

---

## Task 1: Add SymlinkInPath Error Variant

**Files:**
- Modify: `crates/repo-fs/src/error.rs`

**Step 1: Add the new error variant**

Open `crates/repo-fs/src/error.rs` and add after `LockFailed`:

```rust
#[error("Refusing to write through symlink: {path}")]
SymlinkInPath { path: PathBuf },
```

**Step 2: Run existing tests to verify no breakage**

```bash
cd main && cargo test -p repo-fs
```

Expected: All existing tests pass.

**Step 3: Commit**

```bash
git add crates/repo-fs/src/error.rs
git commit -m "feat(repo-fs): add SymlinkInPath error variant

Prepares for symlink vulnerability fix by adding a dedicated error
type for when a write operation is rejected due to symlinks in path."
```

---

## Task 2: Implement Symlink Detection in write_atomic

**Files:**
- Modify: `crates/repo-fs/src/io.rs`

**Step 1: Add the contains_symlink helper function**

Add this function before `write_atomic` in `crates/repo-fs/src/io.rs`:

```rust
/// Check if any component in the path (or its ancestors) is a symlink.
///
/// This prevents symlink-based attacks where writes could escape intended directories.
fn contains_symlink(path: &std::path::Path) -> std::io::Result<bool> {
    use std::path::PathBuf;

    let mut current = PathBuf::from(path);

    // Walk up the path checking each component
    loop {
        if current.exists() {
            let metadata = std::fs::symlink_metadata(&current)?;
            if metadata.file_type().is_symlink() {
                return Ok(true);
            }
        }

        match current.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => {
                current = parent.to_path_buf();
            }
            _ => break,
        }
    }

    Ok(false)
}
```

**Step 2: Add symlink check at the start of write_atomic**

In `write_atomic`, after the `tracing::debug!` line and before `// Ensure parent directory exists`, add:

```rust
    // Security: Reject paths containing symlinks to prevent escape attacks
    if contains_symlink(&native_path).unwrap_or(false) {
        return Err(Error::SymlinkInPath {
            path: native_path.clone(),
        });
    }
```

**Step 3: Run tests**

```bash
cd main && cargo test -p repo-fs
```

Expected: All tests pass (existing tests don't use symlinks).

**Step 4: Commit**

```bash
git add crates/repo-fs/src/io.rs
git commit -m "fix(repo-fs): reject symlinks in write_atomic path

Adds security check to prevent writes through symlinks, which could
allow file writes to escape intended directories. The contains_symlink
helper walks up the path checking each component.

Fixes documented vulnerability in security_audit_tests.rs."
```

---

## Task 3: Update Security Audit Tests

**Files:**
- Modify: `crates/repo-fs/tests/security_audit_tests.rs`

**Step 1: Update test to expect failure instead of documenting vulnerability**

Replace the `test_write_atomic_does_not_follow_symlink_in_path` test:

```rust
#[test]
fn test_write_atomic_rejects_symlink_in_path() {
    let jail = setup_jail();
    let jail_path = jail.path();

    // Create a directory inside the jail and a symlink pointing to it
    let secret_dir_path = jail_path.join("secret_dir");
    fs::create_dir(&secret_dir_path).unwrap();
    let symlink_path = jail_path.join("symlink_dir");
    std::os::unix::fs::symlink(&secret_dir_path, &symlink_path).unwrap();

    // Attempt to write a file inside the symlinked directory
    let path_with_symlink = NormalizedPath::new(symlink_path.join("file.txt"));
    let content = "content";

    let result = io::write_text(&path_with_symlink, content);

    // Should now FAIL with SymlinkInPath error
    assert!(result.is_err(), "Write through symlink should be rejected");

    let err = result.unwrap_err();
    let err_str = format!("{}", err);
    assert!(
        err_str.contains("symlink"),
        "Error should mention symlink, got: {}",
        err_str
    );

    // Verify no file was created
    let secret_file_path = secret_dir_path.join("file.txt");
    assert!(
        !secret_file_path.exists(),
        "File should NOT have been written through symlink"
    );
}
```

**Step 2: Run the security tests**

```bash
cd main && cargo test -p repo-fs --test security_audit_tests
```

Expected: Tests pass with new behavior.

**Step 3: Commit**

```bash
git add crates/repo-fs/tests/security_audit_tests.rs
git commit -m "test(repo-fs): update security tests for symlink rejection

Updates security_audit_tests to verify that write_atomic now correctly
rejects writes through symlinks rather than documenting the vulnerability."
```

---

## Task 4: Add Error Condition Tests

**Files:**
- Create: `crates/repo-fs/tests/error_condition_tests.rs`

**Step 1: Create the new test file**

Create `crates/repo-fs/tests/error_condition_tests.rs`:

```rust
//! Tests for error handling under adverse filesystem conditions
//!
//! These tests verify that repo-fs handles real error conditions gracefully.

use repo_fs::{io, NormalizedPath};
use tempfile::tempdir;

#[test]
fn test_read_text_nonexistent_file() {
    let dir = tempdir().unwrap();
    let path = NormalizedPath::new(dir.path().join("does_not_exist.txt"));

    let result = io::read_text(&path);

    assert!(result.is_err(), "Reading non-existent file should fail");
}

#[cfg(unix)]
mod unix_tests {
    use super::*;
    use std::fs::{self, Permissions};
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn test_write_atomic_permission_denied_directory() {
        let dir = tempdir().unwrap();
        let readonly_dir = dir.path().join("readonly");
        fs::create_dir(&readonly_dir).unwrap();

        // Make directory read-only (no write permission)
        fs::set_permissions(&readonly_dir, Permissions::from_mode(0o444)).unwrap();

        let path = NormalizedPath::new(readonly_dir.join("file.txt"));
        let result = io::write_text(&path, "content");

        // Restore permissions before assertions (for cleanup)
        let _ = fs::set_permissions(&readonly_dir, Permissions::from_mode(0o755));

        assert!(result.is_err(), "Writing to read-only directory should fail");
    }

    #[test]
    fn test_read_text_permission_denied() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("secret.txt");
        fs::write(&file_path, "secret content").unwrap();

        // Make file unreadable
        fs::set_permissions(&file_path, Permissions::from_mode(0o000)).unwrap();

        let path = NormalizedPath::new(&file_path);
        let result = io::read_text(&path);

        // Restore permissions before assertions (for cleanup)
        let _ = fs::set_permissions(&file_path, Permissions::from_mode(0o644));

        assert!(result.is_err(), "Reading unreadable file should fail");
    }

    #[test]
    fn test_write_atomic_parent_not_writable() {
        let dir = tempdir().unwrap();
        let parent = dir.path().join("parent");
        fs::create_dir(&parent).unwrap();

        // Create the file first, then make parent read-only
        let file_path = parent.join("existing.txt");
        fs::write(&file_path, "original").unwrap();
        fs::set_permissions(&parent, Permissions::from_mode(0o555)).unwrap();

        // Try to overwrite - should fail because we can't create temp file
        let path = NormalizedPath::new(&file_path);
        let result = io::write_text(&path, "new content");

        // Restore permissions
        let _ = fs::set_permissions(&parent, Permissions::from_mode(0o755));

        assert!(result.is_err(), "Writing when parent is read-only should fail");
    }
}

#[cfg(windows)]
mod windows_tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_write_atomic_readonly_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("readonly.txt");
        fs::write(&file_path, "original").unwrap();

        // Make file read-only on Windows
        let mut perms = fs::metadata(&file_path).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&file_path, perms).unwrap();

        let path = NormalizedPath::new(&file_path);
        let result = io::write_text(&path, "new content");

        // Restore permissions
        let mut perms = fs::metadata(&file_path).unwrap().permissions();
        perms.set_readonly(false);
        let _ = fs::set_permissions(&file_path, perms);

        // Note: atomic write uses rename, which may succeed on Windows
        // even for read-only files. This test documents the behavior.
        // If it fails, that's actually more secure.
        let _ = result; // Accept either outcome
    }
}
```

**Step 2: Run the new tests**

```bash
cd main && cargo test -p repo-fs --test error_condition_tests
```

Expected: All tests pass.

**Step 3: Commit**

```bash
git add crates/repo-fs/tests/error_condition_tests.rs
git commit -m "test(repo-fs): add error condition tests

Adds tests for real filesystem error conditions:
- Reading non-existent files
- Writing to read-only directories (Unix)
- Reading unreadable files (Unix)
- Writing when parent directory is read-only (Unix)
- Read-only file handling (Windows)

Uses real filesystem permissions, no mocking."
```

---

## Task 5: Add Concurrent Access Tests

**Files:**
- Create: `crates/repo-fs/tests/concurrency_tests.rs`

**Step 1: Create the concurrency test file**

Create `crates/repo-fs/tests/concurrency_tests.rs`:

```rust
//! Concurrent access tests for write_atomic locking
//!
//! Verifies that the fs2-based locking in write_atomic prevents
//! data corruption under concurrent access.

use repo_fs::{io, NormalizedPath, RobustnessConfig};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_concurrent_writes_no_corruption() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("concurrent.txt");
    let path = Arc::new(NormalizedPath::new(&file_path));

    let num_threads = 10;
    let writes_per_thread = 20;
    let barrier = Arc::new(Barrier::new(num_threads));

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let path = Arc::clone(&path);
            let barrier = Arc::clone(&barrier);

            thread::spawn(move || {
                // Synchronize all threads to start simultaneously
                barrier.wait();

                for i in 0..writes_per_thread {
                    let content = format!("thread{}:write{}\n", thread_id, i);
                    // Some writes may fail due to lock timeout - that's acceptable
                    let _ = io::write_text(&path, &content);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread should not panic");
    }

    // Verify file exists and contains valid content (not corrupted/interleaved)
    let content = std::fs::read_to_string(&file_path).unwrap();

    // Content should be a complete write from one thread, not corrupted
    assert!(
        content.starts_with("thread"),
        "Content should start with 'thread', got: {}",
        &content[..content.len().min(50)]
    );
    assert!(
        content.contains(":write"),
        "Content should contain ':write'"
    );
    // Should be a single line (one complete write), not interleaved
    assert!(
        content.matches("thread").count() == 1,
        "Content should have exactly one 'thread' (no interleaving)"
    );
}

#[test]
fn test_concurrent_writes_to_different_files_all_succeed() {
    let dir = tempdir().unwrap();
    let num_threads = 5;
    let barrier = Arc::new(Barrier::new(num_threads));
    let results = Arc::new(std::sync::Mutex::new(Vec::new()));

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let dir_path = dir.path().to_path_buf();
            let barrier = Arc::clone(&barrier);
            let results = Arc::clone(&results);

            thread::spawn(move || {
                barrier.wait();

                let file_path = dir_path.join(format!("file_{}.txt", thread_id));
                let path = NormalizedPath::new(&file_path);
                let result = io::write_text(&path, &format!("content_{}", thread_id));

                results.lock().unwrap().push((thread_id, result.is_ok()));
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread should not panic");
    }

    // All writes to different files should succeed
    let results = results.lock().unwrap();
    for (thread_id, success) in results.iter() {
        assert!(
            *success,
            "Write from thread {} should succeed",
            thread_id
        );
    }
}

#[test]
fn test_lock_timeout_is_respected() {
    use fs2::FileExt;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("locked.txt");
    let lock_path = format!("{}.lock", file_path.display());

    // Create and hold the lock file externally
    let lock_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&lock_path)
        .unwrap();
    lock_file.lock_exclusive().unwrap();

    let path = NormalizedPath::new(&file_path);
    let config = RobustnessConfig {
        lock_timeout: Duration::from_millis(500),
        enable_fsync: false,
    };

    let start = std::time::Instant::now();
    let result = io::write_atomic(&path, b"content", config);
    let elapsed = start.elapsed();

    // Release lock
    drop(lock_file);

    // Should have failed due to lock timeout
    assert!(result.is_err(), "Write should fail when lock is held");

    // Should have respected the timeout (with some tolerance)
    assert!(
        elapsed >= Duration::from_millis(400),
        "Should have waited at least 400ms, waited {:?}",
        elapsed
    );
    assert!(
        elapsed < Duration::from_secs(5),
        "Should not have waited more than 5s, waited {:?}",
        elapsed
    );
}
```

**Step 2: Run the concurrency tests**

```bash
cd main && cargo test -p repo-fs --test concurrency_tests
```

Expected: All tests pass.

**Step 3: Commit**

```bash
git add crates/repo-fs/tests/concurrency_tests.rs
git commit -m "test(repo-fs): add concurrent access tests

Adds stress tests for write_atomic's locking mechanism:
- Concurrent writes to same file (verifies no corruption)
- Concurrent writes to different files (all should succeed)
- Lock timeout is respected when lock is held externally

Uses thread barriers for maximum contention."
```

---

## Task 6: Add SyncOptions Struct to repo-core

**Files:**
- Modify: `crates/repo-core/src/sync/engine.rs`
- Modify: `crates/repo-core/src/sync/mod.rs`
- Modify: `crates/repo-core/src/lib.rs`

**Step 1: Add SyncOptions struct in engine.rs**

Add after the `SyncReport` struct in `crates/repo-core/src/sync/engine.rs`:

```rust
/// Options for sync and fix operations
#[derive(Debug, Clone, Default)]
pub struct SyncOptions {
    /// If true, simulate changes without modifying the filesystem.
    /// Actions will be prefixed with "[dry-run] Would ..."
    pub dry_run: bool,
}
```

**Step 2: Add sync_with_options method**

Add this method to the `SyncEngine` impl block, before the existing `sync` method:

```rust
    /// Synchronize configuration to the filesystem with options
    ///
    /// When `options.dry_run` is true, simulates changes without writing.
    pub fn sync_with_options(&self, options: SyncOptions) -> Result<SyncReport> {
        let ledger = self.load_ledger()?;
        let ledger_path = self.ledger_path();
        let mut report = SyncReport::success();

        if !ledger_path.exists() {
            if options.dry_run {
                report = report.with_action("[dry-run] Would create ledger file".to_string());
            } else {
                self.save_ledger(&ledger)?;
                report = report.with_action("Created ledger file".to_string());
            }
        }

        // TODO: Full sync implementation - apply configuration changes
        // For now, we just ensure the ledger exists

        Ok(report)
    }
```

**Step 3: Update existing sync method to use sync_with_options**

Replace the existing `sync` method:

```rust
    /// Synchronize configuration to the filesystem
    ///
    /// This operation:
    /// 1. Loads the resolved configuration and ledger
    /// 2. Creates/saves the ledger if it doesn't exist
    /// 3. (Future) Applies configuration changes
    ///
    /// # Returns
    ///
    /// A `SyncReport` containing the actions taken.
    pub fn sync(&self) -> Result<SyncReport> {
        self.sync_with_options(SyncOptions::default())
    }
```

**Step 4: Add fix_with_options method**

Add this method before the existing `fix` method:

```rust
    /// Fix synchronization issues with options
    ///
    /// When `options.dry_run` is true, simulates fixes without applying.
    pub fn fix_with_options(&self, options: SyncOptions) -> Result<SyncReport> {
        // For now, fix just re-runs sync
        // In the future, this would also repair drifted/missing projections
        self.sync_with_options(options)
    }
```

**Step 5: Update existing fix method**

Replace the existing `fix` method:

```rust
    /// Fix synchronization issues
    ///
    /// Re-synchronizes to repair any drift or missing files.
    ///
    /// # Returns
    ///
    /// A `SyncReport` containing the actions taken.
    pub fn fix(&self) -> Result<SyncReport> {
        self.fix_with_options(SyncOptions::default())
    }
```

**Step 6: Export SyncOptions in mod.rs**

In `crates/repo-core/src/sync/mod.rs`, update the pub use line:

```rust
pub use engine::{compute_file_checksum, get_json_path, SyncEngine, SyncOptions, SyncReport};
```

**Step 7: Export SyncOptions in lib.rs**

In `crates/repo-core/src/lib.rs`, update the sync export line:

```rust
pub use sync::{CheckReport, CheckStatus, DriftItem, SyncEngine, SyncOptions, SyncReport};
```

**Step 8: Run tests**

```bash
cd main && cargo test -p repo-core
```

Expected: All tests pass.

**Step 9: Commit**

```bash
git add crates/repo-core/src/sync/engine.rs crates/repo-core/src/sync/mod.rs crates/repo-core/src/lib.rs
git commit -m "feat(repo-core): add SyncOptions for dry-run support

Adds SyncOptions struct with dry_run field. When dry_run is true,
sync and fix operations simulate changes without modifying filesystem.

- sync_with_options() and fix_with_options() accept SyncOptions
- Existing sync() and fix() use default options (dry_run: false)
- Dry-run actions are prefixed with '[dry-run] Would ...'"
```

---

## Task 7: Add --dry-run Flag to CLI

**Files:**
- Modify: `crates/repo-cli/src/cli.rs`
- Modify: `crates/repo-cli/src/commands/sync.rs`
- Modify: `crates/repo-cli/src/main.rs`

**Step 1: Add dry_run field to Sync and Fix commands in cli.rs**

In `crates/repo-cli/src/cli.rs`, update the `Sync` variant:

```rust
    /// Synchronize tool configurations
    Sync {
        /// Preview changes without applying them
        #[arg(long)]
        dry_run: bool,
    },
```

Update the `Fix` variant:

```rust
    /// Fix configuration drift automatically
    Fix {
        /// Preview fixes without applying them
        #[arg(long)]
        dry_run: bool,
    },
```

**Step 2: Update run_sync to accept dry_run parameter**

In `crates/repo-cli/src/commands/sync.rs`, update the function signature and implementation:

```rust
use repo_core::{CheckStatus, ConfigResolver, Mode, SyncEngine, SyncOptions};

/// Run the sync command
///
/// Synchronizes configuration from the ledger to the filesystem.
pub fn run_sync(path: &Path, dry_run: bool) -> Result<()> {
    if dry_run {
        println!(
            "{} Previewing sync (dry-run)...",
            "=>".blue().bold()
        );
    } else {
        println!(
            "{} Synchronizing tool configurations...",
            "=>".blue().bold()
        );
    }

    let root = NormalizedPath::new(path);
    let mode = detect_mode(&root)?;
    let engine = SyncEngine::new(root, mode)?;

    let options = SyncOptions { dry_run };
    let report = engine.sync_with_options(options)?;

    if report.success {
        if report.actions.is_empty() {
            println!("{} Already synchronized. No changes needed.", "OK".green().bold());
        } else {
            let prefix = if dry_run { "Would take actions" } else { "Synchronization complete" };
            println!("{} {}:", "OK".green().bold(), prefix);
            for action in &report.actions {
                println!("   {} {}", "+".green(), action);
            }
        }
    } else {
        println!("{} Synchronization failed:", "ERROR".red().bold());
        for error in &report.errors {
            println!("   {} {}", "!".red(), error);
        }
        return Err(CliError::user("Synchronization failed"));
    }

    Ok(())
}
```

**Step 3: Update run_fix to accept dry_run parameter**

Update `run_fix` in the same file:

```rust
/// Run the fix command
///
/// Repairs configuration drift by re-synchronizing.
pub fn run_fix(path: &Path, dry_run: bool) -> Result<()> {
    if dry_run {
        println!(
            "{} Previewing fix (dry-run)...",
            "=>".blue().bold()
        );
    } else {
        println!(
            "{} Fixing configuration drift...",
            "=>".blue().bold()
        );
    }

    let root = NormalizedPath::new(path);
    let mode = detect_mode(&root)?;
    let engine = SyncEngine::new(root, mode)?;

    // First check what's wrong
    let check_report = engine.check()?;

    if check_report.status == CheckStatus::Healthy {
        println!("{} Repository is already healthy. Nothing to fix.", "OK".green().bold());
        return Ok(());
    }

    // Now fix it (or simulate)
    let options = SyncOptions { dry_run };
    let report = engine.fix_with_options(options)?;

    if report.success {
        if report.actions.is_empty() {
            let msg = if dry_run { "No actions needed." } else { "Configuration fixed." };
            println!("{} {}", "OK".green().bold(), msg);
        } else {
            let prefix = if dry_run { "Would take actions" } else { "Configuration fixed" };
            println!("{} {}:", "OK".green().bold(), prefix);
            for action in &report.actions {
                println!("   {} {}", "+".green(), action);
            }
        }
    } else {
        println!("{} Fix operation failed:", "ERROR".red().bold());
        for error in &report.errors {
            println!("   {} {}", "!".red(), error);
        }
        return Err(CliError::user("Fix operation failed"));
    }

    Ok(())
}
```

**Step 4: Update main.rs to pass dry_run**

In `crates/repo-cli/src/main.rs`, update the command handling:

```rust
        Some(Commands::Sync { dry_run }) => {
            commands::sync::run_sync(&cwd, dry_run)?;
        }
        Some(Commands::Fix { dry_run }) => {
            commands::sync::run_fix(&cwd, dry_run)?;
        }
```

**Step 5: Run tests**

```bash
cd main && cargo test -p repo-cli
```

Expected: All tests pass.

**Step 6: Commit**

```bash
git add crates/repo-cli/src/cli.rs crates/repo-cli/src/commands/sync.rs crates/repo-cli/src/main.rs
git commit -m "feat(repo-cli): add --dry-run flag to sync and fix commands

Adds --dry-run flag that previews changes without applying them:
- repo sync --dry-run
- repo fix --dry-run

Output is prefixed with '[dry-run]' and describes what would happen."
```

---

## Task 8: Add ToolRegistry to repo-meta

**Files:**
- Create: `crates/repo-meta/src/tools.rs`
- Modify: `crates/repo-meta/src/lib.rs`

**Step 1: Create the tools.rs file**

Create `crates/repo-meta/src/tools.rs`:

```rust
//! Known tools registry
//!
//! Provides a registry of recognized tool names for validation.

use std::collections::HashSet;

/// Registry of known tool names for validation
#[derive(Debug, Clone)]
pub struct ToolRegistry {
    known_tools: HashSet<&'static str>,
}

impl ToolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            known_tools: HashSet::new(),
        }
    }

    /// Create a registry with built-in known tools
    ///
    /// Includes: claude, claude-desktop, cursor, vscode, windsurf, gemini-cli, antigravity
    pub fn with_builtins() -> Self {
        let known_tools = HashSet::from([
            "claude",
            "claude-desktop",
            "cursor",
            "vscode",
            "windsurf",
            "gemini-cli",
            "antigravity",
        ]);
        Self { known_tools }
    }

    /// Check if a tool name is known
    pub fn is_known(&self, name: &str) -> bool {
        self.known_tools.contains(name)
    }

    /// List all known tools, sorted alphabetically
    pub fn list_known(&self) -> Vec<&'static str> {
        let mut tools: Vec<_> = self.known_tools.iter().copied().collect();
        tools.sort();
        tools
    }

    /// Get the number of known tools
    pub fn len(&self) -> usize {
        self.known_tools.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.known_tools.is_empty()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::with_builtins()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_registry_is_empty() {
        let registry = ToolRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_with_builtins_has_known_tools() {
        let registry = ToolRegistry::with_builtins();
        assert!(!registry.is_empty());
        assert!(registry.is_known("vscode"));
        assert!(registry.is_known("claude"));
        assert!(registry.is_known("cursor"));
    }

    #[test]
    fn test_unknown_tool() {
        let registry = ToolRegistry::with_builtins();
        assert!(!registry.is_known("unknown-tool"));
        assert!(!registry.is_known("vim"));
    }

    #[test]
    fn test_list_known_is_sorted() {
        let registry = ToolRegistry::with_builtins();
        let list = registry.list_known();

        assert!(list.len() >= 7);

        // Verify sorted
        let mut sorted = list.clone();
        sorted.sort();
        assert_eq!(list, sorted);
    }

    #[test]
    fn test_default_uses_builtins() {
        let registry = ToolRegistry::default();
        assert!(registry.is_known("vscode"));
    }
}
```

**Step 2: Export ToolRegistry in lib.rs**

In `crates/repo-meta/src/lib.rs`, add the module and export:

Add after `pub mod schema;`:
```rust
pub mod tools;
```

Add to the pub use section:
```rust
pub use tools::ToolRegistry;
```

**Step 3: Run tests**

```bash
cd main && cargo test -p repo-meta
```

Expected: All tests pass.

**Step 4: Commit**

```bash
git add crates/repo-meta/src/tools.rs crates/repo-meta/src/lib.rs
git commit -m "feat(repo-meta): add ToolRegistry for tool validation

Adds ToolRegistry with known tools from spec:
- claude, claude-desktop, cursor, vscode
- windsurf, gemini-cli, antigravity

Provides is_known() for validation and list_known() for help messages."
```

---

## Task 9: Add Tool/Preset Validation to CLI

**Files:**
- Modify: `crates/repo-cli/src/commands/tool.rs`
- Modify: `crates/repo-cli/Cargo.toml` (if repo-meta not already a dependency)

**Step 1: Add validation to run_add_tool**

In `crates/repo-cli/src/commands/tool.rs`, add the import:

```rust
use repo_meta::{Registry, ToolRegistry};
```

Update `run_add_tool` to add validation after the initial println:

```rust
pub fn run_add_tool(path: &Path, name: &str) -> Result<()> {
    println!(
        "{} Adding tool: {}",
        "=>".blue().bold(),
        name.cyan()
    );

    // Validate tool name
    let tool_registry = ToolRegistry::with_builtins();
    if !tool_registry.is_known(name) {
        eprintln!(
            "{} Unknown tool '{}'. Known tools: {}",
            "warning:".yellow().bold(),
            name,
            tool_registry.list_known().join(", ")
        );
    }

    // ... rest of existing implementation unchanged
```

**Step 2: Add validation to run_add_preset**

Update `run_add_preset` similarly:

```rust
pub fn run_add_preset(path: &Path, name: &str) -> Result<()> {
    println!(
        "{} Adding preset: {}",
        "=>".blue().bold(),
        name.cyan()
    );

    // Validate preset name
    let registry = Registry::with_builtins();
    if !registry.has_provider(name) {
        eprintln!(
            "{} Unknown preset '{}'. Known presets: {}",
            "warning:".yellow().bold(),
            name,
            registry.list_presets().join(", ")
        );
    }

    // ... rest of existing implementation unchanged
```

**Step 3: Run tests**

```bash
cd main && cargo test -p repo-cli
```

Expected: All tests pass.

**Step 4: Commit**

```bash
git add crates/repo-cli/src/commands/tool.rs
git commit -m "feat(repo-cli): add validation warnings for unknown tools/presets

When adding a tool or preset, validates against known registries:
- Unknown tools show warning with list of known tools
- Unknown presets show warning with list of known presets
- Commands still succeed (warning only, not blocking)"
```

---

## Task 10: Change Default Mode to Worktrees

**Files:**
- Modify: `crates/repo-cli/src/cli.rs`

**Step 1: Change the default value**

In `crates/repo-cli/src/cli.rs`, find the `Init` variant and change:

```rust
    /// Initialize a new repository configuration
    Init {
        /// Repository mode (standard or worktree)
        #[arg(short, long, default_value = "worktrees")]
        mode: String,

        /// Tools to enable
        #[arg(short, long)]
        tools: Vec<String>,

        /// Presets to apply
        #[arg(short, long)]
        presets: Vec<String>,
    },
```

**Step 2: Update the unit test**

Find `parse_init_command_defaults` and update:

```rust
    #[test]
    fn parse_init_command_defaults() {
        let cli = Cli::parse_from(["repo", "init"]);
        assert!(matches!(
            cli.command,
            Some(Commands::Init {
                mode,
                tools,
                presets
            }) if mode == "worktrees" && tools.is_empty() && presets.is_empty()
        ));
    }
```

**Step 3: Run CLI tests**

```bash
cd main && cargo test -p repo-cli
```

Expected: Tests pass.

**Step 4: Commit**

```bash
git add crates/repo-cli/src/cli.rs
git commit -m "fix(repo-cli): change default mode to worktrees per spec

Changes default value for --mode from 'standard' to 'worktrees'.
This aligns with the CLI specification which states:
'Default: worktrees (as per user preference)'"
```

---

## Task 11: Update Integration Tests for Default Mode

**Files:**
- Modify: `crates/repo-cli/tests/integration_tests.rs`

**Step 1: Update the default mode test**

Find `test_init_default_mode_is_standard` and rename/update:

```rust
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
```

**Step 2: Run integration tests**

```bash
cd main && cargo test -p repo-cli --test integration_tests
```

Expected: All tests pass.

**Step 3: Commit**

```bash
git add crates/repo-cli/tests/integration_tests.rs
git commit -m "test(repo-cli): update integration tests for worktrees default

Renames test_init_default_mode_is_standard to test_init_default_mode_is_worktrees
and updates assertions to expect worktrees mode and main/ directory."
```

---

## Task 12: Final Verification

**Step 1: Run all tests**

```bash
cd main && cargo test --workspace
```

Expected: All tests pass.

**Step 2: Run clippy**

```bash
cd main && cargo clippy --workspace -- -D warnings
```

Expected: No warnings.

**Step 3: Test CLI manually**

```bash
cd main && cargo build -p repo-cli
./target/debug/repo --help
./target/debug/repo init --help
```

Verify:
- `--dry-run` appears in sync and fix help
- Default mode shows as "worktrees"

**Step 4: Final commit (if any fixes needed)**

```bash
git add -A
git commit -m "chore: final cleanup for audit fixes"
```

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | Add SymlinkInPath error | repo-fs/error.rs |
| 2 | Implement symlink detection | repo-fs/io.rs |
| 3 | Update security tests | repo-fs/tests/security_audit_tests.rs |
| 4 | Add error condition tests | repo-fs/tests/error_condition_tests.rs |
| 5 | Add concurrency tests | repo-fs/tests/concurrency_tests.rs |
| 6 | Add SyncOptions struct | repo-core/sync/engine.rs, mod.rs, lib.rs |
| 7 | Add --dry-run to CLI | repo-cli/cli.rs, commands/sync.rs, main.rs |
| 8 | Add ToolRegistry | repo-meta/tools.rs, lib.rs |
| 9 | Add validation warnings | repo-cli/commands/tool.rs |
| 10 | Change default mode | repo-cli/cli.rs |
| 11 | Update integration tests | repo-cli/tests/integration_tests.rs |
| 12 | Final verification | N/A |

---

Plan complete and saved to `docs/plans/2026-01-26-immediate-shortterm-fixes-implementation.md`.

**Two execution options:**

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

**Which approach?**
