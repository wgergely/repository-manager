---
tags: ["#audit", "#health-audit"]
related: ["[[2026-02-21-supervisor-review]]", "[[2026-02-21-investigator1-core-crates]]", "[[2026-02-21-investigator2-tools-content]]", "[[2026-02-21-investigator3-cli-mcp-integration]]", "[[2026-02-21-testrunner1-results]]", "[[2026-02-21-testrunner2-per-crate-results]]"]
date: 2026-02-21
---

# Repository Manager -- Full Health Audit

**Date:** 2026-02-21
**Scope:** Complete Rust workspace (11 crates + integration tests)
**Team:** 3 investigators, 2 test runners, 1 supervisor
**Methodology:** Independent investigation with supervisor cross-verification and spot-checking

---

## Executive Summary

The Repository Manager is a Rust workspace of 11 crates that generates AI/IDE tool configurations from a single source of truth. The codebase demonstrates strong fundamentals: clean layered architecture, disciplined error handling, no `unsafe` code, and a comprehensive test suite of 1,269 tests at 99.4% pass rate.

However, the project has two critical infrastructure gaps that undermine all other quality work: **no Rust CI workflow** and **CI test failure suppression**. The project's Docker-only CI pipeline does not run `cargo test`, `cargo clippy`, or `cargo fmt`. The integration tests that do run have their failure exit codes silently suppressed. This means the entire Rust test suite -- which the investigators confirmed is largely well-written -- provides zero automated regression protection in CI.

Beyond infrastructure, the most significant code-level issues are: extension management commands that lie about their success status to AI agents, mode detection logic that diverges between CLI and MCP server, and a security-adjacent fail-open behavior in the symlink check for atomic file writes.

The test suite is genuine and mostly non-trivial, with real filesystem operations, real git repositories, and no mocking. The few failures (7 out of 1,269) are attributable to environment assumptions (running as root, Git default branch naming) rather than code bugs.

---

## Codebase Health Score: B-

**Justification:**

The code quality itself is solid B+ territory -- well-structured crates, proper error handling, no unsafe code, thoughtful abstractions. What pulls the score down to B- is the infrastructure layer: no CI for Rust tests, suppressed integration test failures, 7 failing tests, and acknowledged-but-unresolved concurrency issues. The extension stub deception in the MCP server is a reliability concern for AI agent consumers that further weighs on the score.

| Category | Score | Weight | Notes |
|----------|-------|--------|-------|
| Code Quality | B+ | 30% | Clean architecture, proper error handling, no unsafe |
| Test Quality | B | 25% | Genuine tests, good coverage, some gaps in error paths |
| CI/Infrastructure | F | 20% | No Rust CI, failure suppression, critical gap |
| Security Posture | B- | 10% | Good input validation, fail-open symlink check |
| API Correctness | C+ | 15% | Extension stubs lie, mode detection diverges |

---

## Critical Issues (Must Fix)

### CRIT-1: No Rust CI Workflow

**Location:** `.github/workflows/` (only `docker-integration.yml` exists)
**Impact:** Any Rust-level regression -- compile errors, test failures, lint violations, formatting drift -- is undetected by CI. The 1,269 tests provide no automated protection.
**Evidence:** Single workflow file confirmed by filesystem inspection. No `cargo test`, `cargo clippy`, or `cargo fmt` commands exist in any workflow.

**Recommended Fix:** Add `.github/workflows/rust.yml`:
```yaml
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo test --workspace
      - run: cargo clippy --workspace -- -D warnings
      - run: cargo fmt -- --check
```

### CRIT-2: CI Integration Test Failure Suppression

**Location:** `.github/workflows/docker-integration.yml`, lines 256 and 261
**Impact:** Config generation and tool read tests can fail silently. The CI pipeline reports green when core functionality is broken.
**Evidence:**
```yaml
./docker/scripts/test-config-generation.sh || echo "Config gen tests completed"
./docker/scripts/test-tool-reads-config.sh || echo "Tool read tests completed"
```

**Recommended Fix:** Remove `|| echo "..."` from both lines. Let test failures propagate to the job exit code.

---

