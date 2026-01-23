# Superpowers Plan

## Goal

Create detailed Technical Specifications for the **Preset Management System**.
This is a **Research & Design** task. No crates or code will be implemented. The output will be comprehensive Markdown specifications in `docs/design/`.

## Assumptions

- The "System" is a generic Orchestrator (`repo-preset`) managing modular Providers.
- We need to spec the **Rust Traits**, **Configuration Schemas**, and **Behavioral Logic** (Lifecycle).
- The "Skeleton" mentioned earlier refers to the *Architectural Skeleton* (diagrams/signatures), not actual code files.

## Plan

1. **Define Core Interfaces (The "Meta-System")**
    - **Files**: `docs/design/spec-core-interfaces.md`
    - **Change**: Define the `PresetProvider` Rust Trait signatures, the `Registry` logic, and the `Preset` struct structure. Define proper error handling and lifecycle methods (`install`, `verify`, `repair`).
    - **Verify**: `Select-String "trait PresetProvider" docs/design/spec-core-interfaces.md`

2. **Define Configuration & Metadata Schema**
    - **Files**: `docs/design/spec-configuration.md`
    - **Change**: Define the TOML schema for `repo.toml`/`.repository/config.toml`. Explain how Presets are declared, configured, and how dependencies (e.g., `python:backend` includes `env:uv`) are modeled.
    - **Verify**: `Select-String "\[presets\]" docs/design/spec-configuration.md`

3. **Spec Concrete Providers (Case Studies)**
    - **Files**: `docs/design/spec-providers.md`
    - **Change**: Detail the implementation logic for:
        - `env-python` (venv/conda abstraction).
        - `config-*` (file template providers).
        - `tool-*` (binary downloaders).
    - **Verify**: `Select-String "env-python" docs/design/spec-providers.md`

4. **Consolidate & Review**
    - **Files**: `docs/design/README.md`
    - **Change**: Create an index file linking these specs and summarizing the architectural vision.
    - **Verify**: `Test-Path docs/design/README.md`

## Risks & mitigations

- **Risk**: Spec becoming too abstract.
  - **Mitigation**: Use concrete "User Stories" in `spec-providers.md` to ground the design (e.g., "User runs `repo preset add python`...").
- **Risk**: Overlap with existing `preset-manager.md`.
  - **Mitigation**: Validated specs will supersede previous high-level docs. I will mark `preset-manager.md` as deprecated or merge it.

## Rollback plan

- Delete `docs/design/spec-*.md`
