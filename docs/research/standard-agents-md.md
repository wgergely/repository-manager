# AGENTS.md - Universal AI Coding Agent Configuration Standard

The first successful cross-vendor standardization of AI coding agent configuration.

## Overview

| Attribute | Value |
|-----------|-------|
| **Launch** | July 2025 |
| **Backers** | Google, OpenAI, Factory, Sourcegraph, Cursor |
| **Governance** | Agentic AI Foundation (Linux Foundation) |
| **Adoption** | 20,000+ repositories on GitHub |
| **Website** | https://agents.md/ |
| **Spec Repo** | https://github.com/agentsmd/agents.md |

## What is AGENTS.md?

AGENTS.md is a simple, open format for guiding AI coding agents. It provides a universal way to specify project-level instructions that any compliant AI tool can read.

**Analogy**: If MCP is "USB-C for AI tools" (universal connector), AGENTS.md is "README for AI agents" (universal instructions).

## Tool Support

| Tool | Support Level | Notes |
|------|---------------|-------|
| OpenAI Codex | Native | Primary rules source |
| Google Jules | Native | Primary rules source |
| Cursor | Native | Reads alongside .cursorrules |
| GitHub Copilot | Native | Reads alongside copilot-instructions.md |
| Aider | Native | Primary rules source |
| RooCode | Native | Primary rules source |
| Zed | Native | Primary rules source |
| Factory AI | Native | Primary rules source |
| Claude Code | Compatible | Reads as context (not native format) |
| Gemini CLI | Native | Primary rules source |

## File Location

```
./AGENTS.md                  # Repository root (primary)
./subdirectory/AGENTS.md     # Nested for monorepos
~/AGENTS.md                  # User-level defaults (some tools)
```

## Structure Best Practices

Successful AGENTS.md files cover six core areas:

### 1. Commands (Build & Test)

```markdown
## Build & Test
- `npm install` to install dependencies
- `npm test` to run tests
- `npm run lint` to check code style
```

### 2. Testing

```markdown
## Testing
- Tests live in `tests/` mirroring `src/` structure
- Use Jest for unit tests
- Run `npm test -- --coverage` for coverage report
```

### 3. Project Structure

```markdown
## Project Structure
- `src/` - Source code
- `tests/` - Test files (mirror src/ structure)
- `docs/` - Documentation
- `scripts/` - Build and utility scripts
```

### 4. Code Style

```markdown
## Code Style
- TypeScript strict mode
- Prefer functional patterns
- Use named exports
- No `any` types
```

### 5. Git Workflow

```markdown
## Git Workflow
- Branch from `main`
- Conventional commits (feat:, fix:, docs:)
- PRs require one approval
- Squash merge to main
```

### 6. Boundaries

```markdown
## Boundaries
- Never modify package-lock.json manually
- Don't change CI/CD configuration without asking
- Don't commit secrets or API keys
- Don't modify files in `vendor/`
```

## Complete Example

```markdown
# AGENTS.md

## Build & Test
- `npm install` to install dependencies
- `npm test` to run tests
- `npm run lint` to check code style

## Project Structure
- `src/` - Source code
- `tests/` - Test files (mirror src/ structure)
- `docs/` - Documentation

## Code Style
- TypeScript strict mode
- Prefer functional patterns
- Use named exports
- Follow Prettier defaults

## Git Workflow
- Branch from `main`
- Conventional commits (feat:, fix:, docs:)
- PRs require one approval

## Boundaries
- Never modify package-lock.json manually
- Don't change CI/CD configuration without asking
```

## Significance for repo-manager

AGENTS.md should be treated as a **first-class citizen**:

1. **Read**: Parse AGENTS.md to understand project rules
2. **Write**: Generate AGENTS.md from unified configuration
3. **Sync**: Keep AGENTS.md in sync with tool-specific configs
4. **Validate**: Check consistency between AGENTS.md and other rule files

### Integration Strategy

```
.agentic/rules/common.md  (source of truth)
        |
        v
  [repo-manager sync]
        |
   +----+----+----+
   v    v    v    v
AGENTS.md  CLAUDE.md  .cursorrules  copilot-instructions.md
```

## Comparison with Tool-Specific Formats

| Aspect | AGENTS.md | CLAUDE.md | .cursorrules |
|--------|-----------|-----------|--------------|
| Vendor backing | Multi-vendor | Anthropic | Anysphere |
| Governance | Linux Foundation | Anthropic | Cursor |
| Format | Markdown | Markdown | Markdown |
| Hierarchical | Limited | Yes | No |
| Imports | No | Yes (@import) | No |
| Memory | No | Via .claude/memory.md | Via Cursor index |

## Ecosystem Resources

- **Official tooling**: https://github.com/agentsmd/agents.md
- **Template collection**: https://github.com/sammcj/agentic-coding
- **GitHub lessons learned**: https://github.blog/ai-and-ml/github-copilot/how-to-write-a-great-agents-md-lessons-from-over-2500-repositories/

## Sources

- [AGENTS.md Official Site](https://agents.md/)
- [OpenAI Codex AGENTS.md Guide](https://developers.openai.com/codex/guides/agents-md)
- [GitHub Blog - AGENTS.md Lessons](https://github.blog/ai-and-ml/github-copilot/how-to-write-a-great-agents-md-lessons-from-over-2500-repositories/)

---

*Last updated: 2026-01-23*
*Status: Complete*
