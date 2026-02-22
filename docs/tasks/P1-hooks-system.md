# P1: Hooks System — Wire Sync and Agent Events

**Priority**: P1 — High
**Audit ID**: C-3
**Domain**: `crates/repo-core/`, `crates/repo-cli/`
**Status**: Not started

---

## Testing Mandate

> **Inherited from [_index.md](_index.md).** Read and internalize the full
> Testing Mandate before writing any code or any test in this task.

**Domain-specific enforcement:**

- Every hook-event test MUST verify that the hook **actually executed** — not
  that `run_hooks()` returned `Ok`. A stub `run_hooks()` that returns `Ok`
  without executing anything would pass a return-value-only test.
- Test execution by asserting **side effects**: a hook script that creates a
  marker file, writes to a log, or sets an environment variable. The test then
  checks for that side effect.
- Negative test required: configure a hook, run a command that should NOT
  trigger it, verify the hook did NOT execute.

---

## Problem Statement

The hooks system defines 8 events. Only 4 are wired:

| Hook event | Status | Call site |
|------------|--------|-----------|
| `PreBranchCreate` | Fires | `branch.rs:69` |
| `PostBranchCreate` | Fires | `branch.rs:81` |
| `PreBranchDelete` | Fires | `branch.rs:117` |
| `PostBranchDelete` | Fires | `branch.rs:125` |
| **`PreSync`** | **Never fires** | No call site |
| **`PostSync`** | **Never fires** | No call site |
| **`PreAgentComplete`** | **Never fires** | No call site |
| **`PostAgentComplete`** | **Never fires** | No call site |

Users can configure `pre-sync` and `post-sync` hooks in `config.toml`. These
hooks will be silently ignored. No warning is printed. The user believes their
automation runs — it doesn't.

---

## Root Cause

`crates/repo-cli/src/commands/sync.rs` does not call `run_hooks()` at any
point during the sync operation. The `run_hooks` function exists and works
correctly (proven by branch events). The wiring is simply missing.

The agent-complete events (`PreAgentComplete`, `PostAgentComplete`) have no
consumer because the agent subsystem was removed (2026-02-18). These events
are orphaned.

---

## Implementation Plan

### Step 1: Wire `PreSync` and `PostSync` in sync command

**File**: `crates/repo-cli/src/commands/sync.rs`

Add `run_hooks` calls around the sync operation:

```rust
// Before sync engine runs:
run_hooks(&hooks, HookEvent::PreSync, &hook_context, &repo_root)?;

// ... existing sync logic ...

// After sync engine completes successfully:
run_hooks(&hooks, HookEvent::PostSync, &hook_context, &repo_root)?;
```

**Important**: Follow the same pattern as `branch.rs` — load hooks from
config, construct `HookContext` with relevant metadata (which tools synced,
how many files changed, etc.), handle errors with user-friendly warnings.

**Design decision**: If `PreSync` hook fails (returns non-zero), should sync
be aborted? Look at the `PreBranchCreate` pattern — if it aborts on failure,
`PreSync` should too. Consistency matters.

### Step 2: Decide on agent-complete events

The agent subsystem is gone. Two options:

- **Option A (preferred)**: Remove `PreAgentComplete` and `PostAgentComplete`
  from the `HookEvent` enum. Dead code should be deleted. If agent support
  returns in the future, the events can be re-added.
- **Option B**: Keep them but document that they are reserved for future use.
  Add a `#[allow(dead_code)]` annotation.

### Step 3: Add hook context metadata for sync events

**File**: `crates/repo-core/src/hooks.rs`

The `HookContext` struct may need additional fields for sync-specific data
(e.g., list of tools synced, file count changed). Check what metadata the
existing branch hooks pass and extend the context appropriately for sync.

### Step 4: Write tests (after implementation compiles)

**Required tests:**

1. **test_pre_sync_hook_fires** — Configure a `pre-sync` hook that creates a
   marker file. Run `sync`. Assert marker file exists.

2. **test_post_sync_hook_fires** — Configure a `post-sync` hook that creates
   a marker file. Run `sync`. Assert marker file exists.

3. **test_pre_sync_hook_failure_aborts_sync** — Configure a `pre-sync` hook
   that exits with code 1. Run `sync`. Assert that sync did NOT run (no tool
   files created). This tests that pre-hooks are gates, not notifications.

4. **test_post_sync_hook_receives_context** — Configure a `post-sync` hook
   that writes `$REPO_HOOK_EVENT` (or equivalent) to a file. Assert the value
   is `"post-sync"`.

5. **test_sync_with_no_hooks_configured** — Run sync with no hooks in config.
   Assert sync succeeds normally (no regression).

6. **test_hook_not_triggered_by_wrong_event** — Configure only a `pre-sync`
   hook. Run `branch add`. Assert the hook did NOT execute.

**If Option A (remove agent events):**

7. **test_agent_events_removed_from_enum** — Compile-time check. If someone
   re-adds agent events without wiring them, the test should catch it. This
   can be a simple `HookEvent::iter().count()` assertion.

---

## Acceptance Criteria

- [ ] `pre-sync` hooks execute before `repo sync` runs the sync engine
- [ ] `post-sync` hooks execute after `repo sync` completes successfully
- [ ] Failed `pre-sync` hooks abort the sync operation
- [ ] Agent-complete events are either removed or documented as reserved
- [ ] All 6+ tests pass
- [ ] Removing the `run_hooks` calls from `sync.rs` causes tests 1-4 to fail
- [ ] `cargo clippy` clean, `cargo test` passes (all crates)

---

## Files to Modify

| File | Change |
|------|--------|
| `crates/repo-cli/src/commands/sync.rs` | Add `run_hooks` calls for PreSync/PostSync |
| `crates/repo-core/src/hooks.rs` | Remove agent events OR add sync context fields |
| `tests/integration/` or inline `#[cfg(test)]` | Add 6+ tests |

---

## Dependencies

- **Depends on**: Nothing (can start immediately)
- **Blocks**: P3-test-hygiene (hook coverage gaps)
- **Related to**: P0-sync-engine (sync command is modified by both tasks; coordinate)

---

*Task created: 2026-02-22*
*Source: [Deep Implementation Audit](../audits/2026-02-22-deep-implementation-audit.md) — C-3*
