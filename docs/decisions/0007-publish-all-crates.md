# ADR-0007: Publish all workspace crates to crates.io

**Status:** Accepted
**Date:** 2026-02-18
**Decision Makers:** Project Team
**Informed by:** [audits/2026-02-18-rust-distribution-practices.md](../audits/2026-02-18-rust-distribution-practices.md), [audits/2026-02-18-packaging-distribution-audit.md](../audits/2026-02-18-packaging-distribution-audit.md), [audits/2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md)

## Context and Problem Statement

The workspace contains 12 crates, all connected via path dependencies. For `cargo install repo-cli` to work end-to-end, every transitive path dependency must be published to crates.io. Currently none of the crates are published, blocking the primary installation path for users.

The required publishing order (leaf-first) is: repo-fs, repo-content → repo-git, repo-blocks, repo-meta → repo-tools, repo-presets → repo-core → repo-agent → repo-mcp, repo-cli. The integration-tests crate already has `publish = false` and is excluded.

## Decision Drivers

- `cargo install repo-cli` requires all transitive path dependencies to be available on crates.io
- Users expect to install CLI tools via `cargo install` without cloning the repository
- All 11 publishable crates share a single workspace version, simplifying release coordination
- The repository URL in workspace Cargo.toml is currently a placeholder and must be fixed before publishing

## Considered Options

1. Publish all 11 crates to crates.io
2. Publish only the top-level binaries (repo-cli, repo-mcp) and vendor the rest
3. Distribute only via pre-built binaries (GitHub Releases, cargo-binstall) and skip crates.io publishing

## Decision Outcome

**Chosen option:** "Publish all 11 crates", because `cargo install` is a standard, trusted installation path for Rust tools and requires all transitive dependencies to be on crates.io. Vendoring adds complexity. Binary-only distribution limits adoption among Rust developers who prefer source installs.

### Consequences

**Good:**
- `cargo install repo-cli` works out of the box
- Each crate is individually discoverable and usable as a library
- Shared workspace versioning keeps all crates in sync via cargo-release

**Bad:**
- All 11 crates must be published in dependency order on every release
- Placeholder repository URL and missing metadata must be fixed before the first publish
- Breaking changes to internal crates become semver-visible to external consumers

## More Information

- **Related ADRs:** [ADR-0002](0002-cargo-dist-and-cargo-release.md), [ADR-0016](0016-cargo-metadata-completeness.md)
- **Audit Reports:** [audits/2026-02-18-rust-distribution-practices.md](../audits/2026-02-18-rust-distribution-practices.md), [audits/2026-02-18-packaging-distribution-audit.md](../audits/2026-02-18-packaging-distribution-audit.md) (Section 2), [audits/2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md) (P1-2)
- **Implementation:** Fix the placeholder repository URL in workspace Cargo.toml. Add homepage, documentation, keywords, and categories metadata per ADR-0016. Use cargo-release with `publish-order` configuration to publish crates leaf-first.
