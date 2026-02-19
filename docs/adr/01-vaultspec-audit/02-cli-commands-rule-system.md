# VaultSpec Audit: CLI Commands and Rule System

**Date:** 2026-02-19
**Auditor:** vaultspec-cli (Opus agent)
**Source:** `Y:\code\task-worktrees\main\.vaultspec\lib\`

---

## 1. CLI Entry Points

Two CLI scripts in `.vaultspec/lib/scripts/`:

### A) cli.py - Main Resource Manager
```
python .vaultspec/lib/scripts/cli.py <resource> <command> [options]
```
Global flags: `--root <path>`, `--verbose/-v`, `--debug`, `--version/-V`

### B) subagent.py - Sub-agent Launcher
Commands: `run`, `serve`, `a2a-serve`, `list`

## 2. Full Command Reference

| Resource | Commands | Purpose |
|---|---|---|
| `rules` | `list`, `add`, `show`, `edit`, `remove`, `rename`, `sync` | CRUD + sync of markdown rules |
| `agents` | `list`, `add`, `show`, `edit`, `remove`, `rename`, `sync`, `set-tier` | CRUD + sync of agent definitions |
| `skills` | `list`, `add`, `show`, `edit`, `remove`, `rename`, `sync` | CRUD + sync of skill files |
| `constitution` | `show`, `init` | View/create governance principles |
| `config` | `show`, `sync` | Manage tool configs (CLAUDE.md, GEMINI.md, AGENTS.md) |
| `system` | `show`, `sync` | Manage assembled system prompts |
| `sync-all` | (single) | Syncs everything in one shot |
| `test` | categories | Pytest runner |
| `doctor` | (single) | Health check (Python, CUDA, deps, .lance) |
| `init` | `--force` | Scaffolds .vaultspec/ and .vault/ |
| `readiness` | `--json` | 6-dimension governance assessment |
| `hooks` | `list`, `run <event>` | Event-driven hooks |

## 3. Init Command Step-by-Step

1. Read `framework_dir` from config (default `.vaultspec`)
2. Check if `.vaultspec/` exists (error unless `--force`)
3. Create subdirs: `agents/`, `rules/`, `skills/`, `templates/`, `system/`
4. Create `.vault/` subdirs: `adr/`, `audit/`, `exec/`, `plan/`, `reference/`, `research/`
5. Create stub files: `system/framework.md`, `system/project.md`
6. Print summary, tell user to run `sync-all`

## 4. Rule System

### Storage
Markdown files in `.vaultspec/rules/` with YAML frontmatter:
```markdown
---
name: vaultspec-skills
---
# Rule content here
```

### Types
- `*.builtin.md` - Built-in framework rules
- `*.md` - Custom user rules

### Collection
`collect_rules()` globs `*.md` from `RULES_SRC_DIR`, parses frontmatter + body.

### Transformation
`transform_rule()` rewrites frontmatter per target:
- Adds `name` (stripped of `.md`)
- Sets `trigger: always_on`
- Body passed through unchanged

## 5. Sync Destinations

| Tool | Rules Dir | Agents Dir | Skills Dir | Config File | System File |
|---|---|---|---|---|---|
| `claude` | `.claude/rules/` | `.claude/agents/` | `.claude/skills/` | `.claude/CLAUDE.md` | (none) |
| `gemini` | `.gemini/rules/` | `.gemini/agents/` | `.gemini/skills/` | `.gemini/GEMINI.md` | `.gemini/SYSTEM.md` |
| `agents` | (none) | (none) | (none) | `AGENTS.md` (root) | (none) |
| `agent` | `.agent/rules/` | (none) | `.agent/skills/` | (none) | (none) |

Sync flags: `--prune` (delete files not in source), `--dry-run`.

Skills synced to `<dest>/skills/<skill-name>/SKILL.md` (directory-based).
Protected skills (`fd`, `rg`, `sg`, `sd`) never pruned.

## 6. MCP Server Implementation

**File:** `.vaultspec/lib/src/subagent_server/server.py`
**Built on:** `mcp.server.fastmcp.FastMCP`
**Server name:** `vs-subagent-mcp`

### 5 MCP Tools
1. **`list_agents`** - Returns JSON list of agents (name/tier/description)
2. **`dispatch_agent`** - Dispatches sub-agent async. Params: agent, task, model, mode, max_turns, budget, effort, output_format
3. **`get_task_status`** - Returns task status/result/error/lock info
4. **`cancel_task`** - Cancels running task via ACP + asyncio
5. **`get_locks`** - Lists active advisory file locks

### MCP Resources
Dynamic `agents://{name}` resources for each agent file. Auto-registered on startup, polled for changes (default 5s).

