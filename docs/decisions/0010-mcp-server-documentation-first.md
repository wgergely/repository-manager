# ADR-0010: MCP server documentation first, config propagation second

**Status:** Accepted
**Date:** 2026-02-18
**Decision Makers:** Project Team
**Informed by:** [audits/2026-02-18-competitor-analysis.md](../audits/2026-02-18-competitor-analysis.md), [audits/2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md), [audits/2026-02-18-documentation-audit.md](../audits/2026-02-18-documentation-audit.md), [audits/2026-02-18-research-consolidated.md](../audits/2026-02-18-research-consolidated.md)

## Context and Problem Statement

The repo-mcp crate implements a fully functional MCP server with 25 tools, 3 resources, and JSON-RPC over stdio. However, there is zero user-facing documentation explaining how to connect any MCP client to it. Users who discover the binary have no guidance for Claude Desktop, Cursor, VS Code, or other MCP clients.

A separate, related feature - writing MCP server config entries into tool-specific files (.claude/settings.json, .cursor/mcp.json) - is not yet implemented. No known competitor implements this config-propagation feature either, so it provides differentiation but is not a blocker for existing MCP server adoption.

## Decision Drivers

- The MCP server already works; the gap is purely documentation, not implementation
- Users cannot adopt a feature they cannot configure, regardless of how well it works
- Config propagation is a non-trivial feature requiring design and implementation work
- No competitor does config propagation, so it is a differentiator worth building but not urgent
- Documentation is a quick win that unblocks existing users immediately

## Considered Options

1. Documentation first, then implement config propagation as a follow-on feature
2. Implement config propagation first, then document both features together
3. Document only, defer config propagation indefinitely

## Decision Outcome

**Chosen option:** "Documentation first, config propagation second", because the server already works and documentation unlocks its value for all current users with zero implementation risk. Config propagation is a meaningful differentiator and should be implemented, but deferring it does not block MCP server adoption.

### Consequences

**Good:**
- Users can connect Claude Desktop, Cursor, and VS Code to the MCP server immediately after documentation ships
- Documentation effort is low and can be completed in a single PR
- Config propagation can be designed carefully without blocking the documentation win

**Bad:**
- Users must manually add MCP server config entries to their tool files until config propagation is implemented
- Two separate releases are needed to deliver the full vision

## More Information

- **Related ADRs:** [ADR-0004](0004-agentic-workspace-manager-positioning.md)
- **Audit Reports:** [audits/2026-02-18-competitor-analysis.md](../audits/2026-02-18-competitor-analysis.md) (Section 4.2), [audits/2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md) (Sections 1.2, P1-10), [audits/2026-02-18-documentation-audit.md](../audits/2026-02-18-documentation-audit.md) (Section 7), [audits/2026-02-18-research-consolidated.md](../audits/2026-02-18-research-consolidated.md) (Section 4)
- **Implementation:** Phase 1: Create docs/guides/mcp-server.md covering installation, stdio transport configuration, and client-specific connection instructions for Claude Desktop, Cursor, and VS Code. Include example JSON config snippets for each client. Phase 2: Add MCP server config entry writers to the tool integration config writers in repo-tools, so `repo init` or `repo mcp install` can inject the server config into the appropriate tool files.
