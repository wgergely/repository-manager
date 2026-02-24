# Tool Configuration

Each tool integration knows how to translate Repository Manager's rules and settings into the format that specific tool expects.

## How Tools Work

When `repo sync` runs, each enabled tool:

1. Receives the set of active rules.
2. Translates those rules into its own config format (Markdown, JSON, YAML, etc.).
3. Writes the result to the tool's expected config path.

Repository Manager uses a **managed block** system to write only the sections it owns inside a config file, preserving any content you have written manually outside those blocks.

## Tool Categories

### IDE Tools

These tools are code editors with built-in AI assistance.

| Tool      | Config Path              | Format             |
|-----------|--------------------------|--------------------|
| Cursor    | `.cursorrules`           | Markdown           |
| VS Code   | `.vscode/settings.json`  | JSON               |
| Windsurf  | `.windsurfrules`         | Markdown           |
| JetBrains | `.aiassistant/rules/`    | Markdown directory |
| Zed       | `.rules`                 | Markdown           |

### CLI Agent Tools

These tools run as command-line agents and read their instructions from files in your project.

| Tool           | Config Path                  | Format   |
|----------------|------------------------------|----------|
| Claude         | `CLAUDE.md`                  | Markdown |
| Claude Desktop | `claude_desktop_config.json` | JSON     |
| Gemini         | `GEMINI.md`                  | Markdown |
| Aider          | `.aider.conf.yml`            | YAML     |

### Copilot

| Tool    | Config Path                          | Format   |
|---------|--------------------------------------|----------|
| Copilot | `.github/copilot-instructions.md`    | Markdown |

### Autonomous Agent Tools

These tools run agents that can take actions inside your repository.

| Tool       | Config Path        | Format             |
|------------|--------------------|--------------------|
| Cline      | `.clinerules`      | Markdown           |
| Roo        | `.roo/rules/`      | Markdown directory |
| Amazon Q   | `.amazonq/rules/`  | Markdown directory |
| Antigravity| `.agent/rules.md`  | Markdown           |

## Custom Tools

You can define custom tool integrations by placing a tool definition file in `.repository/tools/`. Custom tool definitions let you specify a config path and a template for how rules are rendered into that file.

This feature is useful for tools not included in the 14 built-in integrations.

## Managing Tools

Add a tool:

```bash
repo add-tool windsurf
```

Remove a tool:

```bash
repo remove-tool windsurf
```

List all available tools:

```bash
repo list-tools
```

Filter by category:

```bash
repo list-tools --category ide
repo list-tools --category cli-agent
repo list-tools --category autonomous
repo list-tools --category copilot
```

Show details about a specific tool (config path, capabilities, whether it is active):

```bash
repo tool-info claude
```
