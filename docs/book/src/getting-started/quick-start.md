# Quick Start

This guide walks you through initializing a new project and running your first sync in under five minutes.

## 1. Initialize a Project

Run `repo init` in your project directory. Use `--tools` to specify which AI tools you want to configure:

```bash
cd my-project
repo init --tools cursor,claude,vscode,copilot
```

This creates a `.repository/` directory with a `config.toml` file:

```
my-project/
└── .repository/
    └── config.toml
```

The `config.toml` is now your single source of truth.

## 2. Add a Preset (Optional)

Presets apply sensible defaults for a language or stack. For example, the `rust` preset adds Rust-specific coding guidelines:

```bash
repo add-preset rust
```

Available presets include `rust`, `python`, and `node`.

## 3. Add a Custom Rule

Rules are coding guidelines injected into every tool's configuration. Add one:

```bash
repo add-rule no-unwrap --instruction "Avoid .unwrap() in library code; use ? or handle errors explicitly"
```

Rules are stored as Markdown files in `.repository/rules/`.

## 4. Sync

Generate all tool configuration files from your source of truth:

```bash
repo sync
```

Output:

```
Syncing 4 tools...
  cursor      → .cursorrules               [written]
  claude      → CLAUDE.md                  [written]
  vscode      → .vscode/settings.json      [written]
  copilot     → .github/copilot-instructions.md [written]
Sync complete. 4 files written.
```

## 5. Check Status

Confirm everything is in sync:

```bash
repo status
```

Output:

```
Repository: my-project (standard mode)
Tools:  cursor, claude, vscode, copilot
Rules:  no-unwrap (+ rules from rust preset)
Status: in sync
```

## What to Commit

Commit `.repository/config.toml` and the files in `.repository/rules/` to version control. The generated files (`.cursorrules`, `CLAUDE.md`, etc.) can be committed or gitignored depending on your workflow — committing them means team members without `repo` installed still get correct tool configs.

## Next Step

Read [Your First Sync](first-sync.md) for a deeper walkthrough of how sync works and how to manage ongoing changes.
