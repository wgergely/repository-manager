# Repository Configuration Schema

## Overview

The `.repository` directory serves as the **Single Source of Truth (SSOT)** for the workspace. The Repository Manager CLI uses definitions in this directory to generate, synchronize, and validate the configuration files required by various agentic tools (Claude, Cursor, VSCode, etc.).

## Directory Structure

We adopt a modular, file-based configuration approach using **TOML** for its strong typing and readability.

```text
.repository/
├── config.toml           # The primary manifest (enabled tools, presets, mode)
├── state.lock            # (Optional) Computed state to track sync status
├── tools/                # Registry of available tools and their integration logic
│   ├── claude.toml
│   ├── rules.toml        # (Alternatively, rules logic can vary)
│   └── vscode.toml
├── rules/                # Definitions of prompts/behaviors to enforce
│   ├── python-style.toml
│   └── no-api-keys.toml
└── presets/              # Collections of rules/tools for stacks
    ├── python-web.toml
    └── rust-cli.toml
```

## 1. The Manifest (`config.toml`)

This file defines the high-level configuration of the repository. Tools and rules are top-level arrays. The `[core]` section contains only the workspace mode. Presets are defined as `[presets."type:name"]` table entries.

```toml
# Top-level arrays: tools and rules to enable
tools = ["cursor", "claude", "vscode"]
rules = ["python-style", "no-api-keys"]

[core]
mode = "standard"  # or "worktrees" (default: "worktrees")

[presets."env:python"]
version = "3.12"
provider = "uv"

[presets."env:node"]
version = "20"
```

### Manifest Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `tools` | `string[]` | No | `[]` | List of tool slugs to enable (e.g., `"claude"`, `"vscode"`) |
| `rules` | `string[]` | No | `[]` | List of rule IDs to apply |
| `core.mode` | `string` | No | `"worktrees"` | Workspace mode: `"standard"` or `"worktrees"` |
| `presets.<key>` | `table` | No | - | Preset configurations keyed by `"type:name"` |

> **Note:** The `tools` and `rules` arrays must appear before any `[section]` headers in the TOML file, since they are top-level keys.

## 2. Tool Registration (`tools/*.toml`)

Each file in `tools/` defines a tool's capabilities and how the manager should interact with it. The 13 built-in tools are compiled into the binary and do not require TOML files. Custom tools placed in `.repository/tools/` extend the built-in set.

> **Implementation note:** The current implementation loads built-in tool definitions from `repo-tools::registry` and dispatches to specialized integration modules. The TOML schema below describes the format for custom tool definitions and matches the `ToolDefinition` struct in `repo-meta`.

**Example: `tools/claude.toml`**

```toml
[meta]
name = "Claude Code"
slug = "claude"
description = "Anthropic's Claude Code CLI agent"

[integration]
config_path = ".claude/config.json"
type = "json"
additional_paths = []

[capabilities]
supports_custom_instructions = true
supports_mcp = true
supports_rules_directory = false

[schema]
instruction_key = "global_instructions"
mcp_key = "mcpServers"
```

**Example: `tools/cursor.toml`**

```toml
[meta]
name = "Cursor"
slug = "cursor"
description = "AI-first code editor"

[integration]
config_path = ".cursorrules"
type = "text"

[capabilities]
supports_custom_instructions = true
supports_mcp = true
supports_rules_directory = false
```

### Supported Config Types

| Type | Extension | Description |
|------|-----------|-------------|
| `text` | `.cursorrules`, etc. | Plain text files with managed block markers |
| `json` | `.json` | Structured JSON merge on manager-owned keys |
| `toml` | `.toml` | TOML configuration files |
| `yaml` | `.yaml`, `.yml` | YAML configuration files |
| `markdown` | `.md` | Markdown files (e.g., `CLAUDE.md`) |

## 3. Rule Definitions (`rules/*.toml`)

Rules capture specific behaviors, constraints, or stylistic preferences. They are abstract enough to be unrolled to different tools.

> **Implementation note:** The current implementation loads rules via the CLI `add-rule` command, which stores them in `registry.toml`. The TOML schema below describes the design format and matches the `RuleDefinition` struct in `repo-meta`.

**Example: `rules/code-style-python.toml`**

```toml
[meta]
id = "python-snake-case"
severity = "mandatory" # mandatory, suggestion (default: suggestion)
tags = ["python", "style"]

[content]
instruction = """
All Python variable names and function names must use snake_case.
Classes should use PascalCase.
"""

[examples]
positive = ["my_variable = 1", "def my_function():"]
negative = ["myVariable = 1", "def myFunction():"]

[targets]
files = ["**/*.py"]
```

## 4. Presets (`presets/*.toml`)

Presets allow bulk-enabling of rules and tools.

