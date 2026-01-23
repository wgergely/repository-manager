# OpenAI Codex

OpenAI's open-source CLI coding agent - the originator of AGENTS.md.

## Overview

| Attribute | Value |
|-----------|-------|
| **Company** | OpenAI |
| **Type** | CLI (Terminal-based) + IDE Extension |
| **Language** | Rust (open source) |
| **Models** | GPT-5-Codex, GPT-5.2-Codex |
| **MCP Support** | Native |
| **AGENTS.md** | Native (originator) |

## Significance

Codex is notable for:
- **AGENTS.md originator** - The standard now adopted by Google, Cursor, GitHub, etc.
- **Open source** - Available on GitHub (github.com/openai/codex)
- **MCP adoption** - Integrated MCP in March 2025
- **Rust implementation** - High performance CLI

## Configuration Files

### config.toml (Main Configuration)

**Location**: `~/.codex/config.toml`

Shared between CLI and IDE extension.

```toml
# ~/.codex/config.toml

# Model selection
model = "gpt-5.2-codex"

# Feature flags
[features]
shell_tool = true
web_search_request = false
unified_exec = false
shell_snapshot = false
apply_patch_freeform = false
exec_policy = true
remote_compaction = true

# Project instruction settings
project_doc_max_bytes = 32768          # 32 KiB default
project_doc_fallback_filenames = ["TEAM_GUIDE.md", ".agents.md"]

# Project root detection
project_root_markers = [".git", ".hg", ".sl"]

# MCP servers
[mcp_servers.filesystem]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "."]
```

### AGENTS.md (Project Instructions)

Primary rules file - the standard Codex pioneered.

**Discovery Order** (hierarchical):

1. **Global scope** (`~/.codex/`):
   - `AGENTS.override.md` (if exists)
   - `AGENTS.md` (fallback)
   - Only first non-empty file used

2. **Project scope** (git root → cwd):
   - Walk from project root to current directory
   - At each level: `AGENTS.override.md` → `AGENTS.md` → fallbacks
   - Files concatenated, later overrides earlier

```markdown
# AGENTS.md Example

## Project Overview
This is a TypeScript monorepo using pnpm.

## Commands
- Build: `pnpm build`
- Test: `pnpm test`
- Lint: `pnpm lint`

## Code Style
- TypeScript strict mode
- Functional components
- No `any` types

## Boundaries
- Never modify package-lock.json
- Don't commit secrets
```

### .codex/ Directory

```
.codex/
├── skills/                    # Project-level skills
│   └── skill-name/
│       ├── SKILL.md          # Required
│       └── scripts/          # Optional
└── (future config)
```

### Global Skills

```
~/.codex/
├── config.toml               # Main configuration
├── AGENTS.md                 # Global instructions
├── AGENTS.override.md        # Override instructions
└── skills/                   # User-wide skills
    └── skill-name/
        └── SKILL.md
```

## Skills System

Follows the **Open Agent Skills** specification.

### Skill Structure

```
skill-name/
├── SKILL.md                  # Required - instructions
├── scripts/                  # Optional - supporting scripts
└── resources/                # Optional - reference files
```

### SKILL.md Format

```markdown
---
name: Deploy
description: Deploy application to environment
---

## Instructions

1. Run tests first
2. Build production bundle
3. Deploy to target

## Scripts

Use `./scripts/deploy.sh` for deployment.
```

## AGENTS.md Behavior

### File Merging

- Files concatenated from root down
- Later files override earlier (closer to cwd wins)
- Empty files skipped
- Stops at `project_doc_max_bytes` limit (32 KiB default)

### Override Pattern

`AGENTS.override.md` takes precedence over `AGENTS.md` at any level. At each directory, the override file is checked first; if absent, the standard file is used. Files are concatenated from root to cwd, with later content taking precedence.

## MCP Configuration

```toml
# ~/.codex/config.toml

[mcp_servers.filesystem]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "."]

[mcp_servers.postgres]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-postgres"]
env = { DATABASE_URL = "postgresql://localhost/mydb" }
```

## Capabilities

| Capability | Support | Notes |
|------------|---------|-------|
| Multi-file editing | Full | Agentic mode |
| Terminal access | Full | Configurable |
| Autonomous coding | Full | Agent-first |
| File creation | Full | Native |
| Git operations | Full | Integrated |
| Web search | Optional | Feature flag |
| MCP | Native | Full support |

## IDE Extension

Codex also provides IDE extensions that share `config.toml`:

- Access via gear icon → Codex Settings → Open config.toml
- Same configuration as CLI
- Integrated into editor workflow

## Comparison with Claude Code

| Aspect | Codex | Claude Code |
|--------|-------|-------------|
| Config file | `config.toml` | `settings.json` |
| Rules file | `AGENTS.md` | `CLAUDE.md` |
| Override pattern | `*.override.md` | `*.local.md` |
| Config location | `~/.codex/` | `~/.claude/` |
| Skills location | `.codex/skills/` | `.claude/skills/` |
| Hierarchical | Yes (concatenation) | Yes (merge) |
| Language | Rust | TypeScript |

## Unique Features

1. **AGENTS.md originator** - The standard others adopted
2. **Override files** - Explicit `.override.md` pattern
3. **Open source** - Full source on GitHub
4. **TOML config** - Not JSON
5. **Fallback filenames** - Configurable alternatives

## Quick Reference

```
~/.codex/
├── config.toml               # Main configuration
├── AGENTS.md                 # Global instructions
├── AGENTS.override.md        # Personal overrides
└── skills/                   # Global skills
    └── skill-name/
        └── SKILL.md

./AGENTS.md                   # Project instructions
./AGENTS.override.md          # Project overrides
./.codex/
└── skills/                   # Project skills
    └── skill-name/
        └── SKILL.md
```

## Sources

- [OpenAI Codex CLI](https://developers.openai.com/codex/cli/)
- [AGENTS.md Guide](https://developers.openai.com/codex/guides/agents-md)
- [Configuration Reference](https://developers.openai.com/codex/config-reference/)
- [GitHub Repository](https://github.com/openai/codex)

---

*Last updated: 2026-01-23*
*Status: Complete*
