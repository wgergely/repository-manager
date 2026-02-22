# P2: Preset Providers — Node and Rust Apply Stubs

**Priority**: P2 — High
**Audit IDs**: H-3, H-4
**Domain**: `crates/repo-presets/`
**Status**: Not started

---

## Testing Mandate

> **Inherited from [_index.md](_index.md).** Read and internalize the full
> Testing Mandate before writing any code or any test in this task.

**Domain-specific enforcement:**

- Do NOT test that `apply()` returns `ApplyReport::success()` — that is
  exactly the current (broken) behavior. The stub already returns success.
- If a provider is detection-only, the test must verify that calling `apply()`
  returns a **non-success** status or a clearly-marked detection-only report
  that callers can distinguish from a real apply.
- If a provider performs real setup, the test must verify the **side effect**
  (virtual env created, dependency installed, tool available on PATH).

---

## Problem Statement

### H-3: NodeProvider.apply() — fake success

```rust
// node_provider.rs:124-129
async fn apply(&self, _context: &Context) -> Result<ApplyReport> {
    Ok(ApplyReport::success(vec![
        "Node environment detection complete. This provider is detection-only.".to_string(),
    ]))
}
```

`check()` is real — it detects `package.json`, `node_modules`, `node` on
PATH. But `apply()` does nothing and returns `ApplyReport::success`. A caller
has no way to know that nothing was actually applied without parsing the
message string.

### H-4: RustProvider.apply() — fake success

```rust
// rust_provider.rs:84-90
async fn apply(&self, _context: &Context) -> Result<ApplyReport> {
    Ok(ApplyReport::success(vec![
        "Rust environment provider is detection-only.".to_string(),
        "No actions taken. Use rustup to manage Rust installations.".to_string(),
    ]))
}
```

Same pattern. Detection works; apply is a no-op returning success.

---

## Decision Required

### Option A: Make detection-only status explicit in the type system (recommended)

Add a new `ApplyReport` variant or status:

```rust
pub enum ApplyStatus {
    Success,
    DetectionOnly,  // New — provider checked but does not apply
    Failed,
}
```

Change both providers:

```rust
async fn apply(&self, _context: &Context) -> Result<ApplyReport> {
    Ok(ApplyReport::detection_only(vec![
        "Node environment detected. This provider is detection-only.".to_string(),
    ]))
}
```

**Pros**: Type-safe, callers can pattern-match, no string parsing needed.
**Cons**: Requires modifying `ApplyReport` (may affect other code).

### Option B: Return an error for detection-only providers

```rust
async fn apply(&self, _context: &Context) -> Result<ApplyReport> {
    Err(anyhow::anyhow!(
        "NodeProvider is detection-only and does not support apply(). Use check() instead."
    ))
}
```

**Pros**: Impossible to confuse with success. **Cons**: Callers that iterate
all providers and call `apply()` will need to handle this error gracefully.

### Option C: Implement real apply logic

- **Node**: Run `npm install` or `yarn install` based on lockfile presence
- **Rust**: Run `cargo build` or `rustup component add` based on context

**Pros**: Feature works. **Cons**: Running `npm install` or `cargo build` as a
side effect of `repo sync` is a significant behavior change that needs careful
UX consideration (opt-in? confirmation?).

**Recommendation**: Option A. It's type-safe, honest, and doesn't change
external behavior. Option C is a separate, larger feature request.

---

## Implementation Plan (Option A)

### Step 1: Extend ApplyReport with detection-only status

**File**: `crates/repo-presets/src/` (wherever `ApplyReport` is defined)

Add `DetectionOnly` variant to the status enum. Add a constructor:

```rust
impl ApplyReport {
    pub fn detection_only(messages: Vec<String>) -> Self {
        Self {
            status: ApplyStatus::DetectionOnly,
            messages,
        }
    }

    pub fn is_detection_only(&self) -> bool {
        matches!(self.status, ApplyStatus::DetectionOnly)
    }
}
```

### Step 2: Update NodeProvider.apply()

**File**: `crates/repo-presets/src/node/node_provider.rs:124-129`

```rust
async fn apply(&self, _context: &Context) -> Result<ApplyReport> {
    Ok(ApplyReport::detection_only(vec![
        "Node environment detected. This provider does not perform setup.".to_string(),
        "Install dependencies manually with npm/yarn/pnpm.".to_string(),
    ]))
}
```

### Step 3: Update RustProvider.apply()

**File**: `crates/repo-presets/src/rust/rust_provider.rs:84-90`

```rust
async fn apply(&self, _context: &Context) -> Result<ApplyReport> {
    Ok(ApplyReport::detection_only(vec![
        "Rust environment detected. This provider does not perform setup.".to_string(),
        "Manage Rust installations with rustup.".to_string(),
    ]))
}
```

### Step 4: Update callers that check ApplyReport.status

Search for all code that checks `apply()` results. Ensure they handle
`DetectionOnly` appropriately (don't treat it as full success in progress
reporting, don't treat it as failure).

### Step 5: Write tests

**Required tests:**

1. **test_node_provider_apply_returns_detection_only** — Call
   `NodeProvider.apply()`. Assert `is_detection_only() == true`. Assert
   `status != ApplyStatus::Success`.

2. **test_rust_provider_apply_returns_detection_only** — Same for Rust.

3. **test_node_provider_check_is_real** — Set up a directory with
   `package.json`. Call `check()`. Assert it detects the Node environment.

4. **test_rust_provider_check_is_real** — Set up a directory with
   `Cargo.toml`. Call `check()`. Assert it detects the Rust environment.

5. **test_uv_provider_apply_is_real** — Call `UvProvider.apply()` (the one
   provider that actually does something). Assert `status ==
   ApplyStatus::Success` and observable side effect.

6. **test_detection_only_vs_success_distinguishable** — Create one provider
   that returns `success()` and one that returns `detection_only()`. Assert
   they are distinguishable via `is_detection_only()`.

7. **test_apply_report_serialization** — If `ApplyReport` is serialized
   (e.g., for MCP responses), verify that `DetectionOnly` serializes
   distinctly from `Success`.

---

## Acceptance Criteria

- [ ] `NodeProvider.apply()` returns `DetectionOnly`, not `Success`
- [ ] `RustProvider.apply()` returns `DetectionOnly`, not `Success`
- [ ] Callers can distinguish detection-only from real success via type system
- [ ] No string parsing needed to determine if apply did real work
- [ ] All 7 tests pass
- [ ] Changing `detection_only()` back to `success()` causes tests 1-2 to fail
- [ ] `cargo clippy` clean, `cargo test` passes (all crates)

---

## Files to Modify

| File | Change |
|------|--------|
| `crates/repo-presets/src/` (ApplyReport definition) | Add `DetectionOnly` status |
| `crates/repo-presets/src/node/node_provider.rs` | Return `detection_only()` |
| `crates/repo-presets/src/rust/rust_provider.rs` | Return `detection_only()` |
| Callers of `apply()` across codebase | Handle `DetectionOnly` variant |
| `crates/repo-presets/tests/` | Add 7 tests |

---

## Dependencies

- **Depends on**: Nothing (independent)
- **Can parallelize with**: P2-mcp-server
- **Related to**: UvProvider (for comparison — UV apply is real)

---

*Task created: 2026-02-22*
*Source: [Deep Implementation Audit](../audits/2026-02-22-deep-implementation-audit.md) — H-3, H-4*
