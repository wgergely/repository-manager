# Test Quality Audit Report — Investigator2

**Date:** 2026-02-18
**Scope:** All test code in `Y:/code/repository-manager-worktrees/main/`
**Crates audited:** repo-blocks, repo-cli, repo-content, repo-core, repo-fs, repo-git, repo-meta, repo-mcp, repo-presets, repo-tools, repo-agent

---

## Executive Summary

The workspace's test suite is **generally high quality** for integration-level tests. The test code consistently exercises real production behavior: real git repositories (via git2 and `git` CLI), real filesystems (via tempfile), real TOML/JSON parsing, and real binary invocations (via assert_cmd). No mocking frameworks were found anywhere in the workspace.

However, several **structural and correctness issues** were identified:

1. **Test location violations are pervasive.** Many tests in `crates/*/tests/` directories are pure unit tests (data structure operations, string parsing) that belong in `#[cfg(test)]` inline modules. This is the most widespread issue across the codebase.
2. **`repo-agent` has zero tests** — the crate containing process spawning and subprocess management has no coverage at all.
3. **A known concurrency bug (TOCTOU race) is documented in a test** rather than being fixed. The test asserts last-writer-wins semantics for concurrent ledger saves, treating a data race as expected behavior.
4. **Deprecated API tests persist** in `repo-meta` with `#![allow(deprecated)]`, creating ongoing maintenance burden.
5. **One conditional assertion** in `repo-presets` skips part of its validation when `uv` is not installed, creating environment-dependent test reliability.
6. **test-fixtures golden files** are referenced in `repo-core` fixture tests but could not be located in the repository, raising concerns about whether they are checked in.

Overall, the integration test philosophy is sound — tests validate real behavior with real dependencies. The primary remediation work is structural: relocating misplaced unit tests and filling the `repo-agent` gap.

---

## Test Inventory

### External Test Files (`crates/*/tests/*.rs`)

| Crate | Test File | Lines | Type |
|---|---|---|---|
| repo-blocks | `parser_tests.rs` | ~200 | Unit (misplaced) |
| repo-blocks | `writer_tests.rs` | ~180 | Unit (misplaced) |
| repo-cli | `cli_list_tools.rs` | ~60 | Integration/E2E |
| repo-content | `integration_tests.rs` | ~300 | Integration |
| repo-content | `diff_tests.rs` | ~50 | Unit (misplaced) |
| repo-core | `integration_tests.rs` | ~350 | Integration |
| repo-core | `rules_tests.rs` | ~250 | Integration |
| repo-core | `config_tests.rs` | ~200 | Mixed |
| repo-core | `fixture_tests.rs` | ~120 | Golden-file |
| repo-core | `ledger_locking_tests.rs` | ~100 | Concurrency |
| repo-core | `ledger_tests.rs` | ~200 | Integration |
| repo-core | `mode_tests.rs` | ~150 | Mixed |
| repo-core | `sync_tests.rs` | ~200 | Integration |
| repo-fs | `config_tests.rs` | ~80 | Integration |
| repo-fs | `correctness_tests.rs` | ~150 | Integration |
| repo-fs | `layout_tests.rs` | ~100 | Integration |
| repo-fs | `path_tests.rs` | ~120 | Unit (misplaced) |
| repo-fs | `property_tests.rs` | ~80 | Property-based |
| repo-fs | `security_audit_tests.rs` | ~150 | Security |
| repo-git | `classic_tests.rs` | ~80 | Integration |
| repo-git | `container_tests.rs` | ~250 | Integration |
| repo-git | `git_operations_tests.rs` | ~470 | Integration |
| repo-git | `in_repo_worktrees_tests.rs` | ~200 | Integration |
| repo-git | `naming_tests.rs` | ~80 | Unit (misplaced) |
| repo-git | `robustness_unicode.rs` | ~97 | Integration |
| repo-meta | `config_tests.rs` | ~197 | Deprecated API |
| repo-meta | `schema_tests.rs` | ~254 | Integration |
| repo-mcp | `protocol_compliance_tests.rs` | ~400+ | Integration/E2E |
| repo-presets | `python_tests.rs` | ~120 | Integration |
| repo-tools | `cursor_tests.rs` | ~200 | Integration |
| repo-tools | `vscode_tests.rs` | ~150 | Integration |

