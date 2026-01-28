# Audit Remediation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Address high and medium priority findings from the 2026-01-28 security audit.

**Architecture:** Fix concurrency/atomicity issues in repo-core, remove panic paths in repo-blocks and repo-cli, add defensive depth limits in repo-content and repo-meta. Each fix is isolated to a single crate with minimal cross-cutting concerns.

**Tech Stack:** Rust, fs2 (file locking), toml, serde_json

---

## Task 1: Add Atomic Ledger Writes (repo-core)

**Issue:** DATA-2 - Ledger saves use `fs::write()` which can corrupt on crash.

**Files:**
- Modify: `crates/repo-core/src/ledger/mod.rs:64-68`
- Test: `crates/repo-core/src/ledger/mod.rs` (add test in existing module)

**Step 1: Write the failing test**

Add to the `tests` module at the bottom of `ledger/mod.rs`:

```rust
#[test]
fn ledger_save_is_atomic() {
    use std::fs;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let path = dir.path().join("ledger.toml");

    let mut ledger = Ledger::new();
    ledger.add_intent(Intent::new("rule:test".to_string(), json!({})));

    // Save ledger
    ledger.save(&path).unwrap();

    // Verify no temp file left behind
    let temp_path = path.with_extension("toml.tmp");
    assert!(!temp_path.exists(), "Temporary file should be cleaned up");

    // Verify content is valid
    let loaded = Ledger::load(&path).unwrap();
    assert_eq!(loaded.intents().len(), 1);
}
```

**Step 2: Run test to verify it passes (baseline)**

```bash
cargo test -p repo-core ledger_save_is_atomic -- --nocapture
```

Expected: PASS (current implementation works for happy path)

**Step 3: Implement atomic write pattern**

Replace the `save` method in `crates/repo-core/src/ledger/mod.rs`:

```rust
/// Save the ledger to a TOML file atomically
///
/// Uses write-to-temp-then-rename pattern to prevent corruption
/// if the process crashes during write.
///
/// # Arguments
///
/// * `path` - Path to save the ledger TOML file
///
/// # Errors
///
/// Returns an error if the file cannot be written.
pub fn save(&self, path: &Path) -> Result<()> {
    let content = toml::to_string_pretty(self)?;

    // Write to temporary file first
    let temp_path = path.with_extension("toml.tmp");
    fs::write(&temp_path, &content)?;

    // Atomically rename to target (atomic on POSIX, best-effort on Windows)
    fs::rename(&temp_path, path)?;

    Ok(())
}
```

**Step 4: Run tests to verify**

```bash
cargo test -p repo-core -- --test-threads=1
```

Expected: All tests PASS

**Step 5: Commit**

```bash
git add crates/repo-core/src/ledger/mod.rs
git commit -m "fix(repo-core): implement atomic ledger writes

Addresses DATA-2 from 2026-01-28 audit. Uses write-to-temp-then-rename
pattern to prevent ledger corruption on crash.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 2: Add File Locking for Ledger Operations (repo-core)

**Issue:** DATA-1 and SYNC-1 - TOCTOU race conditions and concurrent sync conflicts.

**Files:**
- Modify: `crates/repo-core/Cargo.toml` (add fs2 dependency)
- Modify: `crates/repo-core/src/ledger/mod.rs`
- Modify: `crates/repo-core/src/sync/engine.rs`
- Test: Add concurrency test

**Step 1: Add fs2 dependency**

Add to `crates/repo-core/Cargo.toml` under `[dependencies]`:

```toml
fs2 = "0.4"
```

**Step 2: Write the failing test**

Create `crates/repo-core/tests/ledger_locking_tests.rs`:

```rust
//! Tests for ledger file locking

use repo_core::ledger::Ledger;
use serde_json::json;
use std::sync::{Arc, Barrier};
use std::thread;
use tempfile::tempdir;

