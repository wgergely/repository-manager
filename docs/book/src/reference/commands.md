# Commands Reference

All commands are invoked as `repo <command>`. Run `repo --help` or `repo <command> --help` for inline documentation.

Global flag: `-v` / `--verbose` enables verbose output and is available on all commands.

---

## Core Commands

### `repo init`

Initialize a new repository configuration. Creates `.repository/config.toml`.

```
repo init [NAME] [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `NAME` | Project name. Creates a subdirectory if not `.` (default: `.`) |
| `-m, --mode <MODE>` | Repository mode: `standard` or `worktrees` (default: `worktrees`) |
| `-t, --tools <TOOLS>` | Tools to enable (repeatable: `-t cursor -t claude`) |
| `-p, --presets <PRESETS>` | Presets to apply |
| `-e, --extensions <EXTS>` | Extensions to enable |
| `-r, --remote <URL>` | Remote repository URL |
| `-i, --interactive` | Guided interactive setup |

Examples:

```bash
repo init
repo init my-project
repo init --mode standard --tools cursor,claude --presets rust
repo init --interactive
```

---

### `repo sync`

Generate all tool configuration files from the source of truth.

```
repo sync [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--dry-run` | Preview changes without writing files |
| `--json` | Output results as JSON for CI/CD integration |

---

### `repo status`

Show a summary of the current repository configuration and sync state.

```
repo status [--json]
```

---

### `repo diff`

Preview what `repo sync` would change, without writing any files.

```
repo diff [--json]
```

---

### `repo check`

Check for configuration drift — generated files that have been modified since the last sync.

```
repo check
```

---

### `repo fix`

Fix configuration drift by restoring generated files to their expected state.

```
repo fix [--dry-run]
```

---

## Tool Management

### `repo add-tool <NAME>`

Add a tool integration to the repository configuration and sync.

```bash
repo add-tool cursor
repo add-tool claude --dry-run
```

### `repo remove-tool <NAME>`

Remove a tool integration.

```bash
repo remove-tool windsurf
```

### `repo list-tools`

List all available tool integrations.

```bash
repo list-tools
repo list-tools --category ide
repo list-tools --category cli-agent
repo list-tools --category autonomous
repo list-tools --category copilot
```

### `repo tool-info <NAME>`

Show details about a specific tool: its config path, format, capabilities, and whether it is active.

```bash
repo tool-info claude
```

### `repo tool` (alias: `repo t`)

Nested subcommand form:

```bash
repo tool add claude
repo tool remove cursor
repo tool list
repo tool info claude
```

---

## Preset Management

### `repo add-preset <NAME>`

Apply a preset to the repository.

```bash
repo add-preset rust
repo add-preset python --dry-run
```

### `repo remove-preset <NAME>`

Remove a preset.

```bash
repo remove-preset rust
```

### `repo list-presets`

List all available presets.

### `repo preset` (alias: `repo p`)

Nested subcommand form:

```bash
repo preset add rust
repo preset remove rust
repo preset list
```

---

## Rule Management

### `repo add-rule <ID>`

Add a new coding rule.

```bash
repo add-rule no-unwrap --instruction "Avoid .unwrap(); use ? or handle errors"
repo add-rule naming --instruction "Use snake_case" --tags style,python
```

| Option | Description |
|--------|-------------|
| `-i, --instruction <TEXT>` | The rule instruction text (required) |
| `-t, --tags <TAGS>` | Optional tags (repeatable) |

### `repo remove-rule <ID>`

Remove a rule by ID.

```bash
repo remove-rule no-unwrap
```

### `repo list-rules`

List all active rules.

### `repo rules-lint`

Validate rules for consistency issues.

```bash
repo rules-lint
repo rules-lint --json
```

### `repo rules-diff`

Show configuration drift between expected and actual state.

```bash
repo rules-diff
repo rules-diff --json
```

### `repo rules-export`

Export rules to `AGENTS.md` format.

```bash
repo rules-export
repo rules-export --format agents
```

### `repo rules-import`

Import rules from an `AGENTS.md` file.

```bash
repo rules-import AGENTS.md
```

### `repo rule` (alias: `repo r`)

Nested subcommand form:

```bash
repo rule add no-unwrap --instruction "..."
repo rule remove no-unwrap
repo rule list
```

---

## Configuration

### `repo config show`

Display the current configuration.

```bash
repo config show
repo config show --json
```

---

## Branch / Worktree Management

Branch commands manage git worktrees in `worktrees` mode and regular branches in `standard` mode.

### `repo branch add <NAME>`

Create a new branch (and worktree in worktrees mode).

```bash
repo branch add feature-x
repo branch add feature-x --base develop
```

### `repo branch remove <NAME>`

Remove a branch and its associated worktree.

```bash
repo branch remove feature-x
```

### `repo branch list`

List all branches/worktrees.

### `repo branch checkout <NAME>`

Switch to a branch (or open its worktree).

### `repo branch rename <OLD> <NEW>`

Rename a branch and its worktree.

---

## Git Wrappers

These commands wrap common git operations to ensure they run in the correct context.

```bash
repo push [--remote <REMOTE>] [--branch <BRANCH>]
repo pull [--remote <REMOTE>] [--branch <BRANCH>]
repo merge <SOURCE>
```

---

## Extensions

Manage Repository Manager extensions.

```bash
repo extension install <URL-OR-PATH> [--no-activate]
repo extension add <NAME>
repo extension init <NAME>
repo extension reinit <NAME>
repo extension remove <NAME>
repo extension list [--json]
```

Alias: `repo ext`

---

## Hooks

Manage lifecycle hooks.

```bash
repo hooks list
repo hooks add <EVENT> <COMMAND> [ARGS...]
repo hooks remove <EVENT>
```

Valid events: `pre-branch-create`, `post-branch-create`, `pre-branch-delete`, `post-branch-delete`, `pre-sync`, `post-sync`

---

## Other Commands

### `repo open <WORKTREE>`

Open a worktree in an editor. Runs sync before opening.

```bash
repo open feature-x
repo open feature-x --tool cursor
repo open feature-x --tool vscode
```

### `repo completions <SHELL>`

Generate shell completion scripts.

```bash
repo completions bash
repo completions zsh
repo completions fish
repo completions powershell
```

---

## Exit Codes

| Code | Meaning                                     |
|------|---------------------------------------------|
| 0    | Success                                     |
| 1    | General error                               |
| 2    | Configuration error                         |
| 3    | Validation error                            |
| 4    | Provider/tool error                         |
| 6    | Sync conflict (drift detected)              |

## Environment Variables

| Variable           | Description                      | Default                    |
|--------------------|----------------------------------|----------------------------|
| `REPO_CONFIG_PATH` | Override config file location    | `.repository/config.toml`  |
| `REPO_NO_COLOR`    | Disable colored output           | `false`                    |
| `REPO_VERBOSE`     | Enable verbose logging           | `false`                    |
| `REPO_DRY_RUN`     | Global dry-run mode              | `false`                    |
