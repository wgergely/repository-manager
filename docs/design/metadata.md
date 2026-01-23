# System Crystallization & Correlation

This document serves as the high-level map correlating the various design documents.

> **Status**: Early Design Phase. Architecture is evolving.

## Subsystem Index

* **[CLI](cli/spec.md)**: Top-level command line tool implementation.
* **[Tools](tools/spec.md)**: Definition and registration of external tools (coding agents, IDEs).
* **[Presets](presets/spec.md)**: Capability provider system (venvs, gitignores, configs).
* **[Repository Management](repository-management/architecture.md)**: Core logic for repository structure.
  * *[Context Research](repository-management/skill-context-management-2026.md)*: Analysis of agentic context patterns.
* **[Metadata System](metadata-system/spec.md)**: The `.repository` directory structure and registry.
* **[File Management](file-management/spec.md)**: Robust I/O utilities.
* **[Git Management](git-management/spec.md)**: Worktree and remote sync management.

## Correlation Goals

* **Presets** provide the capabilities.
* **Tools** consume the environment configured by Presets.
* **Metadata System** connects them by registering which tools and presets are active in a repository.
* **CLI** is the conductor that orchestrates these interactions.