### Benchmark Files (`crates/*/benches/*.rs`)

| Crate | Benchmark File | Notes |
|---|---|---|
| repo-fs | `fs_benchmarks.rs` | write_atomic, WorkspaceLayout::detect |
| repo-git | `git_benchmarks.rs` | ContainerLayout::create_feature |

### Inline Unit Tests (`#[cfg(test)]` modules)

Approximately **111 source files** contain inline `#[cfg(test)]` modules across all crates. Sampling confirmed they contain appropriate unit tests with no mock framework usage. Representative examples:

- `repo-blocks/src/parser.rs` — tests for block marker regex parsing
- `repo-core/src/sync/check.rs` — tests for CheckReport construction
- `repo-core/src/ledger/mod.rs` — tests for version fields and atomic save behavior
- `repo-fs/src/path.rs` — path normalization invariants

### Crates with No Tests

| Crate | Source Files Found | Test Files | Notes |
|---|---|---|---|
| repo-agent | discovery.rs, process.rs, subprocess.rs | **NONE** | Critical gap |

---

## Test Location Audit

### Rule
Per team standards: **unit tests must reside in their source module** (`#[cfg(test)]` inline). Only **integration tests** and **E2E tests** belong in `crates/*/tests/` directories.

### Violations — Tests Misplaced in External `tests/` Directory

#### `repo-blocks/tests/parser_tests.rs`
**Classification: Unit test — MISPLACED**

Tests `parse_blocks()` against string literals. There is no filesystem I/O, no external process, no crate boundary crossing. The function being tested is a pure string-processing function within `repo-blocks`. These tests belong as an inline `#[cfg(test)]` module in `repo-blocks/src/parser.rs` or `lib.rs`.

#### `repo-blocks/tests/writer_tests.rs`
**Classification: Unit test — MISPLACED**

Tests `insert_block`, `update_block`, `remove_block`, `upsert_block` against string inputs. Pure string manipulation with no external dependencies. Should be inline in the writer module.

Notable finding: the cross-block injection attack tests in this file are valuable security tests — they should be preserved and moved, not deleted.

#### `repo-fs/tests/path_tests.rs`
**Classification: Unit test — MISPLACED**

Tests `NormalizedPath` construction, joining, parent/filename operations. This is a data structure with no I/O. Tests belong inline in `repo-fs/src/path.rs`.

#### `repo-git/tests/naming_tests.rs`
**Classification: Unit test — MISPLACED**

Tests `branch_to_directory()` with various string inputs for Slug and Hierarchical naming strategies. Pure string transformation with no git state. Should be inline in the naming strategy module.

#### `repo-content/tests/diff_tests.rs`
**Classification: Unit test — MISPLACED**

Tests `SemanticDiff` struct construction only — creating a struct and asserting field values. No filesystem I/O, no cross-crate interaction. Should be inline in the diff module.

### Correctly Placed Integration Tests

The following files are correctly placed in `tests/` because they cross module/crate boundaries, touch real filesystems, invoke real git processes, or test observable system behavior:

- `repo-git/tests/container_tests.rs` — real git CLI + filesystem
- `repo-git/tests/git_operations_tests.rs` — real git push/pull/merge
- `repo-git/tests/robustness_unicode.rs` — real git with Unicode branch names
- `repo-fs/tests/correctness_tests.rs` — real tempdir filesystem detection
- `repo-fs/tests/security_audit_tests.rs` — real filesystem security checks
- `repo-content/tests/integration_tests.rs` — multi-crate I/O pipeline
- `repo-core/tests/integration_tests.rs` — end-to-end config/sync flow
- `repo-core/tests/sync_tests.rs` — real ledger files + sync engine
- `repo-cli/tests/cli_list_tools.rs` — real binary execution
- `repo-mcp/tests/protocol_compliance_tests.rs` — real server + JSON-RPC

---

## Mock/Stub/Patch Findings

**No mocking frameworks were found in the workspace.**

A search for common Rust mock crate usage (`mockall`, `mock_it`, `double`, `faux`, `mocktopus`, `unimock`) returned zero results across all source and test files.

The word "fake" appears in several test contexts but refers to **realistic test data structures**, not mock framework usage:

- `repo-presets/tests/python_tests.rs` — "fake venv" means a real directory tree created on disk to simulate what a Python virtual environment looks like. Actual filesystem operations are performed.
- `repo-content` — "fake content" refers to literal string values used as test input.
- `repo-core` — comments use "fake" colloquially to describe simplified test data.

**Assessment:** The absence of mocking is a strength. All external dependencies (git, filesystem, config parsing) are tested with real implementations. This maximizes confidence that tests reflect actual production behavior.

---

## False Positive Risk Assessment

### Low Risk (Well-Constructed)

**`repo-git/tests/git_operations_tests.rs`**
Tests perform actual git operations and verify observable state (file existence after merge, error message content for no-remote scenarios). The `test_container_merge_fast_forward` test opens two separate repository handles — the bare `.gt` repo for git operations and the main worktree repo for checkout — which correctly reflects how the production code would behave.

**`repo-fs/tests/security_audit_tests.rs`**
Path traversal tests verify that sanitization functions reject known attack patterns. Symlink rejection tests are gated with `#[cfg(all(test, not(windows)))]` to avoid false results on Windows where symlinks behave differently. Atomic write tests verify that temp files are cleaned up on error.

**`repo-mcp/tests/protocol_compliance_tests.rs`**
Tests the actual JSON-RPC 2.0 protocol responses from the real server implementation. Tests verify ID preservation, correct error codes (-32600, -32601, etc.), version negotiation, and tool invocation with real tool discovery. No protocol simulation.

**`repo-tools/tests/cursor_tests.rs` and `vscode_tests.rs`**
Read back written files and validate content (block markers in cursor, JSON parsing in vscode). These tests would catch regressions in actual file format output.

### Medium Risk (Conditional Logic)

**`repo-presets/tests/python_tests.rs` — `test_uv_check_with_complete_venv_structure`**

```rust
// This test creates a complete fake venv and checks UvProvider behavior.
// The assertion on check() result is conditional based on whether `uv` is installed.
if uv_available {
    assert!(result.is_ok());
} else {
    // May return error or ok depending on implementation
}
```

The conditional assertion means this test provides **weaker guarantees** in CI environments where `uv` is not installed. If the intent is to test the fake venv detection logic (independent of actual `uv` binary), the test should be split: one test for the filesystem structure detection (which should not require `uv`) and a separate integration test gated on `uv` being available.

**`repo-core/tests/mode_tests.rs` — documented behavioral quirks**

The mode tests explicitly document that `StandardBackend` accepts a `.git` directory without a `HEAD` file, and `WorktreeBackend` accepts a missing `main/` directory. These tests validate current behavior but the comments suggest the behavior may not be intentional. Tests that encode bugs rather than intended behavior create false confidence.

### Higher Risk

**`repo-core/tests/ledger_locking_tests.rs` — TOCTOU race documented as passing**

```rust
#[test]
fn test_concurrent_ledger_saves_last_writer_wins() {
    // NOTE: This documents a known TOCTOU race condition.
    // The last thread to write wins; intermediate writes are lost.
    // This is acceptable for now but should be fixed with proper locking.
```

The test spawns multiple threads writing to the same ledger file and asserts that exactly one write survives. This asserts **data loss behavior** as a passing test. While the comment acknowledges it's a known issue, encoding data races as "expected" behavior is a false positive risk: if the race is ever fixed (e.g., by adding file locking), this test would fail not because something broke, but because the behavior improved. The test assertion logic would need to be inverted.

**Recommendation:** This test should be marked `#[ignore]` with a tracking issue reference, not validated as a passing behavior assertion.

---

## Coverage Gap Analysis

### Critical Gap: `repo-agent` — Zero Tests

The `repo-agent` crate contains:
- `discovery.rs` — process discovery logic
- `process.rs` — process management
- `subprocess.rs` — subprocess spawning and communication

**This crate has no tests whatsoever** — no inline `#[cfg(test)]` modules and no external `tests/` directory. Process spawning and subprocess communication are high-risk areas prone to platform-specific failures, race conditions, and error handling gaps. This is the **most critical coverage gap** in the workspace.

### Moderate Gaps

**`repo-mcp` inline unit tests are sparse**
The MCP server has solid external integration tests in `protocol_compliance_tests.rs`, but the individual handler functions and routing logic have minimal inline unit tests. Edge cases in protocol handling (malformed JSON, oversized payloads, invalid tool names with special characters) are not individually tested.