> **Implementation note:** The current implementation provides built-in preset providers in `repo-presets`. The TOML schema below describes the design intent for user-defined presets.

**Example: `presets/python-agentic.toml`**

```toml
[meta]
description = "Standard setup for Python Agentic development"

[requires]
tools = ["cursor", "claude"]

[rules]
include = [
    "python-snake-case",
    "no-api-keys-in-code",
    "always-write-tests"
]
```

## 5. The Sync Process ("Unroll")

The `repo sync` command generates config files while preserving user changes through managed strategies.

**Text Files (e.g., `.cursorrules`)**:
We use "Managed Blocks". The CLI only edits content between specific markers.

```text
# .cursorrules

# User content can go here...
Always be concise.

# --- REPO-MANAGER-START: managed-rules --
# DO NOT EDIT THIS SECTION MANUALLY.
# Generated from: .repository/rules/

1. (python-snake-case) All Python variable names must use snake_case.
2. (no-api-keys) Never commit API keys.

# --- REPO-MANAGER-END ---
```

**JSON Files (e.g., `.claude/config.json`)**:
For JSON, we perform structured merges on manager-owned keys, appending or replacing labeled sections.

### State Tracking (`state.lock`)

To robustly handle `remove-rule` operations, we must know what we previously wrote.
`state.lock` tracks the mapping of `Rule ID` -> `Applied Checksum/Version`.

```toml
# .repository/state.lock
[sync_status]
last_run = "2026-01-23T12:00:00Z"

[installed_files]
".cursorrules" = "hash_of_managed_block"
".claude/config.json" = "hash_of_managed_json_keys"

[active_rules]
python-snake-case = "v1"
```

## Summary of Workflow

1. **User runs**: `repo add-rule "Use snake_case" --tag python`
2. **CLI creates**: `.repository/rules/use-snake-case.toml`
3. **User runs**: `repo sync`
4. **CLI**:
    * Reads `config.toml` -> sees `tools = ["cursor"]`.
    * Reads `tools/cursor.toml` -> sees `config_path = ".cursorrules"`, `type = "text"`.
    * Reads `rules/*.toml`.
    * Generates a text block containing the instructions.
    * Updates `.cursorrules` (between markers).
    * Updates `state.lock`.

This schema aligns with the Rust-based modular architecture and supports the requested modularity and unroll capabilities.

## 6. Rust Data Structures

The following structs show how the schema maps to Rust types. These match the actual implementation.

### Manifest (`repo-core::config::Manifest`)

```rust
/// Core configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreSection {
    /// Repository mode: "standard" or "worktrees"
    #[serde(default = "default_mode")]  // defaults to "worktrees"
    pub mode: String,
}

/// Repository configuration manifest parsed from config.toml
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Manifest {
    /// Core settings
    #[serde(default)]
    pub core: CoreSection,

    /// Preset configurations keyed by "type:name"
    /// e.g., "env:python", "tool:linter", "config:editor"
    #[serde(default)]
    pub presets: HashMap<String, serde_json::Value>,

    /// List of tool slugs to configure
    #[serde(default)]
    pub tools: Vec<String>,

    /// List of rule IDs to apply
    #[serde(default)]
    pub rules: Vec<String>,
}
```

### Tool Definition (`repo-meta::schema::ToolDefinition`)

```rust
/// Complete tool definition loaded from TOML
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolDefinition {
    pub meta: ToolMeta,
    pub integration: ToolIntegrationConfig,
    #[serde(default)]
    pub capabilities: ToolCapabilities,
    #[serde(default, rename = "schema")]
    pub schema_keys: Option<ToolSchemaKeys>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolMeta {
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolIntegrationConfig {
    pub config_path: String,
    #[serde(rename = "type")]
    pub config_type: ConfigType, // text, json, toml, yaml, markdown
    #[serde(default)]
    pub additional_paths: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ToolCapabilities {
    #[serde(default)]
    pub supports_custom_instructions: bool,
    #[serde(default)]
    pub supports_mcp: bool,
    #[serde(default)]
    pub supports_rules_directory: bool,
}
```

### Rule Definition (`repo-meta::schema::RuleDefinition`)

```rust
/// Complete rule definition loaded from TOML
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuleDefinition {
    pub meta: RuleMeta,
    pub content: RuleContent,
    #[serde(default)]
    pub examples: Option<RuleExamples>,
    #[serde(default)]
    pub targets: Option<RuleTargets>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuleMeta {
    pub id: String,
    #[serde(default)]
    pub severity: Severity,  // suggestion (default), mandatory
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuleContent {
    pub instruction: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RuleExamples {
    #[serde(default)]
    pub positive: Vec<String>,
    #[serde(default)]
    pub negative: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RuleTargets {
    #[serde(default, rename = "files")]
    pub file_patterns: Vec<String>,
}
```
