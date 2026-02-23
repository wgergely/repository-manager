# ADR-0014: Detection-only presets with future mise integration

**Status:** Accepted
**Date:** 2026-02-18
**Decision Makers:** Project Team
**Informed by:** [audits/2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md) (Sections 1.2, 8), [audits/2026-02-18-competitor-analysis.md](../audits/2026-02-18-competitor-analysis.md) (Sections 3.1, 8), [audits/2026-02-18-research-consolidated.md](../audits/2026-02-18-research-consolidated.md) (Section 7)

## Context and Problem Statement

The project overview claims presets "automatically install binaries, create virtual environments." The actual behavior is different: 3 language providers (Python/UV+venv, Node, Rust) plus 1 plugins provider detect environments but do not install anything. The `PresetProvider` trait has `check()` and `apply()` methods, but `apply()` performs repair and detection — not installation. This documentation mismatch misleads users and creates false expectations. Competitor mise (14K stars) handles runtime installation and could be integrated to cover this gap.

## Decision Drivers

- Documentation must accurately describe current behavior to maintain user trust
- "Agentic Workspace Manager" positioning depends on config sync, worktrees, and orchestration — not on installation
- Delegating runtime installation to a specialized tool (mise) is preferable to re-implementing it
- Over-promising on installation causes user confusion and support burden

## Considered Options

1. Update documentation to reflect detection-only behavior; plan future mise integration
2. Implement actual installation in presets to match current documentation claims
3. Remove preset installation claims from documentation and deprioritize the feature entirely

## Decision Outcome

**Chosen option:** "Update documentation to reflect detection-only behavior; plan future mise integration", because the current positioning does not require installation, accurate documentation is immediately actionable, and mise integration is a well-scoped future enhancement.

Documentation is updated to accurately describe presets as "environment verification and config generation" rather than "installation." Mise integration is tracked as a future P2 ADR — Repository Manager delegates runtime installation to mise and focuses on config sync, worktrees, and agent orchestration.

### Consequences

**Good:**
- Documentation accurately reflects actual behavior, eliminating user confusion
- Removes maintenance burden of a feature that was never implemented
- Mise integration path is clear and well-scoped for future work
- Agentic Workspace Manager positioning is unaffected — it does not depend on installation

**Bad:**
- Users who expected installation functionality will be disappointed
- Mise integration deferred to P2 means the gap remains open in the near term
- Requires a documentation audit pass to catch all instances of the incorrect claim

## More Information

- **Related ADRs:** [ADR-0004](0004-agentic-workspace-manager-positioning.md)
- **Audit Reports:** [audits/2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md), [audits/2026-02-18-competitor-analysis.md](../audits/2026-02-18-competitor-analysis.md), [audits/2026-02-18-research-consolidated.md](../audits/2026-02-18-research-consolidated.md)
- **Implementation:** Update project-overview.md: change "automatically installs" to "detects and configures." Update README preset description. Create issue for future mise integration. No code changes required.
