# Architecture Design: Modular Preset Management System (`repo-preset`)

## 1. Executive Summary

The `repo-preset` crate is the core orchestration engine for the Repository Manager's capability system. Instead of monolithic managers for specific languages, it defines a generic **Provider Architecture**.

**Presets** are high-level capabilities (e.g., "Python Data Science Stack", "Standard Gitignores") that are fulfilled by discrete **Providers** (e.g., `python-uv`, `python-conda`, `template-file`).

This system acts as the "Meta-Manager" that handles:

1. **Registration**: Knowing which tools/configs are active.
2. **Lifecycle**: Installation, Updates, Health Checks, and Removal.
3. **Interoperability**: Ensuring presets don't conflict.

## 2. Core Concepts

### 2.1 The Preset

A **Preset** is a named configuration unit.

- **ID**: `namespace:name` (e.g., `python:uv`, `config:gitignore`).
- **State**: `Active` | `Inactive` | `Broken`.
- **Config**: Arbitrary JSON/TOML configuration specific to the provider.

### 2.2 The Provider

A **Provider** is a Rust implementation (struct) responsible for realizing a specific family of presets.

- **Example**: `PythonEnvProvider` handles `python:uv` and `python:conda`.
- **Example**: `FileTemplateProvider` handles `config:gitignore` and `config:editorconfig`.

## 3. Architecture

### 3.1 The Crate (`repo-preset`)

The generic library crate that defines the interfaces.

```rust
// The core trait that all providers must implement
pub trait PresetProvider {
    /// Returns the provider's namespace (e.g. "python")
    fn namespace(&self) -> &str;
    
    /// Checks if this provider handles the specific preset variant
    fn supports(&self, variant: &str) -> bool;

    /// Idempotent installation/application of the preset
    fn apply(&self, preset: &Preset, ctx: &Context) -> Result<ApplyResult>;

    /// Checks for drift or broken state
    fn check(&self, preset: &Preset, ctx: &Context) -> Result<CheckResult>;

    /// Returns metadata for Agents (paths, env vars, etc.)
    fn info(&self, preset: &Preset) -> serde_json::Value;
}
```

### 3.2 The Registry

A centralized registry inside `RepositoryManager` holds all available providers.

```rust
pub struct PresetRegistry {
    providers: HashMap<String, Box<dyn PresetProvider>>,
}
```

## 4. Workflows

### 4.1 "Adding" a Preset

User runs: `repo preset add python:uv`

1. **Lookup**: Registry finds provider for namespace `python`.
2. **Validation**: Provider confirms it supports variant `uv`.
3. **Config Update**: `repo` adds entry to `.repository/config.toml`.
4. **Application**: `provider.apply(preset)` is called.
    - *PythonProvider*: Downloads `uv`, creates venv, writes `pyproject.toml`.
    - *FileProvider*: Writes `.gitignore`.

### 4.2 "Checking" State

User runs: `repo check`

1. Iterates all enabled presets in `config.toml`.
2. Calls `provider.check(preset)`.
3. Aggregates results (e.g., "OK", "Venv missing", "Gitignore drifted").

## 5. Configuration Schema

### 5.1 `.repository/config.toml`

The "Manifest" of what is supposed to be installed.

```toml
[presets]
# Format: "namespace:variant" = { config }

"python:uv" = { version = "3.12", lock = true }
"config:gitignore" = { lang = "python", strict = true }
"tool:ruff" = { version = "latest" }
```

## 6. Migration & Extensibility

- **New Languages**: Add a `NodeProvider` in a new module.
- **New Tools**: Add a `ToolProvider` that downloads binaries.
- **Complex Stacks**: A "Meta-Preset" (handled by `MacroProvider`) can simply expand into multiple child presets (e.g., `stack:backend` -> `python:uv`, `tool:ruff`, `tool:docker`).

## 7. Integration with Agents

The system exposes a `repo preset info --json` command.
Agents consume this to know:

- Where the generic interpreter is (`.venv/Scripts/python`).
- What linter rules are active.
- Which files are managed/read-only.
