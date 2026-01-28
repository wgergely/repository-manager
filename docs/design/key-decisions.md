# Architecture & Strategy: Key Design Decisions

This document captures high-level architectural decisions made during the design of the `repository-manager` project.

## Git Operations Implementation

- **Library Selection**: **`gix`** (gitoxide) is preferred over `git2` for core operations due to its modern architecture, focus on performance, and Rust-native safety.
- **Pattern**: Standardized on **Git Worktrees** for managing independent features/branches, enabling isolated environments without context-switching costs.

## Plugin & Extensibility Architecture

- **Decision**: **Separate Binary Plugins** (as subprocesses).
- **Rationale**:
  - **Native Performance**: No overhead compared to WASM.
  - **Language Agnostic**: Plugins can be written in any language.
  - **Simplicity**: Follows the proven Git subcommand pattern.
  - **Full System Access**: Essential for developer-centric tools interacting deep with the OS.
- **Trade-offs**: Sacrifices sandboxing (security) for power and ease of development. Access should be controlled via discovery from trusted directories (e.g., `~/.config/repo-manager/plugins`).

## Core Crate Structure

- **`repo-fs`**: Abstracted file system operations and path normalization.
- **`repo-git`**: Low-level Git manipulations via `gix`.
- **`repo-meta`**: Metadata management and configuration loading.
- **`repo-tools`**: Integration with IDEs and external tools (VSCode, Cursor).
- **`repo-presets`**: Management of project types and tool configurations.
