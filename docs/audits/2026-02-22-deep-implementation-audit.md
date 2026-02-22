# Deep Implementation Audit — Repository Manager

**Date**: 2026-02-22
**Method**: 8 parallel subagents reading every .rs source file, cross-referenced against all ADR/design/plan documents and direct source verification
**Branch**: `claude/product-health-audit-Qb5Cc`

---

## Executive Summary

The codebase is substantially functional but contains **systematic silent gaps** that the February 17 audit did not catch. The most serious problem is not missing code but **code that claims to work but doesn't**: 5 tool integrations declare secondary config paths that are never written, 4 hook events are defined but never fired, and 8 extension system functions return `"(stub - not yet implemented)"` while reporting `success: true`.

A secondary finding is that the February 17 technical audit described features (SuperpowersProvider, MCP superpowers tools) that no longer exist in the codebase, while missing features that do exist (hooks system, governance commands, `repo open`).

---

## Verified Gap Inventory

### CRITICAL — Silent failures (no error returned, wrong behavior)

#### C-1: `additional_paths` never synced in GenericToolIntegration
**File**: `crates/repo-tools/src/generic.rs:278-289`
**Impact**: 5 tools write only their primary config file. Secondary paths are declared, returned by `config_locations()` as metadata, but the `sync()` method dispatches only on primary `config_type` and never iterates `additional_paths`.

| Tool | Secondary path declared | Actually written |
|------|------------------------|-----------------|
| `claude` | `.claude/rules/` | ❌ Never |
| `copilot` | `.github/instructions/` | ❌ Never |
| `cline` | `.clinerules/` | ❌ Never |
| `zed` | `.zed/settings.json` | ❌ Never |
| `aider` | `CONVENTIONS.md` | ❌ Never |

All 5 tools have `supports_rules_directory: true` or secondary JSON paths declared. Tests pass because they only assert that the primary path exists — no test verifies the secondary paths.

**Root cause**:
```rust
// generic.rs:278-289 — never touches additional_paths
fn sync(&self, context: &SyncContext, rules: &[Rule]) -> Result<()> {
    match self.definition.integration.config_type {
        ConfigType::Text     => self.sync_text(context, rules),    // primary only
        ConfigType::Json     => self.sync_json(context, rules),    // primary only
        ConfigType::Markdown => self.sync_markdown(context, rules), // primary only
        ConfigType::Yaml     => self.sync_yaml(context, rules),    // primary only
        ConfigType::Toml     => self.sync_yaml(context, rules),    // primary only
    }
}
```

#### C-2: Antigravity wrong path — rules_dir capability non-functional
**File**: `crates/repo-tools/src/antigravity.rs:22`
**Impact**: `antigravity` declares `supports_rules_directory: true` but its primary path is `.agent/rules.md` (a file). `is_directory_config()` checks for a trailing `/`, returns false, so `sync_to_directory()` is never called. The tool writes one flat file instead of individual rule files.

**Fix**: Change path from `.agent/rules.md` to `.agent/rules/`.

#### C-3: PreSync/PostSync/PreAgentComplete/PostAgentComplete hooks never fire
**File**: `crates/repo-core/src/hooks.rs` defines 8 events; `crates/repo-cli/src/commands/branch.rs` calls `run_hooks` for only 4 of them.

| Hook event | Fires? |
|------------|--------|
| `pre-branch-create` | ✅ Yes — branch.rs:69 |
| `post-branch-create` | ✅ Yes — branch.rs:81 |
| `pre-branch-delete` | ✅ Yes — branch.rs:117 |
| `post-branch-delete` | ✅ Yes — branch.rs:125 |
| `pre-sync` | ❌ Never called |
| `post-sync` | ❌ Never called |
| `pre-agent-complete` | ❌ Never called (no agents) |
| `post-agent-complete` | ❌ Never called (no agents) |

Users can configure `pre-sync` hooks in `config.toml`; they will silently not execute. No warning is issued.

#### C-4: Extension system returns success but does nothing (CLI + MCP)
**Files**: `crates/repo-cli/src/commands/extension.rs`, `crates/repo-mcp/src/handlers.rs`

All extension lifecycle operations print `"(stub - not yet implemented)"` but return `exit 0` / `success: true`. A caller cannot distinguish a stub response from a real one without reading the message text.

| Operation | CLI line | MCP line | Returns |
|-----------|----------|----------|---------|
| `extension install` | ext.rs:25-29 | handlers.rs:686-696 | `OK` / `success:true` |
| `extension add` | ext.rs:47-49 | handlers.rs:705-720 | `OK` / `success:true` |
| `extension init` | ext.rs:76-78 | handlers.rs:737-747 | `OK` / `success:true` |
| `extension remove` | ext.rs:94-96 | handlers.rs:756-766 | `OK` / `success:true` |
| `extension list` | ext.rs:138 | handlers.rs:769-794 | Known list only, no installed tracking |

