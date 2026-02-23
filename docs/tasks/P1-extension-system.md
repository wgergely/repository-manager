# P1: Extension System — Stop Lying About Success

**Priority**: P1 — High
**Audit ID**: C-4
**Domain**: `crates/repo-cli/`, `crates/repo-mcp/`, `crates/repo-extensions/`
**Status**: Not started

---

## Testing Mandate

> **Inherited from [_index.md](_index.md).** Read and internalize the full
> Testing Mandate before writing any code or any test in this task.

**Domain-specific enforcement:**

- Do NOT test that stub functions return `Ok(())` — that is exactly the
  problem this task exists to fix. Stubs returning `Ok` is the bug.
- If extensions remain unimplemented after this task, tests MUST assert that
  calling them returns an **error**, not success.
- If extensions are implemented, tests MUST assert **observable outcomes**
  (extension installed to disk, extension listed in registry, extension
  removed from disk).

---

## Problem Statement

All 5 extension lifecycle operations (install, add, init, remove, list) return
success status in both CLI and MCP interfaces while printing
`"(stub - not yet implemented)"`. A caller (human or AI agent via MCP) cannot
distinguish a stub response from a real one without parsing message text.

This is a trust violation: the tool says it did something, but it didn't.

---

## Affected Operations

| Operation | CLI location | MCP location | Current return |
|-----------|-------------|-------------|---------------|
| `extension install` | `extension.rs:11-33` | `handlers.rs:686-696` | `Ok(())` / `success:true` |
| `extension add` | `extension.rs:36-66` | `handlers.rs:705-728` | `Ok(())` / `success:true` |
| `extension init` | `extension.rs:69-84` | `handlers.rs:737-747` | `Ok(())` / `success:true` |
| `extension remove` | `extension.rs:87-102` | `handlers.rs:756-766` | `Ok(())` / `success:true` |
| `extension list` | `extension.rs:105-147` | `handlers.rs:769-794` | Known list / `success:true` |

---

## Decision Required

Before implementing, decide which path to take:

### Option A: Return proper errors (recommended if extensions are deferred)

Replace all stub functions with:

```rust
pub fn handle_extension_install(source: &str, no_activate: bool) -> Result<()> {
    Err(anyhow::anyhow!(
        "Extension system is not yet implemented. \
         Track progress at: <issue-url>"
    ))
}
```

MCP handlers return `Error::NotImplemented("extension_install")` (same
pattern as `git_push`/`git_pull`/`git_merge` in H-1).

**Pros**: Honest, consistent, easy, prevents false trust.
**Cons**: Extensions don't work. But they don't work now either — this just
stops lying about it.

### Option B: Implement the extension system

Build a real extension lifecycle:

1. Extension registry (`~/.config/repo/extensions/` or `.repository/extensions/`)
2. `install` — clone/download extension to registry
3. `add` — activate extension for current project
4. `init` — scaffold a new extension
5. `remove` — deactivate/uninstall
6. `list` — show installed + available

**Pros**: Feature works. **Cons**: Large scope; probably Phase 3+ work.

### Option C: Remove the extension commands entirely

Strip the CLI subcommands and MCP tools. Re-add when ready to implement.

**Pros**: Cleanest. No dead code. **Cons**: Breaks API contract if anyone
references these commands.

**Recommendation**: Option A for now. It's a 30-minute fix that eliminates the
trust violation. Option B is a separate task for when extensions become a
priority.

---

## Implementation Plan (Option A)

### Step 1: Fix CLI extension handlers

**File**: `crates/repo-cli/src/commands/extension.rs`

Replace each function body:

```rust
pub fn handle_extension_install(source: &str, _no_activate: bool) -> Result<()> {
    Err(anyhow::anyhow!(
        "Extension install is not yet implemented. Source: {source}"
    ))
}
```

Repeat for `add`, `init`, `remove`. For `list`, return only the known
built-in extensions without claiming any are "installed".

### Step 2: Fix MCP extension handlers

**File**: `crates/repo-mcp/src/handlers.rs`

Replace each handler:

```rust
async fn handle_extension_install(arguments: Value) -> Result<Value> {
    Err(Error::NotImplemented("extension_install".to_string()))
}
```

This makes them consistent with the existing `git_push`/`git_pull`/`git_merge`
pattern already in the same file.

### Step 3: Remove TODO comments that claim implementation is coming

Clean up misleading TODO comments in the stub functions. If the feature is
deferred, it should not have inline TODOs suggesting it's about to be
implemented.

### Step 4: Write tests

**Required tests:**

1. **test_extension_install_returns_error** — Call
   `handle_extension_install()`. Assert it returns `Err`, not `Ok`.

2. **test_extension_add_returns_error** — Same for `add`.

3. **test_extension_init_returns_error** — Same for `init`.

4. **test_extension_remove_returns_error** — Same for `remove`.

5. **test_extension_list_returns_known_only** — Call `list`. Assert it returns
   the known extension names without claiming any are installed.

6. **test_mcp_extension_install_returns_not_implemented** — Send MCP request
   for `extension_install`. Assert error response with appropriate error code.

7. **test_mcp_extension_handlers_consistent_with_git** — Verify that
   extension handlers return the same error type as `git_push`/`git_pull`.

---

## Acceptance Criteria

- [ ] No extension operation returns `success:true` or `Ok(())`
- [ ] CLI extension commands return user-friendly error messages
- [ ] MCP extension tools return `Error::NotImplemented`
- [ ] `extension list` still works for known extensions (doesn't error)
- [ ] All 7 tests pass
- [ ] Changing any handler back to `Ok(())` causes the relevant test to fail
- [ ] `cargo clippy` clean, `cargo test` passes (all crates)

---

## Files to Modify

| File | Change |
|------|--------|
| `crates/repo-cli/src/commands/extension.rs` | Return errors instead of `Ok(())` |
| `crates/repo-mcp/src/handlers.rs` | Return `Error::NotImplemented` for extension handlers |
| `crates/repo-cli/tests/` or inline `#[cfg(test)]` | Add 7 tests |

---

## Dependencies

- **Depends on**: Nothing (independent)
- **Blocks**: Nothing
- **Can parallelize with**: P1-hooks-system

---

*Task created: 2026-02-22*
*Source: [Deep Implementation Audit](../audits/2026-02-22-deep-implementation-audit.md) — C-4*
