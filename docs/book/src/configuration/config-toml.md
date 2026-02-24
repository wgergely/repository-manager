# config.toml

`.repository/config.toml` is the single source of truth for your Repository Manager setup. It controls which tools are enabled, which presets are active, and global settings like the operating mode.

## Location

```
.repository/
└── config.toml          # Tracked in version control
└── config.local.toml    # Local overrides (gitignored)
```

`config.local.toml` follows the same format as `config.toml`. Settings in the local file override the shared config. Use it for developer-specific preferences that should not be committed.

## Minimal Example

```toml
[core]
mode = "standard"

[tools]
enabled = ["cursor", "claude", "vscode", "copilot"]

[presets]
enabled = ["rust"]
```

## Full Schema

### `[core]`

```toml
[core]
mode = "standard"   # "standard" | "worktrees"
```

**`mode`** controls the physical layout of the repository.

- `standard` — a normal single-branch git repository. All configuration files are written into the project root.
- `worktrees` — a git worktree container. The `.repository/` directory is shared at the container root. Each worktree gets its own generated tool config files.

### `[tools]`

```toml
[tools]
enabled = ["cursor", "claude", "vscode", "copilot"]
```

**`enabled`** is a list of tool identifiers to register. Each entry must be a valid tool name. See the [Tools Reference](../reference/tools.md) for all available names.

You can also manage tools via CLI:

```bash
repo add-tool gemini
repo remove-tool gemini
```

### `[presets]`

```toml
[presets]
enabled = ["rust"]
```

**`enabled`** is a list of preset names to apply. Presets add language-specific rules and tool settings. See [Presets](presets.md) for details.

Manage presets via CLI:

```bash
repo add-preset python
repo remove-preset python
```

## Rules

Rules are not declared in `config.toml`. They live as individual Markdown files in `.repository/rules/`. Each file is picked up automatically during sync.

```
.repository/
└── rules/
    ├── no-unwrap.md
    ├── error-handling.md
    └── naming-conventions.md
```

Create a rule file directly, or use the CLI:

```bash
repo add-rule no-unwrap --instruction "Avoid .unwrap() in library code"
```

Rules are injected into every tool's configuration on the next sync.

## Rule File Format

A rule file is a Markdown file with optional YAML frontmatter:

```markdown
---
id: no-unwrap
tags:
  - rust
  - safety
---

Avoid `.unwrap()` in library code. Use `?` to propagate errors or handle
them explicitly with `match` or `if let`.
```

If the frontmatter `id` is omitted, the filename (without extension) is used as the rule ID.