## High Priority Issues

### HIGH-1: Extension MCP Stubs Return `success: true`

**Location:** `crates/repo-mcp/src/handlers.rs`, lines 702-810
**Impact:** AI agents calling `extension_install`, `extension_init`, or `extension_remove` via MCP receive `"success": true` even though no operation was performed. This causes agents to believe operations succeeded when nothing happened, leading to incorrect downstream reasoning.
**Evidence:** Verified in source. Example at line 707-711:
```rust
Ok(json!({
    "success": true,
    "source": args.source,
    "message": format!("Extension install from '{}' (stub - not yet implemented)", args.source),
}))
```

**Recommended Fix:** Change stub responses to `"success": false` with a clear error message, or return `Err(Error::NotImplemented(...))` consistent with the git primitives approach at line 39-41.

### HIGH-2: `detect_mode()` Divergence Between CLI and MCP

**Location:**
- CLI: `crates/repo-cli/src/commands/sync.rs`, lines 39-54
- MCP: `crates/repo-mcp/src/handlers.rs`, lines 843-877

**Impact:** Same repository interpreted differently depending on whether the user uses CLI or MCP. Specifically, an uninitialized repository defaults to `Mode::Worktrees` via CLI and `Mode::Standard` via MCP.
**Evidence:** CLI line 44: `return Ok(Mode::Worktrees);` (no config fallback). MCP line 876: `Ok(Mode::Standard)` (no config fallback).

**Recommended Fix:** Extract `detect_mode()` into `repo-core` as a single canonical implementation. Both CLI and MCP should call the same function.

### HIGH-3: Symlink Check Fail-Open in `write_atomic`

**Location:** `crates/repo-fs/src/io.rs`, line 76
**Impact:** When `contains_symlink()` encounters an I/O error (e.g., permission denied on a parent directory), the check returns `false` and the write proceeds. The security control is bypassed on error.
**Evidence:**
```rust
if contains_symlink(&native_path).unwrap_or(false) {
```

**Recommended Fix:** Change to `unwrap_or(true)` (fail-closed) or propagate the error with `?`.

### HIGH-4: `pull()` and `merge()` Force-Checkout Destroys Local Changes

**Location:** `crates/repo-git/src/helpers.rs`, lines 205 and 257
**Impact:** Any uncommitted modifications in the working tree are silently destroyed during pull or merge fast-forward operations. No warning, no backup.
**Evidence:**
```rust
co_repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
```

**Recommended Fix:** Use `CheckoutBuilder::default().safe()` instead of `.force()`. This will fail if local modifications would be overwritten, giving the caller a chance to handle the situation.

### HIGH-5: TOML Formatting Loss on Path Mutations

**Location:** `crates/repo-content/src/document.rs`, lines 313-324
**Impact:** `set_path()` or `remove_path()` on a TOML document destroys all comments, custom key ordering, and inline table formatting. The round-trip goes: TOML source -> `serde_json::Value` -> `toml::Value` -> `toml::to_string_pretty()`.
**Evidence:** `render_from_normalized()` function at line 313-324 confirmed.

**Recommended Fix:** Use `toml_edit` for path mutations on TOML documents, which preserves formatting. The `repo-blocks` crate already depends on `toml_edit` for block operations, so the dependency exists in the workspace.

### HIGH-6: 7 Failing Tests in Suite

**Location:** Multiple crates
**Impact:** Test suite is not green. Failures erode trust in the test infrastructure.

**Failing tests:**
1. `crates/repo-cli/src/commands/branch.rs:403` -- `test_list_branches` fails because test setup does not create a proper repository state for branch listing. Root cause: the test creates a `.repository/config.toml` but the underlying git repo state is insufficient for the branch list operation.

2. `crates/repo-fs/tests/error_condition_tests.rs` -- 3 Unix permission tests fail when running as root (Docker/CI). Tests set file permissions to read-only but root ignores permission restrictions.

3. `crates/repo-git/tests/container_tests.rs` -- 3 container tests fail because `git init --bare` sets HEAD to `refs/heads/master`, but the test creates an orphan `main` branch. When `create_feature()` calls `repo.head()` on the bare repo, it resolves to the nonexistent `refs/heads/master`.

