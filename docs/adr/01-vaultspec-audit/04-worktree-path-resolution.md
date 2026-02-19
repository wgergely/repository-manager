# VaultSpec Audit: Worktree Path Resolution

**Date:** 2026-02-19
**Auditor:** repoman-paths (Opus agent)
**Source:** `repo-fs`, `repo-git`, `repo-core` crates

---

## 1. Layout Modes

Three modes detected by `WorkspaceLayout::detect()`:
- **Container** (`LayoutMode::Container`): `.gt/` dir + `main/` dir
- **InRepoWorktrees** (`LayoutMode::InRepoWorktrees`): `.git` + `.worktrees/` dir
- **Classic** (`LayoutMode::Classic`): `.git` alone

Detection walks up from CWD. `root` is always the container/repo root.

## 2. Where .repository/ Lives

**Always at `{root}/.repository`**, regardless of mode.

```
Container mode:
{container}/
├── .gt/
├── .repository/       <-- config lives here
├── main/              <-- main worktree
└── feature-x/         <-- feature worktree
```

## 3. config_root() vs working_dir()

| | StandardBackend | WorktreeBackend |
|---|---|---|
| `config_root()` | `{repo}/.repository` | `{container}/.repository` |
| `working_dir()` | `{repo}` | `{current_worktree}` (e.g., `{container}/feature-x`) |

**In worktree mode, these diverge.** Config is shared at container level; working_dir is per-worktree.

## 4. Tool Config File Placement

**ALL tool configs are relative to `root`** (container/repo root, NOT worktree):

- `SyncEngine::new(root, mode)` stores `root`
- `ToolSyncer::new(self.root.clone(), ...)` passes root
- `SyncContext::new(self.root.clone())` uses root
- `GenericToolIntegration::config_path()`: `root.join(&definition.integration.config_path)`

For Claude: `CLAUDE.md` at `{container}/CLAUDE.md`, `.claude/rules/` at `{container}/.claude/rules/`
For Gemini: `GEMINI.md` at `{container}/GEMINI.md`

## 5. No Per-Worktree Config Isolation

**ONE shared set of tool configs at container root.** The sync system has zero worktree awareness. `SyncContext.root` is always the container root.

## 6. Implications for VaultSpec Extension

1. `.claude/`, `.gemini/` should go at **container root** (consistent with existing behavior)
2. VaultSpec's `--root` flag set to container root overrides filesystem-relative discovery
3. Extension output paths are relative to root, joined by sync system
4. `.vaultspec/` would also live at container root
5. **No per-worktree VaultSpec configs** - shared across all worktrees

## 7. Critical Paths Summary

| Path | Container Mode | Standard Mode |
|---|---|---|
| `.repository/` | `{container}/.repository/` | `{repo}/.repository/` |
| `CLAUDE.md` | `{container}/CLAUDE.md` | `{repo}/CLAUDE.md` |
| `.claude/rules/` | `{container}/.claude/rules/` | `{repo}/.claude/rules/` |
| `GEMINI.md` | `{container}/GEMINI.md` | `{repo}/GEMINI.md` |
| `.cursorrules` | `{container}/.cursorrules` | `{repo}/.cursorrules` |
| `working_dir()` | `{container}/{branch}/` | `{repo}/` |
| `config_root()` | `{container}/.repository` | `{repo}/.repository` |
