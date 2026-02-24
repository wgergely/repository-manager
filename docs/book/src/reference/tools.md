# Tools Reference

Repository Manager ships with 14 built-in tool integrations. Each tool has a unique identifier used in `config.toml` and CLI commands.

## IDE Tools

These tools are code editors with integrated AI assistance.

### Cursor

| Property      | Value                    |
|---------------|--------------------------|
| Identifier    | `cursor`                 |
| Config path   | `.cursorrules`           |
| Format        | Markdown                 |
| Category      | IDE                      |

Cursor reads project rules from `.cursorrules`. Repository Manager writes your active rules into this file inside a managed block, preserving any content you have added outside that block.

### VS Code

| Property      | Value                       |
|---------------|-----------------------------|
| Identifier    | `vscode`                    |
| Config path   | `.vscode/settings.json`     |
| Format        | JSON                        |
| Category      | IDE                         |

VS Code integration manages AI-related settings keys in `.vscode/settings.json`. Repository Manager merges its managed keys while preserving user-defined keys in the same file.

### Windsurf

| Property      | Value               |
|---------------|---------------------|
| Identifier    | `windsurf`          |
| Config path   | `.windsurfrules`    |
| Format        | Markdown            |
| Category      | IDE                 |

Windsurf uses a rules file similar in structure to `.cursorrules`. Repository Manager writes active rules into `.windsurfrules`.

### JetBrains

| Property      | Value                     |
|---------------|---------------------------|
| Identifier    | `jetbrains`               |
| Config path   | `.aiassistant/rules/`     |
| Format        | Markdown directory        |
| Category      | IDE                       |

JetBrains AI Assistant reads rules from a directory. Repository Manager writes individual rule files into `.aiassistant/rules/`, one file per rule.

### Zed

| Property      | Value     |
|---------------|-----------|
| Identifier    | `zed`     |
| Config path   | `.rules`  |
| Format        | Markdown  |
| Category      | IDE       |

Zed reads project instructions from a `.rules` file at the project root.

---

## CLI Agent Tools

These tools run as command-line AI agents and read their behavioral instructions from files in your project.

### Claude

| Property      | Value       |
|---------------|-------------|
| Identifier    | `claude`    |
| Config path   | `CLAUDE.md` |
| Format        | Markdown    |
| Category      | CLI Agent   |

Claude Code reads project instructions from `CLAUDE.md` at the project root. Repository Manager generates this file from your active rules. Claude also supports per-directory `CLAUDE.md` files for nested context.

### Claude Desktop

| Property      | Value                        |
|---------------|------------------------------|
| Identifier    | `claude-desktop`             |
| Config path   | `claude_desktop_config.json` |
| Format        | JSON                         |
| Category      | CLI Agent                    |

Claude Desktop uses a JSON configuration file. Repository Manager manages MCP server entries and project-level settings within this file.

### Gemini

| Property      | Value       |
|---------------|-------------|
| Identifier    | `gemini`    |
| Config path   | `GEMINI.md` |
| Format        | Markdown    |
| Category      | CLI Agent   |

Gemini CLI reads project instructions from `GEMINI.md`.

### Aider

| Property      | Value              |
|---------------|--------------------|
| Identifier    | `aider`            |
| Config path   | `.aider.conf.yml`  |
| Format        | YAML               |
| Category      | CLI Agent          |

Aider reads its configuration from `.aider.conf.yml`. Repository Manager writes AI-relevant settings into this file.

---

## Copilot

### GitHub Copilot

| Property      | Value                                  |
|---------------|----------------------------------------|
| Identifier    | `copilot`                              |
| Config path   | `.github/copilot-instructions.md`      |
| Format        | Markdown                               |
| Category      | Copilot                                |

GitHub Copilot reads project-level custom instructions from `.github/copilot-instructions.md`. Repository Manager writes your active rules into this file.

---

## Autonomous Agent Tools

These tools run agents capable of taking actions inside your repository (file editing, running commands, etc.).

### Cline

| Property      | Value          |
|---------------|----------------|
| Identifier    | `cline`        |
| Config path   | `.clinerules`  |
| Format        | Markdown       |
| Category      | Autonomous     |

Cline reads project rules from `.clinerules`.

### Roo

| Property      | Value           |
|---------------|-----------------|
| Identifier    | `roo`           |
| Config path   | `.roo/rules/`   |
| Format        | Markdown dir    |
| Category      | Autonomous      |

Roo reads rules from a directory. Repository Manager writes individual rule files into `.roo/rules/`.

### Amazon Q

| Property      | Value              |
|---------------|--------------------|
| Identifier    | `amazon-q`         |
| Config path   | `.amazonq/rules/`  |
| Format        | Markdown dir       |
| Category      | Autonomous         |

Amazon Q Developer reads rules from `.amazonq/rules/`.

### Antigravity

| Property      | Value               |
|---------------|---------------------|
| Identifier    | `antigravity`       |
| Config path   | `.agent/rules.md`   |
| Format        | Markdown            |
| Category      | Autonomous          |

Antigravity reads project rules and skills from the `.agent/` directory.

---

## Custom Tools

In addition to the 14 built-in tools, you can define custom tool integrations by placing definition files in `.repository/tools/`. This lets you add support for tools that are not built into Repository Manager.

Use the CLI to view tool details:

```bash
repo tool-info claude
repo list-tools
repo list-tools --category ide
```