**Recommended Fix:**
- For #1: Fix test setup to ensure valid branch state before listing.
- For #2: Add `#[cfg_attr(not(test_as_root), test)]` or a runtime `is_root()` check that skips these tests in root environments.
- For #3: After bare repo init, run `git symbolic-ref HEAD refs/heads/main` to set HEAD to the correct branch.

---

## Medium Priority Issues

### MED-1: Ledger TOCTOU Race Condition

**Location:** `crates/repo-core/src/ledger/mod.rs`
**Impact:** Concurrent `load() -> modify -> save()` sequences cause last-writer-wins data loss. Acknowledged via `#[ignore]` test.
**Risk Assessment:** Low probability in typical usage (single CLI invocation), but the MCP server could theoretically receive concurrent requests.
**Recommended Fix:** Implement read-modify-write within a single exclusive lock scope.

### MED-2: `ClassicLayout::current_branch()` Opens Wrong Path

**Location:** `crates/repo-git/src/classic.rs`, line 82
**Impact:** Opens `Repository::open(self.git_dir.to_native())` (the `.git` directory) while `open_repo()` at line 33 opens on `self.root`. While `git2` usually handles this, the inconsistency could cause issues with hook or config resolution.
**Recommended Fix:** Change line 82 to use `self.root.to_native()` consistent with `open_repo()`.

### MED-3: Rule ID Validation Duplicated in 3 Locations

**Location:**
- `crates/repo-cli/src/commands/rule.rs`
- `crates/repo-mcp/src/handlers.rs`
- `crates/repo-cli/src/commands/governance.rs`

**Impact:** DRY violation. If validation rules change, all 3 locations must be updated in sync.
**Recommended Fix:** Extract validation into `repo-core` as a shared utility.

### MED-4: `ClassicLayout::list_worktrees()` Swallows Branch Errors

**Location:** `crates/repo-git/src/classic.rs`, line 54
**Impact:** If `current_branch()` fails (corrupted HEAD, detached state), the error is silently replaced with the string `"unknown"`. Callers see a valid-looking response with incorrect data.
**Recommended Fix:** Log the error and/or return an `Option<String>` for the branch field.

### MED-5: `fix_with_options()` Double-Read TOCTOU

**Location:** `crates/repo-core/src/sync/engine.rs`
**Impact:** `fix()` calls `check()` then `sync()`, each reading the ledger independently. The fix summary may report stale counts.
**Recommended Fix:** Have `fix()` use the sync result directly for its summary, not the pre-sync check result.

### MED-6: Asymmetric Sync Triggering in CLI

**Location:** `crates/repo-cli/src/commands/` -- `add-tool` triggers `trigger_sync_and_report()`, but `add-preset` and `remove-preset` do not.
**Impact:** Users adding presets see no immediate effect. Behavioral inconsistency between tool and preset operations.
**Recommended Fix:** Either trigger sync after preset changes, or document the "run `repo sync` after preset changes" requirement in the command output.

### MED-7: CI Path Filters Exclude Integration Tests

**Location:** `.github/workflows/docker-integration.yml`, lines 8-12
**Impact:** Changes to `tests/integration/` do not trigger CI. Only changes in `docker/`, `crates/`, `test-fixtures/`, and `docker-compose*.yml` trigger the workflow.
**Recommended Fix:** Add `'tests/**'` to the paths filter.

---

## Low Priority Issues

### LOW-1: `read_text` and `write_text` Are Public API with TODO Placeholders

**Location:** `crates/repo-fs/src/io.rs`, lines 179, 188
**Impact:** Public functions carry `TODO: PLACEHOLDER - replace with ManagedBlockEditor` comments. Callers may depend on behavior that is scheduled to change.

### LOW-2: Lock Files Accumulate and Are Never Cleaned Up

**Location:** `crates/repo-fs/src/io.rs`, line 88
**Impact:** `.lock` files accumulate in directories over time. Not harmful but untidy.

### LOW-3: Checksum Functions Exported from Two Locations

