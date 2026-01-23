# Windsurf (Codeium)

AI-native IDE with Cascade agentic system and flow-based coding.

## Overview

| Attribute | Value |
|-----------|-------|
| **Company** | Codeium (formerly Exafunction) |
| **Founded** | 2021 |
| **Base** | VS Code fork (Electron) |
| **Core Technology** | Proprietary "Cascade" AI engine |
| **MCP Support** | Full Native |
| **AGENTS.md** | Supported |

## Cascade Architecture

Multi-model orchestration system:
- Fast autocomplete model
- Reasoning model
- Code generation model

Note: Vendor claims 60% latency improvement (unverified).

## Configuration Files

### .windsurfrules / .windsurf/rules/

**Locations**:
- `.windsurfrules` - Repository root
- `.windsurf/rules/` - Modular rules directory

**Format**: Markdown

```markdown
# Project Rules

## Language
This is a TypeScript monorepo using pnpm workspaces.

## Style
- Use functional components with hooks
- Prefer Zod for validation
- No `any` types

## Commands
- Build: `pnpm build`
- Test: `pnpm test`
- Lint: `pnpm lint`

## Architecture
- Service layer in `/services`
- API routes in `/api`
- Shared types in `/types`
```

### .windsurf/ Directory

```
.windsurf/
├── rules/                     # Modular rule files
├── settings.json              # IDE settings
└── cascade.json               # Cascade behavior config
```

### Additional Configuration

- `.codeiumignore` - Files to ignore (project-level)
- `~/.codeium/.codeiumignore` - Global ignore rules
- **Rulebooks** - Invokable via slash commands

## Capabilities

| Capability | Support | Notes |
|------------|---------|-------|
| Multi-file editing | Full | Cascade-coordinated |
| Terminal access | Full | Integrated terminal control |
| Autonomous coding | Full | "Flows" for multi-step tasks |
| File creation | Full | Via Cascade |
| Git operations | Full | Built-in git UI + AI commits |
| Web browsing | Partial | Research mode |
| MCP | Full | Native client support |

## Key Features

### Flows

Visual workflow representation for agentic tasks:
- Contextual understanding of task sequences
- Multi-step planning and execution
- Visual progress tracking

### Cascade Memory

Automatic memory formation from interactions:
- Learns coding patterns
- Remembers architectural decisions
- Tracks conversation context across sessions
- Project-level persistence
- **Format**: Proprietary (not exportable)

### Supercomplete

Context-aware, multi-line completions going beyond single-line suggestions.

## Memory/Persistence

| Type | Persistence | Format | Exportable |
|------|-------------|--------|------------|
| Cascade Memory | Automatic | Proprietary | No |
| Project Context | Project | Proprietary | No |
| User Preferences | User | Internal | No |

## Pricing (2026)

| Tier | Price | Features |
|------|-------|----------|
| Free | $0 | Limited completions, basic Cascade |
| Pro | $15/month | Unlimited completions, full Cascade |
| Teams | $19/user/month | Admin controls, shared settings |
| Enterprise | Custom | SSO, audit logs, SLA |

## Configuration Discovery

```
1. IDE starts
2. Open workspace/folder
3. Scan for .windsurfrules in root
4. Scan .windsurf/ directory
5. Load user settings (~/.windsurf/)
6. Merge: project > user > defaults
7. Initialize Cascade with merged config
```

## Unique Differentiators

1. **Cascade Engine**: Multi-model orchestration for complex tasks
2. **Flows**: Visual workflow representation
3. **Speed**: Optimized for low-latency interactions
4. **Automatic Memory**: Context retention without manual management
5. **Supercomplete**: Beyond single-line suggestions

## Limitations

- Proprietary Cascade system limits interoperability
- Memory format not exportable
- VS Code fork dependency
- Configuration less portable than competitors

## Quick Reference

```
./.windsurfrules               # Project rules (root)
./.windsurf/
├── rules/                     # Modular rule files
├── settings.json              # IDE settings
└── cascade.json               # Cascade config
./.codeiumignore               # Files to ignore
~/.codeium/
└── .codeiumignore             # Global ignore rules
```

## Sources

- [Windsurf Documentation](https://codeium.com/windsurf)

---

*Last updated: 2026-01-23*
*Status: Complete*
