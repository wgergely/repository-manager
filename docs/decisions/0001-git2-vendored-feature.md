# ADR-0001: Use git2 with vendored feature for Git operations

**Status:** Accepted
**Date:** 2026-02-18
**Decision Makers:** Project Team
**Informed by:** 2026-02-18-rust-distribution-practices.md, 2026-02-18-setup-ease-audit.md, 2026-02-18-packaging-distribution-audit.md, 2026-02-18-feature-gap-analysis.md

## Context and Problem Statement

The project uses git2 = " with default features, which requires cmake, a C compiler, and libssl-dev at build time. These are undocumented prerequisites that silently break builds for users who lack them.

Additionally, the design documentation has an unresolved contradiction: key-decisions.md states that gix (gitoxide) is the preferred Git library, while architecture-core.md and the actual codebase use git2. This inconsistency creates confusion about the project's intended direction.

## Decision Drivers

- Distribution blocker: unlisted C build-time dependencies break installs silently
- Developer friction: \ failures for contributors without cmake/libssl-dev
- Documentation integrity: the gix vs git2 contradiction must be resolved
- Minimal disruption: a low-risk fix is preferred over a multi-week refactor

## Considered Options

1. git2 with vendored feature - 1-line Cargo.toml change, statically bundles libgit2, eliminates runtime C deps
2. Switch to gix (gitoxide) - pure Rust, no C deps, but major refactor of repo-git crate taking weeks
3. Keep current git2 defaults - zero effort but keeps undocumented prerequisites

## Decision Outcome

**Chosen option:** \, because it resolves the distribution blocker immediately with a single-line Cargo.toml change. The vendored feature statically links libgit2 into the binary, eliminating all runtime C dependencies without touching any application logic. The gix migration is deferred to a future ADR once its API stabilizes.

### Consequences

**Good:**
- Eliminates undocumented build-time C dependency requirements
- Single-line change with zero application code impact
- Consistent, reproducible builds across all platforms
- Unblocks binary distribution via cargo-dist

**Bad:**
- Slightly larger binary size due to statically linked libgit2
- Longer compile times (vendored build compiles libgit2 from source)
- Does not resolve the gix vs git2 contradiction in spirit - only documents the decision to defer migration

## More Information

- **Related ADRs:** Future ADR for gix migration (P2)
- **Audit Reports:** docs/audits/2026-02-18-rust-distribution-practices.md, docs/audits/2026-02-18-setup-ease-audit.md, docs/audits/2026-02-18-packaging-distribution-audit.md, docs/audits/2026-02-18-feature-gap-analysis.md
- **Implementation:** Add features = ["] to the git2 dependency in workspace Cargo.toml. Update key-decisions.md to resolve the gix vs git2 contradiction and document this deferral decision.
