# Cursor IDE (Anysphere)

AI-first VS Code fork with integrated agentic capabilities.

## Overview

| Attribute | Value |
|-----------|-------|
| **Company** | Anysphere Inc. |
| **Founded** | 2022 |
| **Base** | VS Code fork (Electron) |
| **Models** | Claude 3.5 Sonnet (default), GPT-4, Cursor-small |
| **MCP Support** | Full Native |
| **AGENTS.md** | Native |

## Configuration Files

### .cursorrules (Legacy) / .cursor/rules (Current)

**Locations**:
- `.cursorrules` - Repository root (legacy)
- `.cursor/rules` - Current location
- `~/.cursor/rules` - Global rules

**Format**: Plain text or Markdown

```markdown
# .cursorrules Example

You are an expert TypeScript developer working on a Next.js 14 application.

## Code Style
- Always use TypeScript strict mode
- Prefer server components unless client interactivity is needed
- Use Tailwind CSS for styling

## Patterns
- Use the app router exclusively
- Implement error boundaries for all pages
- Use Zod for runtime validation

## Don't
- Never use `any` type
- Don't create files in the pages/ directory
- Avoid inline styles
```

### .cursor/ Directory

```
.cursor/
├── rules                      # Project rules
├── settings.json              # Cursor-specific settings
├── mcp.json                   # MCP server configuration
└── prompts/                   # Custom prompt templates
```

### MCP Configuration

```json
// .cursor/mcp.json or via Cursor settings
{
  "mcpServers": {
    "context7": {
      "command": "npx",
      "args": ["-y", "@context7/mcp-server"]
    }
  }
}
```

**Note**: Cursor has a 40 tool limit for MCP and supports one-click server installation.

## Capabilities

| Capability | Support | Notes |
|------------|---------|-------|
| Multi-file editing | Full | Composer mode |
| Terminal access | Partial | IDE terminal integration |
| Autonomous coding | Full | Agent mode |
| File creation | Full | Via Composer |
| Git operations | Partial | UI integration |
| Web browsing | Yes | @web for search |
| MCP | Full | Native support (40 tool limit) |

## Key Features

### Composer Mode

Multi-file editing with visual diff preview; accept/reject per-file.

### @-Mentions System

Context references: `@file`, `@folder`, `@codebase`, `@web`, `@docs`.

### RAG/Embeddings

Automatic codebase indexing via vector embeddings for semantic search.

## Context Management

- **Codebase Indexing**: Automatic embedding of repository
- **@-mentions**: Reference files, folders, docs, web content
- **Composer Context**: Multi-file editing context
- **Chat History**: Persistent per project

## Memory/Persistence

| Type | Persistence | Format |
|------|-------------|--------|
| Codebase Index | Project | Proprietary embeddings |
| Chat History | Project | Internal storage |
| Explicit Memory | Limited | Conversation-based |
| RAG | Project | Vector embeddings |

## Pricing (2026)

| Tier | Price | Features |
|------|-------|----------|
| Free | $0 | 2,000 completions/month, limited chat |
| Pro | $20/month | Unlimited, all models |
| Business | $40/user/month | Admin, SSO, audit |

## Configuration Discovery

```
1. IDE starts
2. Open workspace/folder
3. Scan for .cursorrules (legacy) or .cursor/rules
4. Load user rules (~/.cursor/rules)
5. Merge: project > user > defaults
6. Index codebase for RAG
7. Initialize AI with rules context
```

## Unique Differentiators

1. **Composer**: Multi-file planned editing interface
2. **@-mentions**: Rich context system
3. **Inline Editing**: Cmd+K for quick inline changes
4. **Model Flexibility**: Easy model switching
5. **Tab Completion**: Predictive multi-line suggestions
6. **One-click MCP**: Easy MCP server installation

## Limitations

- VS Code ecosystem dependency
- Codebase indexing can be slow for large repos
- Memory tied to Cursor installation (not exportable)
- 40 tool limit for MCP

## Quick Reference

```
./.cursorrules                 # Project rules (legacy)
./.cursor/
├── rules                      # Project rules (current)
├── settings.json              # Cursor settings
└── mcp.json                   # MCP configuration
~/.cursor/
└── rules                      # Global rules
```

## Sources

- [Cursor Documentation](https://docs.cursor.com)

---

*Last updated: 2026-01-23*
*Status: Complete*
