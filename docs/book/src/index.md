# Repository Manager

**One config file. 14 AI tools. Zero drift.**

Repository Manager generates and synchronizes AI tool configurations — Cursor, Claude, VS Code, Copilot, and 10 more — from a single `.repository/config.toml` source of truth.

> **Status: Alpha** — API may change. Feedback welcome.

---

## The Problem

In 2026, a typical project touches a dozen AI tools. Each has its own config format, and they drift apart immediately:

```
.cursorrules                      (manually written)
CLAUDE.md                         (manually written, different rules)
.vscode/settings.json             (manually written, yet another format)
.github/copilot-instructions.md   (manually written, forgotten)
```

Keeping these in sync by hand is tedious and error-prone. Rules written for Cursor don't automatically appear in Claude. Instructions added to Copilot are never applied to Windsurf.

## The Solution

Declare your intent once. `repo sync` generates the rest.

```
.repository/config.toml    ← you edit this
        |
        v  repo sync
        |
        ├── .cursorrules                        (generated)
        ├── CLAUDE.md                           (generated)
        ├── .vscode/settings.json               (generated)
        ├── .github/copilot-instructions.md     (generated)
        └── ... 10 more tools                   (generated)
```

Your rules, coding standards, and project context live in one place. Every registered tool gets an accurate, up-to-date configuration on every sync.

## Key Concepts

**Tools** are the AI IDEs and agents you use — Cursor, Claude Code, VS Code Copilot, etc. Repository Manager ships with 14 built-in tools. You can also define custom tools.

**Rules** are the coding guidelines, project conventions, and behavioral instructions you want every AI tool to follow. They live as Markdown files in `.repository/rules/`.

**Presets** are named bundles of defaults for a language or stack (e.g., `rust`, `python`, `node`). Applying a preset sets sensible rules and tool settings for that environment.

**Sync** is the operation that reads your config and rules, then writes the correct configuration file for each registered tool.

## Next Steps

- [Install Repository Manager](getting-started/installation.md)
- [Quick Start](getting-started/quick-start.md) — initialize a project and run your first sync
- [Tools Reference](reference/tools.md) — see all 14 supported tools
- [Commands Reference](reference/commands.md) — full CLI reference
