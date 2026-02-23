# ADR-0009: Adopt cargo-deny for supply chain security

**Status:** Accepted
**Date:** 2026-02-18
**Decision Makers:** Project Team
**Informed by:** [audits/2026-02-18-packaging-distribution-audit.md](../audits/2026-02-18-packaging-distribution-audit.md), [audits/2026-02-18-marketing-audit-consolidated.md](../audits/2026-02-18-marketing-audit-consolidated.md), [audits/2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md)

## Context and Problem Statement

The project has no deny.toml and no automated supply chain security checks. There is no vulnerability scanning, license compliance enforcement, or duplicate dependency detection in CI. The dependency tree includes crates with known risk surface: serde_yaml pulls in unsafe-libyaml (C bindings), and git2 uses libgit2 via C FFI. Without tooling, these risks go undetected until a security incident occurs.

## Decision Drivers

- No vulnerability advisory scanning means known CVEs in dependencies go unnoticed
- No license compliance checking risks shipping code with incompatible licenses
- serde_yaml and git2 introduce C FFI dependencies that warrant explicit acknowledgment
- cargo-deny is the standard Rust ecosystem tool for this category and integrates cleanly with CI
- The CI workflow (ADR-0006) provides the right integration point

## Considered Options

1. Adopt cargo-deny with full advisories, licenses, bans, and sources checks
2. Use cargo-audit only (advisories only, no license or ban checks)
3. Rely on GitHub Dependabot alerts without local tooling

## Decision Outcome

**Chosen option:** "Adopt cargo-deny with full checks", because it covers all four risk categories (vulnerabilities, licenses, banned crates, untrusted sources) in a single tool with a single CI step. cargo-audit covers only advisories. Dependabot alone provides no local developer workflow and no license enforcement.

### Consequences

**Good:**
- Automated vulnerability scanning catches CVEs in dependencies before release
- License allowlist prevents accidental inclusion of GPL or other incompatible licenses
- Duplicate dependency detection keeps the dependency tree clean
- Explicit allow-list for serde_yaml and git2 documents the accepted risk

**Bad:**
- Initial `cargo deny check` run will likely surface findings that require triage
- deny.toml requires ongoing maintenance as dependencies are added or updated
- False positives in advisory or ban checks may slow down dependency updates

## More Information

- **Related ADRs:** [ADR-0006](0006-rust-ci-workflow.md)
- **Audit Reports:** [audits/2026-02-18-packaging-distribution-audit.md](../audits/2026-02-18-packaging-distribution-audit.md) (P1-10), [audits/2026-02-18-marketing-audit-consolidated.md](../audits/2026-02-18-marketing-audit-consolidated.md) (Section 6), [audits/2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md) (P1-7)
- **Implementation:** Run `cargo deny init` to generate deny.toml. Configure the license allowlist to permit MIT, Apache-2.0, and BSD-* licenses. Add explicit skip entries for known C FFI crates (serde_yaml/unsafe-libyaml, git2/libgit2). Add `cargo deny check` as a CI job in the workflow defined by ADR-0006. Run the initial check and resolve all findings before merging.
