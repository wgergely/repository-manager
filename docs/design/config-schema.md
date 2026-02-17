# Repository Configuration Schema

## Overview

The `.repository` directory serves as the **Single Source of Truth (SSOT)** for the workspace. The Repository Manager CLI uses definitions in this directory to generate, synchronize, and validate the configuration files required by various agentic tools (Claude, Cursor, VSCode, etc.).

## Directory Structure

We adopt a modular, file-based configuration approach using **TOML** for its strong typing and readability.

```text
.repository/
├── config.toml           # The primary manifest (Enabled tools, presets, mode)
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

## 2. Tool Registration (`tools/*.toml`)

Each file in `tools/` defines a tool's capabilities and how the manager should interact with it.

**Example: `tools/claude.toml`**

```toml
[meta]
name = "Claude Desktop"
slug = "claude-desktop"
description = "Anthropic's Claude Desktop App"

[integration]
# Helper methods the CLI uses to find the config
config_path = ".claude/config.json"
type = "json"

[capabilities]
# Does this tool support system prompt injection?
supports_custom_instructions = true
# Does it support MCP servers?
supports_mcp = true

[schema.keys]
# Mapping generic concepts to tool-specific JSON keys
instruction_key = "global_instructions"
mcp_key = "mcpServers"
```

**Example: `tools/cursor.toml`**

```toml
[meta]
name = "Cursor"
slug = "cursor"

[integration]
config_path = ".cursorrules"
type = "text" # Plain text file

[capabilities]
supports_custom_instructions = true
supports_mcp = false
```

## 3. Rule Definitions (`rules/*.toml`)

Rules capture specific behaviors, constraints, or stylistic preferences. They are abstract enough to be unrolled to different tools.

**Example: `rules/code-style-python.toml`**

```toml
[meta]
id = "python-snake-case"
severity = "mandatory" # mandatory, suggestion
tags = ["python", "style"]

[content]
# The core instruction text
instruction = """
All Python variable names and function names must use snake_case. 
Classes should use PascalCase.
"""

[examples]
# Provide few-shot examples that tools can use
positive = ["my_variable = 1", "def my_function():"]
negative = ["myVariable = 1", "def myFunction():"]

[targets]
# Paths this rule applies to
files = ["**/*.py"]
```

## 4. Presets (`presets/*.toml`)

Presets allow bulk-enabling of rules and tools.

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
    * Reads `config.toml` -> sees `tools=["cursor"]`.
    * Reads `tools/cursor.toml` -> sees `config_path=".cursorrules"`, `type="text"`.
    * Reads `rules/*.toml`.
    * Generates a text block containing the instructions.
    * Updates `.cursorrules` (between markers).
    * Updates `state.lock`.

This schema aligns with the Rust-based modular architecture and supports the requested modularity and unroll capabilities.

## 6. Rust Data Structures

These are the actual structs from the codebase (see `repo-core/src/config/manifest.rs`).

```rust
// config.toml -> Manifest (repo-core/src/config/manifest.rs)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(default)]
    pub core: CoreSection,
    #[serde(default)]
    pub presets: HashMap<String, Value>,  // keyed by "type:name", e.g. "env:python"
    #[serde(default)]
    pub tools: Vec<String>,              // top-level array
    #[serde(default)]
    pub rules: Vec<String>,              // top-level array
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreSection {
    #[serde(default = "default_mode")]  // defaults to "worktrees"
    pub mode: String,
}

// tools/*.toml -> ToolDefinition (repo-meta/src/schema.rs)
#[derive(Debug, Deserialize, Serialize)]
pub struct ToolDefinition {
    pub meta: ToolMeta,
    pub integration: ToolIntegrationConfig,
    pub capabilities: ToolCapabilities,
    pub schema_keys: Option<SchemaKeys>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolIntegrationConfig {
    pub config_path: String,
    pub config_type: ConfigType,  // Enum: Json, Text, Toml
    pub additional_paths: Vec<String>,
}
```
