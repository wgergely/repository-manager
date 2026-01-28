# Gap Test Updates

**Generated:** 2026-01-27
**Purpose:** Track test changes needed as gaps are closed by implementation

---

## Overview

This document tracks the specific test updates needed in `tests/integration/src/mission_tests.rs` when gap features are implemented. Each entry shows:
- The gap ID and feature
- The corresponding test that validates it
- Current test state (ignored with `#[ignore]`)
- Required changes when the feature is ready

---

## Gap-006: Antigravity Tool Integration

### Status: **IMPLEMENTED** - Test needs updating

**Implementation Location:**
- `crates/repo-tools/src/antigravity.rs` - Full implementation exists
- Exports: `antigravity_integration()`, `AntigravityIntegration` type alias
- Config path: `.agent/rules.md`

**Current Test (mission_tests.rs:679-685):**
```rust
/// GAP-006: Antigravity tool not implemented
#[test]
#[ignore = "GAP-006: Antigravity tool not implemented"]
fn gap_006_antigravity_tool() {
    // When implemented, should create .agent/rules/ directory
    panic!("Antigravity tool integration not implemented");
}
```

**Required Changes:**
1. Remove `#[ignore]` attribute
2. Replace panic with actual test logic
3. Import `antigravity_integration` from `repo_tools`

**Updated Test:**
```rust
/// M4.5: Antigravity integration name and paths
#[test]
fn m4_5_antigravity_integration_info() {
    let antigravity = antigravity_integration();
    assert_eq!(antigravity.name(), "antigravity");
    assert!(antigravity.config_paths().contains(&".agent/rules.md"));
}

/// M4.6: Antigravity sync creates rules.md
#[test]
fn m4_6_antigravity_sync_creates_rules() {
    let mut repo = TestRepo::new();
    repo.init_git();
    repo.init_repo_manager("standard", &["antigravity"], &[]);

    let root = NormalizedPath::new(repo.root());
    let rules = vec![Rule {
        id: "test-rule".to_string(),
        content: "Test content".to_string(),
    }];
    let context = SyncContext::new(root);

    antigravity_integration().sync(&context, &rules).unwrap();

    repo.assert_file_exists(".agent/rules.md");
    repo.assert_file_contains(".agent/rules.md", "test-rule");
    repo.assert_file_contains(".agent/rules.md", "Test content");
}
```

**Additional Import Required:**
```rust
use repo_tools::antigravity_integration;
```

---

## Gap-007: Windsurf Tool Integration

### Status: **IMPLEMENTED** - Test needs updating

**Implementation Location:**
- `crates/repo-tools/src/windsurf.rs` - Full implementation exists
- Exports: `windsurf_integration()`, `WindsurfIntegration` type alias
- Config path: `.windsurfrules`

**Current Test (mission_tests.rs:687-691):**
```rust
/// GAP-007: Windsurf tool not implemented
#[test]
#[ignore = "GAP-007: Windsurf tool not implemented"]
fn gap_007_windsurf_tool() {
    panic!("Windsurf tool integration not implemented");
}
```

**Required Changes:**
1. Remove `#[ignore]` attribute
2. Replace panic with actual test logic
3. Import `windsurf_integration` from `repo_tools`

**Updated Test:**
```rust
/// M4.7: Windsurf integration name and paths
#[test]
fn m4_7_windsurf_integration_info() {
    let windsurf = windsurf_integration();
    assert_eq!(windsurf.name(), "windsurf");
    assert!(windsurf.config_paths().contains(&".windsurfrules"));
}

/// M4.8: Windsurf sync creates windsurfrules
#[test]
fn m4_8_windsurf_sync_creates_windsurfrules() {
    let mut repo = TestRepo::new();
    repo.init_git();
    repo.init_repo_manager("standard", &["windsurf"], &[]);

    let root = NormalizedPath::new(repo.root());
    let rules = vec![Rule {
        id: "test-rule".to_string(),
        content: "Test content".to_string(),
    }];
    let context = SyncContext::new(root);

    windsurf_integration().sync(&context, &rules).unwrap();

    repo.assert_file_exists(".windsurfrules");
    repo.assert_file_contains(".windsurfrules", "test-rule");
    repo.assert_file_contains(".windsurfrules", "Test content");
}
```

**Additional Import Required:**
```rust
use repo_tools::windsurf_integration;
```

---

## Gap-010: Python venv Provider

### Status: **IMPLEMENTED BUT NOT EXPORTED** - Needs lib.rs update + test update

