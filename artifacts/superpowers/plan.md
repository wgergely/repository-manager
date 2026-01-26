# Rust Crate Review Implementation Plan

## Goal

Fix error handling issues and establish consistent logging patterns across all Rust crates to ensure:

- No errors are silently swallowed
- Warning and failure conditions are properly logged
- Crates follow modern Rust best practices

## Assumptions

1. The existing API contracts remain unchanged (fixes are internal behavior)
2. `tracing` crate is already a workspace dependency and initialized at runtime
3. Tests pass before and after changes
4. Changes follow the fixed pattern already established in `in_repo_worktrees.rs`

## Plan

### Step 1: Fix Swallowed Error in container.rs

**Files**: [container.rs](file:///y:/code/repository-manager-worktrees/feature-rust-core/crates/repo-git/src/container.rs)

**Change**: Replace `let _ = branch.delete()` at line 169 with proper error logging:

```rust
if let Err(e) = branch.delete() {
    tracing::warn!(
        branch = %dir_name,
        error = %e,
        "Failed to delete branch after worktree removal"
    );
}
```

**Verify**:

```bash
cargo test -p repo-git -- container
cargo clippy -p repo-git
```

---

### Step 2: Add tracing instrumentation to key repo-git operations

**Files**: [container.rs](file:///y:/code/repository-manager-worktrees/feature-rust-core/crates/repo-git/src/container.rs), [in_repo_worktrees.rs](file:///y:/code/repository-manager-worktrees/feature-rust-core/crates/repo-git/src/in_repo_worktrees.rs)

**Change**: Add `tracing::debug!` at entry points of key methods:

- `create_feature` - log branch name and base
- `remove_feature` - log branch name

**Verify**:

```bash
cargo test -p repo-git
cargo clippy -p repo-git
```

---

### Step 3: Add tracing instrumentation to repo-fs write operations

**Files**: [io.rs](file:///y:/code/repository-manager-worktrees/feature-rust-core/crates/repo-git/src/io.rs)

**Change**: Add `tracing::debug!` at entry of `write_atomic`:

```rust
tracing::debug!(path = %path.as_str(), "Starting atomic write");
```

**Verify**:

```bash
cargo test -p repo-fs
cargo clippy -p repo-fs
```

---

### Step 4: Fix unused import warning in robustness_tests.rs

**Files**: [robustness_tests.rs](file:///y:/code/repository-manager-worktrees/feature-rust-core/crates/repo-fs/tests/robustness_tests.rs)

**Change**: Remove `use std::fs;` unused import at line 3.

**Verify**:

```bash
cargo test -p repo-fs
```

---

### Step 5: Full verification

**Verify**:

```bash
cargo check --all-targets
cargo test --all
cargo clippy --all -- -D warnings
```

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Adding tracing imports breaks builds | Use `tracing` which is already a workspace dependency |
| New log messages flood output | Use `debug!` level which is off by default |
| Tests fail due to behavior changes | No behavior changes, only additional logging |

## Rollback Plan

All changes are additive logging. Rollback by reverting the commit:

```bash
git revert HEAD
```

---

## Existing Test Commands

The following commands are already available to verify changes:

| Command | Purpose |
|---------|---------|
| `cargo test -p repo-git -- container` | Tests container.rs remove_feature |
| `cargo test -p repo-git -- in_repo` | Tests in_repo_worktrees.rs |
| `cargo test -p repo-fs` | Tests io.rs operations |
| `cargo test --all` | Full test suite |
| `cargo clippy --all` | Lint all crates |
