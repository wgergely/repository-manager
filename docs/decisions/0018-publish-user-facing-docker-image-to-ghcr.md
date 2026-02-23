# ADR-0018: Publish user-facing Docker image to ghcr.io

**Status:** Accepted
**Date:** 2026-02-18
**Decision Makers:** Project Team
**Informed by:** [2026-02-18-rust-distribution-practices.md](../audits/2026-02-18-rust-distribution-practices.md), [2026-02-18-packaging-distribution-audit.md](../audits/2026-02-18-packaging-distribution-audit.md), [2026-02-18-setup-ease-audit.md](../audits/2026-02-18-setup-ease-audit.md), [2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md)

## Context and Problem Statement

Docker infrastructure exists for testing only. `docker/repo-manager/Dockerfile` has a critical PATH bug where the binary is not found at runtime, masked by a `|| echo` fallback. No images are published to any registry. Users have no zero-install trial path — they must build from source or use a package manager channel to try the tool.

## Decision Drivers

- Users need a zero-friction way to try the tool without installing a Rust toolchain
- CI/CD pipelines benefit from a stable, versioned container image
- The existing Dockerfile bug must be fixed regardless; publishing adds minimal additional effort
- Complements binary distribution via cargo-dist

## Considered Options

1. Fix PATH bug, implement multi-stage build (Rust builder + debian:bookworm-slim runtime), publish to ghcr.io on release tags
2. Static musl binary in `FROM scratch` image — smallest possible image
3. No user-facing Docker image — keep Docker infrastructure for testing only

## Decision Outcome

**Chosen option:** "Fix + publish multi-stage image to ghcr.io", because it provides the best balance of zero-install accessibility and runtime compatibility. The debian:bookworm-slim runtime layer ensures dynamic linking works without requiring a fully static build, while keeping image size reasonable. The `docker run ghcr.io/org/repo-manager repo init` experience is a concrete onboarding improvement.

### Consequences

**Good:**
- Zero-install experimentation path for new users
- CI/CD pipeline integration with a versioned image
- Complements existing binary distribution
- PATH bug is fixed, removing silent failures in test infrastructure

**Bad:**
- Ongoing image maintenance burden (base image updates, security patches)
- Image size larger than a musl/scratch approach
- ghcr.io requires GitHub Actions integration to publish

## More Information

- **Related ADRs:** [ADR-0002](0002-cargo-dist-and-cargo-release.md), [ADR-0008](0008-package-manager-channel-rollout.md)
- **Audit Reports:** [2026-02-18-rust-distribution-practices.md (Section 3)](../audits/2026-02-18-rust-distribution-practices.md), [2026-02-18-packaging-distribution-audit.md (P0-3, P0-4, P1-16)](../audits/2026-02-18-packaging-distribution-audit.md), [2026-02-18-setup-ease-audit.md (Section 4)](../audits/2026-02-18-setup-ease-audit.md), [2026-02-18-feature-gap-analysis.md (P2-5)](../audits/2026-02-18-feature-gap-analysis.md)
- **Implementation:** Fix `docker/repo-manager/Dockerfile` PATH bug. Create new multi-stage Dockerfile with Rust builder stage and debian:bookworm-slim runtime stage. Add ghcr.io publish step to release workflow triggered on release tags. Document `docker run` usage in README.
