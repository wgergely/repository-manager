# Repository Manager Design Documentation

This directory contains the design specifications and architectural documents for the Repository Manager.

## 1. Architecture

High-level overview of the system and its core components.

- **[Core Architecture](architecture-core.md)**: Overview of the system modes (Normal vs. Worktrees) and the `repo-manager` crate responsibilities.
- **[Preset System Architecture](architecture-presets.md)**: Deep dive into the "Meta-System" pattern, decomposing capabilities into Presets and Providers.

## 2. Configuration System

Details on how the repository is configured, how configuration is cascaded, and how state is tracked.

- **[Configuration Strategy](config-strategy.md)**: The hierarchical merge strategy (Global -> Repo -> Local) and dependency modeling.
- **[Configuration Schema](config-schema.md)**: The concrete TOML schema for `.repository/config.toml`, `tools/*.toml`, and `rules/*.toml`.
- **[State Ledger](config-ledger.md)**: The "unrolling" mechanism and state tracking via `.repository/ledger.toml`.

## 3. Component Specifications

Detailed specifications for individual subsystems and modules.

- **[CLI Specification](spec-cli.md)**: command-line interface design and argument structure.
- **[API Schema](spec-api-schema.md)**: Unified tool configuration management and TypeScript entity definitions.
- **[Metadata System](spec-metadata.md)**: The structure and responsibilities of the `.repository` directory.
- **[Git Management](spec-git.md)**: Abstraction layer for Git operations in both standard and worktree modes.
- **[File Management](spec-fs.md)**: I/O utilities for robust file handling across modes.
- **[Presets Specification](spec-presets.md)**: Interfaces and logic for the capabilities system.
- **[Tools Specification](spec-tools.md)**: Integration logic for external tools (VSCode, Claude, etc.).
- **[MCP Server Specification](spec-mcp-server.md)**: Design for the Rust-based MCP server exposing repository management tools.

## 4. Reference & Research

Background information and reference material.

- **[Providers Reference](providers-reference.md)**: Reference implementations for Python, Config, and Tool providers.
- **[Context Research](research-context.md)**: Analysis of agentic context management patterns (context for the 2026 design).
