# Repository Manager CLI Specification

## Overview

CLI reference for the `repo` binary. Manages tool configurations, presets, rules, branches, hooks, and extensions for a repository.

Global option available on all commands: `-v, --verbose` (enable debug output).

---

## Command Reference

### `status`

Show repository status overview.

```
repo status [--json]
```

| Flag | Description |
|------|-------------|
| `--json` | Output as JSON for scripting |

---

### `diff`

Preview what `sync` would change without applying it.

```
repo diff [--json]
```

| Flag | Description |
|------|-------------|
| `--json` | Output as JSON for scripting |

---

### `init`

Initialize a new repository configuration. Creates a `.repository/` directory with `config.toml`.

```
repo init [NAME] [OPTIONS]
```

| Argument/Flag | Description | Default |
|---------------|-------------|---------|
| `[NAME]` | Project name; creates folder if not `.` | `.` |
| `-m, --mode <MODE>` | Repository mode: `standard` or `worktrees` | `worktrees` |
| `-t, --tools <TOOLS>` | Tools to enable (repeatable) | — |
| `-p, --presets <PRESETS>` | Presets to apply (repeatable) | — |
| `-e, --extensions <EXTENSIONS>` | Extensions to enable by name or URL (repeatable) | — |
| `-r, --remote <REMOTE>` | Remote repository URL | — |
| `-i, --interactive` | Interactive guided setup | — |

Examples:
```bash
repo init                          # Initialize in current directory
repo init my-project               # Create and initialize my-project/
repo init --interactive            # Guided setup
repo init -t claude -t cursor      # With specific tools
repo init -e vaultspec             # With extensions
```

---

### `check`

Check repository configuration for drift.

```
repo check
```

---

### `sync`

Synchronize tool configurations.

```
repo sync [--dry-run] [--json]
```

| Flag | Description |
|------|-------------|
| `--dry-run` | Preview changes without applying them |
| `--json` | Output as JSON for CI/CD integration |

---

### `fix`

Fix configuration drift automatically.

```
repo fix [--dry-run]
```

| Flag | Description |
|------|-------------|
| `--dry-run` | Preview fixes without applying them |

---

## Tool Management

Both flat and nested forms are available. The nested forms are preferred for new usage.

### Flat forms (backwards-compatible)

#### `add-tool`

```
repo add-tool <NAME> [--dry-run]
```

#### `remove-tool`

```
repo remove-tool <NAME> [--dry-run]
```

#### `list-tools`

```
repo list-tools [-c, --category <CATEGORY>]
```

Categories: `ide`, `cli-agent`, `autonomous`, `copilot`.

#### `tool-info`

```
repo tool-info <NAME>
```

Displays metadata, config paths, capabilities, and active status for the tool.

### Nested form: `tool` (alias: `t`)

```
repo tool <SUBCOMMAND>
repo t <SUBCOMMAND>
```

| Subcommand | Equivalent flat command | Description |
|------------|------------------------|-------------|
| `tool add <NAME> [--dry-run]` | `add-tool` | Add a tool |
| `tool remove <NAME> [--dry-run]` | `remove-tool` | Remove a tool |
| `tool list [-c <CATEGORY>]` | `list-tools` | List available tools |
| `tool info <NAME>` | `tool-info` | Show tool details |

Examples:
```bash
repo tool add claude
repo tool remove cursor
repo tool list --category ide
repo tool info claude
repo t add claude   # short alias
```

---

## Preset Management

Both flat and nested forms are available.

### Flat forms (backwards-compatible)

#### `add-preset`

```
repo add-preset <NAME> [--dry-run]
```

#### `remove-preset`

```
repo remove-preset <NAME> [--dry-run]
```

#### `list-presets`

```
repo list-presets
```

### Nested form: `preset` (alias: `p`)

```
repo preset <SUBCOMMAND>
repo p <SUBCOMMAND>
```

| Subcommand | Equivalent flat command | Description |
|------------|------------------------|-------------|
| `preset add <NAME> [--dry-run]` | `add-preset` | Add a preset |
| `preset remove <NAME> [--dry-run]` | `remove-preset` | Remove a preset |
| `preset list` | `list-presets` | List available presets |

Examples:
```bash
repo preset add typescript
repo preset remove typescript
repo preset list
repo p add typescript   # short alias
```

---

## Rule Management

Both flat and nested forms are available.

### Flat forms (backwards-compatible)

#### `add-rule`

```
repo add-rule <ID> -i <INSTRUCTION> [-t <TAG>]...
```

| Argument/Flag | Description |
|---------------|-------------|
| `<ID>` | Rule identifier (e.g., `python-style`) |
| `-i, --instruction <INSTRUCTION>` | Rule instruction text (required) |
| `-t, --tags <TAGS>` | Optional tags (repeatable) |

#### `remove-rule`

```
repo remove-rule <ID>
```

#### `list-rules`

```
repo list-rules
```

#### `rules-lint`

```
repo rules-lint [--json]
```

Lint configuration for consistency issues.

#### `rules-diff`

```
repo rules-diff [--json]
```

Show config drift between expected and actual rule state.

#### `rules-export`

```
repo rules-export [--format <FORMAT>]
```

Export rules to AGENTS.md format. Default format: `agents`.

#### `rules-import`

```
repo rules-import <FILE>
```

Import rules from an AGENTS.md file.

### Nested form: `rule` (alias: `r`)

```
repo rule <SUBCOMMAND>
repo r <SUBCOMMAND>
```

