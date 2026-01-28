# Final Gap Closure Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Close the 2 remaining critical gaps (GAP-022, GAP-004) to achieve full sync functionality.

**Architecture:** Wire repo-tools integrations into ToolSyncer so that sync() properly generates tool configs using the existing tool integration implementations.

**Tech Stack:** Rust, repo-tools crate, repo-core crate

---

## Current State Analysis

### What's Already Done
- GAP-001/002/003: Push/Pull/Merge CLI commands ✅ (already in cli.rs)
- GAP-012/013: Node/Rust providers ✅ (already exist in repo-presets)
- GAP-018: MCP tool definitions ✅ (already in repo-mcp)

### Critical Gaps Remaining
| Gap | Description | Blocker |
|-----|-------------|---------|
| GAP-022 | ToolSyncer uses hardcoded configs instead of repo-tools | Blocks GAP-004 |
| GAP-004 | sync() doesn't apply projections | Blocked by GAP-022 |

### Root Cause
`ToolSyncer::get_tool_config_files()` has only 3 hardcoded tools:
- cursor → ".cursorrules"
- vscode → ".vscode/settings.json"
- claude → ".claude/config.json"

But `repo-tools` has proper integrations:
- CursorIntegration
- VscodeIntegration
- ClaudeIntegration
- WindsurfIntegration
- AntigravityIntegration
- GeminiIntegration
- GenericIntegration (fallback)

---

## Phase 1: Wire repo-tools into ToolSyncer (GAP-022)

### Task 1: Add repo-tools dependency to repo-core

**Files:**
- Modify: `crates/repo-core/Cargo.toml`

**Step 1: Add the dependency**

Add to `[dependencies]`:
```toml
repo-tools = { path = "../repo-tools" }
```

**Step 2: Verify build**

Run: `cargo check -p repo-core`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/repo-core/Cargo.toml
git commit -m "build(repo-core): add repo-tools dependency

Enables ToolSyncer to use proper tool integrations."
```

---

### Task 2: Refactor ToolSyncer to use repo-tools

**Files:**
- Modify: `crates/repo-core/src/sync/tool_syncer.rs`

**Step 1: Import repo-tools types**

Add to imports:
```rust
use repo_tools::{ToolIntegration, ToolRegistry};
```

**Step 2: Replace get_tool_config_files with repo-tools dispatch**

Replace the `get_tool_config_files` method:

```rust
    /// Get config files for a tool using repo-tools integrations
    ///
    /// Returns a list of (file_path, content) tuples for the tool's configuration files.
    fn get_tool_config_files(&self, tool_name: &str) -> Vec<(String, String)> {
        let registry = ToolRegistry::with_builtins();

        if let Some(integration) = registry.get(tool_name) {
            // Get the tool's config file path and generate content
            let config_path = integration.config_path();
            let content = integration.generate_config(&self.root);

            vec![(config_path.to_string_lossy().to_string(), content)]
        } else {
            // Fallback to generic integration for unknown tools
            let generic = repo_tools::GenericIntegration::new(tool_name);
            let config_path = generic.config_path();
            let content = generic.generate_config(&self.root);

            vec![(config_path.to_string_lossy().to_string(), content)]
        }
    }
```

**Step 3: Remove hardcoded generate methods**

Delete these methods:
- `generate_cursor_rules()`
- `generate_vscode_settings()`
- `generate_claude_config()`

**Step 4: Run tests**

Run: `cargo test -p repo-core`
Expected: Tests may need updates for new file paths

**Step 5: Commit**

```bash
git add crates/repo-core/src/sync/tool_syncer.rs
git commit -m "feat(repo-core): wire ToolSyncer to repo-tools integrations

Closes GAP-022. ToolSyncer now uses repo-tools for proper tool
configuration generation instead of hardcoded content.

Tools now properly supported:
- cursor, vscode, claude (existing)
- windsurf, antigravity, gemini (new)
- Generic fallback for unknown tools"
```

---

### Task 3: Update ToolSyncer tests

**Files:**
- Modify: `crates/repo-core/src/sync/tool_syncer.rs` (tests module)

**Step 1: Update tests to match new behavior**

The tests need to verify:
1. Known tools (cursor, vscode, claude, windsurf, etc.) get proper configs
2. Unknown tools get generic configs via GenericIntegration
3. File paths match what repo-tools expects

**Step 2: Run tests**

Run: `cargo test -p repo-core tool_syncer`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/repo-core/src/sync/tool_syncer.rs
git commit -m "test(repo-core): update ToolSyncer tests for repo-tools integration"
```

---

