# Tools Subsystem Specification

**Crate**: `repo-tools`

## 1. Overview

The Tools subsystem manages the *integration* with external development tools. It acts as a bridge, translating the Repository Manager's internal state (Active Presets, Rules) into tool-specific configuration formats (VSCode JSON, Cursor Rules, Claude Instructions).

## 2. Core Responsibilities

1. **Tool Discovery**: Detecting which tools are installed or relevant for the user.
2. **Config Generation**: Rendering configuration files (`settings.json`, `.cursorrules`).
3. **Instruction Injection**: Prompting agentic tools with system instructions.

## 3. The `ToolIntegration` Trait

```rust
pub trait ToolIntegration {
    /// e.g. "vscode", "kdb"
    fn name(&self) -> &str;

    /// Where does this tool look for config?
    /// Returns path relative to Layout Root.
    fn config_lines(&self) -> Vec<ConfigLocation>;

    /// Apply the accumulated Rules/State to the tool's config.
    fn sync(&self, context: &Context, rules: &[Rule]) -> Result<()>;
}
```

## 4. Supported Tools & Strategies

### 4.1 VSCode (and Forks)

* **Strategy**: JSON manipulation.
* **Target**: `.vscode/settings.json`.
* **Action**: Merges managed keys (e.g., `python.defaultInterpreterPath`) while preserving user keys. Uses the [Ledger](config-ledger.md) to track owned keys.

### 4.2 Cursor

* **Strategy**: Hybrid (JSON + Markdown).
* **Target 1**: `.vscode/settings.json` (Inherited from VSCode).
* **Target 2**: `.cursorrules` (Agentic behavior).
* **Action**: Injects "Managed Blocks" of text into `.cursorrules` based on active rules.

### 4.3 Claude Desktop / CLI

* **Strategy**: JSON / Prompt Files.
* **Target**: `.claude/config.json` or `claude_desktop_config.json`.
* **Action**: Injects MCP server configurations and global system prompts.

### 4.4 Antigravity (.agent)

* **Strategy**: File System Structure.
* **Target**: `.agent/` directory.
* **Action**: Generates `rules/*.md`, `skills/*.md` based on active presets.

## 5. Rule Transpilation

The subsystem is responsible for converting generic "Rules" into tool-specific formats.

* **Input**: `Rule { id: "python-style", content: "Use snake_case" }`
* **Output (VSCode)**: Possibly enabling a linter setting.
* **Output (Cursor)**: Adding "Always use snake_case for Python" to the system prompt.

## 6. MCP Integration

For tools that support the **Model Context Protocol (MCP)**, the Tools subsystem acts as a configuration generator for the MCP servers.

* If `preset:postgres` is active, the Tools subsystem ensures the *Postgres MCP Server* is added to the `claude_desktop_config.json`.