### Server Startup
`mcp.json` launches: `python .vaultspec/lib/scripts/subagent.py serve --root .`
`initialize_server()` requires `root_dir` (from arg or `VAULTSPEC_MCP_ROOT_DIR` env var).

## 7. Root Discovery

**No git-based or marker-file discovery.** Pure filesystem-relative:
```python
_SCRIPTS_DIR = Path(__file__).resolve().parent    # .vaultspec/lib/scripts/
_LIB_DIR = _SCRIPTS_DIR.parent                    # .vaultspec/lib/
ROOT_DIR = _LIB_DIR.parent.parent                 # <project_root>
```

**Override:** `--root <path>` on both CLIs, `VAULTSPEC_MCP_ROOT_DIR` env var for MCP server.

## 8. Configuration System

**File:** `.vaultspec/lib/src/core/config.py`

`VaultSpecConfig` dataclass. Resolution order:
1. Explicit overrides dict
2. `VAULTSPEC_*` environment variables
3. Dataclass defaults

### Key Environment Variables
| Variable | Default | Purpose |
|---|---|---|
| `VAULTSPEC_ROOT_DIR` | (computed) | Workspace root |
| `VAULTSPEC_FRAMEWORK_DIR` | `.vaultspec` | Framework directory name |
| `VAULTSPEC_DOCS_DIR` | `.vault` | Knowledge base directory |
| `VAULTSPEC_CLAUDE_DIR` | `.claude` | Claude output directory |
| `VAULTSPEC_GEMINI_DIR` | `.gemini` | Gemini output directory |
| `VAULTSPEC_MCP_ROOT_DIR` | (from --root) | MCP server root |
| `VAULTSPEC_MCP_PORT` | 10010 | MCP server port |
| `VAULTSPEC_EDITOR` | `zed -w` | Editor command |
| `VAULTSPEC_AGENT_MODE` | (default) | read-write/read-only |

No config file - all env-var or code-default based. Global singleton via `get_config()`.

## 9. Tool-Specific Config Generation

### Claude (.claude/CLAUDE.md)
- Header: `<!-- AUTO-GENERATED by cli.py config sync. -->`
- YAML frontmatter with `system_framework` containing full framework prompt
- Body: project config + constitution + rule references (`@rules/filename.md`)
- Safety guard: skipped if file exists without AUTO-GENERATED header (needs `--force`)

### Gemini (.gemini/GEMINI.md + .gemini/SYSTEM.md)
- GEMINI.md: Same structure as CLAUDE.md
- SYSTEM.md: Assembled from `.vaultspec/system/` parts (ordered by `order` frontmatter)

### AGENTS.md (root-level)
- Uses agents.md standard format with `alwaysApply: true`
- XML tags stripped, converted to `## Heading` sections

### Agent (.agent/rules/)
- Gets synthetic `vaultspec-system.builtin.md` assembled from shared system parts
- Just rules/ and skills/ - no config or system file

## 10. Key Observations for Extension Packaging

1. **No standard entry points** - CLI invoked via `python .vaultspec/lib/scripts/cli.py`
2. **Path discovery is filesystem-relative**, not git-based
3. **Sync model is source-of-truth based**: `.vaultspec/` canonical, tool dirs are derived
4. **Config is env-var only** - no config file
5. **`--root` flag exists** for overriding workspace root
6. **Tool directory names configurable** via env vars
