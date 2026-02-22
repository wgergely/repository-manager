# P3: Config Hierarchy — Global and Organization Layers

**Priority**: P3 — Medium
**Audit ID**: M-1
**Domain**: `crates/repo-core/`
**Status**: Not started

---

## Testing Mandate

> **Inherited from [_index.md](_index.md).** Read and internalize the full
> Testing Mandate before writing any code or any test in this task.

**Domain-specific enforcement:**

- Config resolution tests MUST verify **merge behavior** — not just that a
  config file is loaded. The value of hierarchical config is that layers
  override each other predictably. Test that Layer 3 overrides Layer 1.
- Do NOT test that `resolve()` returns `Ok`. The current code already returns
  `Ok` (it just skips layers 1 and 2). Test that the **resolved values** match
  expectations when multiple layers are present.
- Test the absence case: no global config, no org config — system still works
  with defaults. This is the current behavior and must not regress.

---

## Problem Statement

The config resolver defines a 4-layer hierarchy:

```
Layer 1: Global defaults      (~/.config/repo-manager/config.toml)  ← NOT IMPLEMENTED
Layer 2: Organization config   (org-level shared config)             ← NOT IMPLEMENTED
Layer 3: Repository config     (.repository/config.toml)             ✅ Implemented
Layer 4: Local overrides       (.repository/config.local.toml)       ✅ Implemented
```

Layers 1 and 2 are placeholder comments:

```rust
// resolver.rs:104-110
tracing::debug!("Config layer 1 (global defaults) not yet implemented");
tracing::debug!("Config layer 2 (organization config) not yet implemented");
```

This means there is no way to set user-wide defaults (e.g., always use
certain presets) or share organization-level config across repositories.

---

## Implementation Plan

### Step 1: Implement Layer 1 — Global defaults

**File**: `crates/repo-core/src/config/resolver.rs`

Load config from `~/.config/repo-manager/config.toml` (XDG standard on
Linux, equivalent on macOS/Windows):

```rust
// Layer 1 - Global defaults
let global_config_dir = dirs::config_dir()
    .map(|d| d.join("repo-manager"))
    .unwrap_or_default();
let global_config_path = global_config_dir.join("config.toml");

if global_config_path.is_file() {
    let global_manifest = Manifest::from_file(&global_config_path)?;
    manifest = manifest.merge(global_manifest);
}
```

**Platform paths:**
- Linux: `~/.config/repo-manager/config.toml`
- macOS: `~/Library/Application Support/repo-manager/config.toml`
- Windows: `%APPDATA%\repo-manager\config.toml`

Use the `dirs` crate (already common in Rust ecosystem) for platform-correct
paths.

### Step 2: Implement Layer 2 — Organization config

**Design decision needed**: Where does org config live?

**Option A: Git-hosted shared config**
- A `.repo-org-config.toml` in a shared repository
- Path configured in global config: `org_config_repo = "https://..."`
- Requires git fetch on every resolve — too slow

**Option B: Local org config directory**
- `~/.config/repo-manager/org/<org-name>/config.toml`
- Org name derived from git remote URL or explicitly configured
- Fast, local-first, user controls when to update

**Option C: Symlink/include directive**
- Global config has `include = "/path/to/org/config.toml"`
- Generic, flexible, no org concept needed

**Recommendation**: Option B for initial implementation. It's local-first
(aligns with project principles) and can be synced via any mechanism the org
chooses.

### Step 3: Implement Manifest.merge()

Check if `Manifest::merge()` or equivalent already exists. If not, implement
a merge strategy:

- Tools: union (global tools + repo tools, repo wins on conflict)
- Presets: union (same)
- Mode: repo wins (you don't set standard/worktree globally)
- Hooks: union (global hooks + repo hooks)

### Step 4: Add CLI support

Add `repo config show --resolved` to display the merged config with layer
provenance (which layer each value came from).

### Step 5: Write tests

**Required tests:**

1. **test_global_config_loads** — Create a global config file. Call
   `resolve()`. Assert global values are present in resolved config.

2. **test_repo_config_overrides_global** — Set a tool in global config and a
   different value for the same tool in repo config. Assert repo value wins.

3. **test_org_config_loads** — Create an org config. Assert org values appear.

4. **test_layer_precedence** — Set the same field at all 4 layers. Assert
   Layer 4 > Layer 3 > Layer 2 > Layer 1.

5. **test_no_global_config_defaults_work** — No global config file exists.
   Assert `resolve()` succeeds and returns repo-only values (current behavior).

6. **test_no_org_config_defaults_work** — No org config exists. Assert
   `resolve()` succeeds.

7. **test_tools_union_across_layers** — Global defines tools `[vscode]`, repo
   defines tools `[cursor]`. Assert resolved config has both.

8. **test_global_config_invalid_toml_error** — Put invalid TOML in global
   config file. Assert `resolve()` returns a clear error, not a panic.

---

## Acceptance Criteria

- [ ] Global config loads from platform-appropriate path
- [ ] Organization config loads from org directory
- [ ] Layer precedence is correct: local > repo > org > global
- [ ] Missing layers are silently skipped (no error)
- [ ] Invalid layer files produce clear errors
- [ ] All 8 tests pass
- [ ] `cargo clippy` clean, `cargo test` passes (all crates)

---

## Files to Modify

| File | Change |
|------|--------|
| `crates/repo-core/src/config/resolver.rs` | Implement layers 1 and 2 |
| `crates/repo-core/src/config/manifest.rs` (likely) | Add/verify `merge()` method |
| `Cargo.toml` (repo-core) | Add `dirs` crate dependency |
| `crates/repo-cli/src/commands/config.rs` | Add `--resolved` flag showing provenance |
| `crates/repo-core/tests/` | Add 8 tests |

---

## Dependencies

- **Depends on**: Nothing (independent)
- **Can parallelize with**: P3-test-hygiene
- **Informs**: Phase 3.5 of the roadmap

---

*Task created: 2026-02-22*
*Source: [Deep Implementation Audit](../audits/2026-02-22-deep-implementation-audit.md) — M-1*
