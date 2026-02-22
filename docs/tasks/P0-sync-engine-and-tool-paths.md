# P0: Sync Engine & Tool Paths

**Priority**: P0 — Critical
**Audit IDs**: C-1, C-2, M-3
**Domain**: `crates/repo-tools/`
**Status**: Not started

---

## Testing Mandate

> **Inherited from [_index.md](_index.md).** Read and internalize the full
> Testing Mandate before writing any code or any test in this task.

**Domain-specific enforcement:**

- Every test in this task MUST assert that the **specific secondary path** was
  written — not just the primary path. A test that asserts `CLAUDE.md` exists
  while ignoring `.claude/rules/` is a **false-positive test** and must be
  rejected.
- Every test MUST fail when the `additional_paths` syncing code is removed.
  Verify this by commenting out the new sync code and running the test before
  considering it valid.
- Do NOT write tests that only verify struct fields or function return values.
  Assert **file system side effects**: files exist, contain expected content,
  have correct managed blocks.

---

## Problem Statement

The sync engine writes only the **primary** config path for each tool. Every
tool that declares `additional_paths` in its `ToolIntegrationConfig` has those
paths silently ignored during `sync()`. Users see `repo sync` succeed and
believe all tool configs are up to date — but secondary paths are never
created or updated.

This is the **core value proposition** of the product (tool configuration
synchronization). When it doesn't sync all declared paths, the tool is lying
about what it did.

---

## Affected Tools (7 total, not 5)

| Tool | Primary path | Additional path(s) | Additional type |
|------|-------------|--------------------|--------------------|
| `claude` | `CLAUDE.md` | `.claude/rules/` | Rules directory |
| `copilot` | `.github/copilot-instructions.md` | `.github/instructions/` | Rules directory |
| `cline` | `.clinerules` | `.clinerules/` | Rules directory |
| `zed` | `.rules` | `.zed/settings.json` | JSON config |
| `aider` | `.aider.conf.yml` | `CONVENTIONS.md` | Markdown file |
| `roo` | `.roo/rules/` | `.roomodes` | Config file |
| `jetbrains` | `.aiassistant/rules/` | `.aiignore` | Ignore file |

**Note**: `antigravity` is a separate sub-issue (C-2, see below).

---

## Root Cause Analysis

### C-1: `generic.rs:278-289` — `sync()` dispatches on `config_type` only

```rust
fn sync(&self, context: &SyncContext, rules: &[Rule]) -> Result<()> {
    match self.definition.integration.config_type {
        ConfigType::Text     => self.sync_text(context, rules),
        ConfigType::Json     => self.sync_json(context, rules),
        ConfigType::Markdown => self.sync_markdown(context, rules),
        ConfigType::Yaml     => self.sync_yaml(context, rules),
        ConfigType::Toml     => self.sync_yaml(context, rules),
    }
}
```

This method syncs the **primary** path using the primary `config_type`. It
never reads `self.definition.integration.additional_paths`. Each
`sync_text/json/markdown/yaml` method internally resolves only the primary
`config_path`.

### C-2: `antigravity.rs:22` — wrong path type

```rust
config_path: ".agent/rules.md".into(),  // file path
// but:
supports_rules_directory: true,         // claims directory support
```

The path `.agent/rules.md` is a file. `is_directory_config()` checks for
trailing `/` and returns `false`. So `sync_to_directory()` is never called.
Antigravity writes a single flat rules file instead of individual rule files
per rule.

### M-3: `translator/capability.rs:52-55` — commented-out capability code

```rust
// Rules directory: Future enhancement
// if tool.capabilities.supports_rules_directory {
//     // Different handling for directory-based tools
// }
```

The translator ignores the `supports_rules_directory` capability entirely. The
actual directory sync logic lives in `GenericToolIntegration` path detection
(trailing `/`), not in the translator. This commented code is dead but the
flag is pure metadata with no enforcement.

---

## Implementation Plan

### Step 1: Fix `additional_paths` syncing in `generic.rs`

**File**: `crates/repo-tools/src/generic.rs`

After the primary `sync()` dispatches, iterate `additional_paths` and sync
each one. Each additional path needs its own type inference:

- Paths ending in `/` → directory sync (create dir, write one file per rule)
- Paths ending in `.json` → JSON sync
- Paths ending in `.md` → Markdown sync
- Paths ending in `.yml` or `.yaml` → YAML sync
- Everything else → Text sync

