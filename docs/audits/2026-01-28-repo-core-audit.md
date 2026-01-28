# repo-core Crate Audit - 2026-01-28

## Executive Summary

The `repo-core` crate serves as the central orchestration layer for Repository Manager, coordinating sync operations, ledger management, configuration resolution, and backend abstractions. This audit examined **25 source files** totaling approximately **4,500 lines of code**.

**Overall Assessment: GOOD with minor improvements recommended**

The crate demonstrates solid architecture with proper error handling, good separation of concerns, and defensive coding practices. Key strengths include:
- Comprehensive use of the `Result` type for error propagation
- Symlink-safe file writes via `repo_fs::io::write_text`
- No `unsafe` code blocks
- No explicit `panic!` macros in production code
- Proper use of `thiserror` for error types

Areas requiring attention:
- Non-test `.unwrap()` calls in production paths (3 instances)
- TOCTOU (time-of-check-time-of-use) race conditions in file operations
- No file locking on ledger operations
- Missing atomic write patterns for critical data

## Crate Overview

### Module Structure

| Module | Purpose | Lines |
|--------|---------|-------|
| `sync/` | SyncEngine, check/sync/fix operations | ~1,100 |
| `ledger/` | Intent and projection tracking | ~300 |
| `config/` | Configuration resolution and manifest parsing | ~400 |
| `backend/` | Standard and Worktree mode abstractions | ~450 |
| `backup/` | Tool configuration backup/restore | ~400 |
| `projection/` | ProjectionWriter for filesystem operations | ~350 |
| `rules/` | Rule registry with UUID-based identification | ~300 |
| `error.rs` | Error type definitions | ~80 |
| `mode.rs` | Mode enum (Standard/Worktrees) | ~80 |

### Key Components

1. **SyncEngine** (`sync/engine.rs`): Coordinates state between ledger and filesystem
2. **Ledger** (`ledger/mod.rs`): Persisted registry of intents and projections
3. **ConfigResolver** (`config/resolver.rs`): Hierarchical configuration merge
4. **ModeBackend** (`backend/mod.rs`): Trait for Standard/Worktree operations
5. **BackupManager** (`backup/tool_backup.rs`): Tool configuration backup/restore
6. **ProjectionWriter** (`projection/writer.rs`): Safe filesystem writes

## Findings

### Security

#### SEC-1: Symlink Protection (GOOD)
**Location:** `projection/writer.rs:13-14`, `sync/tool_syncer.rs:17-20`

The crate uses `repo_fs::io::write_text` for file writes, which provides symlink protection:
```rust
fn safe_write(path: &NormalizedPath, content: &str) -> Result<()> {
    repo_fs::io::write_text(path, content).map_err(|e| Error::Io(std::io::Error::other(e.to_string())))
}
```
This prevents path traversal attacks through symbolic links.

#### SEC-2: Shell Command Injection Prevention (GOOD)
**Location:** `backend/standard.rs:45-59`, `backend/worktree.rs:85-99`

Git commands are executed using `Command::new("git").args(...)` which properly escapes arguments:
```rust
fn git_command(&self, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(self.root.to_native())
        .output()
        .map_err(Error::Io)?;
```
Arguments are passed as a slice, not concatenated into a shell string.

#### SEC-3: Configuration Injection Risk (LOW RISK)
**Location:** `config/resolver.rs:96-122`

Configuration files are parsed directly from TOML without sanitization. While this is generally safe due to TOML's structured nature, malicious configuration values could affect tool behavior:
```rust
pub fn resolve(&self) -> Result<ResolvedConfig> {
    // Layer 3 - Repository config
    let repo_config_path = self.root.join(".repository/config.toml");
    if repo_config_path.is_file() {
        let content = fs::read_to_string(repo_config_path.to_native())?;
        let repo_manifest = Manifest::parse(&content)?;
        manifest.merge(&repo_manifest);
    }
}
```
**Recommendation:** Consider validating tool names and paths before use.

#### SEC-4: No Secrets Exposure Risk (GOOD)
The crate does not handle credentials or sensitive data. Configuration files store tool names and rules, not secrets.

### Performance

#### PERF-1: Ledger Save on Every Rule Change (MODERATE)
**Location:** `rules/registry.rs:79-89`

The registry saves to disk after every `add_rule` operation:
```rust
pub fn add_rule(&mut self, id: &str, content: &str, tags: Vec<String>) -> Result<&Rule> {
    let rule = Rule::new(id, content, tags);
    self.rules.push(rule);
    self.save()?;  // Disk I/O on every add
    Ok(self.rules.last().unwrap())
}
```
**Recommendation:** Consider batching saves or providing a `save_later` pattern for bulk operations.