---

### HIGH — Explicit NotImplemented / acknowledged stubs

#### H-1: MCP git primitives return `Error::NotImplemented`
**File**: `crates/repo-mcp/src/handlers.rs:38-41`

```rust
"git_push" => Err(Error::NotImplemented("git_push".to_string())),
"git_pull" => Err(Error::NotImplemented("git_pull".to_string())),
"git_merge" => Err(Error::NotImplemented("git_merge".to_string())),
```

These 3 tools are listed in `get_tool_definitions()` (announced to MCP clients) but always error. The CLI equivalents (`repo push`, `repo pull`, `repo merge`) are fully implemented.

#### H-2: MCP `initialize()` skips validation
**File**: `crates/repo-mcp/src/server.rs:76-79`

```rust
// Repository configuration loading not yet implemented
tracing::warn!("Repository configuration loading not yet implemented");
// Repository structure validation not yet implemented
tracing::warn!("Repository structure validation not yet implemented");
```

The server initializes and sets `initialized = true` without checking whether a valid `.repository/` directory exists. Subsequent tool calls will fail if invoked outside a repository, with potentially confusing error messages.

#### H-3: NodeProvider.apply() is a no-op stub
**File**: `crates/repo-presets/src/node/node_provider.rs:124-129`

```rust
async fn apply(&self, _context: &Context) -> Result<ApplyReport> {
    Ok(ApplyReport::success(vec![
        "Node environment detection complete. This provider is detection-only.".to_string(),
    ]))
}
```

`check()` is real (detects `package.json`, `node_modules`, `node` on PATH). `apply()` is a stub returning fake success. No npm/yarn install is triggered.

#### H-4: RustProvider.apply() is a no-op stub
**File**: `crates/repo-presets/src/rust/rust_provider.rs:84-90`

```rust
async fn apply(&self, _context: &Context) -> Result<ApplyReport> {
    Ok(ApplyReport::success(vec![
        "Rust environment provider is detection-only.".to_string(),
        "No actions taken. Use rustup to manage Rust installations.".to_string(),
    ]))
}
```

Same pattern as NodeProvider. Detection works; apply does nothing.

---

### MEDIUM — Architectural deferrals

#### M-1: Config hierarchy layers 1 & 2 not implemented
**File**: `crates/repo-core/src/config/resolver.rs:104-110`

```rust
// Layer 1 - Global defaults (~/.config/repo-manager/config.toml)
tracing::debug!("Config layer 1 (global defaults) not yet implemented");
// Layer 2 - Organization config
tracing::debug!("Config layer 2 (organization config) not yet implemented");
```

Layers 3 (repo config) and 4 (local overrides) are fully implemented. No global or org-level defaults are ever loaded.

#### M-2: GAP-019 test is stale and misleading
**File**: `tests/integration/src/mission_tests.rs:961-980`

```rust
#[ignore = "GAP-019: add-tool does not yet trigger automatic sync"]
fn gap_019_add_tool_triggers_sync() { ... }
```

The test manually edits `config.toml` and immediately asserts that files appear on disk — an impossible scenario without running a command. Meanwhile, the actual `repo add-tool` CLI command DOES trigger sync (via `trigger_sync_and_report()` in `commands/tool.rs:79`). This test should be rewritten to invoke the CLI or deleted.

#### M-3: `supports_rules_directory` capability in translator is commented out
**File**: `crates/repo-tools/src/translator/capability.rs:52-55`

```rust
// Rules directory: Future enhancement
// if tool.capabilities.supports_rules_directory {
//     // Different handling for directory-based tools
// }
```

Directory syncing works via GenericToolIntegration path detection (primary path with `/`), not through the translator. The commented code is truly dead. However, this means the capability flag is pure metadata with no enforcement in the translation layer.

---

### LOW — Minor design gaps

#### L-1: YAML and TOML use TextWriter (full file replacement)
**File**: `crates/repo-tools/src/writer/registry.rs:28-29`

YAML and TOML config files are overwritten entirely on each sync instead of using AST-preserving writes. Intentional (noted in comment), but means user edits to YAML/TOML configs are lost on next sync.

#### L-2: `unreachable!()` in diff.rs
**File**: `crates/repo-content/src/diff.rs:118`
`(None, None) => unreachable!()` — this is defensive programming in a pattern match where both values being None is logically impossible. Low risk; not a functional gap.

---

## Features Removed Since February 17 Audit

The following were described as "fully implemented" in the Feb 17 technical audit but no longer exist in the codebase:

