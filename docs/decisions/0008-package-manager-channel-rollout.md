# ADR-0008: Package manager channel rollout priorities

**Status:** Accepted
**Date:** 2026-02-18
**Decision Makers:** Project Team
**Informed by:** [audits/2026-02-18-rust-distribution-practices.md](../audits/2026-02-18-rust-distribution-practices.md), [audits/2026-02-18-research-consolidated.md](../audits/2026-02-18-research-consolidated.md), [audits/2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md)

## Context and Problem Statement

The project has multiple potential distribution channels: GitHub Releases, cargo-binstall, Homebrew, Winget, Scoop, Docker, Chocolatey, APT/RPM, and nixpkgs. Attempting to support all channels simultaneously would spread effort too thin. A phased rollout is needed to deliver value quickly while deferring lower-priority channels.

## Decision Drivers

- Phase 1 channels (GitHub Releases, cargo-binstall, Homebrew tap) are provided free by cargo-dist with minimal additional work
- Windows users need a native package manager path (Winget/Scoop) but these require additional GitHub Actions and registry submissions
- Docker is already partially implemented but has a known PATH bug blocking production use
- Chocolatey, APT/RPM, and nixpkgs require significant maintenance overhead and are appropriate only at v1.0 scale

## Considered Options

1. Phased rollout: GitHub Releases + cargo-binstall + Homebrew now; Winget + Scoop + Docker next; system package managers at v1.0
2. Prioritize Windows-first: ship Winget and Scoop before Homebrew
3. Binary-only via GitHub Releases with no package manager integration until v1.0

## Decision Outcome

**Chosen option:** "Phased rollout", because Phase 1 delivers three channels essentially for free via cargo-dist (ADR-0002), Phase 2 fills the Windows gap and fixes the existing Docker investment, and Phase 3 defers high-maintenance system package managers until the project has the user base to justify them.

### Consequences

**Good:**
- Phase 1 ships immediately as a side effect of implementing ADR-0002
- macOS users get Homebrew; Linux/Windows users get cargo-binstall; all users get GitHub Releases
- Winget and Scoop (Phase 2) cover Windows users without requiring Chocolatey maintenance

**Bad:**
- Windows users in Phase 1 must use cargo-binstall or build from source rather than a native package manager
- Docker PATH bug must be fixed before Phase 2 Docker rollout
- Each new channel adds ongoing maintenance surface

## More Information

- **Related ADRs:** [ADR-0002](0002-cargo-dist-and-cargo-release.md), [ADR-0007](0007-publish-all-crates.md)
- **Audit Reports:** [audits/2026-02-18-rust-distribution-practices.md](../audits/2026-02-18-rust-distribution-practices.md) (Sections 1, 6, 10), [audits/2026-02-18-research-consolidated.md](../audits/2026-02-18-research-consolidated.md) (Section 5), [audits/2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md) (P1-3, P1-4)
- **Implementation:** Phase 1 is automatic from ADR-0002 (cargo-dist). Phase 2: add winget-releaser GitHub Action, create Scoop bucket JSON manifest, fix Docker PATH bug and publish image to ghcr.io. Phase 3: submit to Chocolatey, set up APT/RPM repos, open nixpkgs PR.
