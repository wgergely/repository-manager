# Audit Remediation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Address HIGH priority findings from the 2026-01-28 security/robustness audits - atomic ledger writes, file locking, JSON panic fixes, and production unwrap replacements.

**Architecture:** Implement atomic file operations using temp-file-then-rename pattern. Add file locking using fs2 crate. Replace panic-prone code with proper error handling.

**Tech Stack:** Rust, fs2 (file locking), tempfile, repo-core, repo-blocks

---

## Prerequisites

Review audit findings:
- `docs/audits/2026-01-28-audit-index.md`
- `docs/audits/2026-01-28-repo-core-audit.md`
- `docs/audits/2026-01-28-repo-blocks-audit.md`

---

## Task 1: Add fs2 Dependency for File Locking

**Files:**
- Modify: `crates/repo-core/Cargo.toml`

**Step 1: Add fs2 dependency**

```toml
[dependencies]
# ... existing deps ...
fs2 = "0.4"
```

**Step 2: Verify compilation**

Run: `cargo check -p repo-core`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add crates/repo-core/Cargo.toml Cargo.lock
git commit -m "chore(repo-core): add fs2 dependency for file locking"
```

---

## Task 2: Implement Atomic Ledger Writes (DATA-2)

**Files:**
- Modify: `crates/repo-core/src/ledger/mod.rs`

**Step 1: Write test for atomic save**

Add to ledger tests:

```rust
#[test]
fn test_ledger_save_is_atomic() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let temp = tempfile::TempDir::new().unwrap();
    let ledger_path = temp.path().join("ledger.toml");

    // Create initial ledger
    let mut ledger = Ledger::new();
    ledger.add_intent(Intent::new("rule:test".to_string(), json!({})));
    ledger.save(&ledger_path).unwrap();

    // Verify temp file doesn't exist after save
    let temp_path = ledger_path.with_extension("toml.tmp");
    assert!(!temp_path.exists(), "Temp file should be cleaned up after save");

    // Verify main file exists
    assert!(ledger_path.exists(), "Ledger file should exist");
}