| Feature | Feb 17 claim | Reality |
|---------|-------------|---------|
| `SuperpowersProvider` | "Full lifecycle: clone from GitHub, install, enable in Claude settings.json, uninstall" | **Entirely removed.** No superpowers module anywhere. |
| `repo superpowers install/status/uninstall` CLI commands | "Fully implemented" | **Removed.** No `superpowers.rs` in commands/. |
| MCP `superpowers_install/status/uninstall` tools | Listed as implemented | **Replaced** by extension system (all stubs). |
| 10 ignored tests (GAP-004, GAP-019, etc.) | "10 ignored" | GAP-004 now passes; GAP-019 still ignored (stale); others removed. |

**Likely cause**: Superpowers were part of the vaultspec agent subsystem removed on 2026-02-18 per the roadmap update.

---

## Features Implemented But Not in ADR Tracking

The following exist and work but were listed as Phase 3 roadmap items or not mentioned at all:

| Feature | File | Status |
|---------|------|--------|
| `repo open` — launch editor in worktree | `commands/open.rs` (263 lines) | ✅ Real |
| `repo hooks list/add/remove` | `commands/hooks.rs` (276 lines) | ✅ Real |
| Hooks system (branch pre/post events) | `repo-core/src/hooks.rs` (409 lines) | ✅ Real |
| `repo rules-lint` | `commands/governance.rs` | ✅ Real |
| `repo rules-diff` | `commands/governance.rs` | ✅ Real |
| `repo rules-export --format agents` | `commands/governance.rs` | ✅ Real |
| `repo rules-import AGENTS.md` | `commands/governance.rs` | ✅ Real |
| `repo branch rename` | `commands/branch.rs:189` | ✅ Real |
| `repo config show` | `commands/config.rs` | ✅ Real |
| Extension system infrastructure | `commands/extension.rs`, `handlers.rs` | ⚠️ Scaffold (CLI+MCP stubs) |
| `Claude Desktop` tool integration | `claude_desktop.rs` | ✅ Real (14th tool, not in ADR count) |

---

## Test Coverage Gaps Discovered

| Gap | Risk |
|-----|------|
| No test asserts secondary paths are written (`.claude/rules/`, `.github/instructions/`, etc.) | HIGH — C-1 went undetected for this reason |
| No test that `pre-sync` or `post-sync` hooks fire during `repo sync` | MEDIUM — C-3 went undetected |
| GAP-019 test logic is wrong (doesn't test CLI, tests manual config edit) | MEDIUM — gives false impression of gap |
| No test for MCP tool invocation at runtime (only protocol structure) | MEDIUM |
| No test for extension list showing installed extensions | LOW |

---

## Phase 0 Status (Roadmap Cross-Reference)

| Item | Status | Evidence |
|------|--------|---------|
| 0.1 README rewrite | ✅ Done | README has value prop, tool table, install, quick start |
| 0.2 Config schema fix | Unknown | Not verified in this audit |
| 0.3 `repo init` → sync gap | ⚠️ By design | init does NOT auto-sync; prints "Next: run `repo sync`" |
| 0.4 GAP-004 integration test | ✅ Done | `gap_004_sync_applies_projections` is a passing test |

---

## Summary Table

| ID | Severity | Description | File:Line |
|----|----------|-------------|-----------|
| C-1 | Critical | `additional_paths` never synced (5 tools) | `generic.rs:278-289` |
| C-2 | Critical | Antigravity wrong path, rules_dir non-functional | `antigravity.rs:22` |
| C-3 | Critical | pre-sync/post-sync/agent hooks never fire | `hooks.rs` (no call sites in sync) |
| C-4 | Critical | Extension system returns `success:true` but does nothing | `extension.rs:25-96`, `handlers.rs:686-766` |
| H-1 | High | MCP git_push/pull/merge return NotImplemented | `handlers.rs:38-41` |
| H-2 | High | MCP initialize() skips repo validation | `server.rs:76-79` |
| H-3 | High | NodeProvider.apply() is detection-only stub | `node_provider.rs:124-129` |
| H-4 | High | RustProvider.apply() is detection-only stub | `rust_provider.rs:84-90` |
| M-1 | Medium | Config layers 1 & 2 (global/org) not implemented | `resolver.rs:104-110` |
| M-2 | Medium | GAP-019 test is stale and misleading | `mission_tests.rs:961` |
| M-3 | Medium | `supports_rules_directory` in translator commented out | `capability.rs:52-55` |
| L-1 | Low | YAML/TOML use full-file replacement (no AST preserve) | `registry.rs:28-29` |
| L-2 | Low | `unreachable!()` in diff.rs | `diff.rs:118` |

---

*Audit completed: 2026-02-22*
*Method: 8 parallel subagents + direct source verification*
*Branch: `claude/product-health-audit-Qb5Cc`*