**Implementation Location:**
- `crates/repo-presets/src/python/venv.rs` - Full implementation exists
- Exported from `python/mod.rs` as `VenvProvider`
- **NOT exported** from `crates/repo-presets/src/lib.rs` (only `UvProvider` is exported)

**Current Test (mission_tests.rs:702-707):**
```rust
/// GAP-010: Python venv provider not implemented
#[test]
#[ignore = "GAP-010: Python venv provider not implemented"]
fn gap_010_python_venv_provider() {
    // Should support provider = "venv" in addition to "uv"
    panic!("Python venv provider not implemented");
}
```

**Required Changes:**

### Step 1: Update `crates/repo-presets/src/lib.rs`
Add `VenvProvider` to exports:
```rust
pub use python::UvProvider;
pub use python::VenvProvider;  // Add this line
```

### Step 2: Update Mission Test
1. Remove `#[ignore]` attribute
2. Replace panic with actual test logic

**Updated Test:**
```rust
/// M5.5: Venv provider ID
#[test]
fn m5_5_venv_provider_id() {
    let provider = VenvProvider::new();
    assert_eq!(provider.id(), "env:python");
}

/// M5.6: Venv provider check returns non-healthy when venv missing
#[tokio::test]
async fn m5_6_venv_provider_check_missing() {
    let repo = TestRepo::new();
    let layout = WorkspaceLayout {
        root: NormalizedPath::new(repo.root()),
        active_context: NormalizedPath::new(repo.root()),
        mode: LayoutMode::Classic,
    };

    let context = Context::new(layout, HashMap::new());
    let provider = VenvProvider::new();

    let report = provider.check(&context).await.unwrap();

    // Should not be healthy since no venv exists
    assert_ne!(
        report.status,
        PresetStatus::Healthy,
        "Expected non-healthy status when venv is missing"
    );
}
```

**Additional Import Required:**
```rust
use repo_presets::VenvProvider;
```

---

## Summary Table

| Gap ID | Feature | Implementation Status | Test File Location | Action Required |
|--------|---------|----------------------|-------------------|-----------------|
| GAP-006 | Antigravity tool | **COMPLETE** | mission_tests.rs:679-685 | Remove `#[ignore]`, add real test |
| GAP-007 | Windsurf tool | **COMPLETE** | mission_tests.rs:687-691 | Remove `#[ignore]`, add real test |
| GAP-010 | Python venv provider | **COMPLETE** (not exported) | mission_tests.rs:702-707 | Export from lib.rs, remove `#[ignore]`, add real test |

---

## Implementation Notes

### Discoveries During Analysis

1. **GAP-006 and GAP-007 are already implemented** - The tool integrations for Antigravity and Windsurf exist in `crates/repo-tools/src/` with full functionality including unit tests.

2. **GAP-010 is implemented but not exported** - The `VenvProvider` exists at `crates/repo-presets/src/python/venv.rs` with a complete implementation, but the crate's `lib.rs` only exports `UvProvider`. This is likely an oversight.

3. **Unit tests already exist** - Each implementation file contains its own unit tests that verify the core functionality. The mission tests need to be updated to reflect that these features now work.

### Test Summary Update

When these gaps are closed, update the `test_summary()` function in mission_tests.rs:

```rust
// Before:
println!("Mission 4 (Tools):    3/7 tools implemented");
println!("Mission 5 (Presets):  1 provider (uv) implemented");

// After:
println!("Mission 4 (Tools):    5/7 tools implemented");  // +antigravity, +windsurf
println!("Mission 5 (Presets):  2 providers (uv, venv) implemented");  // +venv
```

### GAP_TRACKING.md Updates

When closing these gaps, move entries from "Medium Gaps" to "Closed Gaps" section:

```markdown
### Closed Gaps

| ID | Feature | Closed Date | PR/Commit |
|----|---------|-------------|-----------|
| GAP-006 | Antigravity tool | 2026-01-27 | PR #XXX |
| GAP-007 | Windsurf tool | 2026-01-27 | PR #XXX |
| GAP-010 | Python venv provider | 2026-01-27 | PR #XXX |
```

---

## Appendix: File Locations

| File | Purpose |
|------|---------|
| `tests/integration/src/mission_tests.rs` | Integration tests with gap documentation |
| `docs/testing/GAP_TRACKING.md` | Gap registry and status dashboard |
| `crates/repo-tools/src/antigravity.rs` | Antigravity integration implementation |
| `crates/repo-tools/src/windsurf.rs` | Windsurf integration implementation |
| `crates/repo-presets/src/python/venv.rs` | Venv provider implementation |
| `crates/repo-presets/src/lib.rs` | Preset crate exports (needs VenvProvider) |