**Location:** `repo_fs::checksum` and `repo_core::sync::engine`
**Impact:** Two public API paths to the same function. Refactoring hazard.

### LOW-4: 14 Clippy Warnings Across 2 Crates

**Location:** 10 in `repo-tools` (`mcp_installer.rs`, `mcp_translate.rs`, `capability.rs`), 4 in `repo-core` (`hooks.rs`, `registry.rs`, `engine.rs`)
**Impact:** Code style inconsistencies. No functional impact.
**Details:** Mostly `collapsible_if`, `unnecessary_map_or`, `manual_inspect`, and `for_kv_map` lints. See [[2026-02-21-testrunner1-results]] for full list.

### LOW-5: Test Helper Duplication Across Test Files

**Location:** Multiple crates (`repo-git`, `repo-tools`, `repo-presets`)
**Impact:** `setup_classic_repo_with_git()`, `create_test_context()`, `create_rule()` duplicated across test files. Maintenance burden grows with test count.

### LOW-6: `file_stem().unwrap()` in CLI Rule Listing

**Location:** `crates/repo-cli/src/commands/rule.rs`, line 106
**Impact:** Safe today (paths from `read_dir` always have file stems), but fragile if the code is refactored.

### LOW-7: Mode String Aliasing Not Normalized on Write

**Location:** `crates/repo-cli/src/commands/init.rs`
**Impact:** Both `"worktree"` and `"worktrees"` are accepted as input, but the user-provided string is written verbatim to config. Config files may inconsistently contain either form.

### LOW-8: `_repo_managed` JSON Key Collision Risk

**Location:** `crates/repo-blocks/src/formats/json.rs`
**Impact:** If a user's JSON document contains a `_repo_managed` key, the block system may misinterpret it as a managed block section. No collision detection or warning exists.

### LOW-9: Stale Documentation

**Location:** `docs/testing/framework.md`
**Impact:** Coverage matrix shows "0% implemented" for MCP Server and Git Operations, which is outdated. Misleads contributors about project status.

---

## Test Suite Assessment

### Overall Statistics

| Metric | Value |
|--------|-------|
| Total tests | 1,269 (per-crate run) |
| Passed | 1,261 (99.4%) |
| Failed | 7 (0.6%) |
| Ignored | 3 |
| Clippy warnings | 14 |
| Compilation | Clean (all 11 crates + integration tests) |

### Test Quality Verdict: B

**Strengths:**
- Tests are genuine: real filesystem operations via `tempfile::TempDir`, real `git2` repository operations, real `git` CLI invocations. No mocking or stubbing detected in library tests.
- Good coverage of happy paths across all crates.
- Several high-quality integration tests: `test_full_vertical_slice`, `test_tool_sync_creates_files`, protocol compliance tests in `repo-mcp`.
- Fixture-based golden file tests in `repo-core` validate real output content.
- Security regression tests in `repo-extensions` (parent traversal rejection, template chaining prevention).
- Idempotency testing in `repo-tools` (sync three times, compare).

**Weaknesses:**
- Error path coverage is thin across all crates. Tests predominantly exercise happy paths.
- `UvProvider::apply()` (the only preset that actively modifies the environment) has zero test coverage.
- No MCP end-to-end tests for `repo_sync`, `repo_fix`, `branch_create`, `branch_delete`, `tool_add`, `tool_remove`, `preset_add`, `preset_remove`.
- Extension command tests only assert `result.is_ok()` on stubs -- they verify nothing.
- `governance.rs`, `hooks.rs`, `projection/`, `backup/` modules in `repo-core` have no dedicated test files.
- 7 failing tests indicate environment assumptions not properly gated.
- CI does not run any Rust tests, negating the entire test suite's value as a regression gate.

### Failure Root Causes

All 7 failures are environment-related, not code bugs:

| Failure Category | Count | Root Cause |
|-----------------|-------|------------|
| Unix permission tests running as root | 3 | Root user ignores file permission restrictions |
| Container tests with wrong HEAD target | 3 | `git init --bare` defaults HEAD to `refs/heads/master` |
| CLI branch list test | 1 | Test setup produces insufficient repository state |

