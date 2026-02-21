# ADR-005: Extension CLI and Init Integration

**Status:** Approved (all decisions confirmed)
**Date:** 2026-02-19
**Context:** CLI command structure and init flow integration for extensions

---

## Context

The extension system needs CLI commands for install/add/init/remove/list and integration with the existing `repo init` flow.

## Decisions

### 5.1 Command Structure

**Decision: Nested with short alias.**

```
repo extension install|add|init|remove|list
repo ext install|add|init|remove|list    (alias)
```

Matches the existing `branch`/`config`/`hooks` nested subcommand pattern. Short alias reduces typing.

**Implementation:**
- `ExtensionAction` enum in `cli.rs` (alongside `BranchAction`, `ConfigAction`, `HooksAction`)
- `Extension` variant in `Commands` with `#[command(subcommand)]`
- `commands/extension.rs` module with handler functions
- `Ext` as alias via clap `#[command(alias = "ext")]`

### 5.2 Init Flow Integration

**Decision: Init flag with interactive support.**

`repo init --extensions vaultspec` for non-interactive mode.
Interactive mode (`repo init -i`) adds extension selection to the `dialoguer` MultiSelect prompts, alongside tool and preset selection.

This makes extensions a first-class part of repo setup.

**Implementation:**
- Add `--extensions/-e` repeatable flag to `Init` command
- Add `MultiSelect` prompt in `interactive_init()` for extensions
- Extension selection sources from a built-in known list + custom URL entry option

### 5.3 Extension State in config.toml

**Decision: Table with config (overrides only).**

```toml
[extensions."vaultspec"]
source = "https://github.com/org/vaultspec"
ref = "v0.1.0"

# Config section is OPTIONAL - only populated for overrides.
# Empty or absent means "use extension defaults."
# Values here override the extension's own defaults if the
# extension supports env-based or config-based overrides.
[extensions."vaultspec".config]
# Example: override VaultSpec's default output directories
# claude_dir = ".claude"
# gemini_dir = ".gemini"
```

Matches the presets pattern (`[presets."env:python"]`). Each extension is a named table entry with source, ref, and optional config overrides.

**Key principle:** The `[config]` sub-table carries **overrides only**. Default values come from the extension itself (its `repo_extension.toml` or internal defaults). An absent or empty config section means all defaults apply. This only works for extensions that support external configuration (e.g., VaultSpec's `VAULTSPEC_*` env vars).

**Rationale:**
- Table format allows pinning source + ref alongside the extension name
- Per-extension config overrides enable customization without modifying the extension
- Consistent with presets pattern already in the manifest

**Rejected alternatives:**
- Simple list: can't carry source/ref/config
- Array of tables: heavier syntax, doesn't match existing patterns

### 5.4 Extension MCP Tools

**Decision: 5 new MCP tools.**

| Tool | Required Params | Description |
|---|---|---|
| `extension_install` | source | Install extension from URL or local path |
| `extension_add` | name | Activate an installed extension |
| `extension_init` | name | Run extension's initialization logic |
| `extension_remove` | name | Deactivate and uninstall |
| `extension_list` | (none) | List extensions and their status |

These follow the existing MCP tool naming convention (`tool_add`, `preset_list`, etc.).

### 5.5 Extension in Interactive Init

**Decision: Built-in known list + URL entry.**

Interactive init (`repo init -i`) shows a curated `MultiSelect` of well-known extensions (e.g., VaultSpec) plus an option to enter a custom URL. Similar to how tools show a built-in list from `ToolRegistry`.

**Implementation:**
- Built-in extension registry with known extensions (name, description, source URL)
- `MultiSelect` prompt populated from this registry
- Final option: "Custom (enter URL)" -> switches to `Input` prompt
- Selected extensions are installed and registered during init

### 5.6 Auto-Install on Sync

**Decision: Yes, auto-install missing extensions on `repo sync`.**

If `config.toml` declares an extension that isn't installed locally, `repo sync` automatically fetches and installs it. This ensures consistency when cloning a repo or switching machines.

**Rationale:** Matches `npm install` reading `package.json` behavior. When a developer clones a repo with extensions declared in config.toml, `repo sync` makes everything work without manual steps. The `extensions.lock` file ensures the exact same versions are installed.

**Implementation:**
- `SyncEngine::sync()` checks for declared-but-missing extensions before tool/rule sync
- Missing extensions are fetched, deps installed, and activated
- Uses `extensions.lock` for version pinning (deterministic)
- Prints what was auto-installed so the user is aware

**Rejected alternatives:**
- Explicit install required: poor DX when cloning repos
- Prompt the user: breaks non-interactive workflows (CI, scripts)

## Consequences

- `repo extension` command group with 5 subcommands + `repo ext` alias
- Extension selection integrated into `repo init` interactive flow via known list + URL
- `config.toml` carries extension source/ref/config as named tables
- Config overrides are optional; empty means use extension defaults
- `repo sync` auto-installs missing extensions for seamless clone experience
- 5 new MCP tools for AI-driven extension management
- Extensions appear in `repo status` output
- Built-in extension registry for known extensions (expandable)
