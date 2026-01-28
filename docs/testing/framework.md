# Conceptual Testing Framework for Repository Manager

**Date:** 2026-01-27
**Purpose:** Production-grade testing framework to validate specs against implementation
**Strategy:** Test what specs claim, reveal implementation gaps

---

## Executive Summary

This document defines a comprehensive testing framework organized around **mission success criteria**. Rather than testing individual functions, we test *production scenarios* - what users actually need to accomplish.

### Testing Philosophy

```
Specs define WHAT should work
Tests verify IF it actually works
Gaps reveal WHERE implementation is incomplete
```

### Coverage Matrix

| Mission Category | Spec Claims | Implemented | Tested | Gap Score |
|------------------|-------------|-------------|--------|-----------|
| Repository Init | 100% | ~80% | 60% | HIGH |
| Branch Management | 100% | 90% | 70% | MEDIUM |
| Configuration Sync | 100% | 30% | 20% | CRITICAL |
| Tool Integration | 100% | 40% | 40% | HIGH |
| Preset Providers | 100% | 15% | 10% | CRITICAL |
| Git Operations | 100% | 0% | 0% | CRITICAL |
| MCP Server | 100% | 0% | 0% | CRITICAL |

---

## Part 1: Mission Categories

### Mission 1: Repository Initialization

**Spec Source:** `docs/design/spec-cli.md`, `docs/design/architecture-core.md`

**Success Criteria:**
1. Empty folder -> fully configured repo
2. Existing git repo -> add repository manager config
3. Mode selection (standard vs worktrees) affects physical layout
4. Tools specified -> tool configs created
5. Presets specified -> presets applied

**Test Scenarios:**

```
M1.1 Init Empty Folder (Standard Mode)
  Given: Empty folder
  When: `repo init --mode standard`
  Then:
    - .git/ directory exists
    - .repository/config.toml exists
    - config.toml has mode = "standard"

M1.2 Init Empty Folder (Worktrees Mode)
  Given: Empty folder
  When: `repo init --mode worktrees`
  Then:
    - .git/ directory (bare or hybrid)
    - main/ worktree directory exists
    - .repository/config.toml at container level
    - config.toml has mode = "worktrees"

M1.3 Init with Tools
  Given: Empty folder
  When: `repo init --tools vscode cursor claude`
  Then:
    - .vscode/settings.json created
    - .cursorrules created
    - CLAUDE.md created
    - config.toml lists tools

M1.4 Init with Presets
  Given: Empty folder
  When: `repo init --presets env:python`
  Then:
    - Python environment created (venv or uv)
    - VSCode python settings configured (if vscode tool active)
    - config.toml lists presets

M1.5 Init Idempotency
  Given: Already initialized repo
  When: `repo init` again
  Then:
    - No destructive changes
    - Config preserved or updated cleanly
    - Exit code indicates already initialized
```

### Mission 2: Branch & Worktree Management

**Spec Source:** `docs/design/spec-git.md`, `docs/design/spec-cli.md`

**Success Criteria:**
1. Create branch -> works in both modes
2. Standard mode uses git checkout
3. Worktree mode creates physical directory + git worktree
4. Remove cleans up completely
5. List shows accurate state

**Test Scenarios:**

```
M2.1 Branch Add (Standard Mode)
  Given: Standard mode repo on main
  When: `repo branch add feature-x`
  Then:
    - New branch 'feature-x' exists
    - Switched to feature-x
    - No new directories created

M2.2 Branch Add (Worktrees Mode)
  Given: Worktrees mode container
  When: `repo branch add feature-x`
  Then:
    - New directory {container}/feature-x/ exists
    - Git worktree registered for feature-x
    - Branch feature-x exists
    - Worktree has .git file pointing to main repo

M2.3 Branch Add with Base
  Given: Repo with branch 'develop'
  When: `repo branch add feature-y --base develop`
  Then:
    - feature-y created from develop HEAD
    - Not from main

M2.4 Branch Remove (Worktrees Mode)
  Given: Worktree feature-x exists
  When: `repo branch remove feature-x`
  Then:
    - Directory {container}/feature-x/ deleted
    - Worktree unregistered
    - Branch feature-x deleted
    - No orphan .git references

M2.5 Branch List
  Given: Multiple branches/worktrees
  When: `repo branch list`
  Then:
    - All branches listed
    - Current branch indicated
    - Worktree paths shown (if worktrees mode)

M2.6 Branch Name Sanitization
  Given: Worktrees mode
  When: `repo branch add feat/user-auth`
  Then:
    - Directory name is 'feat-user-auth' (safe slug)
    - Git branch is 'feat/user-auth' (original)
```