**`repo-core/tests/fixture_tests.rs` — golden files not confirmed**
The fixture tests reference golden files via:
```rust
let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("../../test-fixtures");
```
A filesystem search for `test-fixtures/` at the workspace root returned no results. If these files are not checked into the repository, the fixture tests would silently fail to run meaningful validation. This needs verification.

**`repo-content` — no concurrent edit tests**
The content block system (insert/update/remove) has no tests for concurrent modification scenarios. If two processes write to the same managed file simultaneously, the outcome is unspecified. Given the use of atomic writes elsewhere in the codebase, this gap may be intentional, but it should be explicitly addressed.

**`repo-git` — no tests for detached HEAD state**
The git operations tests cover normal branch scenarios but do not test behavior when the repository is in a detached HEAD state. `layout.current_branch()` is called throughout, but its behavior in detached HEAD is not tested.

**`repo-presets` — only Python preset tested**
The `repo-presets` crate presumably supports multiple language presets, but only the Python/uv preset has tests. If other presets exist (Node, Ruby, etc.), they have no coverage.

### Minor Gaps

**`repo-fs/tests/property_tests.rs`**
Property tests cover `NormalizedPath` normalization but do not cover `WorkspaceLayout` detection or `ConfigStore` serialization with fuzz inputs. Adding proptest-based fuzzing for config parsing would strengthen resilience.

**`repo-meta` schema tests**
The schema tests verify loading from real TOML files but do not test every schema field combination. The `schema/tool.rs`, `schema/rule.rs`, and `schema/preset.rs` inline unit tests (referenced in the file header comment) may cover remaining cases, but this could not be confirmed without reading all inline test modules for that crate.

---

## Stale/Dead Tests

### `repo-meta/tests/config_tests.rs` — Deprecated API Tests

```rust
//! Note: These tests exercise the deprecated RepositoryConfig/load_config API.
//! New code should use repo_core::Manifest::parse() instead.

#![allow(deprecated)]
```

This entire file tests an API that the codebase itself documents as deprecated. The tests are not dead (they still run and pass), but they are **maintenance debt**: every change to the deprecated API requires updating these tests, and the deprecated API cannot be removed while this test file exists.

**Recommendation:** Either remove the deprecated API and this test file together, or add a tracking issue to schedule the removal. The `allow(deprecated)` suppression should not persist indefinitely.

### `repo-core/tests/mode_tests.rs` — Quirk Documentation Tests

Several tests in this file document unexpected behavior:
```rust
// Accepts .git without HEAD — this may be a bug
#[test]
fn test_standard_backend_accepts_git_without_head() { ... }
```

If these tests exist to document known bugs, they should reference tracking issues so they can be updated when the bugs are fixed. Currently there is no linkage between the "this is a quirk" comment and any issue tracker.

### Benchmark Utility

Both benchmark files (`git_benchmarks.rs`, `fs_benchmarks.rs`) benchmark real operations and appear maintained. No stale benchmarks were identified. The `fs_benchmarks.rs` benchmarks `write_atomic` and `WorkspaceLayout::detect` — both core operations worth ongoing performance tracking.

---

## Recommendations

### P0 — Critical

**1. Add tests for `repo-agent` crate**
The process discovery, management, and subprocess communication code has zero test coverage. At minimum, add:
- Unit tests for `discovery.rs` process detection logic (can use mock process lists or test against known system binaries)
- Integration tests for `subprocess.rs` spawning a simple command (e.g., `echo`) and verifying output capture
- Error handling tests: what happens when the target process does not exist, crashes mid-execution, or writes to stderr

**2. Verify and check in `test-fixtures/` directory**
Confirm that the golden files referenced in `repo-core/tests/fixture_tests.rs` exist at the workspace root and are committed to the repository. If they are missing, the fixture tests are not validating anything meaningful.

### P1 — High Priority

**3. Fix or suppress the TOCTOU concurrency test**
`repo-core/tests/ledger_locking_tests.rs` `test_concurrent_ledger_saves_last_writer_wins` asserts data loss behavior. Options:
- Fix the underlying race by implementing file locking in the ledger save path, then update the test to assert safe concurrent behavior
- Mark with `#[ignore = "known TOCTOU race: see issue #N"]` and create a tracking issue

