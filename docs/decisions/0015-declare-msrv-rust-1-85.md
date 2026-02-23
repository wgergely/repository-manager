# ADR-0015: Declare MSRV at Rust 1.85

**Status:** Accepted
**Date:** 2026-02-18
**Decision Makers:** Project Team
**Informed by:** [audits/2026-02-18-packaging-distribution-audit.md](../audits/2026-02-18-packaging-distribution-audit.md), [audits/2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md)

## Context and Problem Statement

No `rust-version` field is present in any Cargo.toml in the workspace. The workspace uses `edition = "2024"` which requires Rust 1.85+. Without an explicit MSRV declaration, users have no indication of the minimum Rust version required to build the project, and CI does not verify compatibility with that floor.

Declaring MSRV is a crates.io best practice and helps users and downstream consumers know minimum requirements before attempting to build.

## Decision Drivers

- The 2024 edition already requires Rust 1.85+ as a hard floor — declaring it costs nothing
- crates.io and the Rust ecosystem treat `rust-version` as a first-class field for discoverability and compatibility signaling
- CI should validate the declared MSRV to prevent accidental breakage
- Prerequisite for professional crates.io publishing (ADR-0007)

## Considered Options

1. Declare `rust-version = "1.85"` at workspace level in `[workspace.package]`
2. Declare MSRV per-crate with varying versions
3. Do not declare MSRV and leave it undocumented

## Decision Outcome

**Chosen option:** "Declare `rust-version = \"1.85\"` at workspace level", because the 2024 edition already enforces this floor, all crates inherit it automatically, and no code changes are required. This is the minimal-effort, highest-value action.

### Consequences

**Good:**
- Users immediately know the minimum Rust version required
- crates.io displays the MSRV prominently on crate pages
- CI can verify the declared floor remains accurate
- All 11 crates inherit the field without per-crate changes

**Bad:**
- CI matrix grows by one job (Rust 1.85 specific run)
- Future language features above 1.85 would require bumping the declared MSRV

## More Information

- **Related ADRs:** [ADR-0006](0006-rust-ci-workflow.md), [ADR-0007](0007-publish-all-crates.md)
- **Audit Reports:** [audits/2026-02-18-packaging-distribution-audit.md](../audits/2026-02-18-packaging-distribution-audit.md) (P1-13), [audits/2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md) (Phase 2, item 18)
- **Implementation:** Add `rust-version = "1.85"` to `[workspace.package]` in the root `Cargo.toml`. Add a CI job that tests with Rust 1.85 specifically (alongside stable). Document MSRV in README.
