# Phase 5: Tautological Test Elimination

**Priority**: P3 (test hygiene)
**Audit ID**: TH-5
**Domain**: `crates/repo-tools/tests/`, `crates/repo-tools/src/` (inline tests)
**Status**: Not started
**Prerequisite**: [Phase 3](2026-02-22-test-health-phase3-format-validation.md) (format validators replace tautological checks)
**Estimated scope**: ~120 tests reviewed, ~40 upgraded, ~20 deleted

---

## Testing Mandate

> **Inherited from [tasks/_index.md](../tasks/_index.md).** Read and internalize the
> full Testing Mandate before modifying any test file.

**Domain-specific enforcement:**
- A tautological test is one that **cannot fail** unless `fs::write` or `String::contains`
  is broken. These tests test the standard library, not the product.
- Deletion is preferable to keeping a test that provides false confidence. A test that
  can never fail is worse than no test — it inflates coverage numbers while hiding gaps.
- Every surviving test must be **provably capable of failing** when the feature it
  guards is broken. Document the failure mode in a comment.

---

## Problem Statement

~40% of tool-layer tests follow this pattern:

```rust
let rules = vec![Rule { id: "X", content: "Y" }];
integration.sync(&context, &rules).unwrap();
let content = fs::read_to_string(file).unwrap();
assert!(content.contains("Y"));
```

This proves that writing "Y" to a file and reading it back produces "Y". It does NOT
test that the output is correctly formatted for the target tool, that managed block
markers are present, that existing user content is preserved, or that the sync operation
is idempotent.