### Mission 3: Configuration Synchronization

**Spec Source:** `docs/design/config-ledger.md`, `docs/design/spec-tools.md`

**Success Criteria:**
1. Sync applies ledger intents to filesystem
2. Check detects drift between ledger and files
3. Fix repairs drift automatically
4. Managed blocks preserved, user content preserved

**Test Scenarios:**

```
M3.1 Sync Creates Tool Configs
  Given: Config with tools = ["vscode", "cursor"]
  When: `repo sync`
  Then:
    - .vscode/settings.json exists with managed keys
    - .cursorrules exists with managed blocks
    - Ledger updated with projections

M3.2 Check Detects Missing File
  Given: Ledger expects .vscode/settings.json
  And: File deleted by user
  When: `repo check`
  Then:
    - Status: Missing
    - Report lists missing file

M3.3 Check Detects Drift
  Given: Ledger expects editor.fontSize = 14
  And: User changed to fontSize = 12
  When: `repo check`
  Then:
    - Status: Drifted
    - Report shows expected vs actual

M3.4 Check Healthy
  Given: All files match ledger
  When: `repo check`
  Then:
    - Status: Healthy

M3.5 Fix Repairs Drift
  Given: Drifted configuration
  When: `repo fix`
  Then:
    - Files restored to ledger state
    - User non-managed content preserved

M3.6 Managed Block Preservation
  Given: .cursorrules with user content + managed blocks
  When: `repo sync` with updated rules
  Then:
    - Managed blocks updated
    - User content between blocks preserved

M3.7 JSON Key Preservation
  Given: .vscode/settings.json with user keys + managed keys
  When: `repo sync` with updated settings
  Then:
    - Managed keys updated
    - User keys preserved
    - JSON formatting preserved
```

### Mission 4: Tool Management

**Spec Source:** `docs/design/spec-tools.md`

**Success Criteria:**
1. Add tool -> config updated + files synced
2. Remove tool -> config updated + files cleaned
3. All specified tools have working integrations

**Test Scenarios:**

```
M4.1 Add Tool Triggers Sync
  Given: Repo without vscode tool
  When: `repo add-tool vscode`
  Then:
    - config.toml updated with vscode
    - .vscode/settings.json created
    - Ledger updated

M4.2 Remove Tool Cleans Up
  Given: Repo with cursor tool active
  When: `repo remove-tool cursor`
  Then:
    - config.toml updated (cursor removed)
    - .cursorrules managed blocks removed (file may remain)
    - Ledger updated

M4.3 VSCode Integration
  Given: vscode tool active, python preset
  When: `repo sync`
  Then:
    - settings.json has python.defaultInterpreterPath
    - settings.json has managed key marker

M4.4 Cursor Integration
  Given: cursor tool active, rules defined
  When: `repo sync`
  Then:
    - .cursorrules has managed blocks
    - Each rule creates a block

M4.5 Claude Integration
  Given: claude tool active, rules defined
  When: `repo sync`
  Then:
    - CLAUDE.md has managed blocks
    - .claude/rules/ directory may exist

M4.6 Antigravity Integration [EXPECTED FAILURE - NOT IMPLEMENTED]
  Given: antigravity tool active
  When: `repo sync`
  Then:
    - .agent/rules/ directory created
    - Rule files generated

M4.7 Windsurf Integration [EXPECTED FAILURE - NOT IMPLEMENTED]
  Given: windsurf tool active
  When: `repo sync`
  Then:
    - Appropriate config created
```

### Mission 5: Preset Management

**Spec Source:** `docs/design/spec-presets.md`

**Success Criteria:**
1. Add preset -> preset applied
2. Remove preset -> preset unapplied
3. Preset check reports status
4. Multiple preset types supported

**Test Scenarios:**

