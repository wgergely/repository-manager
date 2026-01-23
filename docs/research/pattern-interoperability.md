# Cross-Platform Interoperability

Strategies for sharing rules, memory, and skills across agentic coding tools.

## The Interoperability Challenge

Each tool has its own:
- Rules file format and location
- Memory/context system
- Skill/plugin architecture
- Configuration schema

**Goal**: Define once, use everywhere.

## Rules Sharing

### Universal Standard: AGENTS.md

AGENTS.md provides the best path to universal rules:

| Tool | Support |
|------|---------|
| OpenAI Codex | Native |
| Google Jules | Native |
| Cursor | Native |
| GitHub Copilot | Native |
| Aider | Native |
| Zed | Native |
| Claude Code | Compatible |

### Rules File Locations

| Tool | Primary | Additional |
|------|---------|------------|
| Claude Code | `CLAUDE.md` | `.claude/rules/*.md` |
| Cursor | `.cursorrules` | `.cursor/rules` |
| Copilot | `.github/copilot-instructions.md` | - |
| Windsurf | `.windsurfrules` | `.windsurf/rules/` |
| Zed | `.zed/settings.json` | - |
| Universal | `AGENTS.md` | - |

### Sync Strategy

```
.repository/rules/common.md    (source of truth)
        |
   [repo-manager sync]
        |
   +----+----+----+----+
   v    v    v    v    v
AGENTS.md  CLAUDE.md  .cursorrules  copilot-instructions.md  .windsurfrules
```

### What's Portable

| Content Type | Portability | Notes |
|--------------|-------------|-------|
| Code style guidelines | High | Universal Markdown |
| Project structure | High | Universal Markdown |
| Build commands | High | Shell commands |
| Git workflow | High | Universal concepts |
| Tool-specific syntax | None | @import, @codebase, etc. |

### What's NOT Portable

| Feature | Tool | Why |
|---------|------|-----|
| `@import` | Claude Code | Proprietary syntax |
| `@codebase` | Cursor | Relies on Cursor's RAG |
| Cascade memory | Windsurf | Proprietary format |
| Thread persistence | OpenAI | API-specific |

## Memory/Context Sharing

### Current State: No Portability

| Tool | Memory Type | Format | Exportable |
|------|-------------|--------|------------|
| Claude Code | Explicit files | Markdown | Yes |
| Cursor | Codebase index | Proprietary | No |
| Windsurf | Cascade memory | Proprietary | No |
| Copilot | Session only | N/A | N/A |

### Potential Solutions

**1. Explicit Memory Files**

Claude Code pattern that could be universal:
```markdown
# .repository/memory.md

## Architecture Decisions
- Using event sourcing for audit trail
- PostgreSQL for primary database
- Redis for caching

## Team Conventions
- PR reviews required before merge
- Use conventional commits
- Deploy on Friday is forbidden
```

**2. Structured Memory Format**

```yaml
# .repository/memory.yaml
decisions:
  - date: 2026-01-15
    topic: Database choice
    decision: PostgreSQL
    rationale: ACID compliance, JSON support

context:
  team_size: 5
  deployment: Kubernetes
  ci_cd: GitHub Actions
```

**3. Memory Sync Service (Hypothetical - not implemented)**

Hypothetical service that syncs context across tools (doesn't exist yet).

## MCP as Interop Layer

MCP provides tool/resource portability:

### MCP Adoption

| Tool | Status |
|------|--------|
| Claude Code | Full Native |
| Cursor | Full Native |
| Windsurf | Full Native |
| Zed | Full Native |
| Amazon Q | Native |
| OpenAI | Native |
| Google | Native |
| Copilot | Limited |

### Shared MCP Configuration

```yaml
# .repository/mcp/servers.yaml
servers:
  filesystem:
    command: npx
    args: [-y, "@modelcontextprotocol/server-filesystem", "."]

  postgres:
    command: npx
    args: [-y, "@modelcontextprotocol/server-postgres"]
    env:
      DATABASE_URL: ${DATABASE_URL}
```

Generated to tool-specific formats:
- `.claude/settings.json`
- Cursor MCP settings
- Windsurf MCP config
- Zed settings.json

## Skill/Plugin Portability

### Current State: No Portability

| Tool | Skill Format | Location |
|------|--------------|----------|
| Claude Code | Markdown | `.claude/skills/` |
| Cursor | VS Code extensions | Extension store |
| Copilot | Extensions (beta) | GitHub Marketplace |
| Continue | JSON commands | `config.json` |

### Potential Abstraction (Proposed Format - Not Implemented)

```yaml
# .repository/skills/deploy.yaml
name: deploy
description: Deploy application to environment

steps:
  - run: npm test
    description: Run tests
  - run: npm run build
    description: Build production
  - run: ./scripts/deploy.sh
    description: Deploy to target

permissions:
  - bash: npm test
  - bash: npm run build
  - bash: ./scripts/deploy.sh
```

Could generate:
- Claude Code skill (`.claude/skills/deploy.md`)
- Continue command (in `config.json`)
- Custom scripts for others

## Behavioral Drift Mitigation

### The Problem

Same rules, different AI behavior across tools.

### Mitigation Strategies

**1. Explicit, Unambiguous Rules**

```markdown
# Bad - ambiguous
Use good variable names.

# Good - explicit
Variables MUST be camelCase.
Function names MUST start with a verb.
Boolean variables MUST start with is/has/can.
```

**2. Concrete Examples**

```markdown
## Variable Naming

Good:
- `isEnabled`
- `hasPermission`
- `calculateTotal()`

Bad:
- `flag`
- `check`
- `doStuff()`
```

**3. Validation Hooks**

Pre-commit hooks that enforce rules regardless of which AI generated the code.

**4. Regular Audits**

Periodic comparison of AI outputs across tools using the same rules.

## Implementation Priority for repo-manager

### Phase 1: Rules Sync (High Value)
- Read from `.repository/rules/`
- Generate AGENTS.md, CLAUDE.md, .cursorrules, etc.
- Watch for changes, re-sync

### Phase 2: MCP Config Sync (High Value)
- Read from `.repository/mcp/`
- Generate tool-specific MCP configs
- Validate server availability

### Phase 3: Memory Format (Medium Value)
- Define `.repository/memory.yaml` format
- Generate Claude Code memory.md
- (Others as they support explicit memory)

### Phase 4: Skills Abstraction (Lower Value)
- Define `.repository/skills/` format
- Generate Claude Code skills
- (Limited value due to tool diversity)

## Summary

| Area | Interoperability | Best Strategy |
|------|------------------|---------------|
| Rules | High | AGENTS.md + sync to tool formats |
| MCP | High | Central config + tool-specific generation |
| Memory | Low | Wait for standards / manual export |
| Skills | Low | Tool-specific for now |

---

*Last updated: 2026-01-23*
*Status: Complete*
