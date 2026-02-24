# Your First Sync

This page explains what happens during a sync and how to manage configuration over time.

## How Sync Works

When you run `repo sync`, Repository Manager:

1. Reads `.repository/config.toml` to find which tools are enabled and which presets are active.
2. Loads all rule files from `.repository/rules/`.
3. For each enabled tool, generates a configuration file in the format that tool expects.
4. Writes each file, recording a checksum so drift can be detected later.

Each tool has its own output path and format:

| Tool    | Output Path                           |
|---------|---------------------------------------|
| Cursor  | `.cursorrules`                        |
| Claude  | `CLAUDE.md`                           |
| VS Code | `.vscode/settings.json`               |
| Copilot | `.github/copilot-instructions.md`     |

See the [Tools Reference](../reference/tools.md) for a complete list.

## Previewing Changes

Before applying a sync, you can preview what would change:

```bash
repo diff
```

Or use the `--dry-run` flag:

```bash
repo sync --dry-run
```

Neither command writes any files; they only show what would be written.

## Detecting Drift

Drift occurs when a generated file has been edited manually after a sync. Repository Manager tracks checksums of all generated files and can detect when they no longer match.

Check for drift:

```bash
repo check
```

See a detailed diff of what drifted:

```bash
repo rules-diff
```

Fix drift automatically (restores generated files to the expected state):

```bash
repo fix
```

## Adding and Removing Tools

Add a new tool to your configuration:

```bash
repo add-tool windsurf
```

This registers the tool in `config.toml` and runs a sync automatically.

Remove a tool:

```bash
repo remove-tool windsurf
```

This removes the tool from `config.toml`. The previously generated config file is left in place unless you remove it manually.

## Managing Rules

List all active rules:

```bash
repo list-rules
```

Add a new rule:

```bash
repo add-rule error-handling \
  --instruction "Always handle errors explicitly; do not silently swallow exceptions"
```

Remove a rule:

```bash
repo remove-rule error-handling
```

After adding or removing rules, run `repo sync` to apply the changes to all tool config files.

## Worktrees Mode

If your project uses git worktrees, initialize with `--mode worktrees`:

```bash
repo init --mode worktrees --tools cursor,claude
```

In worktrees mode, `.repository/` lives at the container root and is shared across all worktrees. Each worktree gets its own generated tool config files. See [config.toml](../configuration/config-toml.md) for details on the `mode` setting.
