# Phase 2: Shared Test Fixture Library

**Priority**: P3 (test hygiene)
**Audit ID**: TH-2
**Domain**: New `crates/repo-test-utils/` crate + fixture consolidation
**Status**: Not started
**Prerequisite**: None — fully independent
**Estimated scope**: 1 new crate, ~200 lines new code, 6+ files updated to use shared fixtures

---

## Testing Mandate

> **Inherited from [tasks/_index.md](../tasks/_index.md).** Read and internalize the
> full Testing Mandate before writing any code.

**Domain-specific enforcement:**
- Extracted fixtures must produce **identical test behavior** — no test should change
  its pass/fail result.
- Every fixture function must have a doc-comment explaining what it creates and what
  "realism level" it provides (fake filesystem vs real git).
- Do NOT add new tests in this phase. This is purely structural refactoring.

---

## Problem Statement

`setup_git_repo()` is defined 4+ times across the codebase with inconsistent
implementations (some fake, some real). The best fixture (`TestRepo` builder in
`mission_tests.rs`) is not shared. Only 1 shared test module exists in the entire
workspace (`repo-presets/tests/common/mod.rs`, 13 lines).

See: [Test Health Audit TH-2](../audits/2026-02-22-test-health-audit.md#finding-th-2-fixture-duplication)

---

## Implementation Plan

### Step 1: Create `crates/repo-test-utils/` crate

Create a dev-dependency-only crate for shared test infrastructure.

**`Cargo.toml`:**
```toml
[package]
name = "repo-test-utils"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
tempfile = { workspace = true }
git2 = { workspace = true }
repo-fs = { path = "../repo-fs" }
```

**`src/lib.rs`:**
```rust
//! Shared test utilities for the repository-manager workspace.
//!
//! This crate provides standardized test fixtures to eliminate duplication
//! across crate test suites. It is a dev-dependency only — never published.

pub mod git;
pub mod repo;
```

### Step 2: Extract `TestRepo` builder from `mission_tests.rs`

The `TestRepo` struct in `tests/integration/src/mission_tests.rs:31-123` is the best
fixture in the codebase. Extract it into `repo-test-utils/src/repo.rs`.

**Source:** `tests/integration/src/mission_tests.rs` lines 31-123
**Target:** `crates/repo-test-utils/src/repo.rs`

The extracted builder should provide:
- `TestRepo::new()` — empty temp directory
- `TestRepo::init_git()` — real git repo via `git2::Repository::init()`
- `TestRepo::init_repo_manager(mode, tools, presets)` — writes valid `config.toml`
- `TestRepo::root()` → `&Path`
- `TestRepo::assert_file_exists(path)`
- `TestRepo::assert_file_not_exists(path)`
- `TestRepo::assert_file_contains(path, content)`

### Step 3: Extract git fixture functions

Create `crates/repo-test-utils/src/git.rs` with two clearly-labeled fixture levels:

```rust
//! Git repository fixtures at different realism levels.

/// Creates a fake .git directory structure (no real git state).
/// Use for tests that only check path existence, not git operations.
pub fn fake_git_dir(path: &Path) { ... }

/// Creates a real git repository via git2.
/// Use for tests that need actual git operations (branch, commit, etc).
pub fn real_git_repo(path: &Path) -> git2::Repository { ... }

/// Creates a real bare git repo + worktree structure for container mode.
/// Use for tests that exercise ContainerLayout.
pub fn real_container_repo(path: &Path) -> (git2::Repository, PathBuf) { ... }
```

### Step 4: Update `mission_tests.rs` to use shared crate

**File:** `tests/integration/src/mission_tests.rs`
**Change:** Replace the local `TestRepo` definition with `use repo_test_utils::repo::TestRepo;`

Add `repo-test-utils` as a dev-dependency in `tests/integration/Cargo.toml`.

**Verification:** `cargo test -p integration-tests` — identical results.

### Step 5: Update `repo-core` test files to use shared fixtures

Replace duplicated `setup_git_repo()` / `setup_standard_repo()` in:

| File | Current Fixture | Replace With |
|------|----------------|--------------|
| `crates/repo-core/src/backend/standard.rs:170-176` | `setup_git_repo()` (fake) | `repo_test_utils::git::fake_git_dir()` |
| `crates/repo-core/tests/mode_tests.rs:57-71` | `setup_standard_repo()` (fake) | `repo_test_utils::git::fake_git_dir()` |
| `crates/repo-core/tests/sync_tests.rs:14-20` | `setup_git_repo()` (fake) | `repo_test_utils::git::fake_git_dir()` |

Add `repo-test-utils` as a dev-dependency in `crates/repo-core/Cargo.toml`.

**Verification:** `cargo test -p repo-core` — identical results.

### Step 6: Update `repo-cli` test fixtures

**File:** `crates/repo-cli/src/commands/branch.rs:172-183`
**Change:** Replace inline `setup_git_repo()` with `repo_test_utils::git::real_git_repo()`.

Add `repo-test-utils` as a dev-dependency in `crates/repo-cli/Cargo.toml`.

**Verification:** `cargo test -p repo-cli` — identical results.

### Step 7: Update workspace `Cargo.toml`

Add `repo-test-utils` to workspace members and dev-dependencies:

```toml
[workspace]
members = [
    "crates/*",
    "tests/integration",
]

[workspace.dependencies]
repo-test-utils = { path = "crates/repo-test-utils" }
```

### Step 8: Delete `repo-presets/tests/common/mod.rs`

The existing 13-line `common/mod.rs` creates a `Context` with hardcoded values. Replace
callers with `TestRepo::new().init_repo_manager(...)` and derive the context from that.

If the `create_test_context()` function serves a distinct purpose (preset-specific
context creation), move it into `repo-test-utils/src/presets.rs` instead.

---

## Acceptance Criteria

- [ ] `crates/repo-test-utils/` exists with `lib.rs`, `git.rs`, `repo.rs`
- [ ] `TestRepo` builder is shared from `repo-test-utils`, not duplicated in `mission_tests.rs`
- [ ] Zero copies of `setup_git_repo()` remain outside `repo-test-utils`
- [ ] Zero copies of `setup_standard_repo()` remain outside `repo-test-utils`
- [ ] Every fixture function has a doc-comment with realism level
- [ ] `cargo test` passes with identical test count across all crates
- [ ] `cargo clippy` clean

---

## Files to Create

| File | Purpose |
|------|---------|
| `crates/repo-test-utils/Cargo.toml` | Crate manifest |
| `crates/repo-test-utils/src/lib.rs` | Module root |
| `crates/repo-test-utils/src/repo.rs` | `TestRepo` builder (extracted from mission_tests.rs) |
| `crates/repo-test-utils/src/git.rs` | Git fixture functions (3 levels) |

## Files to Modify

| File | Change |
|------|--------|
| `Cargo.toml` (workspace) | Add member + workspace dep |
| `tests/integration/Cargo.toml` | Add dev-dep |
| `tests/integration/src/mission_tests.rs` | Import from shared crate |
| `crates/repo-core/Cargo.toml` | Add dev-dep |
| `crates/repo-core/src/backend/standard.rs` | Use shared fixture |
| `crates/repo-core/tests/mode_tests.rs` | Use shared fixture |
| `crates/repo-core/tests/sync_tests.rs` | Use shared fixture |
| `crates/repo-cli/Cargo.toml` | Add dev-dep |
| `crates/repo-cli/src/commands/branch.rs` | Use shared fixture |

---

## Dependencies

- **Depends on**: Nothing
- **Blocks**: [Phase 3](2026-02-22-test-health-phase3-format-validation.md) (format validation tests will use `TestRepo`)
- **Can parallelize with**: [Phase 1](2026-02-22-test-health-phase1-taxonomy.md)

---

## Cross-References

- **Source finding**: [Test Health Audit TH-2](../audits/2026-02-22-test-health-audit.md#finding-th-2-fixture-duplication)
- **Best existing pattern**: `tests/integration/src/mission_tests.rs:31-123` (the `TestRepo` to extract)
- **Related task**: [P3 Test Hygiene](../tasks/P3-test-hygiene.md)
- **Research**: [Testing practices](../research/) — Rust test organization patterns
- **Chain**: Phase 2 of 5 — prev: [Phase 1](2026-02-22-test-health-phase1-taxonomy.md), next: [Phase 3](2026-02-22-test-health-phase3-format-validation.md)

---

*Plan created: 2026-02-22*
