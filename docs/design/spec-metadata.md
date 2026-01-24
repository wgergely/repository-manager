# Metadata System Specification

**Crate**: `repo-meta`

## 1. Overview

The Metadata System is the database of the Repository Manager. It is responsible for parsing, validating, and managing the `.repository` directory. It acts as the Registry for all known Tools, Rules, and Presets.

## 2. Directory Structure Ownership

The subsystem owns the definition of the `.repository` folder:

```text
.repository/
├── config.toml       # The Root Manifest
├── state.lock        # The State Ledger
├── tools/            # Tool Definitions
├── rules/            # Rule Definitions
└── presets/          # Preset Definitions
```

## 3. Core Responsibilities

### 3.1 Configuration Resolution

The subsystem implements the "Cascade" logic described in [Configuration Strategy](config-strategy.md).

```rust
pub struct ResolvedConfig {
    pub mode: RepositoryMode,
    pub active_tools: Vec<ToolDefinition>,
    pub active_presets: Vec<PresetDefinition>,
    pub rules: Vec<RuleDefinition>,
}

impl MetadataStore {
    /// Loads config.toml and recursively resolves all referenced tools and presets
    pub fn load_resolved(&self) -> Result<ResolvedConfig>;
}
```

### 3.2 Validity Checking

Before any synchronization happens, the Metadata System validates the graph:

1. **Dependency Check**: Do we have a provider for every requested preset?
2. **Conflict Check**: Do two tools claim the same config file? (e.g., two Linters trying to write `.pylintrc`).
3. **Schema Check**: Are the `*.toml` files syntactically correct?

### 3.3 The Registry Concept

The `Registry` is the runtime catalog of available components.

* **Built-in Registry**: Hardcoded support for common tools (VSCode, Claude).
* **Local Registry**: `.repository/tools/*.toml` files.
* **Remote Registry**: (Future) Fetching definitions from a shared URL.

## 4. State Ledger Management

The metadata subsystem manages the `ledger.toml`.

* **Recording Intents**: specific methods to add `Intent` entries when a tool writes a file.
* **Querying State**: `get_projections_for_file(path)` -> Returns which rules modified this file.

## 5. Metadata API

Other crates consume this crate to answer questions:

* *"What is the Python version?"* -> `meta.get_preset("env:python").property("version")`
* *"Is Claude enabled?"* -> `meta.is_tool_enabled("claude")`

## 6. Migration & Versioning

The `config.toml` includes a `version = "1.0"` field. The Metadata System is responsible for:

* Detecting outdated versions.
* Performing migrations (e.g., renaming keys) if the schema evolves.
