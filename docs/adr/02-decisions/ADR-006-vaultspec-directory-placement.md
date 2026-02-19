# ADR-006: VaultSpec Directory Placement and Path Decoupling

**Status:** Directive (pending VaultSpec team implementation)
**Date:** 2026-02-19
**Context:** Embedding VaultSpec as an extension in both worktree and standard layout modes

---

## Context

The repository manager (Rust) will embed VaultSpec as an extension. The repo manager handles:
- Cloning VaultSpec source to `.repository/extensions/vaultspec/source/`
- Creating a managed Python venv at `.repository/extensions/vaultspec/.venv/`
- Installing VaultSpec's dependencies into that venv
- Invoking VaultSpec's CLI and MCP server with explicit path arguments

The repo manager operates in two layout modes:

**Standard mode** (single repo root):
```
my-project/
  .repository/extensions/vaultspec/     <- framework + venv (not git-tracked)
  .vaultspec/                           <- authored content (git-tracked)
  .claude/                              <- generated output
  .gemini/
  src/
```

**Container/worktree mode** (bare repo + worktrees):
```
my-project/                             <- container root (NOT a git working tree)
  .gt/                                  <- bare git repo
  .repository/extensions/vaultspec/     <- framework + venv
  .claude/                              <- generated output (shared)
  .gemini/                              <- generated output (shared)
  main/                                 <- git worktree
    .vaultspec/                         <- authored content (git-tracked)
    src/
  feature-branch/                       <- another worktree
    .vaultspec/                         <- same content, different branch
    src/
```

## The Problem

VaultSpec currently assumes **one root** for everything. `_paths.py` computes `ROOT_DIR` by walking up 2 levels from the script location, and all paths (content source, output destinations, framework code) are relative to that single root. The `--root` flag overrides it, but it's still one value controlling everything.

When embedded by the repo manager, VaultSpec needs to handle **three independent paths**:

| Path | What | Example (worktree mode) |
|---|---|---|
| **Framework path** | Python code, scripts, lib/ | `.repository/extensions/vaultspec/source/.vaultspec/lib/` |
| **Content path** | Rules, agents, skills, system prompts, constitution | `main/.vaultspec/` (inside the git worktree) |
| **Output root** | Where `.claude/`, `.gemini/`, `.agent/`, `AGENTS.md` get written | Container root (`my-project/`) |

Today these three are all derived from one `ROOT_DIR`. They need to be independently configurable.

## Required Changes

### 1. Three-path configuration support

Add two new environment variables (or CLI flags) so that content source and output destination can be specified independently:

| Variable | Purpose | Default (backwards-compat) |
|---|---|---|
| `VAULTSPEC_ROOT_DIR` | **Output root** - where `.claude/`, `.gemini/`, `.agent/` get written | CWD (existing) |
| `VAULTSPEC_CONTENT_DIR` | **Content source** - where `rules/`, `agents/`, `skills/`, `system/`, `constitution.md` are read from | `{ROOT_DIR}/{FRAMEWORK_DIR}` (existing behavior) |
| `VAULTSPEC_FRAMEWORK_DIR` | **Framework dir name** | `.vaultspec` (existing) |

The repo manager would invoke VaultSpec like:

```bash
# Standard mode - content and output at same root (backwards compatible)
VAULTSPEC_ROOT_DIR=/path/to/repo \
  /path/to/.venv/bin/python /path/to/source/.vaultspec/lib/scripts/cli.py sync-all

# Worktree mode - content in worktree, output at container root
VAULTSPEC_ROOT_DIR=/path/to/container \
VAULTSPEC_CONTENT_DIR=/path/to/container/main/.vaultspec \
  /path/to/.venv/bin/python /path/to/source/.vaultspec/lib/scripts/cli.py sync-all
```

The critical change is in `cli.py`'s `init_paths()` function. Today it derives all source dirs (RULES_SRC_DIR, AGENTS_SRC_DIR, etc.) from `root / framework_dir`. It needs to derive them from the **content path** instead, while keeping output destinations (TOOL_CONFIGS entries) relative to the **output root**.

### 2. Remove reliance on `_paths.py` structural path computation

The current `ROOT_DIR = _LIB_DIR.parent.parent` computation in `_paths.py` breaks when the framework is installed at a different location from the content. The Python scripts will live in `.repository/extensions/vaultspec/source/.vaultspec/lib/scripts/`, so walking up 2 levels gives `.repository/extensions/vaultspec/source/` -- which is wrong for both content and output.

The fix: when `VAULTSPEC_ROOT_DIR` or `VAULTSPEC_CONTENT_DIR` are set, those take full precedence. The structural computation is only a fallback for standalone/development use.

### 3. Fix Python packaging

The repo manager will run `pip install -r requirements.txt` (or `pip install .`) into a managed venv. Currently:

- `pyproject.toml` package discovery is misconfigured (empty `top_level.txt`)
- `requirements.txt` is stale (lists `mcp>=0.1.0` vs pyproject.toml's `mcp>=1.20.0`, missing `claude-agent-sdk`, `sse-starlette`)

**Needed:** A `requirements.txt` at the repo root that accurately lists all runtime dependencies. It doesn't need to be an installable package -- just accurate dependency declarations so the repo manager can `pip install -r requirements.txt` into the venv.

### 4. Add `extension.toml` to repo root

The repo manager reads this to understand what VaultSpec provides and requires:

```toml
[extension]
name = "vaultspec"
version = "0.1.0"
description = "Development workflow rules, agents, and skills framework"

[requires.python]
version = ">=3.13"

[runtime]
type = "python"
install = "pip install -r requirements.txt"

[entry_points]
cli = "{runtime.python} {source}/.vaultspec/lib/scripts/cli.py"
mcp = "{runtime.python} {source}/.vaultspec/lib/scripts/subagent.py serve"

[provides]
mcp = "mcp.json"
content_types = ["rules", "agents", "skills", "system", "constitution"]

[outputs]
claude_dir = ".claude/"
gemini_dir = ".gemini/"
agent_dir = ".agent/"
agents_md = "AGENTS.md"
```

## What Does NOT Need to Change

- The sync logic itself (rules -> tool dirs) is fine
- The MCP server implementation is fine (already accepts `--root`)
- The env-var config system works well -- we're extending it, not replacing it
- The `init` command scaffolding is fine
- Agent/skill/rule CRUD operations are fine
- Standalone (non-embedded) usage stays exactly the same -- all new env vars have backwards-compatible defaults

## Deliverables Summary

| # | Change | Effort |
|---|---|---|
| 1 | Add `VAULTSPEC_CONTENT_DIR` env var + `--content-dir` CLI flag | Medium |
| 2 | Update `init_paths()` to derive source dirs from content path, output dirs from root | Medium |
| 3 | Make `_paths.py` ROOT_DIR a fallback, not the authority when env vars are set | Small |
| 4 | Fix `requirements.txt` to match actual dependencies | Small |
| 5 | Add `extension.toml` to repo root | Small |

Items 1-3 are the critical path. Item 4 is a bugfix regardless. Item 5 is a new file with no code impact.

## Consequences

- VaultSpec authored content (rules, agents, skills) lives inside git worktrees and is version-controlled per-branch
- Generated output (.claude/, .gemini/) lives at the shared container root
- Framework code lives in `.repository/extensions/vaultspec/` managed by the repo manager
- Standalone VaultSpec usage is unaffected (all defaults preserve current behavior)
- The repo manager passes explicit paths via env vars when invoking VaultSpec CLI/MCP
