# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for the Repository Manager project. Each ADR documents a significant architectural or strategic decision, the context that drove it, the options considered, and the chosen outcome.

ADRs follow the [MADR (Markdown Architectural Decision Records)](https://adr.github.io/madr/) format with fields for Status, Date, Decision Makers, and the audit reports that informed the decision.

New ADRs should be numbered sequentially and named `NNNN-short-title.md`.

---

## Status Legend

| Status | Meaning |
|--------|---------|
| **Proposed** | Decision is under review, not yet ratified |
| **Accepted** | Decision is ratified and in effect |
| **Deprecated** | Decision was accepted but is no longer applicable |
| **Superseded** | Decision was replaced by a later ADR |

---

## ADR Index by Priority

### P0 - Critical (Before Public Alpha)

These decisions unblock distribution, fix trust-destroying bugs, and establish the minimum viable quality bar required before the tool can be publicly promoted.

| # | Title | Status | Date | Key Impact |
|---|-------|--------|------|------------|
| [0001](0001-git2-vendored-feature.md) | Use git2 with vendored feature for Git operations | Accepted | 2026-02-18 | Eliminates undocumented C build-time dependencies; unblocks binary distribution |
| [0002](0002-cargo-dist-and-cargo-release.md) | Adopt cargo-dist and cargo-release for release pipeline | Accepted | 2026-02-18 | Drops time-to-first-value from 10-20 min to <60 sec; enables one-line install |
| [0003](0003-release-profile-optimizations.md) | Add release profile optimizations | Accepted | 2026-02-18 | 30-50% binary size reduction via LTO + strip; complements cargo-dist |
| [0004](0004-agentic-workspace-manager-positioning.md) | Position as "Agentic Workspace Manager" | Accepted | 2026-02-18 | Establishes clear market differentiation; drives messaging and feature priority |
| [0005](0005-comma-delimited-tools-flag.md) | Support comma-delimited values in --tools flag | Accepted | 2026-02-18 | Fixes trust-destroying silent failure when users follow documented syntax |
| [0006](0006-rust-ci-workflow.md) | Add comprehensive Rust CI workflow | Accepted | 2026-02-18 | Closes credibility gap; gates PRs on tests, lint, formatting, and supply chain |

### P1 - Important (Before Beta/GA)

These decisions build the distribution ecosystem, secure the supply chain, establish the agentic workflow features, and ensure crates.io publishing produces a professional result.

| # | Title | Status | Date | Key Impact |
|---|-------|--------|------|------------|
| [0007](0007-publish-all-crates.md) | Publish all workspace crates to crates.io | Accepted | 2026-02-18 | Enables `cargo install repo-cli`; all 11 crates published leaf-first |
| [0008](0008-package-manager-channel-rollout.md) | Package manager channel rollout priorities | Accepted | 2026-02-18 | Phased rollout: Homebrew/cargo-binstall now; Winget/Scoop/Docker next |
| [0009](0009-cargo-deny-supply-chain-security.md) | Adopt cargo-deny for supply chain security | Accepted | 2026-02-18 | Automates vulnerability scanning, license compliance, and duplicate detection |
| [0010](0010-mcp-server-documentation-first.md) | MCP server documentation first, config propagation second | Accepted | 2026-02-18 | Unlocks existing 25-tool MCP server for users; config propagation follows |
| [0011](0011-default-standard-mode.md) | Default to standard mode with worktrees recommended | Accepted | 2026-02-18 | Eliminates surprising `main/` subdirectory in non-interactive init |
| [0012](0012-snap-mode-agent-lifecycle.md) | Implement snap mode for agent lifecycle | Accepted | 2026-02-18 | Delivers one-command agentic workflow matching competitor feature parity |
| [0013](0013-expand-tool-support-generic-integration.md) | Expand tool support to 20+ via generic integration | Accepted | 2026-02-18 | Closes competitive gap with Ruler (30+) and rulesync (20+) |
| [0014](0014-detection-only-presets-mise-integration.md) | Detection-only presets with future mise integration | Accepted | 2026-02-18 | Corrects documentation overpromise; defers mise integration to P2 |
| [0015](0015-declare-msrv-rust-1-85.md) | Declare MSRV at Rust 1.85 | Accepted | 2026-02-18 | Signals minimum build requirements; prerequisite for professional crates.io presence |
| [0016](0016-complete-cargo-metadata.md) | Complete cargo metadata for all crates | Accepted | 2026-02-18 | Fixes broken placeholder repository URL; enables crates.io discoverability |
| [0017](0017-vaultspec-optional-subsystem.md) | Keep vaultspec subprocess integration as optional documented subsystem | Accepted | 2026-02-18 | Formalizes optional integration; fixes stale hardcoded version; clears user expectations |

### P2 - Nice-to-Have (Post-GA)

These decisions add significant value and differentiation but are not required for the initial public release.

| # | Title | Status | Date | Key Impact |
|---|-------|--------|------|------------|
| [0018](0018-publish-user-facing-docker-image-to-ghcr.md) | Publish user-facing Docker image to ghcr.io | Accepted | 2026-02-18 | Provides zero-install trial path; fixes existing PATH bug in Docker infra |
| [0019](0019-auto-generate-json-schema-from-rust-config-types.md) | Auto-generate JSON Schema from Rust config types | Accepted | 2026-02-18 | Editor autocompletion for `config.toml`; schema always in sync with Rust types |
| [0020](0020-defer-read-only-enforcement-update-documentation.md) | Defer read-only enforcement, update documentation | Accepted | 2026-02-18 | Aligns docs with actual drift detection + repair implementation |
| [0021](0021-git-based-remote-rule-includes.md) | Git-based remote rule includes | Accepted | 2026-02-18 | Enables team/org rule sharing with versioning; git-only (no unauthenticated HTTP) |
| [0022](0022-import-top-3-tool-config-formats.md) | Import from top 3 tool config formats | Accepted | 2026-02-18 | Reduces migration friction; imports from `.cursorrules`, `CLAUDE.md`, `.windsurfrules` |
| [0023](0023-comprehensive-repo-doctor-command.md) | Implement comprehensive repo doctor command | Accepted | 2026-02-18 | Self-service diagnostics; actionable pass/fail for all environment checks |

---

## Dependency Graph

ADRs that build on or constrain other ADRs:

```
0001 (git2-vendored) ← 0021 (remote-rule-includes: git fetch implementation)

0002 (cargo-dist+release) ← 0003 (release profile: optimizations applied to dist builds)
                          ← 0007 (crates.io: cargo-release manages workspace publish order)
                          ← 0008 (pkg channels: Phase 1 channels free via cargo-dist)
                          ← 0018 (Docker: release workflow publishes image on release tags)

0004 (agentic positioning) ← 0011 (default mode: worktrees promoted in interactive init)
                           ← 0012 (snap mode: delivers the agentic one-command workflow)
                           ← 0013 (tool expansion: breadth supports positioning claims)
                           ← 0014 (detection-only: accurate docs protect positioning credibility)
                           ← 0017 (vaultspec optional: core value independent of vaultspec)
                           ← 0020 (read-only deferral: honest positioning on enforcement)

0006 (CI workflow) ← 0009 (cargo-deny: supply chain check runs in CI)
                  ← 0015 (MSRV: CI validates declared floor with Rust 1.85 job)

0007 (crates.io publishing) ← 0015 (MSRV: rust-version field prerequisite for publishing)
                            ← 0016 (cargo metadata: complete metadata prerequisite for publishing)

0008 (pkg channels) ← 0018 (Docker: Phase 2 channel rollout includes ghcr.io image)

0010 (MCP documentation) ← 0019 (JSON Schema: schema improves MCP client config understanding)

0011 (default standard mode) ← 0022 (import: standard mode is the default for migrating users)

0012 (snap mode) ← 0017 (vaultspec optional: snap mode degrades gracefully without vaultspec)

0013 (tool expansion) ← 0022 (import: top-3 import targets are among the expanded tool set)
```

---

## Audit Reports Referenced

The following audit reports were produced on 2026-02-18 and informed these decisions. They are located in [`../audits/`](../audits/).

| Report | Key Findings Used In |
|--------|---------------------|
| [2026-02-18-competitor-analysis.md](../audits/2026-02-18-competitor-analysis.md) | ADR-0004, ADR-0010, ADR-0012, ADR-0013, ADR-0014, ADR-0019, ADR-0020, ADR-0021, ADR-0022 |
| [2026-02-18-ai-ecosystem-landscape.md](../audits/2026-02-18-ai-ecosystem-landscape.md) | ADR-0004 |
| [2026-02-18-rust-distribution-practices.md](../audits/2026-02-18-rust-distribution-practices.md) | ADR-0001, ADR-0002, ADR-0007, ADR-0008, ADR-0018 |
| [2026-02-18-research-consolidated.md](../audits/2026-02-18-research-consolidated.md) | ADR-0004, ADR-0008, ADR-0010, ADR-0013, ADR-0014, ADR-0019, ADR-0020, ADR-0021, ADR-0022 |
| [2026-02-18-documentation-audit.md](../audits/2026-02-18-documentation-audit.md) | ADR-0005, ADR-0010, ADR-0011, ADR-0017 |
| [2026-02-18-setup-ease-audit.md](../audits/2026-02-18-setup-ease-audit.md) | ADR-0001, ADR-0005, ADR-0011, ADR-0018, ADR-0019, ADR-0023 |
| [2026-02-18-packaging-distribution-audit.md](../audits/2026-02-18-packaging-distribution-audit.md) | ADR-0001, ADR-0002, ADR-0003, ADR-0006, ADR-0007, ADR-0008, ADR-0009, ADR-0015, ADR-0016, ADR-0018 |
| [2026-02-18-marketing-audit-consolidated.md](../audits/2026-02-18-marketing-audit-consolidated.md) | ADR-0002, ADR-0003, ADR-0005, ADR-0006, ADR-0009, ADR-0016 |
| [2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md) | All ADRs |

---

## Implementation Plan

The phased implementation plan coordinating all ADRs is located at:

- [`../plans/2026-02-18-implementation-plan.md`](../plans/2026-02-18-implementation-plan.md)

The plan is organized into three phases matching the priority tiers above:

- **Phase 1 (P0):** Ship Alpha — ADRs 0001-0006
- **Phase 2 (P1):** Adoption Readiness — ADRs 0007-0017
- **Phase 3 (P2):** Post-GA — ADRs 0018-0023
