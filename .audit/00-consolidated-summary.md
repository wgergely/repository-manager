# Consolidated Code Health Audit Summary

**Date:** 2026-02-18
**Project:** repository-manager (Rust workspace, 12 crates)
**Auditors:** Investigator1 (Architecture), Investigator2 (Tests), Investigator3 (Code Quality)

---

## Top-Level Findings

### Critical / High Priority

| # | Finding | Source | Category |
|---|---------|--------|----------|
| 1 | Silent save-error discard in `RuleRegistry::remove_rule` — `.save().ok()?` swallows IO errors | Inv3 F1 | Error Handling |
| 2 | Silent IO error swallowing in MCP resource handlers — errors replaced with default content | Inv3 F4 | Error Handling |
| 3 | `repo-agent` crate has ZERO tests — process spawning/discovery completely untested | Inv2 P0 | Test Coverage |
| 4 | `test-fixtures/` golden files referenced but existence not verified | Inv2 P0 | Test Integrity |
| 5 | TOCTOU race in ledger documented as passing test (asserts data loss as expected) | Inv2 Higher Risk | Concurrency |
| 6 | `Manifest::core.mode` is raw `String` instead of typed enum — loses compile-time safety | Inv1 Arch1 | Type Safety |
| 7 | `governance::diff_configs` bypasses file locking by reading ledger directly | Inv1 Arch3 | Concurrency |
| 8 | Checksum format inconsistency: `"sha256:abc"` vs `"abc"` across crates | Inv3 F3 | Data Integrity |
| 9 | SHA-256 checksum logic duplicated in 4 places | Inv1 Dep | Code Duplication |
| 10 | Non-workspace dependency versions (chrono, fs2, dirs, tokio features) bypass governance | Inv1 Dep | Dependency Mgmt |

### Medium Priority

| # | Finding | Source |
|---|---------|--------|
| 11 | `ToolRegistry` naming collision between repo-meta and repo-tools | Inv3 F2 |
| 12 | `CheckReport` naming collision between repo-core and repo-presets | Inv3 F10 |
| 13 | `Mode` / `RepositoryMode` duplicate enums with different defaults | Inv1 Arch1 |
| 14 | `VSCodeIntegration` is sole bespoke impl; all others use GenericToolIntegration | Inv3 F5 |
| 15 | `sync_yaml` does full file replacement (destructive, no content preservation) | Inv3 F9 |
| 16 | `tracing-subscriber` in library crate (repo-tools) — should be in binaries only | Inv1 Arch4 |
| 17 | `toml_edit` imported but not used for round-trip editing in `Document::set_path` | Inv1 Arch6 |
| 18 | Error type conversion loses info in `ProjectionWriter` (JSON→io::Error via string) | Inv3 F11 |
| 19 | Hardcoded tool/preset list in repo-meta must be manually synced with repo-tools | Inv3 F12 |
| 20 | `BackupManager` filename collision risk for nested config paths | Inv3 F13 |
| 21 | `ManagedBlock` naming collision (repo-blocks::formats vs repo-content::block) | Inv1 Arch2 |

### Structural / Test Issues

| # | Finding | Source |
|---|---------|--------|
| 22 | 5 unit test files misplaced in tests/ dirs (should be #[cfg(test)] inline) | Inv2 P1 |
| 23 | Deprecated `repo-meta::RepositoryConfig` tests maintained with `#[allow(deprecated)]` | Inv2 P2 |
| 24 | Environment-dependent test assertion in repo-presets (conditional on `uv` binary) | Inv2 P2 |
| 25 | Documented behavioral quirks in mode_tests.rs lack issue tracker references | Inv2 P2 |
| 26 | Integration test crate imports repo-mcp but doesn't test it | Inv1 Arch7 |

### Low Priority

| # | Finding | Source |
|---|---------|--------|
| 27 | `CapabilityTranslator` is zero-sized struct with only static methods | Inv3 F6 |
| 28 | `name()` returns slug, requires Clippy suppression | Inv3 F7 |
| 29 | Deprecated `new()` functions lack `#[deprecated]` attribute | Inv3 F8 |
| 30 | `repo-agent` is extremely thin — may not need to be separate crate | Inv1 |
| 31 | `repo-content::path` module is fully public but should be pub(crate) | Inv1 |
| 32 | Plugin version hardcoded in CLI default_value | Inv3 F18 |
| 33 | `Format::from_content` heuristics undocumented and ambiguous | Inv3 F15 |
| 34 | `BUILTIN_COUNT` constant leaked in public API | Inv1 |
| 35 | tokio features="full" in repo-mcp (wasteful) | Inv1 |

---

## Positive Observations

1. **No `unwrap()` in production code** — all error paths use `?` or explicit handling
2. **No mocking frameworks anywhere** — all tests use real git, real filesystem, real I/O
3. **Consistent `thiserror` usage** — every crate defines typed error enums
4. **Atomic file writes** with symlink protection, fsync, and backoff retry
5. **Schema-driven tool integrations** — adding a new tool requires only a factory function
6. **Cross-platform path handling** via `NormalizedPath`
7. **Strong CLI test coverage** — parse tests for every command variant
8. **Well-typed MCP protocol** — strongly-typed request/response structs
9. **Platform-aware tests** — Unix-only tests properly gated
10. **No circular dependencies** — crate layering is sound

---

## Detailed Reports

- [Architecture & Dependencies](./investigator1-architecture.md)
- [Test Quality & Coverage](./investigator2-test-quality.md)
- [Code Quality & Best Practices](./investigator3-code-quality.md)
