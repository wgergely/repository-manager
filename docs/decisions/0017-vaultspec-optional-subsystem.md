# ADR-0017: Keep vaultspec subprocess integration as optional documented subsystem

**Status:** Accepted
**Date:** 2026-02-18
**Decision Makers:** Project Team
**Informed by:** [audits/2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md), [audits/2026-02-18-documentation-audit.md](../audits/2026-02-18-documentation-audit.md)

## Context and Problem Statement

`repo-agent` discovers and invokes vaultspec as a subprocess, requiring Python 3.13+ and a `.vaultspec/` directory. The integration gracefully degrades when vaultspec is unavailable. The MCP server exposes `agent_spawn`, `status`, and `stop` operations backed by this integration. Vaultspec is actively developed with AGENTS.md integration in progress.

However, the `plugins` command hardcodes `vaultspec v4.1.1`, which will become stale as vaultspec evolves. Additionally, users have no clear documentation explaining which features require vaultspec and which work standalone, creating confusion about the tool's requirements.

## Decision Drivers

- The graceful degradation design confirms optional integration is already the intended model — this decision formalizes it
- Hardcoded version strings in user-facing output create a maintenance burden and erode trust when they become stale
- Users need to know upfront which features are core (no external deps) vs optional (require vaultspec)
- AGENTS.md integration in vaultspec is in progress, indicating the subsystem is actively evolving
- Clear documentation reduces support burden and sets correct expectations

## Considered Options

1. Keep vaultspec integration as optional, document it clearly, and make version dynamic
2. Promote vaultspec to a required dependency and drop the graceful degradation
3. Remove vaultspec integration entirely and treat agent spawning as out of scope

## Decision Outcome

**Chosen option:** "Keep vaultspec integration as optional, document it clearly, and make version dynamic", because the graceful degradation is already implemented and working, vaultspec is actively developed making it viable long-term, and the optional model correctly represents the tool's value proposition — core workspace features work standalone.

### Consequences

**Good:**
- Users without Python/vaultspec can still use all core features (config sync, worktrees, drift detection)
- Dynamic version display stays accurate as vaultspec evolves without code changes
- Clear documentation sets correct expectations and reduces user confusion
- Implementation status matrix gives contributors and users a clear picture of feature completeness

**Bad:**
- Two-tier documentation (core vs optional) adds complexity to onboarding materials
- Dynamic version lookup introduces a subprocess call or file read at plugins command time
- Maintaining an accurate feature status matrix requires ongoing updates as features land

## More Information

- **Related ADRs:** [ADR-0004](0004-agentic-workspace-manager-positioning.md), [ADR-0012](0012-snap-mode-agent-lifecycle.md)
- **Audit Reports:** [audits/2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md) (Sections 1.1, 1.2, 8), [audits/2026-02-18-documentation-audit.md](../audits/2026-02-18-documentation-audit.md) (Section 6)
- **Implementation:**
  1. Create `docs/guides/agent-spawning.md` explaining vaultspec setup requirements and configuration
  2. Make the `plugins` command version-dynamic instead of hardcoding `vaultspec v4.1.1`
  3. Update README to clearly distinguish core features from optional vaultspec-dependent features
  4. Add an implementation status matrix showing done/partial/planned feature states
