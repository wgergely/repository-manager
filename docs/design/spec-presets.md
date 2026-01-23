# Presets Subsystem Specification

**Crate**: `repo-presets`

## 1. Overview

The Presets subsystem is the execution engine of the Repository Manager. While the `repo-meta` crate *reads* the configuration, `repo-presets` *enacts* it. It manages the lifecycle of "Capabilities" (Providers).

## 2. Typology

Presets are categorized by their functionality domain as described in the [Providers Reference](providers-reference.md).

* **`env:*`**: Environment Providers (Python Virtual Envs, Node modules).
* **`config:*`**: Configuration Providers (EditorConfig, GitIgnore).
* **`tool:*`**: Tooling Providers (Ruff, Cargo, Pytest).

## 3. The `PresetProvider` Trait

This is the core interface for any capability.

```rust
#[async_trait]
pub trait PresetProvider {
    /// The unique identifier (e.g., "env:python")
    fn id(&self) -> &str;

    /// 1. Check: Measure the difference between Current State and Desired State.
    /// Returns a report of what is missing/broken.
    async fn check(&self, context: &Context) -> Result<CheckReport>;

    /// 2. Apply: Execute operations to bring system to Desired State.
    async fn apply(&self, context: &Context) -> Result<ApplyReport>;
}
```

### 3.1 The `CheckReport`

A structured diagnosis:

```rust
pub struct CheckReport {
    pub status: PresetStatus, // Healthy, Missing, Drifted, Broken
    pub details: Vec<String>, // "venv missing", "pip outdated"
    pub remedial_action: ActionType, // None, Install, Repair
}
```

## 4. Helper Logic (The SDK)

The subsystem provides a toolkit for common operations to make building providers easy:

* **`PythonHelper`**: `locate_python()`, `create_venv(path)`, `pip_install(packages)`.
* **`FileHelper`**: `ensure_line_in_file(path, line)`, `write_template(path, template_data)`.
* **`DownloadHelper`**: `download_binary(url, hash)`.

## 5. Preset Resolution Logic

When `repo sync` runs:

1. It fetches the list of active preset strings (e.g., `["env:python", "tool:ruff"]`).
2. It asks the `Registry` (from `repo-meta`) for the matching Provider structs.
3. It builds a **Dependency Graph**.
    * *Example*: `tool:ruff` depends on `env:python`.
4. It executes `check()` and then `apply()` in topological order.

## 6. Built-in vs. Dynamic Providers

* **Built-in**: Compiled into the binary (e.g., standard Python/Rust/Node support).
* **Dynamic**: Defined purely in TOML?
  * *Current Design*: Only simple "Config Providers" (writes this file content) can be purely dynamic. Complex logic (installing python) requires a compiled Provider.

## 7. Interaction with git-worktrees

Providers must be context-aware.

* **Global Installation**: Installing a tool to `~/.cargo/bin` or `.bin` in the container root.
* **Local Installation**: Creating a `venv` inside the specific worktree `{root}/feat-branch/venv`.

The `Context` struct passed to `apply()` contains the `WorkspaceLayout` (from `repo-fs`) so the provider knows where to target.