**4. Relocate misplaced unit tests**
Move these pure unit tests from external `tests/` directories to inline `#[cfg(test)]` modules:
- `repo-blocks/tests/parser_tests.rs` → `repo-blocks/src/parser.rs`
- `repo-blocks/tests/writer_tests.rs` → `repo-blocks/src/writer.rs` (or relevant module)
- `repo-fs/tests/path_tests.rs` → `repo-fs/src/path.rs`
- `repo-git/tests/naming_tests.rs` → `repo-git/src/naming.rs` (or relevant module)
- `repo-content/tests/diff_tests.rs` → `repo-content/src/diff.rs` (or relevant module)

Note: preserve the cross-block injection security tests from `writer_tests.rs` — they are valuable and must not be lost during relocation.

### P2 — Medium Priority

**5. Fix environment-dependent assertion in `repo-presets`**
Split `test_uv_check_with_complete_venv_structure` into:
- A test that validates the fake venv directory structure is detected correctly (should not require `uv` binary)
- A separate test gated on `uv` availability that tests actual execution

**6. Plan deprecation of `repo-meta` config API**
Schedule removal of the deprecated `RepositoryConfig`/`load_config` API and its corresponding test file (`repo-meta/tests/config_tests.rs`). Until removal, the `#![allow(deprecated)]` in the test file is acceptable but the timeline should be documented.

**7. Add tracking issues for documented behavioral quirks**
`repo-core/tests/mode_tests.rs` documents quirks (accepting `.git` without `HEAD`, accepting missing `main/`) without referencing issue numbers. Each documented quirk should have a corresponding issue so it can be resolved or formally accepted.

### P3 — Low Priority

**8. Extend `repo-presets` coverage**
If presets beyond Python are planned or implemented, add corresponding test files. A single-preset test suite creates false confidence that the preset infrastructure is fully tested.

**9. Extend `repo-git` tests for edge cases**
Add tests for:
- Detached HEAD state behavior in `current_branch()`
- Repository with no commits (empty repo)
- Worktree with corrupted git state

**10. Extend `repo-fs` property tests**
Add proptest fuzzing for:
- `ConfigStore` serialization roundtrip with arbitrary key/value content
- `WorkspaceLayout::detect()` with arbitrary directory trees (should never panic)

---

## Appendix: Test Infrastructure Quality Notes

### Positive Patterns Found

- **Real git operations everywhere in `repo-git`**: Tests use both `git2::Repository` and `Command::new("git")` for setup. This catches bugs that would only manifest with real git state (e.g., index state, commit graph, worktree registration).
- **Real filesystem I/O in all storage tests**: `tempfile::TempDir` and `tempfile::tempdir()` are used consistently. No in-memory filesystem abstractions that could hide real I/O bugs.
- **Parameterized security tests**: `repo-fs/tests/security_audit_tests.rs` uses `rstest` to parameterize path traversal attack vectors. Easy to add new attack patterns.
- **Platform-aware tests**: Unix-only symlink tests are correctly gated with `#[cfg(all(test, not(windows)))]`.
- **Snapshot tests with path masking**: `repo-fs/tests/snapshot_tests.rs` masks temporary directory paths with `[ROOT]` before snapshot comparison, preventing flaky tests from non-deterministic paths.
- **CLI binary tests**: `repo-cli/tests/cli_list_tools.rs` uses `assert_cmd` to test the actual compiled binary, not a library function that simulates CLI behavior.

### Testing Libraries in Use

| Library | Usage |
|---|---|
| `tempfile` | Real filesystem isolation in all storage/git tests |
| `git2` | Rust-native git operations in container/unicode tests |
| `assert_cmd` | CLI binary testing in repo-cli |
| `predicates` | Output assertions paired with assert_cmd |
| `proptest` | Property-based fuzzing in repo-fs |
| `insta` | Snapshot testing in repo-fs |
| `rstest` | Parameterized test cases in repo-fs security tests |
| `tokio::test` | Async test runtime in repo-presets |
| `criterion` | Benchmark harness in repo-fs and repo-git |

No mocking frameworks (`mockall`, `mock_it`, `faux`, `unimock`, etc.) are present anywhere in the workspace.