#### PERF-2: Linear Search in Ledger (LOW IMPACT)
**Location:** `ledger/mod.rs:83-101`

Intent lookups are O(n) linear scans:
```rust
pub fn remove_intent(&mut self, uuid: Uuid) -> Option<Intent> {
    let pos = self.intents.iter().position(|i| i.uuid == uuid)?;
    Some(self.intents.remove(pos))
}

pub fn find_by_rule(&self, rule_id: &str) -> Vec<&Intent> {
    self.intents.iter().filter(|i| i.id == rule_id).collect()
}
```
For typical use cases (tens to hundreds of intents), this is acceptable. For larger repositories, consider adding a HashMap index.

#### PERF-3: Full File Read for JSON Key Operations (LOW IMPACT)
**Location:** `projection/writer.rs:113-148`

JSON key updates require reading and rewriting the entire file:
```rust
fn write_json_key(&self, path: &NormalizedPath, key_path: &str, value: &str) -> Result<String> {
    let existing = if path.exists() {
        fs::read_to_string(path.as_ref())?
    } else {
        "{}".to_string()
    };
    // ... parse, modify, rewrite entire file
}
```
This is necessary for JSON integrity but could be slow for very large config files.

### Memory Safety

#### MEM-1: No Unsafe Code (EXCELLENT)
The crate contains zero `unsafe` blocks. All operations use safe Rust abstractions.

#### MEM-2: Bounded String Operations (GOOD)
String operations use Rust's safe string handling. Path operations use `NormalizedPath` which provides safe path manipulation.

#### MEM-3: No Unbounded Allocations (GOOD)
Data structures grow proportionally to repository size. No patterns that could cause unbounded memory growth.

### Data Integrity

#### DATA-1: TOCTOU Race Conditions (MODERATE RISK)
**Location:** Multiple files

Several check-then-act patterns exist without locking:

```rust
// sync/engine.rs:120
if path.exists() {
    Ledger::load(path.as_ref())
} else {
    Ok(Ledger::new())
}

// backup/tool_backup.rs:94
if source.exists() {
    // ... copy operations
}

// projection/writer.rs:155
if path.exists() {
    fs::remove_file(path.as_ref())?;
}
```

Between `exists()` and the subsequent operation, another process could modify the file. This is especially concerning for ledger operations where concurrent syncs could cause data loss.

**Recommendation:**
1. Use file locking (e.g., `fs2::FileExt::lock_exclusive`) for ledger operations
2. Use atomic write patterns (write to temp, then rename)

#### DATA-2: Non-Atomic Ledger Writes (MODERATE RISK)
**Location:** `ledger/mod.rs:64-68`

Ledger saves are not atomic:
```rust
pub fn save(&self, path: &Path) -> Result<()> {
    let content = toml::to_string_pretty(self)?;
    fs::write(path, content)?;
    Ok(())
}
```
If the process crashes during `fs::write`, the ledger file could be corrupted.

**Recommendation:** Write to a temporary file, then atomically rename:
```rust
pub fn save(&self, path: &Path) -> Result<()> {
    let content = toml::to_string_pretty(self)?;
    let temp_path = path.with_extension("toml.tmp");
    fs::write(&temp_path, content)?;
    fs::rename(&temp_path, path)?;
    Ok(())
}
```

#### DATA-3: Backup Integrity (GOOD)
**Location:** `backup/tool_backup.rs`

Backup operations properly:
1. Create backup directory before copying files
2. Store metadata with file list and timestamp
3. Verify source existence before backup
4. Create parent directories on restore

#### DATA-4: Checksum Verification (GOOD)
**Location:** `sync/engine.rs:176-209`, `projection/writer.rs:272-276`