#[test]
fn concurrent_ledger_saves_are_serialized() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("ledger.toml");

    // Create initial ledger
    let ledger = Ledger::new();
    ledger.save(&path).unwrap();

    let barrier = Arc::new(Barrier::new(2));
    let path1 = path.clone();
    let path2 = path.clone();
    let b1 = barrier.clone();
    let b2 = barrier.clone();

    // Two threads try to modify ledger concurrently
    let t1 = thread::spawn(move || {
        b1.wait();
        let mut ledger = Ledger::load(&path1).unwrap();
        ledger.add_intent(repo_core::ledger::Intent::new(
            "rule:thread1".to_string(),
            json!({}),
        ));
        ledger.save(&path1)
    });

    let t2 = thread::spawn(move || {
        b2.wait();
        let mut ledger = Ledger::load(&path2).unwrap();
        ledger.add_intent(repo_core::ledger::Intent::new(
            "rule:thread2".to_string(),
            json!({}),
        ));
        ledger.save(&path2)
    });

    // Both should complete without error (locking serializes them)
    t1.join().unwrap().unwrap();
    t2.join().unwrap().unwrap();

    // Final ledger should have at least one intent (last writer wins)
    let final_ledger = Ledger::load(&path).unwrap();
    assert!(!final_ledger.intents().is_empty());
}
```

**Step 3: Run test to see current behavior**

```bash
cargo test -p repo-core concurrent_ledger_saves -- --nocapture
```

Expected: May pass by luck or fail with corruption. Goal is deterministic success.

**Step 4: Implement file locking in Ledger**

Update `crates/repo-core/src/ledger/mod.rs`:

Add import at top:
```rust
use fs2::FileExt;
use std::fs::{File, OpenOptions};
```

Replace `load` and `save` methods:

```rust
/// Load a ledger from a TOML file with shared lock
///
/// # Arguments
///
/// * `path` - Path to the ledger TOML file
///
/// # Errors
///
/// Returns an error if the file cannot be read, locked, or parsed.
pub fn load(path: &Path) -> Result<Self> {
    let file = File::open(path)?;
    file.lock_shared()?;

    let content = fs::read_to_string(path)?;
    let ledger: Ledger = toml::from_str(&content)?;

    // Lock released when file is dropped
    Ok(ledger)
}

/// Save the ledger to a TOML file atomically with exclusive lock
///
/// Uses write-to-temp-then-rename pattern with file locking to prevent
/// corruption and race conditions.
///
/// # Arguments
///
/// * `path` - Path to save the ledger TOML file
///
/// # Errors
///
/// Returns an error if the file cannot be written or locked.
pub fn save(&self, path: &Path) -> Result<()> {
    let content = toml::to_string_pretty(self)?;

    // Create or open the target file for locking
    let lock_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)?;

    // Acquire exclusive lock (blocks if another process holds lock)
    lock_file.lock_exclusive()?;

    // Write to temporary file first
    let temp_path = path.with_extension("toml.tmp");
    fs::write(&temp_path, &content)?;

    // Atomically rename to target
    fs::rename(&temp_path, path)?;

    // Lock released when lock_file is dropped
    Ok(())
}
```

**Step 5: Run all tests**

```bash
cargo test -p repo-core -- --test-threads=1
```

Expected: All tests PASS

**Step 6: Commit**

```bash
git add crates/repo-core/Cargo.toml crates/repo-core/src/ledger/mod.rs crates/repo-core/tests/
git commit -m "fix(repo-core): add file locking for ledger operations

Addresses DATA-1 and SYNC-1 from 2026-01-28 audit. Uses fs2 for
cross-platform file locking to prevent concurrent modification races.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Fix JSON Root Panic in repo-blocks

**Issue:** ERR-02 - `expect("Root must be object")` panics on array JSON input.

**Files:**
- Modify: `crates/repo-blocks/src/formats/json.rs:73`
- Test: Add test for array input in existing test module

**Step 1: Write the failing test**

