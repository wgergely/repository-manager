# Design: Immediate & Short-Term Fixes

**Date**: 2026-01-26
**Status**: Approved
**Scope**: Address audit findings for robustness, testing, and spec compliance

---

## Overview

This design addresses six concerns identified in the code audit:

| Priority | Item | Approach |
|----------|------|----------|
| Immediate | Dry-run flag for sync/fix | Engine-level `SyncOptions` struct |
| Immediate | Symlink vulnerability | Detect and reject symlinks in path |
| Immediate | Error injection tests | Real filesystem conditions |
| Short-term | Concurrent access tests | Multi-threaded stress tests |
| Short-term | Tool/preset validation | Registry-based with warnings |
| Short-term | Default mode correction | Change to `worktrees` |

---

## 1. Dry-Run Feature

### Design

Add `SyncOptions` struct to `repo-core/src/sync/engine.rs`:

```rust
#[derive(Debug, Clone, Default)]
pub struct SyncOptions {
    pub dry_run: bool,
}
```

Modify `SyncEngine`:
- Add `sync_with_options(&self, options: SyncOptions) -> Result<SyncReport>`
- Add `fix_with_options(&self, options: SyncOptions) -> Result<SyncReport>`
- Existing `sync()` and `fix()` call `*_with_options` with defaults

When `dry_run: true`:
- Prefix actions with `[dry-run] Would ...`
- Skip all filesystem writes
- Still perform validation and checks

### CLI Changes

Add to `Sync` and `Fix` commands in `cli.rs`:
```rust
#[arg(long)]
dry_run: bool,
```

Output prefixed with `[dry-run]` when active.

---

## 2. Symlink Vulnerability Fix

### Design

Add symlink detection before any write in `repo-fs/src/io.rs`:

```rust
fn contains_symlink(path: &Path) -> std::io::Result<bool> {
    let mut current = path.to_path_buf();
    while let Some(parent) = current.parent() {
        if current.exists() {
            let metadata = std::fs::symlink_metadata(&current)?;
            if metadata.file_type().is_symlink() {
                return Ok(true);
            }
        }
        current = parent.to_path_buf();
        if current.as_os_str().is_empty() {
            break;
        }
    }
    Ok(false)
}
```

Add error variant to `repo-fs/src/error.rs`:
```rust
#[error("Refusing to write through symlink: {path}")]
SymlinkInPath { path: PathBuf },
```

Call `contains_symlink` at start of `write_atomic`, return error if true.

### Test Updates

Update `security_audit_tests.rs` to expect failure instead of documenting vulnerability.

---

## 3. Error Injection Tests

### New File: `repo-fs/tests/error_condition_tests.rs`

Tests using real filesystem conditions:

| Test | Condition | Platform |
|------|-----------|----------|
| `test_write_atomic_permission_denied_directory` | Read-only parent dir | Unix |
| `test_write_atomic_permission_denied_existing_file` | Read-only target file | Unix |
| `test_read_text_nonexistent_file` | Missing file | All |
| `test_read_text_permission_denied` | Unreadable file | Unix |

Windows tests use `FILE_ATTRIBUTE_READONLY` where applicable.

All tests restore permissions in cleanup to avoid test pollution.

---

## 4. Concurrent Access Tests

### New File: `repo-fs/tests/concurrency_tests.rs`

| Test | Purpose |
|------|---------|
| `test_concurrent_writes_no_corruption` | 10 threads, 20 writes each, verify no corruption |
| `test_concurrent_writes_all_succeed_eventually` | Different files, all should succeed |
| `test_lock_timeout_respected` | Manually hold lock, verify timeout behavior |

Uses `std::sync::Barrier` to synchronize thread start for maximum contention.

---

## 5. Tool/Preset Validation

### New File: `repo-meta/src/tools.rs`

```rust
pub struct ToolRegistry {
    known_tools: HashSet<&'static str>,
}

impl ToolRegistry {
    pub fn with_builtins() -> Self {
        // claude, claude-desktop, cursor, vscode, windsurf, gemini-cli, antigravity
    }

    pub fn is_known(&self, name: &str) -> bool;
    pub fn list_known(&self) -> Vec<&'static str>;
}
```

### CLI Integration

In `run_add_tool` and `run_add_preset`:
- Check against registry
- Print warning (yellow) if unknown
- Continue execution (don't block)

---

## 6. Default Mode Change

### Change

In `repo-cli/src/cli.rs`, `Init` command:
```rust
#[arg(short, long, default_value = "worktrees")]  // was "standard"
mode: String,
```

### Test Updates

- Rename `test_init_default_mode_is_standard` to `test_init_default_mode_is_worktrees`
- Update assertions to expect `worktrees` mode
- Verify `main/` directory is created by default

---

## Files Changed

| Crate | File | Change |
|-------|------|--------|
| repo-fs | `src/io.rs` | Add symlink detection |
| repo-fs | `src/error.rs` | Add `SymlinkInPath` variant |
| repo-fs | `tests/error_condition_tests.rs` | New file |
| repo-fs | `tests/concurrency_tests.rs` | New file |
| repo-fs | `tests/security_audit_tests.rs` | Update expectations |
| repo-core | `src/sync/engine.rs` | Add `SyncOptions`, `*_with_options` methods |
| repo-meta | `src/tools.rs` | New file |
| repo-meta | `src/lib.rs` | Export `ToolRegistry` |
| repo-cli | `src/cli.rs` | Add `--dry-run`, change default mode |
| repo-cli | `src/commands/sync.rs` | Pass dry_run to engine |
| repo-cli | `src/commands/tool.rs` | Add validation warnings |
| repo-cli | `tests/integration_tests.rs` | Update default mode tests |

---

## Testing Strategy

All changes tested via:
1. Unit tests in respective modules
2. Integration tests in `repo-cli/tests/`
3. Existing test suite must pass

No mocking - all tests use real filesystem operations.

---

## Rollout

1. Implement in feature branch
2. Run full test suite
3. Manual testing of CLI commands
4. PR with audit findings as context
