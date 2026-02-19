# VaultSpec Audit: Synthesis and Extension Packaging Verdict

**Date:** 2026-02-19
**Author:** Team Lead (synthesis of 4 auditor reports)

---

## The Fundamental Reality

VaultSpec and the repo manager are doing the same job through different mechanisms:

- **Repo manager**: `.repository/rules/` -> sync engine -> tool configs
- **VaultSpec**: `.vaultspec/rules/` + agents + skills + system -> `cli.py sync-all` -> `.claude/`, `.gemini/`, `.agent/`, `AGENTS.md`

They would **directly conflict** if both managed the same output files.

---

## 1. PACKAGING

VaultSpec is not a Python library. It's an embedded framework - a `.vaultspec/` directory that lives in the repo.

**The "install" is:**
1. Fetch VaultSpec source repository
2. Place `.vaultspec/` in the target repo (container root in worktree mode)
3. Create managed venv at `.repository/extensions/vault-spec/.venv/`
4. `pip install` dependencies into venv (deps only, not VaultSpec itself)

**Draft extension.toml:**
```toml
[extension]
name = "vault-spec"
version = "0.1.0"
description = "Development workflow rules, agents, and skills framework"

[source]
embed_dir = ".vaultspec"

[requires.python]
version = ">=3.13"

[runtime]
type = "python"
install = "pip install -r requirements.txt"

[entry_points]
cli = "{runtime.python} .vaultspec/lib/scripts/cli.py --root {root}"
init = "{runtime.python} .vaultspec/lib/scripts/cli.py --root {root} init"
sync = "{runtime.python} .vaultspec/lib/scripts/cli.py --root {root} sync-all"

[provides]
mcp = "mcp.json"

[outputs]
claude_dir = ".claude/"
gemini_dir = ".gemini/"
agent_dir = ".agent/"
agents_md = "AGENTS.md"
vault_dir = ".vault/"
```

## 2. INSTALLATION FLOW

```
repo extension install <vault-spec-url>
  1. Fetch VaultSpec source repo (git clone to cache)
  2. Read extension.toml from source
  3. Check requires.python (>=3.13) against system (via uv or PATH)
  4. Create venv at .repository/extensions/vault-spec/.venv/
  5. pip install dependencies into venv
  6. Copy .vaultspec/ from source to {root}/.vaultspec/
  7. Create .vault/ scaffolding
  8. Register in .repository/config.toml under [extensions]
  9. Write extensions.lock with resolved version
```

## 3. INITIALIZATION

`repo extension init vault-spec` invokes:
```
{venv}/bin/python .vaultspec/lib/scripts/cli.py --root {container_root} init
```
Then: `sync-all` to populate `.claude/`, `.gemini/`, `.agent/`.

The `--root` flag solves the worktree problem: repo manager passes container root, overriding VaultSpec's filesystem-relative discovery.

## 4. OUTPUT MAPPING

All outputs go at **container root** (worktree mode) or **repo root** (standard mode). Consistent with how repo manager handles all tool configs.

VaultSpec's `--root` flag set to container root. Env var overrides (`VAULTSPEC_CLAUDE_DIR`, `VAULTSPEC_GEMINI_DIR`) available for non-default paths.

The `[outputs]` section declares what VaultSpec will write. Repo manager resolves paths relative to root. Ledger tracks as `FileManaged` projections with `intent_id = "ext:vault-spec"`.

## 5. COEXISTENCE: The Overlap Problem

### Option A: Territory Split (Recommended for now)
VaultSpec owns its declared output dirs (`.claude/`, `.gemini/`, `.agent/`, `AGENTS.md`).
Repo manager owns everything else (`.cursorrules`, `.vscode/`, etc.).
The `[outputs]` declaration tells repo manager which paths to yield.

### Option B: VaultSpec as Content Provider (Long-term ideal)
VaultSpec's rules/agents/skills become `RuleDefinition` entries in repo manager's pipeline.
Repo manager handles all distribution. VaultSpec's sync bypassed.
Only VaultSpec's MCP server and content authoring CLI are used.

### Option C: Layered Coexistence
Both write to same files using non-overlapping markers.
Fragile but possible.

**Recommendation: Start with A, evolve toward B.**

## 6. RESTRUCTURING RECOMMENDATIONS

### Minimal (for immediate integration)
1. Add `extension.toml` to VaultSpec repo root
2. Fix `pyproject.toml` - add `[project.scripts]` entry points, fix `packages.find.where`
3. Accept `--root` consistently (already exists)

### Larger (for long-term Option B)
4. Expose rules/agents/skills as machine-readable content API
5. Separate content (markdown files) from runtime (Python MCP server + CLI)
6. Content could live in `.repository/extensions/vault-spec/` consumed by repo manager's sync engine
