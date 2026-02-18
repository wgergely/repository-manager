# Repository Configuration Schema

## Overview

The `.repository` directory serves as the **Single Source of Truth (SSOT)** for the workspace. The Repository Manager CLI uses definitions in this directory to generate, synchronize, and validate the configuration files required by various agentic tools (Claude, Cursor, VSCode, etc.).

## Directory Structure

We adopt a modular, file-based configuration approach using **TOML** for the manifest and **Markdown** for rule files.

```text
.repository/
├── config.toml           # The primary manifest (enabled tools, presets, mode)
├── tools/                # Custom tool definitions (TOML); built-in tools are compiled in
│   └── my-custom-tool.toml
├── rules/                # Rule files created by `repo add-rule`
│   ├── python-style.md
│   └── no-api-keys.md
└── presets/              # Custom preset definitions (TOML); built-in presets are compiled in
    └── my-custom-preset.toml
```

## 1. The Manifest (`config.toml`)

This file defines the high-level configuration of the repository. It is parsed into the `Manifest` struct in `repo-core/src/config/manifest.rs`.

**Important**: `tools` and `rules` are top-level arrays that must appear before any `[section]` headers in the TOML file. (In TOML, once a `[section]` header appears, subsequent keys belong to that section. Placing `tools` and `rules` first ensures they are parsed as top-level fields of the `Manifest` struct, not nested under `[core]`.) There is no `[active]`, `[project]`, or `[sync]` section.

```toml
# Top-level arrays (must appear before [core])
tools = ["claude", "cursor", "vscode"]
rules = ["python-style", "no-api-keys"]

[core]
# "standard" or "worktrees" (default: "worktrees")
mode = "worktrees"

[presets]
# Preset configurations keyed by "type:name"
"env:python" = { version = "3.12" }
"rust" = {}
```

### Manifest Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `tools` | `string[]` | No | `[]` | List of tool slugs to enable (e.g., `"claude"`, `"vscode"`) |
| `rules` | `string[]` | No | `[]` | List of rule IDs to apply |
| `core.mode` | `string` | No | `"worktrees"` | Workspace mode: `"standard"` or `"worktrees"` |
| `presets.<key>` | `table` | No | - | Preset configurations keyed by `"type:name"` |

> **Note:** The `tools` and `rules` arrays must appear before any `[section]` headers in the TOML file, since they are top-level keys.

## 2. Tool Registration

The 13 built-in tools are compiled into the binary via `repo-tools::registry`. They do not require TOML files on disk. Custom tools can be placed in `.repository/tools/` as TOML files and are loaded by the `DefinitionLoader` in `repo-meta`.

The `ToolDefinition` struct (in `repo-meta::schema`) defines the schema for both built-in and custom tool definitions:

**Example: `tools/my-custom-tool.toml`**

```toml
[meta]
name = "My Custom Tool"
slug = "my-custom-tool"
description = "A custom tool integration"

[integration]
config_path = ".my-tool-config"
type = "text"
additional_paths = []

[capabilities]
supports_custom_instructions = true
supports_mcp = false
supports_rules_directory = false

[schema]
instruction_key = "instructions"
```

### Built-in Tool Examples

The Claude integration uses `CLAUDE.md` (markdown type) and `.claude/rules/` as its additional path. The Cursor integration uses `.cursorrules` (text type).

### Supported Config Types

| Type | Extension | Description |
|------|-----------|-------------|
| `text` | `.cursorrules`, etc. | Plain text files with managed block markers |
| `json` | `.json` | Structured JSON merge with `__repo_managed__` key |
| `toml` | `.toml` | TOML configuration files |
| `yaml` | `.yaml`, `.yml` | YAML configuration files |
| `markdown` | `.md` | Markdown files (e.g., `CLAUDE.md`) |

## 3. Rule Files (`rules/*.md`)

Rules capture specific behaviors, constraints, or stylistic preferences. The CLI `add-rule` command creates rules as Markdown files in `.repository/rules/`.

**Example: creating a rule via CLI**

```bash
repo add-rule python-snake-case "All Python variable names must use snake_case." --tag python --tag style
```

This creates `.repository/rules/python-snake-case.md`:

```markdown
tags: python, style

All Python variable names must use snake_case.
```

The `DefinitionLoader` in `repo-meta` also supports loading structured TOML rule definitions from `.repository/rules/*.toml` for advanced use cases. The TOML format matches the `RuleDefinition` struct:

**Example: `rules/python-snake-case.toml`**

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

Presets allow bulk-enabling of rules and tools. Built-in preset providers are compiled into the binary via `repo-presets`. Custom presets can be placed in `.repository/presets/` as TOML files.

**Example: `presets/python-agentic.toml`**

```toml
[meta]
id = "python-agentic"
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

The `repo sync` command generates tool config files while preserving user content through managed blocks.

Each rule is assigned a UUID (stored in `.repository/rules/registry.toml`). The UUID is used as the managed block marker in tool config files, allowing individual rules to be inserted, updated, or removed independently.

### Managed Block Formats

**Markdown/Text Files** (e.g., `CLAUDE.md`, `.cursorrules`):

Uses HTML comment markers with UUID-based block identifiers:

```markdown
# User content can go here...
Always be concise.

<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->
All Python variable names must use snake_case.
<!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 -->
```

**Markdown Files (CLAUDE.md)**:
For Markdown, we use managed blocks with `<!-- repo:block:{id} -->` markers, same as `.cursorrules`.

**YAML/TOML Files**:

Uses hash-comment markers:

```yaml
# repo:block:550e8400-e29b-41d4-a716-446655440000
some_key: managed_value
# /repo:block:550e8400-e29b-41d4-a716-446655440000
```

**JSON Files** (e.g., `.vscode/settings.json`):

Uses a reserved `__repo_managed__` key to store managed blocks:

```json
{
  "user_setting": true,
  "__repo_managed__": {
    "550e8400-e29b-41d4-a716-446655440000": {
      "content": "managed value"
    }
  }
}
```

## Summary of Workflow

1. **User runs**: `repo add-rule python-style "Use snake_case" --tag python`
2. **CLI creates**: `.repository/rules/python-style.md`
3. **User runs**: `repo sync`
4. **CLI**:
    * Reads `config.toml` -> sees `tools = ["cursor"]`.
    * Looks up tool integration (built-in or from `tools/*.toml`) -> gets `config_path = ".cursorrules"`, `type = "text"`.
    * Reads rules from `.repository/rules/`.
    * Generates managed blocks containing the rule instructions (keyed by UUID).
    * Updates `.cursorrules` (inserting/updating managed blocks, preserving user content).

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