```
M5.1 Python UV Provider
  Given: Clean directory
  When: `repo add-preset env:python` (with uv)
  Then:
    - .venv/ directory created (or uv-managed location)
    - Python binary available

M5.2 Python Venv Provider [EXPECTED FAILURE - NOT IMPLEMENTED]
  Given: Clean directory with system python
  When: `repo add-preset env:python --provider venv`
  Then:
    - .venv/ created via python -m venv

M5.3 Node Provider [EXPECTED FAILURE - NOT IMPLEMENTED]
  Given: Clean directory
  When: `repo add-preset env:node`
  Then:
    - node_modules management configured
    - package.json interaction

M5.4 Rust Provider [EXPECTED FAILURE - NOT IMPLEMENTED]
  Given: Clean directory
  When: `repo add-preset env:rust`
  Then:
    - Cargo.toml recognized
    - rust-analyzer settings configured

M5.5 Preset Check Reports Status
  Given: env:python configured but venv missing
  When: `repo check`
  Then:
    - Preset status: Missing or Broken
    - Remediation suggested

M5.6 Preset Dependencies
  Given: tool:ruff requires env:python
  When: `repo add-preset tool:ruff` (without python)
  Then:
    - Either auto-add env:python
    - Or error with dependency message
```

### Mission 6: Git Operations (Wrappers)

**Spec Source:** `docs/design/spec-cli.md`, `docs/design/spec-git.md`

**Test Scenarios:**

```
M6.1 Push [EXPECTED FAILURE - NOT IMPLEMENTED]
  Given: Commits on feature branch, remote configured
  When: `repo push`
  Then:
    - Branch pushed to origin
    - Upstream tracking set

M6.2 Pull [EXPECTED FAILURE - NOT IMPLEMENTED]
  Given: Remote has new commits
  When: `repo pull`
  Then:
    - Local updated from remote
    - Works from any worktree

M6.3 Merge [EXPECTED FAILURE - NOT IMPLEMENTED]
  Given: Feature branch with commits
  When: `repo merge main` (from feature)
  Then:
    - main merged into feature
    - Conflicts reported if present

M6.4 Push from Worktree [EXPECTED FAILURE - NOT IMPLEMENTED]
  Given: In worktree mode, on feature-x worktree
  When: `repo push`
  Then:
    - Pushes feature-x branch
    - Correct upstream tracking
```

### Mission 7: Rule Management

**Spec Source:** `docs/design/spec-cli.md`

**Test Scenarios:**

```
M7.1 Add Rule
  Given: Initialized repo
  When: `repo add-rule python-style -i "Use snake_case"`
  Then:
    - Rule stored in .repository/rules/python-style.toml
    - Rule available for sync

M7.2 Remove Rule
  Given: Rule python-style exists
  When: `repo remove-rule python-style`
  Then:
    - Rule file removed
    - Managed blocks for rule removed on next sync

M7.3 List Rules
  Given: Multiple rules defined
  When: `repo list-rules`
  Then:
    - All rules listed with IDs
    - Tags shown if present

M7.4 Rule Tags
  Given: Rule with tags
  When: Querying/filtering by tag
  Then:
    - Can filter rules by tag
```

---

## Part 2: Gap Detection Tests

These tests are specifically designed to **fail** when features are missing, documenting the gap.

### Gap Test Template

```rust
#[test]
#[should_panic(expected = "not implemented")]  // Or expect error
fn gap_test_feature_x() {
    // Attempt to use feature X as documented in spec
    // This test documents the gap and will pass when feature is implemented
}
```

### Identified Gaps from Spec Analysis

| Gap ID | Spec Claim | Implementation Status |
|--------|------------|----------------------|
| GAP-001 | `repo push` command | Not implemented |
| GAP-002 | `repo pull` command | Not implemented |
| GAP-003 | `repo merge` command | Not implemented |
| GAP-004 | `sync()` applies projections | Only creates ledger |
| GAP-005 | `fix()` repairs drift | Calls sync (stub) |
| GAP-006 | Antigravity tool integration | Not implemented |
| GAP-007 | Windsurf tool integration | Not implemented |
| GAP-008 | Gemini CLI tool integration | Not implemented |
| GAP-009 | JetBrains tool integration | Not implemented |
| GAP-010 | Python venv provider | Not implemented |
| GAP-011 | Python conda provider | Not implemented |
| GAP-012 | Node env provider | Not implemented |
| GAP-013 | Rust env provider | Not implemented |
| GAP-014 | EditorConfig provider | Not implemented |
| GAP-015 | GitIgnore provider | Not implemented |
| GAP-016 | tool:ruff provider | Not implemented |
| GAP-017 | tool:pytest provider | Not implemented |
| GAP-018 | MCP Server crate | Not started |
| GAP-019 | add-tool triggers sync | Only updates config |
| GAP-020 | remove-tool cleans up | Only updates config |

---

## Part 3: Test Infrastructure

### Test Fixtures