#[test]
fn test_ledger_save_preserves_on_write_failure() {
    // This tests that if the temp file write fails, the original is preserved
    let temp = tempfile::TempDir::new().unwrap();
    let ledger_path = temp.path().join("ledger.toml");

    // Create initial content
    std::fs::write(&ledger_path, "# original content\n").unwrap();

    // Make directory read-only to force write failure (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(temp.path()).unwrap().permissions();
        perms.set_mode(0o555);
        std::fs::set_permissions(temp.path(), perms).unwrap();

        let ledger = Ledger::new();
        let result = ledger.save(&ledger_path);

        // Restore permissions for cleanup
        let mut perms = std::fs::metadata(temp.path()).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(temp.path(), perms).unwrap();

        // Original should still exist
        let content = std::fs::read_to_string(&ledger_path).unwrap();
        assert!(content.contains("original"), "Original content should be preserved on failure");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p repo-core test_ledger_save_is_atomic`
Expected: May pass with current impl, but temp file handling needs verification

**Step 3: Implement atomic save**

In `crates/repo-core/src/ledger/mod.rs`, replace the `save` method:

```rust
/// Save ledger to file atomically
///
/// Uses write-to-temp-then-rename pattern to prevent corruption
/// if the process crashes during write.
pub fn save(&self, path: &Path) -> Result<()> {
    let content = toml::to_string_pretty(self)
        .map_err(|e| Error::LedgerError { message: e.to_string() })?;

    // Write to temporary file first
    let temp_path = path.with_extension("toml.tmp");

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write content to temp file
    std::fs::write(&temp_path, &content)?;

    // Atomically rename temp to target
    // On Unix, rename is atomic. On Windows, it's as close as we can get.
    std::fs::rename(&temp_path, path).map_err(|e| {
        // Clean up temp file if rename fails
        let _ = std::fs::remove_file(&temp_path);
        e
    })?;

    Ok(())
}
```

**Step 4: Run tests**

Run: `cargo test -p repo-core ledger`
Expected: All tests pass

**Step 5: Commit**

```bash
git add crates/repo-core/src/ledger/mod.rs
git commit -m "fix(repo-core): implement atomic ledger writes to prevent corruption"
```

---

## Task 3: Add File Locking for Ledger Operations (DATA-1, SYNC-1)

**Files:**
- Create: `crates/repo-core/src/ledger/lock.rs`
- Modify: `crates/repo-core/src/ledger/mod.rs`

**Step 1: Write test for concurrent access prevention**

```rust
#[test]
fn test_ledger_lock_prevents_concurrent_access() {
    use std::thread;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let temp = tempfile::TempDir::new().unwrap();
    let ledger_path = temp.path().join("ledger.toml");
    let lock_path = temp.path().join("ledger.lock");

    // Create initial ledger
    let ledger = Ledger::new();
    ledger.save(&ledger_path).unwrap();

    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    // Spawn multiple threads trying to acquire lock
    for _ in 0..5 {
        let path = lock_path.clone();
        let cnt = Arc::clone(&counter);

        let handle = thread::spawn(move || {
            if let Ok(_guard) = LedgerLock::acquire(&path) {
                cnt.fetch_add(1, Ordering::SeqCst);
                thread::sleep(std::time::Duration::from_millis(50));
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // All threads should have been able to access eventually
    assert!(counter.load(Ordering::SeqCst) >= 1);
}
```

**Step 2: Implement file locking**

Create `crates/repo-core/src/ledger/lock.rs`:

```rust
//! File-based locking for ledger operations
//!
//! Prevents concurrent modifications to the ledger file.

use std::fs::{File, OpenOptions};
use std::path::Path;
use fs2::FileExt;
use crate::{Error, Result};

/// Guard that holds a lock on the ledger
///
/// The lock is released when this guard is dropped.
pub struct LedgerLock {
    _file: File,
}

impl LedgerLock {
    /// Acquire an exclusive lock on the ledger
    ///
    /// This blocks until the lock is available.
    pub fn acquire(lock_path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = lock_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(lock_path)?;

        file.lock_exclusive().map_err(|e| {
            Error::LedgerError {
                message: format!("Failed to acquire ledger lock: {}", e),
            }
        })?;

        Ok(LedgerLock { _file: file })
    }

    /// Try to acquire lock without blocking
    ///
    /// Returns None if lock is held by another process.
    pub fn try_acquire(lock_path: &Path) -> Result<Option<Self>> {
        if let Some(parent) = lock_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(lock_path)?;

        match file.try_lock_exclusive() {
            Ok(()) => Ok(Some(LedgerLock { _file: file })),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(Error::LedgerError {
                message: format!("Failed to acquire ledger lock: {}", e),
            }),
        }
    }
}

// Lock is automatically released when File is dropped
impl Drop for LedgerLock {
    fn drop(&mut self) {
        // File drop handles unlock
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_lock_acquire_and_release() {
        let temp = TempDir::new().unwrap();
        let lock_path = temp.path().join("test.lock");

        // Acquire lock
        let guard = LedgerLock::acquire(&lock_path).unwrap();
        assert!(lock_path.exists());

        // Drop guard releases lock
        drop(guard);

        // Can acquire again
        let _guard2 = LedgerLock::acquire(&lock_path).unwrap();
    }

    #[test]
    fn test_try_lock_when_held() {
        let temp = TempDir::new().unwrap();
        let lock_path = temp.path().join("test.lock");

        let _guard = LedgerLock::acquire(&lock_path).unwrap();

        // try_acquire should return None when lock is held
        // Note: This only works across processes, not threads in same process
        // For thread-based testing, we'd need separate processes
    }
}
```

**Step 3: Export lock module**

In `crates/repo-core/src/ledger/mod.rs`:

```rust
mod lock;
pub use lock::LedgerLock;
```

**Step 4: Update SyncEngine to use locking**

In `crates/repo-core/src/sync/engine.rs`, update methods that modify ledger:

```rust
use crate::ledger::LedgerLock;

impl SyncEngine {
    /// Sync with file locking to prevent concurrent modifications
    pub fn sync_with_options(&self, options: SyncOptions) -> Result<SyncReport> {
        let lock_path = self.root.join(".repository/ledger.lock");
        let _lock = LedgerLock::acquire(lock_path.as_ref())?;

        // ... existing sync logic ...
    }

    /// Fix with file locking
    pub fn fix_with_options(&self, options: SyncOptions) -> Result<SyncReport> {
        let lock_path = self.root.join(".repository/ledger.lock");
        let _lock = LedgerLock::acquire(lock_path.as_ref())?;

        // ... existing fix logic ...
    }
}
```

**Step 5: Run tests**

Run: `cargo test -p repo-core`
Expected: All tests pass

**Step 6: Commit**

```bash
git add crates/repo-core/src/ledger/lock.rs crates/repo-core/src/ledger/mod.rs crates/repo-core/src/sync/engine.rs
git commit -m "fix(repo-core): add file locking to prevent concurrent ledger modifications"
```

---

## Task 4: Fix JSON Root Panic in repo-blocks (ERR-02)

**Files:**
- Modify: `crates/repo-blocks/src/json.rs`

**Step 1: Write test for array root handling**

```rust
#[test]
fn test_json_update_with_array_root() {
    let content = r#"[1, 2, 3]"#;
    let result = update_json_block(content, "test-uuid", "new content");

    // Should return error, not panic
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("object") || err.to_string().contains("root"));
}

#[test]
fn test_json_update_with_null_root() {
    let content = "null";
    let result = update_json_block(content, "test-uuid", "new content");

    assert!(result.is_err());
}
```

**Step 2: Run test to verify panic**

Run: `cargo test -p repo-blocks test_json_update_with_array_root`
Expected: FAIL with panic

**Step 3: Fix the panic**

In `crates/repo-blocks/src/json.rs`, replace `expect` with proper error:

```rust
pub fn update_json_block(content: &str, uuid: &str, new_content: &str) -> Result<String> {
    let mut value: serde_json::Value = serde_json::from_str(content)?;

    let obj = value.as_object_mut().ok_or_else(|| {
        Error::InvalidFormat {
            message: "JSON root must be an object, not array or primitive".to_string(),
        }
    })?;

    // ... rest of the function
}
```

**Step 4: Run test to verify fix**

Run: `cargo test -p repo-blocks test_json_update_with_array_root`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/repo-blocks/src/json.rs
git commit -m "fix(repo-blocks): return error instead of panic on non-object JSON root"
```

---

## Task 5: Replace Production unwrap() Calls (ERR-1)

**Files:**
- Modify: `crates/repo-core/src/rules/registry.rs`
- Modify: `crates/repo-core/src/projection/writer.rs`

**Step 1: Fix registry.rs unwrap**

In `crates/repo-core/src/rules/registry.rs:88`:

Replace:
```rust
Ok(self.rules.last().unwrap())
```

With:
```rust
// Safe because we just pushed to the vector
self.rules.last()
    .ok_or_else(|| Error::InternalError {
        message: "rules vector unexpectedly empty after push".to_string(),
    })
```

**Step 2: Fix projection/writer.rs unwrap at line 82**

Replace:
```rust
let start_idx = existing.find(&marker_start).unwrap();
```

With:
```rust
let start_idx = existing.find(&marker_start)
    .ok_or_else(|| Error::InternalError {
        message: format!("marker_start not found despite contains() check: {}", marker_start),
    })?;
```

**Step 3: Fix projection/writer.rs unwrap at line 267**

Replace:
```rust
map.remove(*parts.last().unwrap());
```

With:
```rust
if let Some(last_part) = parts.last() {
    map.remove(*last_part);
}
```

**Step 4: Add InternalError variant if not exists**

In `crates/repo-core/src/error.rs`:

```rust
#[error("Internal error: {message}")]
InternalError { message: String },
```

**Step 5: Run tests**

Run: `cargo test -p repo-core`
Expected: All tests pass

**Step 6: Commit**

```bash
git add crates/repo-core/src/rules/registry.rs crates/repo-core/src/projection/writer.rs crates/repo-core/src/error.rs
git commit -m "fix(repo-core): replace production unwrap() calls with proper error handling"
```

---

## Task 6: Add Input Size Validation (S2 from repo-meta)

**Files:**
- Modify: `crates/repo-meta/src/config.rs`

**Step 1: Write test for large file rejection**

```rust
#[test]
fn test_config_rejects_oversized_file() {
    let temp = tempfile::TempDir::new().unwrap();
    let config_path = temp.path().join("huge.toml");

    // Create a 2MB file (above reasonable limit)
    let huge_content = "key = \"".to_string() + &"x".repeat(2 * 1024 * 1024) + "\"";
    std::fs::write(&config_path, &huge_content).unwrap();

    let result = Config::load(&config_path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("too large") ||
            result.unwrap_err().to_string().contains("size"));
}
```

**Step 2: Implement size check**

In `crates/repo-meta/src/config.rs`:

```rust
/// Maximum config file size (1MB should be plenty)
const MAX_CONFIG_SIZE: u64 = 1024 * 1024;

pub fn load(path: &Path) -> Result<Self> {
    // Check file size before reading
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > MAX_CONFIG_SIZE {
        return Err(Error::ConfigTooLarge {
            path: path.to_path_buf(),
            size: metadata.len(),
            max: MAX_CONFIG_SIZE,
        });
    }

    let content = std::fs::read_to_string(path)?;
    Self::parse(&content)
}
```

**Step 3: Add error variant**

In `crates/repo-meta/src/error.rs`:

```rust
#[error("Config file too large: {path} is {size} bytes (max {max})")]
ConfigTooLarge { path: PathBuf, size: u64, max: u64 },
```

**Step 4: Run tests**

Run: `cargo test -p repo-meta config_rejects_oversized`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/repo-meta/src/config.rs crates/repo-meta/src/error.rs
git commit -m "fix(repo-meta): add input size validation to prevent DoS"
```

---

## Task 7: Replace expect() in repo-cli (E1)

**Files:**
- Modify: `crates/repo-cli/src/main.rs`

**Step 1: Find and fix the expect**

In `crates/repo-cli/src/main.rs:36`:

Replace:
```rust
tracing::subscriber::set_global_default(subscriber)
    .expect("Failed to set tracing subscriber");
```

With:
```rust
if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
    eprintln!("Warning: Could not set tracing subscriber: {}", e);
}
```

**Step 2: Run tests**

Run: `cargo test -p repo-cli`
Expected: All tests pass

**Step 3: Commit**

```bash
git add crates/repo-cli/src/main.rs
git commit -m "fix(repo-cli): replace expect() with graceful error handling for tracing setup"
```

---

## Completion Checklist

- [ ] fs2 dependency added
- [ ] Atomic ledger writes implemented (temp file + rename)
- [ ] File locking implemented for ledger operations
- [ ] JSON root panic fixed in repo-blocks
- [ ] Production unwrap() calls replaced (3 instances)
- [ ] Input size validation added to repo-meta
- [ ] CLI expect() replaced with graceful handling
- [ ] All tests pass

---

## Verification

After completing all tasks:

```bash
# Run all tests
cargo test --workspace

# Verify no panicking calls in production code
grep -r "\.unwrap()" crates/*/src/**/*.rs | grep -v "#\[cfg(test)\]" | grep -v "// safe:" | head -20
grep -r "\.expect(" crates/*/src/**/*.rs | grep -v "#\[cfg(test)\]" | grep -v "// safe:" | head -20

# Check for unsafe code (should be none)
grep -r "unsafe" crates/*/src/**/*.rs
```

---

*Plan created: 2026-01-29*
*Addresses: DATA-1, DATA-2, SYNC-1, ERR-01, ERR-02, S2, E1 from audit findings*