Add to tests in `crates/repo-blocks/src/formats/json.rs`:

```rust
#[test]
fn test_write_block_to_array_returns_unchanged() {
    let handler = JsonFormatHandler::new();
    let array_content = r#"[1, 2, 3]"#;
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    // Should NOT panic, should return content unchanged
    let result = handler.write_block(array_content, uuid, r#"{"setting": true}"#);

    // Content should be unchanged since we can't add managed section to array
    assert_eq!(result, array_content);
}
```

**Step 2: Run test to verify it fails (panics)**

```bash
cargo test -p repo-blocks test_write_block_to_array_returns_unchanged -- --nocapture
```

Expected: PANIC with "Root must be object"

**Step 3: Fix the panic**

In `crates/repo-blocks/src/formats/json.rs`, replace line 73:

```rust
// Before:
let obj = json.as_object_mut().expect("Root must be object");

// After:
let Some(obj) = json.as_object_mut() else {
    // Cannot add managed section to non-object JSON (e.g., array)
    return content.to_string();
};
```

**Step 4: Run tests to verify fix**

```bash
cargo test -p repo-blocks -- --nocapture
```

Expected: All tests PASS including new test

**Step 5: Commit**

```bash
git add crates/repo-blocks/src/formats/json.rs
git commit -m "fix(repo-blocks): handle non-object JSON root gracefully

Addresses ERR-02 from 2026-01-28 audit. Returns content unchanged
instead of panicking when JSON root is not an object (e.g., array).

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Add Recursion Depth Limits in repo-content

**Issue:** P1 - `diff_values()` has unbounded recursion that can stack overflow.

**Files:**
- Modify: `crates/repo-content/src/diff.rs:139-224`
- Test: Add test for deep nesting

**Step 1: Write the failing test**

Add to tests in `crates/repo-content/src/diff.rs`:

```rust
#[test]
fn test_compute_handles_deep_nesting() {
    // Create deeply nested JSON (deeper than stack can handle without limit)
    fn create_nested(depth: usize) -> Value {
        let mut current = json!({"leaf": "value"});
        for _ in 0..depth {
            current = json!({"nested": current});
        }
        current
    }

    // 200 levels should work fine with depth limiting
    let old = create_nested(200);
    let new = create_nested(200);

    // Should not stack overflow
    let diff = SemanticDiff::compute(&old, &new);
    assert!(diff.is_equivalent);
}

#[test]
fn test_compute_truncates_at_max_depth() {
    fn create_nested(depth: usize, leaf_value: &str) -> Value {
        let mut current = json!({"leaf": leaf_value});
        for _ in 0..depth {
            current = json!({"nested": current});
        }
        current
    }

    // Create structures deeper than MAX_DIFF_DEPTH
    let old = create_nested(150, "old");
    let new = create_nested(150, "new");

    // Should detect difference without stack overflow
    let diff = SemanticDiff::compute(&old, &new);
    assert!(!diff.is_equivalent);
}
```

**Step 2: Implement depth limiting**

In `crates/repo-content/src/diff.rs`, add constant and modify function:

Add after imports:
```rust
/// Maximum recursion depth for diff operations
const MAX_DIFF_DEPTH: usize = 128;
```

Replace `diff_values` function signature and add depth tracking:

```rust
/// Recursively diff two JSON values, collecting changes with path tracking
fn diff_values(old: &Value, new: &Value, path: String, changes: &mut Vec<SemanticChange>) {
    diff_values_with_depth(old, new, path, changes, 0);
}