---

## Architecture Assessment

### Strengths

1. **Clean dependency layering.** `repo-fs` has no internal dependencies. `repo-git` and `repo-meta` depend only on `repo-fs`. `repo-core` orchestrates everything. No circular dependencies.

2. **`NormalizedPath` as cross-cutting concern.** Used consistently across all crates. The normalization boundary (path sanitization at construction, `to_native()` at filesystem access) is respected throughout.

3. **Error handling discipline.** All crates use `thiserror`-derived error types. No `unwrap()` calls in production paths (one exception: `file_stem().unwrap()` in CLI). Error variants are meaningful and carry context.

4. **No `unsafe` code anywhere in the workspace.** Confirmed across all crates.

5. **Separation of concerns between block systems.** `repo-blocks` (tool config markers) and `repo-content` (document format handlers) serve distinct purposes despite conceptual overlap.

### Weaknesses

1. **`repo-core` is too wide.** It depends on 6 sibling crates and functions as the application layer disguised as a library. Any change in any dependency crate potentially touches `repo-core`.

2. **`repo-fs` carries format-specific dependencies.** `serde_yaml`, `serde_json`, `toml` in a filesystem abstraction crate. `ConfigStore` belongs in `repo-meta`.

3. **`LayoutMode` (in `repo-fs`) and `LayoutProvider` (in `repo-git`) are parallel concepts without compile-time connection.** Callers must manually map between them, which is error-prone.

4. **Two separate `detect_mode()` implementations** in CLI and MCP with different behavior. Should be a single function in `repo-core`.

5. **Extension system is entirely vaporware.** The `repo-extensions` crate provides manifest parsing and MCP config resolution, but the CLI and MCP server commands that would use it are all stubs. The extension system exists as infrastructure without a consumer.

---

## Recommendations (Prioritized Action Items)

### Immediate (Block Release)

1. **Add a Rust CI workflow** (`.github/workflows/rust.yml`) that runs `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, and `cargo fmt -- --check` on every push and PR. This is the single highest-impact change. Without it, none of the other test quality improvements matter.

2. **Remove CI test failure suppression.** Delete the `|| echo "..."` suffixes in `docker-integration.yml` lines 256 and 261.

3. **Change extension MCP stubs to return `success: false`** or `Err(Error::NotImplemented(...))`. AI agents should not be told operations succeeded when they did not.

### Short-Term (Next Sprint)

4. **Fix 7 failing tests.**
   - Gate permission tests on non-root environment.
   - Fix container test setup to set bare repo HEAD to `refs/heads/main`.
   - Fix `test_list_branches` setup.

5. **Centralize `detect_mode()` into `repo-core`.** Both CLI and MCP should call the same function with the same default.

6. **Fix symlink check fail-open.** Change `unwrap_or(false)` to `unwrap_or(true)` in `crates/repo-fs/src/io.rs:76`.

7. **Change `pull()` and `merge()` to use safe checkout** instead of force checkout.

8. **Address 14 clippy warnings.** These are trivial fixes.

### Medium-Term (Next Quarter)

9. **Fix TOML formatting loss.** Use `toml_edit` for path mutations to preserve comments and formatting.

10. **Centralize rule ID validation** into `repo-core` to eliminate the 3-location DRY violation.

11. **Add error-path integration tests** for tool sync (unwritable directories, malformed files), MCP tool calls (invalid arguments, missing repos), and preset application.

12. **Resolve ledger TOCTOU** with read-modify-write under a single lock scope.

13. **Add `tests/integration/` to CI path filters.**

### Long-Term (Future Planning)

14. **Decide on extension system ship date or deprecation.** The infrastructure exists (`repo-extensions` crate), but no consumer works. Either implement the CLI/MCP commands or remove the stubs entirely to avoid misleading users.

15. **Consider splitting `repo-core`** into `repo-core` (sync engine, ledger) and `repo-orchestrator` (config resolution, mode detection) to reduce the dependency fan-out.

16. **Add property tests** for `NormalizedPath` (idempotency, no traversal, no network paths) and for the block system (round-trip invariants).
