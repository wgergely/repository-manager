# ADR-0006: Add comprehensive Rust CI workflow

**Status:** Accepted
**Date:** 2026-02-18
**Decision Makers:** Project Team
**Informed by:** 2026-02-18-packaging-distribution-audit.md (P1-14, P1-15), 2026-02-18-marketing-audit-consolidated.md (recommendation #7), 2026-02-18-feature-gap-analysis.md (P0-6)

## Context and Problem Statement

The only CI workflow currently in the repository is `docker-integration.yml`, which tests Docker-based integration scenarios. No `cargo test`, `cargo clippy`, or `cargo fmt --check` runs exist in CI. This means PRs can merge with broken tests, lint failures, or formatting inconsistencies without any automated gate. For a project positioning itself as production-quality infrastructure for agentic workflows, the absence of basic Rust CI is a credibility gap and a practical risk to code quality.

## Decision Drivers

- PRs can currently merge with broken tests, clippy violations, or formatting issues
- No security/license gate exists for dependency changes
- A project used as infrastructure tooling requires higher reliability guarantees
- The combined CI runtime (~5 minutes) is acceptable for a per-PR gate
- Cross-platform support (Windows, macOS, Linux) matters given the tool's target audience
- Automated dependency updates reduce maintenance burden and security exposure

## Considered Options

1. Full Rust CI workflow - `cargo test --workspace` + `cargo clippy --workspace` + `cargo fmt --check` + `cargo deny check` on every PR. Complete quality gate.
2. Minimal - `cargo test --workspace` only. Catches test failures but misses lint violations, formatting issues, and security advisories.
3. Multi-tier - fast check on PR (test + fmt), full suite (clippy + deny) on merge to main. Reduces per-PR latency at the cost of catching issues later.

## Decision Outcome

**Chosen option:** "Full Rust CI workflow on every PR", because the combined runtime is ~5 minutes which is acceptable, and catching issues at PR time is strictly better than catching them at merge time. A multi-tier approach optimizes for speed at the cost of earlier feedback; given the team size and PR frequency, this trade-off is not warranted. Add Dependabot for automated dependency updates.

### Consequences

**Good:**
- Broken tests, clippy violations, and formatting issues are caught before merge
- Cross-platform matrix (ubuntu, windows, macos) surfaces platform-specific issues early
- `cargo deny check` provides a security and license gate on every dependency change
- Dependabot keeps dependencies current with minimal manual effort
- Establishes baseline quality expectations for contributors

**Bad:**
- ~5 minute CI runtime adds latency to PR review cycle
- `cargo deny check` requires a `deny.toml` configuration file (see ADR for cargo-deny, P1); the deny job may need to be gated until that file exists
- Windows and macOS runners consume more CI minutes than Linux-only

## More Information

- **Related ADRs:** ADR-0002 (release pipeline), ADR-0009 (cargo-deny - P1)
- **Audit Reports:** docs/audits/2026-02-18-packaging-distribution-audit.md (P1-14, P1-15), docs/audits/2026-02-18-marketing-audit-consolidated.md (recommendation #7), docs/audits/2026-02-18-feature-gap-analysis.md (P0-6)
- **Implementation:** Create `.github/workflows/ci.yml` with:
  - Matrix: ubuntu-latest, windows-latest, macos-latest
  - Steps: `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --check`
  - Separate job for `cargo deny check` (requires deny.toml - see ADR-0009)
  - Add `.github/dependabot.yml` for Cargo dependency updates (weekly schedule, auto-assign reviewers)
