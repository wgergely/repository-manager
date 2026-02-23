# ADR-0013: Expand tool support to 20+ via generic integration

**Status:** Accepted
**Date:** 2026-02-18
**Decision Makers:** Project Team
**Informed by:** [audits/2026-02-18-competitor-analysis.md](../audits/2026-02-18-competitor-analysis.md) (Section 5), [audits/2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md) (Section 2.1, P1-6), [audits/2026-02-18-research-consolidated.md](../audits/2026-02-18-research-consolidated.md) (Section 7)

## Context and Problem Statement

Repository Manager currently supports 14 tools (not 13 as documented in the README). Competitor Ruler supports 30+, and rulesync supports 20+. This perception gap weakens the tool's competitive position. The existing GenericToolIntegration framework means adding new tool support is largely a configuration task using the ToolDefinition schema from repo-meta — not new Rust code. Expanding to 20+ tools is achievable with modest effort per tool.

## Decision Drivers

- Competitive tools support significantly more integrations (30+)
- GenericToolIntegration framework makes new tool additions low-effort
- README incorrectly states "13 tools" — accurate documentation is needed regardless
- Community contribution is more tractable with clear per-tool effort expectations

## Considered Options

1. Expand to 20+ tools via generic integration framework with contributor documentation
2. Keep current tool set and focus on depth (better support for existing tools)
3. Build a plugin system allowing external tool definitions without code changes

## Decision Outcome

**Chosen option:** "Expand to 20+ tools via generic integration framework with contributor documentation", because the generic framework already exists and the per-tool effort is low, making this a high-value, low-risk expansion.

Priority tool additions: OpenCode, Kilo Code, Continue.dev, Amp, Codex CLI, Kiro. Each requires: (1) research config file format and location, (2) create tool definition in .repository/tools/ or add to built-in registry, (3) test config generation. Community-driven expansion is enabled by contributor documentation. The README is updated from "13" to the actual current count immediately.

### Consequences

**Good:**
- Closes the competitive gap with Ruler (30+) and rulesync (20+)
- Low per-tool effort (1-2 days) via existing generic framework
- Community contributions become tractable with clear documentation
- Corrects the inaccurate "13 tools" claim in documentation

**Bad:**
- Each new tool requires maintenance when those tools change their config formats
- Expanding tool count without usage data may add low-value integrations
- Contributor documentation requires upfront investment

## More Information

- **Related ADRs:** [ADR-0004](0004-agentic-workspace-manager-positioning.md)
- **Audit Reports:** [audits/2026-02-18-competitor-analysis.md](../audits/2026-02-18-competitor-analysis.md), [audits/2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md), [audits/2026-02-18-research-consolidated.md](../audits/2026-02-18-research-consolidated.md)
- **Implementation:** For each new tool: (1) research config file format and location, (2) create tool definition in .repository/tools/ or add to built-in registry, (3) test config generation. Create CONTRIBUTING.md section on adding new tool integrations. Update README tool count. Estimated effort: 1-2 days per tool.
