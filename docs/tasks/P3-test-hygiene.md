# P3: Test Hygiene — Stale Tests and Coverage Gaps

**Priority**: P3 — Medium
**Audit IDs**: M-2, coverage gaps from audit
**Domain**: `tests/integration/`, all `crates/*/tests/`
**Status**: Not started
**Prerequisite**: Run AFTER P0, P1, P2 tasks are complete

---

## Testing Mandate

> **Inherited from [_index.md](_index.md).** Read and internalize the full
> Testing Mandate before writing any code or any test in this task.

**Domain-specific enforcement — this task IS about test quality:**

- This task exists because the test suite failed to catch C-1, C-2, C-3, and
  C-4. Every test added here must be **proven to fail** when the feature it
  guards is broken. The audit found tests that passed while features were
  non-functional. That cannot happen again.
- Do NOT add tests that duplicate tests written in P0/P1/P2 tasks. This task
  covers gaps NOT addressed by those domain tasks.
- The `#[ignore]` annotation is treated as a code smell. Any test marked
  `#[ignore]` must have a tracking issue linked. Bare `#[ignore]` without
  justification is banned.

---

## Problem Statement

The test suite has structural problems that allowed silent failures to persist
undetected:

1. **GAP-019 test is stale and misleading** — Tests a scenario that's
   impossible (manual config edit expecting automatic file creation) while the
   actual CLI command `repo add-tool` DOES trigger sync.
2. **No tests verify secondary paths** — Every tool test only checks the
   primary config file, allowing C-1 to go undetected.
3. **No tests verify hook execution during sync** — Allowing C-3 to go
   undetected.
4. **No MCP runtime invocation tests** — Only protocol structure is tested.
5. **No tests verify extension failure behavior** — Allowing C-4 to go
   undetected.

---

## Inventory of Known Issues

### Stale tests

| Test | File:Line | Problem | Action |
|------|-----------|---------|--------|
| `gap_019_add_tool_triggers_sync` | `mission_tests.rs:962` | Tests impossible scenario (manual config edit → expects auto file creation). The `repo add-tool` CLI already triggers sync. | Rewrite to test `add-tool` CLI or delete |

### Coverage gaps NOT addressed by P0/P1/P2

| Gap | Risk | Covered by |
|-----|------|-----------|
| Secondary paths never asserted | HIGH | **P0** covers this |
| No hook execution test during sync | MEDIUM | **P1** covers this |
| Extension success-on-stub | MEDIUM | **P1** covers this |
| MCP runtime invocation | MEDIUM | **P2** covers this |
| No end-to-end CLI test (`repo init && repo sync && verify files`) | MEDIUM | **This task** |
| No test for `repo check` detecting real drift | MEDIUM | **This task** |
| No test for `repo fix` correcting real drift | MEDIUM | **This task** |
| No test for ledger recording after sync | LOW | **This task** |
| No test for `repo add-tool` then `repo sync` sequence | LOW | **This task** |

---

## Implementation Plan

### Step 1: Fix or delete GAP-019 test

**File**: `tests/integration/src/mission_tests.rs:960-980`

The current test manually edits `config.toml` and expects files to appear —
this is impossible without running a command. Two options:

**Option A (preferred)**: Rewrite to test the actual `add-tool` CLI:
```rust
#[test]
fn gap_019_add_tool_triggers_sync() {
    let mut repo = TestRepo::new();
    repo.init_git();
    repo.init_repo_manager("standard", &[], &[]);

    // Use the actual add-tool command
    repo.run_command(&["add-tool", "vscode"]);

    // Verify sync was triggered
    let vscode_exists = repo.root().join(".vscode/settings.json").exists();
    assert!(
        vscode_exists,
        "add-tool should trigger sync and create .vscode/settings.json"
    );
}
```

**Option B**: Delete the test entirely if `add-tool` triggering sync is
already tested elsewhere.

### Step 2: Add end-to-end CLI pipeline test

**File**: New test in `tests/integration/`

Test the full user journey:

```
repo init --mode standard --tools vscode,claude --presets env:python
repo sync
→ verify .vscode/settings.json exists with correct content
→ verify CLAUDE.md exists with managed block
→ verify .claude/rules/ directory exists (secondary path)
→ verify .repository/ledger.toml records the sync
```

This is the "smoke test" that catches category-level regressions.

### Step 3: Add `repo check` drift detection test

Test that `repo check` correctly identifies when a managed file has been
modified outside of `repo sync`:

1. `repo init && repo sync` → healthy
2. Manually delete `CLAUDE.md`
3. `repo check` → reports missing file
4. `repo sync` → recreates it

### Step 4: Add `repo fix` correction test

Test that `repo fix` (which delegates to sync) actually corrects drift:

1. `repo init && repo sync` → healthy
2. Manually corrupt a managed block in `.cursorrules`
3. `repo fix` → restores correct content
4. `repo check` → healthy

### Step 5: Add ledger recording test

After `repo sync`, verify that `.repository/ledger.toml` (or equivalent)
contains entries recording what was synced, when, and the hash/content.

### Step 6: Audit all existing `#[ignore]` annotations

Search for every `#[ignore]` in the test codebase. For each one:

- If the feature is now implemented → remove `#[ignore]` and verify test passes
- If the feature is still missing → add a comment with a tracking reference
- If the test logic is wrong → fix or delete

### Step 7: Review test assertion quality

Scan all test files for these patterns (automated where possible):

- `assert!(result.is_ok())` without checking side effects
- `assert_eq!(result, Ok(()))` without checking what `Ok` means
- Tests with only one assertion that checks a return value

Flag these for review. Not all need changing, but each should be evaluated
for whether it would catch a regression.

---

## Acceptance Criteria

- [ ] GAP-019 test is fixed or removed (no `#[ignore]` annotation)
- [ ] End-to-end CLI pipeline test exists and passes
- [ ] `repo check` drift detection is tested
- [ ] `repo fix` correction is tested
- [ ] Ledger recording after sync is tested
- [ ] Zero `#[ignore]` tests without tracking references
- [ ] All new tests fail when the feature they guard is broken
- [ ] `cargo clippy` clean, `cargo test` passes (all crates)

---

## Files to Modify

| File | Change |
|------|--------|
| `tests/integration/src/mission_tests.rs` | Fix/delete GAP-019 test |
| `tests/integration/src/` | Add end-to-end CLI pipeline test |
| `tests/integration/src/` | Add check/fix drift tests |
| All test files with `#[ignore]` | Audit and resolve |

---

## Dependencies

- **Depends on**: P0, P1, P2 (run this task LAST — it verifies the others)
- **Blocks**: Nothing
- **Must not duplicate**: Tests already created by P0/P1/P2 tasks

---

## Anti-Pattern Checklist (use during implementation)

Before marking this task complete, verify that NONE of these exist in the
test suite:

- [ ] No `#[ignore]` without a linked tracking issue
- [ ] No `assert!(result.is_ok())` as the sole assertion in a test
- [ ] No test that passes when the feature under test is commented out
- [ ] No test that only checks struct construction, not method behavior
- [ ] No test with TODO/FIXME comments indicating incomplete logic
- [ ] No test that asserts primary path exists when secondary paths are the
      subject of the test

---

*Task created: 2026-02-22*
*Source: [Deep Implementation Audit](../audits/2026-02-22-deep-implementation-audit.md) — M-2, coverage gaps*
