# Research: Modern Rust Testing & Dependencies

**Date**: 2026-01-23

## 1. Dependency Version Check

The project is using **cutting-edge** versions of key ecosystems.

| Crate | Current Project Version | Status | Notes |
| :--- | :--- | :--- | :--- |
| `thiserror` | **2.0** | ✅ Frontier | Major version 2.0 is very recent. |
| `tempfile` | **3.14** | ✅ Frontier | 3.14 is the latest. |
| `rstest` | **0.23** | ✅ Frontier | Current standard for fixtures/cases. |
| `toml` | **0.8** | ✅ Frontier | Uses the new parser `toml_edit` internally. |
| `serde` | **1.0** | ✅ Stable | Industry standard. |

**Conclusion**: The repository is already adhering to a "bleeding edge" update policy. No immediate updates are required for the existing stack.

## 2. Frontier Testing Practices

For a "Repository Manager" filesystem crate (`repo-fs`), the following modern practices are recommended to elevate quality beyond standard unit tests:

### A. Snapshot Testing (`insta`)

**Why**: Testing filesystem structure or complex config objects is brittle with assert_eq! strings.
**Recommendation**: Use `insta` to serialize the `WorkspaceLayout` debug view or directory tree structures and compare against stored snapshots.

```rust
insta::assert_yaml_snapshot!(layout);
```

### B. Property-Based Testing (`proptest`)

**Why**: Edge cases in path normalization (`NormalizedPath`) are hard to manually enumerate.
**Recommendation**: Use `proptest` to generate random strings (including Unicode, control characters, overly long paths) to fuzz the `NormalizedPath::new` constructor and ensure it never panics or produces invalid paths.

### C. Mutation Testing (`cargo-mutants`)

**Why**: To verify test coverage quality, not just quantity.
**Recommendation**: Run `cargo mutants` occasionally. It modifies source code (e.g., changes `==` to `!=`) and checks if tests fail. If tests pass with mutated code, the tests are weak.

### D. Mocking (`mockall`)

**Why**: Testing IO failure scenarios (e.g., "disk full", "permission denied" on specific calls) is hard with real FS.
**Recommendation**: Abstract filesystem operations behind a trait (e.g., `FileSystem`) and use `mockall` to simulate rare errors.

## 3. Recommended Next Steps

1. **Adopt `insta`**: Add to `dev-dependencies` for layout and tree verification.
2. **Adopt `proptest`**: Add to `dev-dependencies` for `NormalizedPath` hardening.
3. **Add `walkdir` & `ignore`**: As identified in the Audit, these are critical functional gaps for a repository manager, even if not strictly "testing" libraries.
