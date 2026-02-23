# ADR-0004: Position as "Agentic Workspace Manager"

**Status:** Accepted
**Date:** 2026-02-18
**Decision Makers:** Project Team
**Informed by:** 2026-02-18-competitor-analysis.md (Section 6), 2026-02-18-ai-ecosystem-landscape.md, 2026-02-18-research-consolidated.md, 2026-02-18-feature-gap-analysis.md (Section 7.2)

## Context and Problem Statement

The project needs a clear marketing position that differentiates it from competitors and accurately reflects its unique value. AGENTS.md generation is being handled by the vaultspec project (in progress). Repository Manager's unique value is the combination of worktrees + multi-agent orchestration + config sync + MCP server that no competitor replicates. Without a clear position, messaging becomes diluted and the product fails to stand out in an increasingly crowded AI tooling space.

## Decision Drivers

- No competitor replicates the full combination of worktrees + multi-agent orchestration + config sync + MCP server
- AGENTS.md generation is owned by the vaultspec project, reducing overlap
- Teams adopting multiple AI agents need an operational backbone to manage parallel workstreams
- The MCP server provides a unique integration point that strengthens the agentic positioning
- Config sync across 13+ tools is a strong secondary value proposition but not the primary differentiator

## Considered Options

1. "Universal Config Control Plane" - lead with config sync across 13+ tools, drift detection. AGENTS.md as one of many outputs.
2. "Agentic Workspace Manager" - lead with multi-agent workflow story: worktrees for parallel agents, preset system, MCP server, snap mode (future). Position as operational backbone for teams using multiple AI agents.
3. Defer positioning - move to P1, focus P0 on technical decisions only.
4. "Complementary to vaultspec" - explicitly position as complement to vaultspec (agent lifecycle + AGENTS.md) while Repository Manager handles config sync + worktrees + drift detection.

## Decision Outcome

**Chosen option:** "Agentic Workspace Manager", because it positions around the unique combination that no competitor replicates. The agentic workspace narrative (worktrees for parallel agents, unified config sync, MCP server integration) tells a cohesive story that resonates with teams scaling AI-assisted development. AGENTS.md is handled by vaultspec; Repository Manager owns the workspace layer.

### Consequences

**Good:**
- Clear differentiation from competitors who focus on single-agent or config-only solutions
- Coherent narrative connecting worktrees, config sync, and MCP server as a unified product
- Positions well for snap mode (P1) and other future agentic features
- Avoids overlap with vaultspec's AGENTS.md ownership

**Bad:**
- "Agentic Workspace Manager" may be unfamiliar terminology requiring more explanation
- De-emphasizing config sync as a primary hook may undersell a genuinely strong feature
- Relies on multi-agent workflows becoming mainstream adoption; early market positioning risk

## More Information

- **Related ADRs:** ADR-0012 (MCP server strategy - P1), ADR-0015 (snap mode - P1)
- **Audit Reports:** docs/audits/2026-02-18-competitor-analysis.md (Section 6), docs/audits/2026-02-18-ai-ecosystem-landscape.md, docs/audits/2026-02-18-research-consolidated.md, docs/audits/2026-02-18-feature-gap-analysis.md (Section 7.2)
- **Implementation:** Update README.md tagline and project-overview.md. Lead messaging with: worktree management for parallel AI agents, unified config sync for 13+ tools, drift detection and repair, MCP server for agent integration. De-emphasize AGENTS.md as a primary hook (keep as a supported output format). Clarify relationship with vaultspec in docs.
