# ADR-0016: Complete cargo metadata for all crates

**Status:** Accepted
**Date:** 2026-02-18
**Decision Makers:** Project Team
**Informed by:** [audits/2026-02-18-packaging-distribution-audit.md](../audits/2026-02-18-packaging-distribution-audit.md), [audits/2026-02-18-marketing-audit-consolidated.md](../audits/2026-02-18-marketing-audit-consolidated.md)

## Context and Problem Statement

All 11 crates in the workspace are missing `homepage`, `documentation`, `keywords`, and `categories` fields. The `repository` URL is a placeholder (`https://github.com/user/repository-manager`) rather than the actual project URL. These fields are required for a professional crates.io presence and are used by crates.io for search indexing and discoverability.

Without complete metadata, the crates will appear incomplete on crates.io, rank poorly in searches, and provide no useful links to documentation or the project homepage.

## Decision Drivers

- crates.io uses `keywords` and `categories` for search and browsing — missing them hurts discoverability
- The placeholder `repository` URL will break for anyone who follows the link
- `documentation` pointing to docs.rs enables one-click access to API docs once published
- All crates can inherit from `[workspace.package]`, so this is a single-point fix
- Prerequisite for ADR-0007 (crates.io publishing) to produce a professional result

## Considered Options

1. Set full metadata at workspace level in `[workspace.package]` — all crates inherit automatically
2. Set metadata per-crate individually with varying values
3. Set only the required fields (fix repository URL) and leave optional fields empty

## Decision Outcome

**Chosen option:** "Set full metadata at workspace level", because all crates share the same project identity, workspace inheritance eliminates repetition, and this is the standard approach for multi-crate workspaces. All crates inherit the fields without per-crate changes.

### Consequences

**Good:**
- All 11 crates receive complete metadata in a single change
- crates.io search and category browsing will surface the crates appropriately
- The broken placeholder repository URL is replaced with the real URL
- docs.rs documentation link is pre-configured for when crates are published

**Bad:**
- The actual GitHub repository URL must be confirmed and hardcoded before publishing
- Keywords and categories require deliberate choices that reflect the project's identity
- Any crate needing to override workspace-level metadata must do so explicitly

## More Information

- **Related ADRs:** [ADR-0007](0007-publish-all-crates.md), [ADR-0002](0002-cargo-dist-and-cargo-release.md)
- **Audit Reports:** [audits/2026-02-18-packaging-distribution-audit.md](../audits/2026-02-18-packaging-distribution-audit.md) (Section 2, P1-12), [audits/2026-02-18-marketing-audit-consolidated.md](../audits/2026-02-18-marketing-audit-consolidated.md)
- **Implementation:** Update `[workspace.package]` in the root `Cargo.toml`:
  - Fix `repository` to the actual GitHub URL
  - Add `homepage` (same as repository or project website)
  - Add `documentation` (docs.rs URL pattern: `https://docs.rs/{crate-name}`)
  - Add `keywords = ["cli", "workspace", "ai-agent", "configuration", "devtools"]`
  - Add `categories = ["command-line-utilities", "development-tools", "config"]`
