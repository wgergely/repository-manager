# P2: MCP Server — Git Primitives and Initialization

**Priority**: P2 — High
**Audit IDs**: H-1, H-2
**Domain**: `crates/repo-mcp/`
**Status**: Not started

---

## Testing Mandate

> **Inherited from [_index.md](_index.md).** Read and internalize the full
> Testing Mandate before writing any code or any test in this task.

**Domain-specific enforcement:**

- MCP tests MUST test **end-to-end tool invocation**, not just protocol
  structure. The existing `protocol_compliance_tests.rs` verifies JSON-RPC
  framing — that is necessary but insufficient. Tests must verify that calling
  `git_push` actually performs a push (or returns a meaningful error).
- Do NOT test that `handle_tool_call("git_push")` returns a `Value` — test
  that the return value contains expected fields with expected types.
- Initialize validation tests must verify that the server **rejects** requests
  when outside a repository, not just that it initializes.

---

## Problem Statement

### H-1: Git primitives announced but always error

The MCP server advertises `git_push`, `git_pull`, and `git_merge` in
`get_tool_definitions()` (clients see them in the tools list). But all three
always return `Error::NotImplemented`:

```rust
// handlers.rs:38-41
"git_push"  => Err(Error::NotImplemented("git_push".to_string())),
"git_pull"  => Err(Error::NotImplemented("git_pull".to_string())),
"git_merge" => Err(Error::NotImplemented("git_merge".to_string())),
```

The CLI equivalents (`repo push`, `repo pull`, `repo merge`) are fully
implemented. The MCP handlers just need to delegate to the same underlying
logic.

### H-2: `initialize()` skips validation

```rust
// server.rs:76-79
tracing::warn!("Repository configuration loading not yet implemented");
tracing::warn!("Repository structure validation not yet implemented");
```

The server sets `initialized = true` without checking whether a valid
`.repository/` directory exists. Subsequent tool calls fail with confusing
errors when invoked outside a valid repository.

---

## Implementation Plan

### Step 1: Implement MCP git_push handler

**File**: `crates/repo-mcp/src/handlers.rs`

Replace the `NotImplemented` return with actual logic. The handler should:

1. Parse arguments (remote name, branch name, force flag)
2. Resolve the repository root from server context
3. Call the same git push logic used by `repo-cli/src/commands/push.rs`
4. Return structured result (success/failure, remote URL, ref pushed)

**Important**: The CLI command already works. Do not re-implement git push
from scratch. Extract the core logic into a shared function (if not already
shared) and call it from both CLI and MCP.

### Step 2: Implement MCP git_pull handler

Same pattern as Step 1. Parse arguments, delegate to shared git pull logic.

### Step 3: Implement MCP git_merge handler

Same pattern. Parse arguments (branch to merge, strategy), delegate to shared
git merge logic.

### Step 4: Add repository validation to `initialize()`

**File**: `crates/repo-mcp/src/server.rs`

Replace the `tracing::warn!` stubs with actual checks:

```rust
pub async fn initialize(&mut self) -> Result<()> {
    // Check .repository/ directory exists
    let config_dir = self.root.join(".repository");
    if !config_dir.is_dir() {
        return Err(Error::InvalidRequest(
            "Not a repository-manager project. Run `repo init` first.".into()
        ));
    }

    // Load and validate config
    let config_path = config_dir.join("config.toml");
    if !config_path.is_file() {
        return Err(Error::InvalidRequest(
            "Missing .repository/config.toml. Run `repo init` to create it.".into()
        ));
    }

    // Proceed with initialization
    self.initialized = true;
    Ok(())
}
```

### Step 5: Write tests

**Required tests:**

1. **test_mcp_git_push_delegates_to_core** — Set up a git repo with a remote,
   call `git_push` via MCP handler. Assert the remote received the push.

2. **test_mcp_git_pull_delegates_to_core** — Set up a git repo with upstream
   changes, call `git_pull` via MCP. Assert local repo has the changes.

3. **test_mcp_git_merge_delegates_to_core** — Set up a git repo with two
   branches, call `git_merge` via MCP. Assert merge completed.

4. **test_mcp_git_push_returns_structured_result** — Call `git_push`. Assert
   the return `Value` has `success`, `remote`, `ref` fields (not just any
   `Ok`).

5. **test_mcp_git_push_error_on_no_remote** — Call `git_push` on a repo with
   no remote configured. Assert meaningful error, not a panic.

6. **test_mcp_initialize_rejects_non_repository** — Create a temp dir without
   `.repository/`. Call `initialize()`. Assert it returns an error.

7. **test_mcp_initialize_rejects_missing_config** — Create `.repository/` but
   no `config.toml`. Call `initialize()`. Assert error.

8. **test_mcp_initialize_succeeds_in_valid_repo** — Create `.repository/`
   with `config.toml`. Call `initialize()`. Assert success.

9. **test_mcp_tool_call_before_initialize** — Call any tool before
   `initialize()`. Assert appropriate error about server not being
   initialized.

---

## Acceptance Criteria

- [ ] `git_push`, `git_pull`, `git_merge` MCP tools perform real operations
- [ ] MCP `initialize()` validates repository structure before proceeding
- [ ] MCP `initialize()` returns clear error when outside a repository
- [ ] All 9 tests pass
- [ ] `cargo clippy` clean, `cargo test` passes (all crates)

---

## Files to Modify

| File | Change |
|------|--------|
| `crates/repo-mcp/src/handlers.rs` | Implement git_push, git_pull, git_merge handlers |
| `crates/repo-mcp/src/server.rs` | Add repository validation to `initialize()` |
| `crates/repo-git/src/` (possibly) | Extract shared logic if CLI-only currently |
| `crates/repo-mcp/tests/` | Add 9 tests |

---

## Dependencies

- **Depends on**: P0-sync-engine (uses same tool definitions)
- **Related to**: P1-extension-system (MCP handlers in same file)
- **Can parallelize with**: P2-preset-providers

---

*Task created: 2026-02-22*
*Source: [Deep Implementation Audit](../audits/2026-02-22-deep-implementation-audit.md) — H-1, H-2*
