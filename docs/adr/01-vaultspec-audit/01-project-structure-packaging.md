# VaultSpec Audit: Project Structure and Python Packaging

**Date:** 2026-02-19
**Auditor:** vaultspec-structure (Opus agent)
**Source:** `Y:\code\task-worktrees\main\`

---

## 1. Build System

- **Build backend:** setuptools (>=61.0) via `pyproject.toml`
- **No `setup.py` or `setup.cfg`** - pure pyproject.toml
- **Package name:** `vaultspec`
- **Version:** `0.1.0`
- **Requires Python:** `>=3.13`
- **License:** MIT

### CRITICAL: Package Discovery is Misconfigured

```toml
[tool.setuptools.packages.find]
where = [".vaultspec/lib/src"]
```

`vaultspec.egg-info/top_level.txt` is **EMPTY**. The package installs nothing importable. Source modules exist under `.vaultspec/lib/src/` but have no top-level `vaultspec` Python package wrapping them.

**No `[project.scripts]` or `[project.gui-scripts]`** defined. CLI is invoked via `python .vaultspec/lib/scripts/cli.py`.

## 2. Dependencies

### Core Dependencies
| Dependency | Version | Purpose |
|---|---|---|
| `a2a-sdk>=0.3.22` | Google's Agent-to-Agent protocol SDK |
| `agent-client-protocol>=0.8.1` | ACP protocol client |
| `claude-agent-sdk>=0.1.30` | Claude agent spawning |
| `httpx>=0.27.0` | Async HTTP client |
| `pydantic>=2.0.0` | Data validation/models |
| `uvicorn>=0.23.0` | ASGI server for A2A |
| `starlette>=0.27.0` | ASGI framework for A2A |
| `sse-starlette>=1.0.0` | Server-Sent Events for A2A |
| `mcp>=1.20.0` | Model Context Protocol SDK |
| `PyYAML>=6.0` | YAML parsing for frontmatter |

### Optional: RAG Dependencies
`torch>=2.5.0`, `sentence-transformers>=3.0.0`, `lancedb>=0.15.0`, `einops>=0.7.0`

### Stale: requirements.txt
Lists `mcp>=0.1.0` vs pyproject.toml's `mcp>=1.20.0`. Missing `claude-agent-sdk` and `sse-starlette`. The file is outdated.

## 3. MCP Server Declaration

```json
{
  "mcpServers": {
    "vs-subagent-mcp": {
      "command": "python",
      "args": [".vaultspec/lib/scripts/subagent.py", "serve", "--root", "."],
      "env": {}
    }
  }
}
```

Single MCP server `vs-subagent-mcp` running the subagent script in serve mode.

## 4. Module Structure

All source under `.vaultspec/lib/src/` (10 modules):

| Module | Purpose |
|---|---|
| `core` | Config via `VaultSpecConfig` dataclass, env var resolution |
| `graph` | Vault document graph analysis, hotspots, orphans |
| `hooks` | Event-driven hooks system (YAML-defined) |
| `metrics` | Vault metrics collection |
| `orchestration` | Sub-agent dispatch, task orchestration |
| `protocol` | Protocol abstraction layer |
| `protocol/a2a` | A2A protocol with Claude/Gemini executors |
| `protocol/acp` | ACP protocol bridge |
| `protocol/providers` | Provider abstraction with tier-based model resolution |
| `rag` | RAG pipeline: embedding, indexing, semantic search |
| `subagent_server` | MCP server implementation |
| `vault` | Document parsing, frontmatter, wiki-links, templates |
| `verification` | Document verification, integrity checks |

### Path Resolution
`_paths.py` computes `ROOT_DIR` as 3 levels up from itself:
```
scripts -> lib -> .vaultspec -> root
```
Adds `.vaultspec/lib/src` to `sys.path` for bare module imports.

## 5. .vaultspec/ Directory Structure

```
.vaultspec/
  constitution.md          -- Immutable governance principles
  agents/                  -- 9 agent persona definitions (.md with YAML frontmatter)
  hooks/                   -- Event-driven hook definitions (YAML)
  lib/
    scripts/               -- CLI entry points (cli.py, subagent.py, docs.py)
    src/                   -- Library source (10 modules)
    tests/                 -- Test suite
  logs/                    -- Runtime log files
  rules/                   -- Behavioral rules (2 builtin + .gitignore)
  skills/                  -- 14 skill definitions
  system/                  -- System prompt fragments (base.md, framework.md, etc.)
  templates/               -- Document templates (ADR, PLAN, RESEARCH, etc.)
```

## 6. .vault/ Directory (Knowledge Base)

Obsidian-compatible markdown vault with wiki-links and YAML frontmatter:
```
.vault/
  .obsidian/              -- Obsidian config
  adr/                    -- Architecture Decision Records
  audit/                  -- Audit documents
  exec/                   -- Execution records
  plan/                   -- Plan documents
  reference/              -- Reference documents
  research/               -- Research documents
```

## 7. Tool Output Directories

### .claude/
```
.claude/
  CLAUDE.md               -- AUTO-GENERATED (system prompt + rules)
  settings.local.json     -- Permission allowlist (~229 entries)
  agents/                 -- 9 agent definitions (synced from .vaultspec/agents/)
  rules/                  -- 3 rules (builtins)
  skills/                 -- 12 skill directories (SKILL.md each)
```

### .gemini/
```
.gemini/
  GEMINI.md               -- AUTO-GENERATED config
  SYSTEM.md               -- AUTO-GENERATED assembled system prompt
  settings.json           -- Enables checkpointing, agents
  agents/                 -- 9 agent definitions (synced)
  rules/                  -- 2 rules (builtins)
  skills/                 -- 12 skill directories (synced)
```

### .agent/
```
.agent/
  rules/                  -- 3 rules (builtins)
  skills/                 -- 12 skill directories (synced)
```

## 8. Key Architectural Observations

1. **Multi-provider sync engine** is VaultSpec's core: takes canonical `.vaultspec/` definitions, transforms and distributes to `.claude/`, `.gemini/`, `.agent/`
2. **Not an installable package** - despite pyproject.toml, produces nothing importable
3. **sys.path manipulation** for imports - no proper package namespace
4. **Config via environment** only (`VAULTSPEC_*` env vars)
5. **uv.lock present** - uses uv as package manager, pinned to Python >=3.13
