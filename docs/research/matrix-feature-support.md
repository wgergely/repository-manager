# Feature Support Matrix

Comprehensive feature comparison across major agentic coding tools.

## Rules/Instructions Support

| Tool | Format | File Name(s) | Hierarchical | AGENTS.md |
|------|--------|--------------|--------------|-----------|
| **Claude Code** | Markdown | `CLAUDE.md`, `.claude/rules/` | Yes | Compatible |
| **Claude Desktop** | JSON | `claude_desktop_config.json` | No | N/A |
| **Cursor** | Markdown | `.cursorrules`, `.cursor/rules` | Limited | Native |
| **Windsurf** | Markdown | `.windsurfrules`, `.windsurf/rules/` | Limited | Supported |
| **Antigravity** | Markdown | `.agent/rules/`, `SKILL.md` | Yes | Supported |
| **Copilot** | Markdown | `copilot-instructions.md` | Partial | Native |
| **Zed** | JSON | `.zed/settings.json` | Yes | Native |
| **Gemini** | JSON | `.gemini/settings.json` | Yes | Native |
| **Amazon Q** | JSON | `.amazonq/` | Unknown | Unknown |
| **Continue** | JSON | `config.json` | Yes | Partial |
| **Aider** | YAML | `.aider.conf.yml` | Yes | Native |

## MCP Support

| Tool | Support Level | Configuration | Notes |
|------|---------------|---------------|-------|
| **Claude Code** | Full Native | `.claude/settings.json` | First-class citizen |
| **Claude Desktop** | Full Native | `claude_desktop_config.json` | Desktop Extensions (.mcpb) |
| **Cursor** | Full Native | Cursor settings | 40 tool limit |
| **Windsurf** | Full Native | Windsurf settings | Native support |
| **Antigravity** | Native | Via Gemini integration | Native support |
| **Zed** | Full Native | `.zed/settings.json` | Native support |
| **Amazon Q** | Native | IDE settings | MCP in IDE |
| **OpenAI (Codex)** | Native | - | Adopted March 2025 |
| **Google (Gemini)** | Native | - | DeepMind adoption |
| **Copilot** | Limited | - | Experimental |
| **Continue** | Partial | Context providers | Similar concept |
| **Aider** | None | - | Not supported |

## Memory/Context Persistence

| Tool | Type | Format | Exportable |
|------|------|--------|------------|
| **Claude Code** | Explicit files | Markdown | Yes |
| **Cursor** | Codebase index | Proprietary | No |
| **Windsurf** | Cascade memory | Proprietary | No |
| **Copilot** | Session only | N/A | N/A |
| **Zed** | Editor state | Internal | No |
| **OpenAI** | Thread-based | API | Via API |
| **Continue** | Config-based | JSON | Partial |
| **Aider** | Git-based | Commits | Yes (git) |

## Multi-File Editing

| Tool | Mode | Preview | Undo |
|------|------|---------|------|
| **Claude Code** | Sequential + Parallel | Yes | Git-based |
| **Claude Desktop** | Via bundled CLI | Yes | Git-based |
| **Cursor** | Composer | Yes | IDE undo |
| **Windsurf** | Cascade | Yes | IDE undo |
| **Antigravity** | Agent Manager | Yes | IDE undo |
| **Copilot** | Workspace | Yes | IDE undo |
| **Zed** | Multi-buffer | Yes | IDE undo |
| **Aider** | Native | No | Git revert |

## Terminal/Shell Access

| Tool | Execution | Sandboxing | Timeout |
|------|-----------|------------|---------|
| **Claude Code** | Direct | Configurable | Yes |
| **Cursor** | IDE terminal | None | No |
| **Windsurf** | Cascade | Configurable | Yes |
| **Copilot** | Via Workspace | GitHub sandbox | Yes |
| **Aider** | Direct | None | No |
| **OpenAI** | Code Interpreter | Sandboxed | Yes |

## API/Programmatic Access

| Tool | API Type | SDK |
|------|----------|-----|
| **Claude Code** | CLI + MCP | TypeScript |
| **Cursor** | N/A | N/A |
| **Windsurf** | N/A | N/A |
| **Copilot** | GitHub API | REST |
| **Gemini** | REST/gRPC | Python, JS |
| **Amazon Q** | AWS SDK | Multi-lang |
| **OpenAI** | REST | Python, JS |
| **Aider** | CLI + Python | Python |

## Pricing Overview (2026)

| Tool | Free Tier | Pro | Business |
|------|-----------|-----|----------|
| **Cursor** | 2K completions | $20/mo | $40/user/mo |
| **Windsurf** | Limited | $15/mo | $19/user/mo |
| **Antigravity** | Preview (limited) | ~$20/mo | ~$40-60/user/mo |
| **Copilot** | - | $10/mo | $19/user/mo |
| **Zed** | Core only | $20/mo | - |
| **Aider** | Free (OSS) | - | - |
| **Continue** | Free (OSS) | - | - |

## Interoperability Summary

### High Portability
- Rules/instructions (Markdown is universal)
- Code style guidelines
- Build commands

### Low/No Portability
- Memory/context (all proprietary)
- Skills/plugins (format varies)
- Tool-specific syntax (@import, @codebase)

---

*Last updated: 2026-01-23*
*Status: Complete*