```rust
/// Standard test repository setup
pub struct TestRepo {
    temp_dir: TempDir,
    mode: LayoutMode,
}

impl TestRepo {
    /// Create uninitialized test directory
    pub fn new() -> Self;

    /// Create initialized standard mode repo
    pub fn standard() -> Self;

    /// Create initialized worktrees mode container
    pub fn worktrees() -> Self;

    /// Create with specific config
    pub fn with_config(config: &str) -> Self;

    /// Get root path
    pub fn root(&self) -> &Path;

    /// Run CLI command
    pub fn run(&self, args: &[&str]) -> CommandResult;

    /// Assert file exists with content
    pub fn assert_file(&self, path: &str, contains: &str);

    /// Assert file does not exist
    pub fn assert_no_file(&self, path: &str);
}
```

### Test Categories

```rust
mod mission_tests {
    mod init;      // M1.x tests
    mod branch;    // M2.x tests
    mod sync;      // M3.x tests
    mod tools;     // M4.x tests
    mod presets;   // M5.x tests
    mod git_ops;   // M6.x tests (mostly expected failures)
    mod rules;     // M7.x tests
}

mod gap_tests {
    mod documented_gaps;  // Tests that document missing features
}

mod regression_tests {
    mod fixed_bugs;  // Tests for previously fixed issues
}

mod robustness_tests {
    mod error_handling;
    mod concurrent_access;
    mod unicode_paths;
}
```

---

## Part 4: Implementation Priority

### Phase 1: Critical Path Tests (Must Have)

1. **M1.1-M1.3**: Init in both modes with tools
2. **M2.1-M2.4**: Branch add/remove in both modes
3. **M3.1-M3.4**: Basic sync/check cycle
4. **M4.3-M4.5**: VSCode/Cursor/Claude integration

### Phase 2: Feature Completion Tests

1. **M3.5-M3.7**: Fix and content preservation
2. **M4.1-M4.2**: Add/remove tool with sync
3. **M5.1**: Python UV provider
4. **M7.1-M7.3**: Rule management

### Phase 3: Gap Documentation Tests

1. All GAP-xxx tests (expected to fail, documenting gaps)
2. As features are implemented, tests should pass

### Phase 4: Production Hardening

1. Robustness tests
2. Concurrent access tests
3. Error recovery tests
4. Performance benchmarks

---

## Part 5: Success Metrics

### Test Health Dashboard

```
Mission Success Rate:
  M1 (Init):     ████████░░ 80%
  M2 (Branch):   ███████░░░ 70%
  M3 (Sync):     ██░░░░░░░░ 20%
  M4 (Tools):    ████░░░░░░ 40%
  M5 (Presets):  █░░░░░░░░░ 10%
  M6 (Git):      ░░░░░░░░░░ 0%
  M7 (Rules):    ███░░░░░░░ 30%

Gap Closure:
  Open Gaps:     20
  Closed Gaps:   0
  In Progress:   0
```

### Graduation Criteria

A mission is considered "production ready" when:
- All tests in that category pass
- No expected failures remain
- Edge cases covered
- Error handling verified

---

## Appendix A: Existing Test Inventory

### Unit Tests (per crate)

| Crate | Test Files | Coverage Focus |
|-------|-----------|----------------|
| repo-fs | 12 | Path normalization, atomic I/O, security |
| repo-git | 5 | Layout providers, worktree ops |
| repo-core | 5 | Ledger, sync engine, modes |
| repo-content | 12 | Format handlers, block operations |
| repo-blocks | 2 | Block parsing/writing |
| repo-meta | - | Config loading, validation |
| repo-tools | - | Tool integrations |
| repo-presets | - | Provider implementations |
| repo-cli | 1 | CLI parsing |

### Integration Tests

| Test File | Coverage |
|-----------|----------|
| `tests/integration/src/integration_test.rs` | Vertical slice: config->preset->tool |

### Missing Test Categories

1. **CLI End-to-End Tests**: Full command execution
2. **Cross-Mode Tests**: Same operation in both modes
3. **Multi-Worktree Tests**: Operations across worktrees
4. **Error Recovery Tests**: Graceful failure handling
5. **Upgrade/Migration Tests**: Config version changes

---

## Appendix B: Test Command Reference

```bash
# Run all tests
cargo test -- --test-threads=1

# Run specific mission tests
cargo test mission_init
cargo test mission_branch
cargo test mission_sync

# Run gap tests (expect failures)
cargo test gap_tests

# Run with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration
```
