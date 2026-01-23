# Claude Code (Anthropic)

Anthropic's official CLI and agentic coding tool for terminal-based AI-assisted development.

## Overview

| Attribute | Value |
|-----------|-------|
| **Company** | Anthropic |
| **Type** | CLI (Terminal-based) |
| **Model** | Claude (exclusive) |
| **MCP Support** | Full Native |
| **AGENTS.md** | Compatible (reads as context) |

## Configuration Files

### CLAUDE.md (Project Instructions)

Primary rules file using hierarchical Markdown.

**Locations** (in priority order):
1. `./CLAUDE.md` - Project root
2. `./subdirectory/CLAUDE.md` - Nested directories
3. `~/.claude/CLAUDE.md` - User-wide defaults

**Features**:
- Hierarchical: Files read from all directories in path
- Merged: Child overrides parent for conflicts
- Markdown-native: Standard Markdown with optional sections

```markdown
# CLAUDE.md Example

## Project Overview
This is a TypeScript monorepo using pnpm workspaces.

## Code Style
- Use functional components with hooks
- Prefer named exports over default exports
- Always use strict TypeScript

## Commands
- Build: `pnpm build`
- Test: `pnpm test`
- Lint: `pnpm lint`

## Architecture
[Description of project structure and patterns]
```

### .claude/ Directory

```
.claude/
├── settings.json          # Local settings + MCP servers
├── settings.local.json    # Personal settings (gitignored)
├── rules/                 # Modular rule files
│   └── *.md
├── skills/                # Custom skill definitions
│   └── *.md
└── memory.md              # Persistent memory/context
```

### settings.json

```json
{
  "permissions": {
    "allow": ["Bash", "Read", "Write", "Edit"],
    "deny": ["WebFetch"]
  },
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "."]
    },
    "postgres": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-postgres"],
      "env": {
        "DATABASE_URL": "postgresql://localhost/mydb"
      }
    }
  }
}
```

### Memory System

- **File**: `.claude/memory.md` or project-level
- **Format**: Markdown
- **Persistence**: Stored between sessions
- **Scope**: Global (`~/.claude/`) or project-specific

## Skills System

Skills are Markdown files with structured instructions.

**Location**: `.claude/skills/` or plugin directories

**Invocation**: `/skill-name` commands or automatic detection

```markdown
# commit

## Description
Create a well-formatted git commit with conventional commit messages.

## Instructions
1. Run `git status` to see changes
2. Run `git diff` to understand modifications
3. Create a commit message following conventional commits
4. Use co-author attribution

## Commit Format
<type>(<scope>): <description>

Types: feat, fix, docs, style, refactor, test, chore
```

## Capabilities

| Capability | Support | Notes |
|------------|---------|-------|
| Multi-file editing | Full | Sequential + parallel |
| Terminal access | Full | Direct shell with permissions |
| Autonomous coding | Full | Agentic mode |
| File creation | Full | Via Write tool |
| Git operations | Full | Native git integration |
| Web browsing | Yes | Via WebFetch tool |
| MCP | Full | Native client support |

## Context Management

- Hierarchical file reading (CLAUDE.md files in path)
- Explicit context via Read tool
- MCP-based context providers
- Conversation history within session
- No automatic codebase indexing (unlike Cursor)

## Unique Features

1. **Hierarchical Rules**: Only tool with full directory-tree rule inheritance
2. **Explicit Memory**: User-controlled memory files (not automatic)
3. **Skills System**: Markdown-based workflow definitions
4. **MCP First-Class**: Deepest MCP integration
5. **CLI Native**: Terminal-first, not IDE-bound

## Limitations

- No GUI (CLI only)
- No automatic codebase indexing
- Claude models only (no GPT-4, etc.)
- Memory requires manual management
- Skills format not portable to other tools

## Quick Reference

```
./CLAUDE.md                    # Project instructions
./CLAUDE.local.md              # Personal (gitignored)
./.claude/
├── settings.json              # Tool settings, MCP servers
├── settings.local.json        # Personal settings
├── rules/
│   └── *.md                   # Modular rules
├── skills/
│   └── *.md                   # Custom skills
└── memory.md                  # Project memory
~/.claude/
├── CLAUDE.md                  # User-wide instructions
├── settings.json              # Global settings
└── memory.md                  # Global memory
```

## Sources

- [Claude Code Documentation](https://docs.anthropic.com/en/docs/claude-code)

---

*Last updated: 2026-01-23*
*Status: Complete*