**Key constraint**: The additional path may have a DIFFERENT `ConfigType` than
the primary path (e.g., Zed's primary is `.rules` (Text) but additional is
`.zed/settings.json` (JSON)). Do NOT reuse the primary `config_type`.

**Approach**: Add a method `fn sync_additional_paths(&self, context: &SyncContext, rules: &[Rule]) -> Result<()>` that iterates additional paths, infers type from extension, and dispatches. Call it at the end of `sync()`.

### Step 2: Fix antigravity path

**File**: `crates/repo-tools/src/antigravity.rs`

Change line 22:
```rust
// FROM:
config_path: ".agent/rules.md".into(),
// TO:
config_path: ".agent/rules/".into(),
```

This makes `is_directory_config()` return `true`, enabling per-rule file
creation in `.agent/rules/`.

### Step 3: Clean up translator dead code

**File**: `crates/repo-tools/src/translator/capability.rs`

Either:
- **Option A**: Remove the commented-out code entirely (it's dead)
- **Option B**: Implement the capability check if it adds value

Prefer Option A unless there's a concrete use case for translator-level
capability awareness.

### Step 4: Write tests (after implementation is complete and compiling)

**Required tests** (minimum — add more as implementation reveals edge cases):

1. **test_sync_writes_claude_rules_directory** — Sync Claude tool, assert
   both `CLAUDE.md` AND `.claude/rules/` exist with correct content.

2. **test_sync_writes_copilot_instructions_directory** — Sync Copilot, assert
   both `.github/copilot-instructions.md` AND `.github/instructions/` exist.

3. **test_sync_writes_zed_settings_json** — Sync Zed, assert both `.rules`
   AND `.zed/settings.json` exist with correct JSON structure.

4. **test_sync_writes_aider_conventions** — Sync Aider, assert both
   `.aider.conf.yml` AND `CONVENTIONS.md` exist.

5. **test_sync_writes_cline_rules_directory** — Sync Cline, assert both
   `.clinerules` AND `.clinerules/` directory exists.

6. **test_sync_writes_roo_roomodes** — Sync Roo, assert both `.roo/rules/`
   AND `.roomodes` exist.

7. **test_sync_writes_jetbrains_aiignore** — Sync JetBrains, assert both
   `.aiassistant/rules/` AND `.aiignore` exist.

8. **test_antigravity_uses_rules_directory** — Sync Antigravity, assert
   `.agent/rules/` is a directory containing individual rule files (not a
   single `.agent/rules.md` file).

9. **test_additional_path_type_inference** — Verify that `.json` paths get
   JSON sync, `.md` paths get Markdown sync, `/`-ending paths get directory
   sync.

10. **test_additional_path_content_correctness** — Verify that secondary path
    files contain managed blocks with correct rule content, not empty files.

**Negative tests:**

11. **test_empty_additional_paths_no_extra_files** — Tool with
    `additional_paths: vec![]` produces only the primary file.

12. **test_additional_path_failure_does_not_corrupt_primary** — If a secondary
    path write fails (e.g., permission denied), the primary file should still
    be correct.

---

## Acceptance Criteria

- [ ] `repo sync` with Claude tool creates both `CLAUDE.md` and `.claude/rules/<rule>.md`
- [ ] `repo sync` with all 7 affected tools writes ALL declared paths
- [ ] `repo sync` with Antigravity creates `.agent/rules/` as a directory
- [ ] All 12+ tests pass
- [ ] Removing the `sync_additional_paths` method causes tests 1-10 to fail
- [ ] `cargo clippy` clean, `cargo test` passes (all crates)

---

## Files to Modify

| File | Change |
|------|--------|
| `crates/repo-tools/src/generic.rs` | Add `sync_additional_paths()`, call from `sync()` |
| `crates/repo-tools/src/antigravity.rs` | Fix `config_path` to `.agent/rules/` |
| `crates/repo-tools/src/translator/capability.rs` | Remove dead commented code |
| `crates/repo-tools/tests/` or inline `#[cfg(test)]` | Add 12+ tests |

---

## Out of Scope

- Changes to tool definitions (tool.toml files in `repo-meta`)
- Changes to the ledger system
- MCP handler changes (covered in P2-mcp-server.md)
- Hook integration (covered in P1-hooks-system.md)

---

*Task created: 2026-02-22*
*Source: [Deep Implementation Audit](../audits/2026-02-22-deep-implementation-audit.md) — C-1, C-2, M-3*
