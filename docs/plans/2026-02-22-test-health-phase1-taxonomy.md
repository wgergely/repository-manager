# Phase 1: Test Taxonomy & Renaming

**Priority**: P3 (test hygiene)
**Audit ID**: TH-1
**Domain**: File renaming and doc-comment corrections across 3 crates
**Status**: Not started
**Prerequisite**: None — fully independent
**Estimated scope**: 3 files renamed, ~10 doc-comment lines changed

---

## Testing Mandate

> **Inherited from [tasks/_index.md](../tasks/_index.md).** Read and internalize the
> full Testing Mandate before modifying any test file.

**Domain-specific enforcement:**
- This phase is about **naming accuracy**, not test logic. No test assertions change.
- After renaming, `cargo test` must produce identical pass/fail results.
- No new tests are added in this phase.

---

## Problem Statement

Three test files are labeled "integration" but contain zero integration behavior. This
mislabeling hides the absence of real integration testing — developers checking for
coverage gaps see "integration_tests.rs" and conclude the gap is filled.

See: [Test Health Audit TH-1](../audits/2026-02-22-test-health-audit.md#finding-th-1-mislabeled-test-categories)

---

## Implementation Plan

### Step 1: Rename `repo-content` test files

**Current state:**
- `crates/repo-content/tests/integration_tests.rs` — 10 tests, all in-memory `Document` API calls
- `crates/repo-content/tests/diff_integration_tests.rs` — 8 tests, all in-memory `doc.diff()` calls

**Actions:**
1. `git mv crates/repo-content/tests/integration_tests.rs crates/repo-content/tests/lifecycle_tests.rs`
2. Update doc-comment line 1: `//! Lifecycle tests for repo-content crate` (replace "Integration tests")
3. Update doc-comment line 3: `//! These tests verify document lifecycle behavior across multiple operations and formats.` (replace "end-to-end")
4. `git mv crates/repo-content/tests/diff_integration_tests.rs crates/repo-content/tests/diff_tests.rs`
5. Update doc-comment line 1: `//! Tests for semantic diff` (replace "Integration tests for semantic diff")

**Verification:** `cargo test -p repo-content` — identical pass count.

### Step 2: Rename `repo-tools` integration test file

**Current state:**
- `crates/repo-tools/tests/integration_tests.rs` — 28 tests, all create TempDir + call `sync()` + read file

**Actions:**
1. `git mv crates/repo-tools/tests/integration_tests.rs crates/repo-tools/tests/sync_output_tests.rs`
2. Update doc-comment line 1: `//! Sync output tests for tool integrations` (replace "Integration tests")
3. Update doc-comment line 3: `//! These tests verify that tool sync operations produce expected file output.` (replace "end-to-end")

**Verification:** `cargo test -p repo-tools` — identical pass count.

### Step 3: Standardize test category doc-comments

Add a brief category label to the top of each test file that currently lacks one.
Format: `//! Category: <unit|component|cli|scenario>`

Only modify the files touched in Steps 1-2. Do NOT bulk-update all test files — that
would create noise without value.

### Step 4: Verify no broken imports or references

Search the codebase for any references to the old filenames (unlikely in Rust, but
check CI configs and documentation):

```bash
rg "integration_tests" docs/ .github/ Cargo.toml
```

Fix any stale references.

---

## Acceptance Criteria

- [ ] `crates/repo-content/tests/lifecycle_tests.rs` exists (renamed)
- [ ] `crates/repo-content/tests/diff_tests.rs` exists (renamed)
- [ ] `crates/repo-tools/tests/sync_output_tests.rs` exists (renamed)
- [ ] No file named `integration_tests.rs` exists in `repo-content` or `repo-tools`
- [ ] Doc-comments accurately describe what the tests do
- [ ] `cargo test` passes with identical test count
- [ ] `cargo clippy` clean

---

## Files to Modify

| File (current name) | Action | New Name |
|---------------------|--------|----------|
| `crates/repo-content/tests/integration_tests.rs` | Rename + update doc | `lifecycle_tests.rs` |
| `crates/repo-content/tests/diff_integration_tests.rs` | Rename + update doc | `diff_tests.rs` |
| `crates/repo-tools/tests/integration_tests.rs` | Rename + update doc | `sync_output_tests.rs` |

---

## Dependencies

- **Depends on**: Nothing
- **Blocks**: Nothing (purely cosmetic, but enables accurate coverage analysis)
- **Can parallelize with**: [Phase 2](2026-02-22-test-health-phase2-fixtures.md)

---

## Cross-References

- **Source finding**: [Test Health Audit TH-1](../audits/2026-02-22-test-health-audit.md#finding-th-1-mislabeled-test-categories)
- **Related task**: [P3 Test Hygiene](../tasks/P3-test-hygiene.md) — broader test cleanup
- **Chain**: Phase 1 of 5 — next: [Phase 2](2026-02-22-test-health-phase2-fixtures.md)

---

*Plan created: 2026-02-22*