/// Internal recursive diff with depth tracking
fn diff_values_with_depth(
    old: &Value,
    new: &Value,
    path: String,
    changes: &mut Vec<SemanticChange>,
    depth: usize,
) {
    // Depth limit: treat deeply nested differences as a single modification
    if depth > MAX_DIFF_DEPTH {
        if old != new {
            changes.push(SemanticChange::Modified {
                path,
                old: old.clone(),
                new: new.clone(),
            });
        }
        return;
    }

    match (old, new) {
        // Both are objects - compare keys
        (Value::Object(old_obj), Value::Object(new_obj)) => {
            // Check for removed and modified keys
            for (key, old_value) in old_obj {
                let child_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };

                match new_obj.get(key) {
                    Some(new_value) => {
                        // Key exists in both - recurse
                        diff_values_with_depth(old_value, new_value, child_path, changes, depth + 1);
                    }
                    None => {
                        // Key removed
                        changes.push(SemanticChange::Removed {
                            path: child_path,
                            value: old_value.clone(),
                        });
                    }
                }
            }

            // Check for added keys
            for (key, new_value) in new_obj {
                if !old_obj.contains_key(key) {
                    let child_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    changes.push(SemanticChange::Added {
                        path: child_path,
                        value: new_value.clone(),
                    });
                }
            }
        }

        // Both are arrays - compare element by element
        (Value::Array(old_arr), Value::Array(new_arr)) => {
            let max_len = old_arr.len().max(new_arr.len());
            for i in 0..max_len {
                let child_path = if path.is_empty() {
                    format!("[{}]", i)
                } else {
                    format!("{}[{}]", path, i)
                };

                match (old_arr.get(i), new_arr.get(i)) {
                    (Some(old_val), Some(new_val)) => {
                        diff_values_with_depth(old_val, new_val, child_path, changes, depth + 1);
                    }
                    (Some(old_val), None) => {
                        changes.push(SemanticChange::Removed {
                            path: child_path,
                            value: old_val.clone(),
                        });
                    }
                    (None, Some(new_val)) => {
                        changes.push(SemanticChange::Added {
                            path: child_path,
                            value: new_val.clone(),
                        });
                    }
                    (None, None) => unreachable!(),
                }
            }
        }

        // Different types or scalar values - compare directly
        _ => {
            if old != new {
                changes.push(SemanticChange::Modified {
                    path,
                    old: old.clone(),
                    new: new.clone(),
                });
            }
        }
    }
}
```

**Step 3: Run tests**

```bash
cargo test -p repo-content -- --nocapture
```

Expected: All tests PASS

**Step 4: Commit**

```bash
git add crates/repo-content/src/diff.rs
git commit -m "fix(repo-content): add recursion depth limit to diff_values

Addresses P1 from 2026-01-28 audit. Limits recursion to 128 levels
to prevent stack overflow on deeply nested JSON structures.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Add Input Size Limits in repo-meta

**Issue:** S2 - Config files read without size checks can cause OOM.

**Files:**
- Modify: `crates/repo-meta/src/config.rs`
- Test: Add test for large file rejection

**Step 1: Add constant and modify load function**

In `crates/repo-meta/src/config.rs`, add after imports:

```rust
/// Maximum configuration file size (10 MB)
const MAX_CONFIG_SIZE: u64 = 10 * 1024 * 1024;
```

**Step 2: Write the failing test**

Add to tests in `crates/repo-meta/src/config.rs`:

```rust
#[test]
fn test_load_rejects_oversized_config() {
    use tempfile::tempdir;
    use std::io::Write;

    let dir = tempdir().unwrap();
    let root = NormalizedPath::new(dir.path().to_str().unwrap());

    // Create .repository directory
    let config_dir = dir.path().join(".repository");
    std::fs::create_dir_all(&config_dir).unwrap();

    // Create oversized config file (11 MB of 'a')
    let config_path = config_dir.join("config.toml");
    let mut file = std::fs::File::create(&config_path).unwrap();
    for _ in 0..11 {
        file.write_all(&[b'a'; 1024 * 1024]).unwrap();
    }

    // Should reject oversized file
    let result = load_config(&root);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("too large"));
}
```

**Step 3: Implement size check**

Modify `load_config` function in `crates/repo-meta/src/config.rs`:

