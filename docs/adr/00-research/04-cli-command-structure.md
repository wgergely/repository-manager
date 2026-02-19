# Research: CLI Command Structure and Init Flow

**Date:** 2026-02-19
**Researcher:** cli-researcher (Opus agent)
**Source:** `repo-cli` and `repo-mcp` crates

---

## 1. Full CLI Command Tree

```
repo [--verbose/-v] <COMMAND>

  status [--json]
  diff [--json]
  init [name] [--mode/-m] [--tools/-t...] [--presets/-p...] [--remote/-r] [--interactive/-i]
  check
  sync [--dry-run] [--json]
  fix [--dry-run]
  add-tool <name> [--dry-run]
  remove-tool <name> [--dry-run]
  add-preset <name> [--dry-run]
  remove-preset <name> [--dry-run]
  add-rule <id> --instruction/-i <text> [--tags/-t...]
  remove-rule <id>
  list-rules
  rules-lint [--json]
  rules-diff [--json]
  rules-export [--format agents]
  rules-import <file>
  list-tools [--category/-c <cat>]
  list-presets
  tool-info <name>
  completions <shell>
  push [--remote/-r] [--branch/-b]
  pull [--remote/-r] [--branch/-b]
  merge <source>

  branch add|remove|list|checkout|rename
  config show [--json]
  hooks list|add|remove
  open <worktree> [--tool/-t <slug>]
```

## 2. Repo Init Flow

1. **Parse CLI args**: name (default "."), mode (default "worktrees"), tools, presets, remote, interactive
2. **Interactive mode** (`-i`): dialoguer prompts for project name, mode, tools (MultiSelect), presets (MultiSelect), remote URL
3. **Determine target path**: sanitize name -> create folder or use cwd
4. **Initialize**: create `.repository/`, generate `config.toml`, `git init` if needed, create `main/` if worktree mode
5. **Add remote** if specified
6. **Print** next-step guidance

## 3. Tool Add/Remove Lifecycle

### add-tool
1. Validate against `KnownToolSlugs` (warn if unknown, don't block)
2. Load manifest from `.repository/config.toml`
3. Check if already configured
4. Add to `manifest.tools`
5. Save manifest
6. **Trigger sync** via `SyncEngine::sync()`

### remove-tool
1. Load manifest
2. Find and remove from `manifest.tools`
3. Save manifest
4. Trigger sync

## 4. Preset Add/Remove Lifecycle

### add-preset
1. Validate against `Registry::with_builtins()`
2. Load manifest
3. Insert into `manifest.presets` with empty JSON object
4. Save manifest
5. **Does NOT trigger sync** (unlike tool add)

### remove-preset
Same pattern, does NOT trigger sync.

## 5. Rule Add/Remove Lifecycle

### add-rule
1. Validate rule ID (no path traversal, alphanumeric + hyphens/underscores)
2. Create `.repository/rules/` if needed
3. Write `{id}.md` with optional tags header + instruction content
4. Overwrites existing silently

### remove-rule
1. Validate ID, check exists, remove file

## 6. MCP Server: 17 Tools, 3 Resources

### Tools
| Category | Tool | Required Params |
|---|---|---|
| Lifecycle | `repo_init` | name |
| Lifecycle | `repo_check` | (none) |
| Lifecycle | `repo_sync` | (none) |
| Lifecycle | `repo_fix` | (none) |
| Branch | `branch_create` | name |
| Branch | `branch_delete` | name |
| Branch | `branch_list` | (none) |
| Git | `git_push` | (not implemented) |
| Git | `git_pull` | (not implemented) |
| Git | `git_merge` | (not implemented) |
| Config | `tool_add` | name |
| Config | `tool_remove` | name |
| Config | `rule_add` | id, content |
| Config | `rule_remove` | id |
| Preset | `preset_list` | (none) |
| Preset | `preset_add` | name |
| Preset | `preset_remove` | name |

### Resources
- `repo://config` - `.repository/config.toml`
- `repo://state` - `.repository/ledger.toml`
- `repo://rules` - Aggregated rules markdown

## 7. Extension Command Integration

**Recommended: Nested subcommand** (matches branch/config/hooks pattern):
```
repo extension install|add|init|remove|list
```

Would require:
- `ExtensionAction` enum in `cli.rs`
- `Extension` variant in `Commands`
- `commands/extension.rs` module
- 5 new MCP tools: `extension_install`, `extension_add`, `extension_init`, `extension_remove`, `extension_list`

## 8. Error Handling Patterns

```rust
pub enum CliError {
    Core(repo_core::Error),
    Fs(repo_fs::Error),
    Git(repo_git::Error),
    Io(std::io::Error),
    Dialoguer(dialoguer::Error),
    Json(serde_json::Error),
    Presets(repo_presets::Error),
    User { message: String },
}
```

Conventions: non-fatal warnings for unknown names, idempotent operations, dry-run pattern, sync failures non-fatal during add/remove.