See: [Test Health Audit TH-5](../audits/2026-02-22-test-health-audit.md#finding-th-5-tautological-tests)

---

## Implementation Plan

### Step 1: Inventory all tautological tests

Search for the pattern across `repo-tools`:

```bash
# Tests that only assert content.contains(rule_content)
rg 'assert.*contains.*content' crates/repo-tools/tests/
rg 'assert.*contains.*content' crates/repo-tools/src/
```

Classify each test into one of three buckets:

| Bucket | Criteria | Action |
|--------|----------|--------|
| **Upgrade** | Test covers a real scenario but uses weak assertions | Add format checks from Phase 3 |
| **Delete** | Test is fully redundant with another test or with format validation | Remove |
| **Keep** | Test covers a unique scenario with adequate assertions | No change |

### Step 2: Upgrade pattern — add format assertions

For tests in the "Upgrade" bucket, strengthen assertions by adding format validation
alongside the content check:

**Before (tautological):**
```rust
#[test]
fn test_cursor_sync_with_rules() {
    let rules = vec![Rule { id: "test", content: "Test content" }];
    cursor_integration().sync(&context, &rules).unwrap();
    let content = fs::read_to_string(".cursorrules").unwrap();
    assert!(content.contains("Test content"));
}
```

**After (meaningful):**
```rust
#[test]
fn test_cursor_sync_produces_valid_markdown_with_blocks() {
    let rules = vec![Rule { id: "test", content: "Test content" }];
    cursor_integration().sync(&context, &rules).unwrap();
    let content = fs::read_to_string(".cursorrules").unwrap();

    // Format: valid managed block structure
    assert!(content.contains("<!-- repo:block:test -->"));
    assert!(content.contains("<!-- /repo:block:test -->"));
    let opens = content.matches("<!-- repo:block:").count();
    let closes = content.matches("<!-- /repo:block:").count();
    assert_eq!(opens, closes, "Unbalanced block markers");

    // Content: rule content is inside the block
    // (This assertion is now meaningful because the block structure is validated above)
    assert!(content.contains("Test content"));
}
```

### Step 3: Delete pattern — remove fully redundant tests

Tests that are redundant with:
- Phase 3 format validation tests (which are strictly more thorough)
- Other tests in the same file that cover the same scenario with better assertions

**Deletion criteria (all must be true):**
1. The test's only assertion is `content.contains(rule_content)`
2. Another test already verifies the same tool's sync produces output
3. A Phase 3 format validation test covers the output format

### Step 4: Apply upgrades to `repo-tools/tests/` integration tests

The 5 test files in `crates/repo-tools/tests/`:

| File | Tests | Expected Upgrades | Expected Deletes |
|------|-------|-------------------|------------------|
| `integration_tests.rs` (now `sync_output_tests.rs`) | 28 | ~15 | ~5 |
| `claude_tests.rs` | 10 | ~5 | ~2 |
| `cursor_tests.rs` | 9 | ~5 | ~1 |
| `vscode_tests.rs` | 8 | ~4 | ~1 |
| `dispatcher_tests.rs` | 13 | ~3 | ~0 |

### Step 5: Apply upgrades to `repo-tools/src/` inline tests

The 30+ files with inline `#[cfg(test)]` modules in `repo-tools/src/`:

Priority files (most tautological tests):
- `src/cursor.rs` — 5 tests
- `src/claude.rs` — 5 tests
- `src/gemini.rs` — 5 tests
- `src/windsurf.rs` — 5 tests
- `src/copilot.rs` — 3 tests
- `src/aider.rs` — 2 tests
- `src/antigravity.rs` — 4 tests

Lower priority (less likely tautological):
- `src/mcp_installer.rs` — 33 tests (likely meaningful — installs MCP configs)
- `src/mcp_translate.rs` — 29 tests (likely meaningful — translates formats)
- `src/dispatcher.rs` — 9 tests (likely meaningful — routing logic)
- `src/generic.rs` — 14 tests (likely meaningful — generic framework)

### Step 6: Verify no coverage regression

After all changes, run:
```bash
cargo test -p repo-tools 2>&1 | grep "test result"
```

Document the before/after test count. It's acceptable (and expected) for the total to
decrease — fewer, better tests are preferable to many tautological ones.

---

## Acceptance Criteria

- [ ] Every surviving test in `repo-tools` can demonstrably fail when its feature is broken
- [ ] Zero tests exist whose sole assertion is `content.contains(rule_content)` without
  accompanying format validation
- [ ] Test count change is documented (expected: slight decrease)
- [ ] No test was deleted that was the **only** test covering a unique scenario
- [ ] `cargo test -p repo-tools` passes
- [ ] `cargo clippy` clean

---

## Files to Modify

| File | Expected Changes |
|------|-----------------|
| `crates/repo-tools/tests/sync_output_tests.rs` | Upgrade ~15, delete ~5 |
| `crates/repo-tools/tests/claude_tests.rs` | Upgrade ~5, delete ~2 |
| `crates/repo-tools/tests/cursor_tests.rs` | Upgrade ~5, delete ~1 |
| `crates/repo-tools/tests/vscode_tests.rs` | Upgrade ~4, delete ~1 |
| `crates/repo-tools/src/cursor.rs` | Upgrade inline tests |
| `crates/repo-tools/src/claude.rs` | Upgrade inline tests |
| `crates/repo-tools/src/gemini.rs` | Upgrade inline tests |
| `crates/repo-tools/src/windsurf.rs` | Upgrade inline tests |
| `crates/repo-tools/src/copilot.rs` | Upgrade inline tests |
| `crates/repo-tools/src/aider.rs` | Upgrade inline tests |
| `crates/repo-tools/src/antigravity.rs` | Upgrade inline tests |

---

## Dependencies

- **Depends on**: [Phase 3](2026-02-22-test-health-phase3-format-validation.md) (format validators replace tautological assertions)
- **Blocks**: Nothing — this is the final phase
- **Can parallelize with**: [Phase 4](2026-02-22-test-health-phase4-golden-files.md) (both depend on Phase 3, independent of each other)

---

## Cross-References

- **Source finding**: [Test Health Audit TH-5](../audits/2026-02-22-test-health-audit.md#finding-th-5-tautological-tests)
- **Testing mandate**: [tasks/_index.md](../tasks/_index.md) — Rule 2 (no false-positive tests)
- **Related task**: [P3 Test Hygiene](../tasks/P3-test-hygiene.md) — broader test cleanup
- **Phase 3 format tests**: [Phase 3](2026-02-22-test-health-phase3-format-validation.md) — provides the format validators this phase uses
- **Chain**: Phase 5 of 5 — prev: [Phase 4](2026-02-22-test-health-phase4-golden-files.md), first: [Phase 1](2026-02-22-test-health-phase1-taxonomy.md)

---

*Plan created: 2026-02-22*