SHA-256 checksums are computed for drift detection:
```rust
pub fn compute_checksum(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

### Error Handling

#### ERR-1: Production Unwrap Calls (LOW RISK)
**Location:** 3 instances in non-test code

1. `rules/registry.rs:88`:
```rust
Ok(self.rules.last().unwrap())
```
This is safe because we just pushed to the vector, but should use `.expect()` for clarity.

2. `projection/writer.rs:82`:
```rust
let start_idx = existing.find(&marker_start).unwrap();
```
This is after a `contains()` check, so it's safe, but could be refactored to avoid the redundant check.

3. `projection/writer.rs:267`:
```rust
map.remove(*parts.last().unwrap());
```
Safe because we check `!parts.is_empty()` earlier, but could be clearer.

**Recommendation:** Replace with `.expect()` with descriptive messages or restructure to avoid `.unwrap()`.

#### ERR-2: Comprehensive Error Types (EXCELLENT)
**Location:** `error.rs`

The error enum properly wraps all underlying crate errors:
```rust
pub enum Error {
    ConfigNotFound { path: PathBuf },
    InvalidMode { mode: String },
    LedgerError { message: String },
    // ... transparent wrappers for dependencies
    #[error(transparent)]
    Fs(#[from] repo_fs::Error),
    #[error(transparent)]
    Git(#[from] repo_git::Error),
    // ...
}
```

#### ERR-3: Graceful Degradation (GOOD)
**Location:** `sync/engine.rs:155-167`

Ledger load errors are handled gracefully:
```rust
let ledger = match self.load_ledger() {
    Ok(l) => l,
    Err(e) => {
        return Ok(CheckReport::broken(format!("Failed to load ledger: {}", e)));
    }
};
```

#### ERR-4: Silent Failure in remove_rule (LOW RISK)
**Location:** `rules/registry.rs:122-127`

```rust
pub fn remove_rule(&mut self, uuid: Uuid) -> Option<Rule> {
    let pos = self.rules.iter().position(|r| r.uuid == uuid)?;
    let rule = self.rules.remove(pos);
    self.save().ok()?;  // Save errors silently ignored
    Some(rule)
}
```
**Recommendation:** Return `Result` or log the error.

### Sync Engine Race Conditions

#### SYNC-1: Concurrent Sync Operations (MODERATE RISK)
**Location:** `sync/engine.rs:344-406`

Multiple sync operations running simultaneously could:
1. Both read the same ledger state
2. Make conflicting changes
3. One overwrites the other's changes

The engine has no locking mechanism:
```rust
pub fn sync_with_options(&self, options: SyncOptions) -> Result<SyncReport> {
    let mut ledger = self.load_ledger()?;
    // ... modifications
    if !options.dry_run {
        self.save_ledger(&ledger)?;
    }
}
```

**Recommendation:** Implement file locking or use a lock file pattern.

#### SYNC-2: Git Command Races (LOW RISK)
**Location:** `backend/standard.rs`, `backend/worktree.rs`

Git commands execute without coordination. Running multiple commands simultaneously could cause Git lock errors, but Git handles this internally.

### Backend Pattern Consistency

#### BACKEND-1: Consistent Interface (EXCELLENT)
Both `StandardBackend` and `WorktreeBackend` implement the `ModeBackend` trait consistently:
- `config_root()` - returns configuration directory
- `working_dir()` - returns working directory
- `create_branch()` - creates branch/worktree
- `delete_branch()` - removes branch/worktree
- `list_branches()` - enumerates branches
- `switch_branch()` - changes active context

#### BACKEND-2: Error Handling Consistency (GOOD)
Both backends return consistent error types using the shared `Error` enum.

## Recommendations

### Critical Priority

1. **Add File Locking for Ledger Operations**
   - Use `fs2::FileExt::lock_exclusive()` or similar
   - Prevents concurrent modification corruption
   - Estimated effort: 2-4 hours

2. **Implement Atomic Ledger Writes**
   - Write to temporary file, then rename
   - Prevents corruption on crash
   - Estimated effort: 1-2 hours

### High Priority

3. **Add Sync Operation Locking**
   - Create a lock file during sync operations
   - Prevents concurrent syncs from conflicting
   - Estimated effort: 2-4 hours

4. **Replace Production `unwrap()` Calls**
   - 3 instances to fix
   - Use `.expect()` with descriptive messages or restructure
   - Estimated effort: 30 minutes

### Medium Priority

5. **Add Configuration Validation**
   - Validate tool names against allowed list
   - Validate file paths for suspicious patterns
   - Estimated effort: 2-4 hours

6. **Consider Batch Registry Operations**
   - Add `add_rules_batch()` method
   - Reduces disk I/O for bulk imports
   - Estimated effort: 1-2 hours

### Low Priority

7. **Add Ledger Index for Large Repositories**
   - HashMap index by intent ID
   - Only needed if performance becomes an issue
   - Estimated effort: 2-4 hours

8. **Add Telemetry/Metrics**
   - Track sync duration, file counts
   - Useful for debugging and optimization
   - Estimated effort: 4-8 hours

## Test Coverage Notes

The crate has comprehensive test coverage:
- All major components have unit tests
- Tests use `tempfile::TempDir` for isolation
- Mock/stub patterns used appropriately

Most `.unwrap()` calls in the codebase are in test code, which is acceptable practice.

## Conclusion

The `repo-core` crate is well-designed with solid error handling and good security practices. The main areas for improvement are around concurrent access and atomic operations. The recommended changes would bring the crate to production-ready status for multi-user or high-reliability scenarios.

---
*Audit performed by Claude Opus 4.5*
*Files reviewed: 25*
*Lines of code: ~4,500*