| Subcommand | Equivalent flat command | Description |
|------------|------------------------|-------------|
| `rule add <ID> -i <INSTRUCTION> [-t <TAG>]...` | `add-rule` | Add a rule |
| `rule remove <ID>` | `remove-rule` | Remove a rule |
| `rule list` | `list-rules` | List all active rules |

Examples:
```bash
repo rule add python-style --instruction "Use snake_case for variables."
repo rule add naming -i "Follow consistent naming." -t style -t python
repo rule remove python-style
repo rule list
repo r list   # short alias
```

---

## Branch Management

### `branch`

Manage branches (worktree mode).

```
repo branch <SUBCOMMAND>
```

| Subcommand | Description |
|------------|-------------|
| `branch add <NAME> [--base <BASE>]` | Add a new branch worktree (default base: `main`) |
| `branch remove <NAME>` | Remove a branch worktree |
| `branch list` | List all branch worktrees |
| `branch checkout <NAME>` | Switch to a branch or worktree |
| `branch rename <OLD> <NEW>` | Rename a branch and its worktree |

Examples:
```bash
repo branch add feature-x
repo branch add feature-x --base develop
repo branch remove feature-x
repo branch list
repo branch checkout feature-x
repo branch rename old-name new-name
```

### `push`

```
repo push [-r, --remote <REMOTE>] [-b, --branch <BRANCH>]
```

### `pull`

```
repo pull [-r, --remote <REMOTE>] [-b, --branch <BRANCH>]
```

### `merge`

```
repo merge <SOURCE>
```

---

## Configuration

### `config`

Manage repository configuration.

```
repo config <SUBCOMMAND>
```

| Subcommand | Description |
|------------|-------------|
| `config show [--json]` | Display the current configuration |

---

## Hooks

### `hooks`

Manage lifecycle hooks. Events: `pre-branch-create`, `post-branch-create`, `pre-branch-delete`, `post-branch-delete`, `pre-sync`, `post-sync`.

```
repo hooks <SUBCOMMAND>
```

| Subcommand | Description |
|------------|-------------|
| `hooks list` | List all configured hooks |
| `hooks add <EVENT> <COMMAND> [ARGS]...` | Add a hook for an event |
| `hooks remove <EVENT>` | Remove all hooks for an event |

Examples:
```bash
repo hooks list
repo hooks add post-branch-create npm -- install
repo hooks remove post-branch-create
```

---

## Extensions

### `extension` (alias: `ext`)

Manage extensions. Install, add, initialize, remove, and list extensions.

```
repo extension <SUBCOMMAND>
repo ext <SUBCOMMAND>
```

| Subcommand | Description |
|------------|-------------|
| `extension install <SOURCE> [--no-activate]` | Install from URL or local path |
| `extension add <NAME>` | Add a known extension by name |
| `extension init <NAME>` | Initialize a new extension scaffold |
| `extension reinit <NAME>` | Re-fire post-install hooks for an installed extension |
| `extension remove <NAME>` | Remove an installed extension |
| `extension list [--json]` | List installed and known extensions |

Examples:
```bash
repo extension list
repo extension install https://github.com/example/ext.git
repo extension install ./local-ext --no-activate
repo ext add my-extension   # short alias
repo extension init my-ext
repo extension reinit my-ext
repo extension remove my-ext
```

---

## Workspace

### `open`

Open a worktree in an editor/IDE. Runs sync before opening.

```
repo open <WORKTREE> [-t, --tool <TOOL>]
```

| Argument/Flag | Description |
|---------------|-------------|
| `<WORKTREE>` | Name of the worktree to open |
| `-t, --tool <TOOL>` | Editor: `cursor`, `vscode`, `zed`. Auto-detected if omitted. |

Examples:
```bash
repo open feature-x
repo open feature-x --tool cursor
repo open feature-x --tool vscode
```

---

## Shell Completions

### `completions`

Generate shell completion scripts.

```
repo completions <SHELL>
```

Supported shells: `bash`, `elvish`, `fish`, `powershell`, `zsh`.

Examples:
```bash
repo completions bash > ~/.local/share/bash-completion/completions/repo
repo completions zsh > ~/.zfunc/_repo
repo completions fish > ~/.config/fish/completions/repo.fish
```

---

## Command Summary

| Command | Alias | Description |
|---------|-------|-------------|
| `status` | — | Repository status |
| `diff` | — | Preview sync changes |
| `init` | — | Initialize repository |
| `check` | — | Check for drift |
| `sync` | — | Synchronize configs |
| `fix` | — | Fix drift automatically |
| `add-tool` | `tool add` | Add a tool |
| `remove-tool` | `tool remove` | Remove a tool |
| `list-tools` | `tool list` | List available tools |
| `tool-info` | `tool info` | Show tool details |
| `tool` | `t` | Nested tool management |
| `add-preset` | `preset add` | Add a preset |
| `remove-preset` | `preset remove` | Remove a preset |
| `list-presets` | `preset list` | List available presets |
| `preset` | `p` | Nested preset management |
| `add-rule` | `rule add` | Add a rule |
| `remove-rule` | `rule remove` | Remove a rule |
| `list-rules` | `rule list` | List active rules |
| `rules-lint` | — | Lint rules config |
| `rules-diff` | — | Show rules drift |
| `rules-export` | — | Export rules to AGENTS.md |
| `rules-import` | — | Import rules from file |
| `rule` | `r` | Nested rule management |
| `branch` | — | Branch/worktree management |
| `push` | — | Push to remote |
| `pull` | — | Pull from remote |
| `merge` | — | Merge branch |
| `config` | — | Repository configuration |
| `hooks` | — | Lifecycle hook management |
| `extension` | `ext` | Extension management |
| `open` | — | Open worktree in editor |
| `completions` | — | Generate shell completions |
