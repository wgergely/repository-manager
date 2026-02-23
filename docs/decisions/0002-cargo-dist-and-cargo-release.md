# ADR-0002: Adopt cargo-dist and cargo-release for release pipeline

**Status:** Accepted
**Date:** 2026-02-18
**Decision Makers:** Project Team
**Informed by:** 2026-02-18-rust-distribution-practices.md, 2026-02-18-packaging-distribution-audit.md, 2026-02-18-marketing-audit-consolidated.md, 2026-02-18-feature-gap-analysis.md

## Context and Problem Statement

The project has zero release automation. There are no pre-built binaries, no GitHub Releases, no install script, and no crates.io publications. Every audit report flagged this as the #1 blocker to adoption. The current time-to-first-value for a new user is 10-20 minutes of manual setup, which is unacceptable for a developer tool competing in a space where alternatives offer one-line installs.

The project also has 11 workspace crates whose versions must be managed in sync, adding complexity to any release process.

## Decision Drivers

- Time-to-first-value must drop from 10-20 minutes to under 60 seconds
- Multi-platform binary distribution (Linux, macOS, Windows) is required
- 11 workspace crates require coordinated version bumping
- Release tooling should be low-maintenance and follow Rust ecosystem conventions
- Shell installer and Homebrew tap are expected by target users

## Considered Options

1. cargo-dist only - generates GitHub Actions workflows for multi-platform binaries, shell installers, and Homebrew tap
2. Manual GitHub Actions matrix - full control but must maintain all workflows manually, including artifact signing and upload
3. cargo-dist + cargo-release (complementary) - cargo-dist for binary distribution and installers, cargo-release for version bumps, tagging, and crates.io publishing across all 11 workspace crates

## Decision Outcome

**Chosen option:** "Option 3 - cargo-dist + cargo-release", because these tools are complementary and together cover the full release lifecycle. cargo-dist handles binary artifact production and distribution (GitHub Releases, shell installer, Homebrew tap); cargo-release handles the version management workflow across all 11 workspace crates. This combination is the established standard used by major Rust projects including ripgrep, bat, and delta.

### Consequences

**Good:**
- One-line shell installer generated automatically by cargo-dist
- Multi-platform binaries (Linux x86_64/ARM, macOS x86_64/ARM, Windows x86_64) built in CI
- Homebrew tap support via cargo-dist
- Coordinated workspace-wide version bumping via cargo-release
- Follows conventions already familiar to Rust users
- Minimal ongoing maintenance once bootstrapped

**Bad:**
- cargo-dist adds a configuration file and workflow that must be kept current with cargo-dist releases
- Initial setup requires running `cargo dist init` and reviewing generated workflows
- cargo-release requires a `.config/release.toml` to configure workspace-wide behavior correctly

## More Information

- **Related ADRs:** ADR-0003 (release profile optimizations), ADR-0004 (crates.io publishing strategy - P1)
- **Audit Reports:** docs/audits/2026-02-18-rust-distribution-practices.md (Sections 2.3, 5.3, 8, 10), docs/audits/2026-02-18-packaging-distribution-audit.md (P0-2), docs/audits/2026-02-18-marketing-audit-consolidated.md, docs/audits/2026-02-18-feature-gap-analysis.md (P0-1)
- **Implementation:** Run `cargo dist init` from the workspace root to bootstrap the GitHub Actions release workflow. Create `.config/release.toml` to configure cargo-release for workspace-wide version bumps and tagging.