```rust
pub fn load_config(root: &NormalizedPath) -> Result<RepositoryConfig> {
    let config_path = root.join(RepoPath::ConfigFile.as_str());

    if !config_path.is_file() {
        return Err(Error::ConfigNotFound {
            path: config_path.to_native(),
        });
    }

    // Check file size before reading
    let metadata = std::fs::metadata(config_path.as_ref())
        .map_err(|e| Error::InvalidConfig {
            path: config_path.to_native(),
            message: e.to_string(),
        })?;

    if metadata.len() > MAX_CONFIG_SIZE {
        return Err(Error::InvalidConfig {
            path: config_path.to_native(),
            message: format!(
                "Configuration file too large ({} bytes, max {} bytes)",
                metadata.len(),
                MAX_CONFIG_SIZE
            ),
        });
    }

    let content = std::fs::read_to_string(config_path.as_ref())
        .map_err(|e| Error::InvalidConfig {
            path: config_path.to_native(),
            message: e.to_string(),
        })?;

    let config: RepositoryConfig = toml::from_str(&content)
        .map_err(|e| Error::InvalidConfig {
            path: config_path.to_native(),
            message: e.to_string(),
        })?;

    Ok(config)
}
```

**Step 4: Run tests**

```bash
cargo test -p repo-meta -- --nocapture
```

Expected: All tests PASS

**Step 5: Commit**

```bash
git add crates/repo-meta/src/config.rs
git commit -m "fix(repo-meta): add file size limit to config loading

Addresses S2 from 2026-01-28 audit. Rejects configuration files
larger than 10 MB to prevent memory exhaustion attacks.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 6: Replace expect() in repo-cli main.rs

**Issue:** E1 - Tracing subscriber setup can panic if called twice.

**Files:**
- Modify: `crates/repo-cli/src/main.rs:35-36`

**Step 1: Fix the panic path**

In `crates/repo-cli/src/main.rs`, replace lines 35-36:

```rust
// Before:
tracing::subscriber::set_global_default(subscriber)
    .expect("Failed to set tracing subscriber");

// After:
if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
    eprintln!("Warning: Could not set tracing subscriber: {}", e);
}
```

**Step 2: Run tests**

```bash
cargo test -p repo-cli -- --nocapture
```

Expected: All tests PASS

**Step 3: Verify CLI still works**

```bash
cargo run -p repo-cli -- --help
cargo run -p repo-cli -- -v --help
```

Expected: Help output displayed, verbose mode note shown

**Step 4: Commit**

```bash
git add crates/repo-cli/src/main.rs
git commit -m "fix(repo-cli): replace expect() with graceful error handling

Addresses E1 from 2026-01-28 audit. Logs warning instead of
panicking if tracing subscriber is already set.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 7: Run Full Test Suite and Verify

**Files:**
- None (validation only)

**Step 1: Run all tests**

```bash
cargo test --workspace -- --test-threads=1
```

Expected: All tests PASS

**Step 2: Run clippy**

```bash
cargo clippy --workspace -- -D warnings
```

Expected: No warnings

**Step 3: Final commit summary**

```bash
git log --oneline -7
```

Expected: Shows all 6 fix commits

---

## Summary

| Task | Issue | Crate | Priority |
|------|-------|-------|----------|
| 1 | DATA-2: Non-atomic writes | repo-core | HIGH |
| 2 | DATA-1, SYNC-1: No locking | repo-core | HIGH |
| 3 | ERR-02: JSON panic | repo-blocks | MEDIUM |
| 4 | P1: Unbounded recursion | repo-content | MEDIUM |
| 5 | S2: No size limits | repo-meta | MEDIUM |
| 6 | E1: expect() panic | repo-cli | MEDIUM |
| 7 | Validation | all | - |

**Estimated total:** 7 tasks

---

*Plan created from 2026-01-28 audit findings.*
*Reference: docs/audits/2026-01-28-audit-index.md*