## Phase 2: Verify sync() Works End-to-End (GAP-004)

### Task 4: Add integration test for sync with tools

**Files:**
- Create: `crates/repo-core/tests/sync_integration_tests.rs`

**Step 1: Create integration test**

```rust
//! Integration tests for sync functionality

use repo_core::{Ledger, ToolSyncer};
use repo_fs::NormalizedPath;
use tempfile::tempdir;

#[test]
fn test_sync_creates_cursor_config() {
    let dir = tempdir().unwrap();
    let root = NormalizedPath::new(dir.path());
    let syncer = ToolSyncer::new(root.clone(), false);
    let mut ledger = Ledger::new();

    let actions = syncer.sync_tool("cursor", &mut ledger).unwrap();

    // Should have created the config file
    assert!(actions.iter().any(|a| a.contains("Created")));

    // File should exist with content
    let config_path = root.join(".cursorrules");
    assert!(config_path.exists());

    // Content should be from repo-tools, not hardcoded
    let content = std::fs::read_to_string(config_path.as_ref()).unwrap();
    assert!(content.contains("Cursor") || content.contains("cursor"));
}

#[test]
fn test_sync_creates_windsurf_config() {
    let dir = tempdir().unwrap();
    let root = NormalizedPath::new(dir.path());
    let syncer = ToolSyncer::new(root.clone(), false);
    let mut ledger = Ledger::new();

    // This should now work with repo-tools integration
    let actions = syncer.sync_tool("windsurf", &mut ledger).unwrap();

    assert!(actions.iter().any(|a| a.contains("Created")));
    assert!(!ledger.intents().is_empty());
}

#[test]
fn test_sync_unknown_tool_uses_generic() {
    let dir = tempdir().unwrap();
    let root = NormalizedPath::new(dir.path());
    let syncer = ToolSyncer::new(root.clone(), false);
    let mut ledger = Ledger::new();

    let actions = syncer.sync_tool("my-custom-tool", &mut ledger).unwrap();

    // Should still create a config using generic integration
    assert!(actions.iter().any(|a| a.contains("Created")));
}
```

**Step 2: Run integration tests**

Run: `cargo test -p repo-core sync_integration`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/repo-core/tests/sync_integration_tests.rs
git commit -m "test(repo-core): add sync integration tests

Verifies sync() properly creates tool configs via repo-tools.
Tests cursor, windsurf, and unknown tools."
```

---

### Task 5: Update GAP_TRACKING.md

**Files:**
- Modify: `docs/testing/GAP_TRACKING.md`

**Step 1: Mark GAP-022 and GAP-004 as closed**

Update the gap registry tables to show these as closed.

**Step 2: Update dashboard**

```
Gap Status (2026-01-28 Final):
================================

  Critical:  0 open  |  ░░░░░░░░░░░░░░░░░░░░ 0%
  High:      0 open  |  ░░░░░░░░░░░░░░░░░░░░ 0%
  Medium:    4 open  |  ████████████████████ 100% (optional providers)
  Low:       4 open  |  ████████████████████ 100% (nice to have)
  ------------------------
  Total:     8 open  |  13 closed

Production Readiness: 95%
  - Init/Branch: Ready
  - Sync/Fix: READY (GAP-004, GAP-022 closed)
  - Tools: 7/7 implemented
  - Presets: 4 providers (uv, venv, node, rust)
  - Git Ops: Ready (push/pull/merge)
  - MCP: Ready (tool definitions)
```

**Step 3: Commit**

```bash
git add docs/testing/GAP_TRACKING.md
git commit -m "docs: mark GAP-004 and GAP-022 as closed

Production readiness now at 95%. All critical and high priority
gaps are closed."
```

---

## Phase 3: Run Full Verification

### Task 6: Run full test suite

**Step 1: Run all workspace tests**

Run: `cargo test --workspace`
Expected: PASS

**Step 2: Run clippy**

Run: `cargo clippy --workspace`
Expected: No warnings

**Step 3: Final commit (if any fixes needed)**

```bash
git add -A
git commit -m "chore: fix any test/clippy issues from gap closure"
```

---

## Summary

| Task | Gap | Description |
|------|-----|-------------|
| 1 | GAP-022 | Add repo-tools dependency |
| 2 | GAP-022 | Refactor ToolSyncer to use repo-tools |
| 3 | GAP-022 | Update ToolSyncer tests |
| 4 | GAP-004 | Add sync integration tests |
| 5 | - | Update GAP_TRACKING.md |
| 6 | - | Full verification |

**Total tasks:** 6
**Estimated commits:** 6
