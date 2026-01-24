# Superpowers Brainstorm

## Goal

Architect a modular **Preset Management System** in Rust for the `repository-manager`. This system must handle diverse capabilities (venvs, gitignores, linter configs) via discrete **Providers**. It serves as the "Meta-System" for managing interconnected configuration, updates, and registration of tools across different languages (Python, Node, Rust) and types (Env, Config, Tooling).

## Constraints

- **Modular Providers**: Must support distinct providers like `python-venv`, `python-conda`, `node-modules`, `gitignore-python`.
- **Composition**: Presets should be able to depend on or imply others (e.g., `preset:python` -> includes `venv`, `gitignore`, `black`).
- **Unified Interface**: A single CLI surface (`repo preset add ...`) that delegates to the correct provider.
- **State Management**: Must track what is installed, its version/hash, and manage updates/repairs.
- **Language Agnostic**: The core system cares about "Presets" and "Providers", not specific languages.

## Known context

- User rejected the "monolithic python venv tool" idea.
- The system deals with `{type}-{lang}` pairs (e.g., `env-python`, `ignore-python`).
- A "Preset" is likely a higher-order concept that groups these lower-level capabilities.
- We need a **Registry** to track available providers.
- We need a **Configuration Schema** that allows providers to declare their inputs/outputs.

## Risks

- **Over-Abstraction**: Creating a system so generic it's hard to implement simple things like "write a file".
- **Dependency Hell**: Resolving conflicts if two presets want to manage the same file (e.g., conflicting `gitignore` rules).
- **State Drift**: Tracking "installed" presets vs "actual" filesystem state.

## Options

1. **Trait-Based Plugin System** (Rust):
    - Define a `PresetProvider` trait in a core crate.
    - Implement specific providers in separate crates (`preset-python`, `preset-node`).
    - Core loads them (statically linked for now, strictly).
    - *Pros*: Type-safe, fast.
    - *Cons*: Recompile to add providers.

2. **Declarative "Recipe" System**:
    - Presets are just YAML/TOML files describing files to write and commands to run.
    - A single "Engine" interprets these recipes.
    - *Pros*: extremely easy to add new simple presets (gitignores).
    - *Cons*: dynamic behavior (venv logic, complex updates) is strictly limited or requires messy scripting.

3. **Hybrid "Handler" System (Recommended)**:
    - Core defines a `Manifest` of installed presets.
    - specialized "Handlers" (Rust modules/crates) register for specific `namespaces` (e.g., `python/*`).
    - Simple things (gitignores) are consistent "Templates". Complex things (venvs) are code.

## Recommendation

**Option 3: Hybrid Handler Architecture**.

- **Core Crate (`repo_presets`)**: Defines the `PresetManager`, `Registry`, and `Preset` trait.
- **Providers (Crates)**:
  - `provider_venv`: Implements logic for `python-venv`, `python-conda`.
  - `provider_file`: Implements logic for `gitignore`, `dockerignore` (template based).
- **Schema**:
  - `PresetId`: `provider:variant` (e.g., `env:uv`, `config:python-git`).
  - `Lifecycle`: `install()`, `update()`, `check()`, `remove()`.

## Acceptance criteria

- [ ] Architecture design for `repo_presets` crate.
- [ ] Diagram/Description of the `PresetProvider` Trait.
- [ ] Definition of the `Preset` struct (metadata, config).
- [ ] Strategy for "Meta-management" (how `repo` coordinates updates across all active presets).
