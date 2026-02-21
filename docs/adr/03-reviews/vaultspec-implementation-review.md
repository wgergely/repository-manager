# VaultSpec Implementation Review: Workspace Path Decoupling

**Date:** 2026-02-19
**Reviewer:** Repository Manager team
**Subject:** VaultSpec's implementation of ADR-006 requirements

---

## Verdict: ACCEPTED

All ADR-006 requirements met. Three flagged issues resolved in follow-up.

## Requirements Checklist

| ADR-006 Requirement | Status | Location |
|---|---|---|
| `VAULTSPEC_CONTENT_DIR` env var | PASS | `config.py:70,321-327` |
| `VAULTSPEC_ROOT_DIR` as output root | PASS | `_paths.py:38` → `resolve_workspace()` |
| `init_paths()` splits source/dest | PASS | `cli.py:180-186` (content), `cli.py:208-240` (output) |
| `_paths.py` structural fallback only | PASS | Structural `LIB_SRC_DIR` first, then `resolve_workspace()` |
| Bootstrap order (sys.path before import) | PASS | `_paths.py:19-25` structural → `_paths.py:35` import |
| `--content-dir` CLI flag | PASS | `cli.py:2031-2036`, `subagent.py:219-224` |
| Fix `requirements.txt` | PASS | All 10 deps match `pyproject.toml`, `mcp>=1.20.0` |
| Add `repo_extension.toml` | PASS | All 6 sections present |
| `mcp.json` intact | PASS | `vs-subagent-mcp` matches `repo_extension.toml` provides |

## Critical Check: Container Mode

When `.gt/` detected, `content_root = container_root / ".vaultspec"` (NOT `worktree / ".vaultspec"`).
Confirmed by `test_standalone_container_git` in `test_workspace.py:212-228`.
Matches ADR-006 Appendix A.

## Implementation Highlights

### New Module: `core/workspace.py` (381 lines)

- `WorkspaceLayout` frozen dataclass: content_root, output_root, vault_root, framework_root, mode, git
- `LayoutMode` enum: STANDALONE, EXPLICIT (WORKTREE/EMBEDDED folded into STANDALONE with GitInfo metadata)
- `discover_git()`: walks up for `.gt/` (priority) then `.git` (files AND dirs), parses gitdir pointers
- `resolve_workspace()`: explicit env vars > git detection > structural fallback > CWD
- `_validate()`: 3 checks with actionable error messages including env var names

### Bootstrap (`_paths.py`)

Correct two-step: structural `LIB_SRC_DIR` → `sys.path.insert` → import `core.workspace` → `resolve_workspace()`.
`_FRAMEWORK_ROOT` always structural from `__file__`, never from env vars.

### `init_paths()` Decoupling (`cli.py`)

- Source dirs (`RULES_SRC_DIR`, `AGENTS_SRC_DIR`, etc.) from `layout.content_root`
- Output dirs (`TOOL_CONFIGS` entries) from `layout.output_root`
- Backward compatible: accepts both `WorkspaceLayout` and plain `Path`

### `repo_extension.toml`

```toml
[extension]
name = "vaultspec"
version = "0.1.0"
[requires.python]
version = ">=3.13"
[runtime]
type = "python"
install = "pip install -e '.[dev]'"
[entry_points]
cli = ".vaultspec/lib/scripts/cli.py"
mcp = ".vaultspec/lib/scripts/subagent.py serve"
[provides]
mcp = ["vs-subagent-mcp"]
content_types = ["rules", "agents", "skills", "system", "templates"]
[outputs]
claude_dir = ".claude"
gemini_dir = ".gemini"
agent_dir = ".agent"
agents_md = "AGENTS.md"
```

## Issues Found and Resolved

### Issue 1: `list_available_agents()` wrong root (Fixed)

Was: `root / framework_dir / "rules" / "agents"` (output_root).
Now: function takes explicit `content_root: Path` parameter, call site passes `_layout.content_root`.

### Issue 2: `content_override` alone silently ignored (Fixed)

Was: fell through to git detection, content_override never consumed.
Now: dedicated branch handles content-only override, derives output_root from git/structural fallback.

### Issue 3: Test gaps (Fixed)

- `TestContentDirCLI`: integration test for `--content-dir` through `init_paths()` and `collect_rules()`
- `TestPathsEnvBridge`: tests `resolve_workspace()` with env-var-equivalent parameters
- `test_validation_output_root_parent_missing`: covers the previously untested validation branch

## Test Coverage: 21+ unit tests, 6+ integration tests

All layout modes, container detection, gitdir parsing, env var overrides, validation errors, frozen immutability, and CLI flag integration covered.
