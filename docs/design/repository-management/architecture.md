# Repository Manager Architecture

## Overview

The **Repository Manager** is a Rust-based system designed to orchestrate the structure and configuration of development workspaces. It abstracts over the underlying physical layout of the repository to support different "modes" of operation while providing a unified API for tool integration and context management.

## Core Concepts

### 1. Modes

The system operates in two distinct modes. The chosen mode determines how files, configurations, and git branches are physically mapped to the filesystem.

#### A. Normal Mode (Standard Git)

* **Description**: Traditional single-directory Git repository.
* **Structure**: Project root contains `.git`, source code, and configuration files directly.
* **Config Strategy**: Tool configuration files (e.g., `.vscode`, `.claude`, `.cursor`) are committed directly to history in the main branch.

#### B. Worktrees Mode (Container Strategy)

* **Description**: A "container" folder hosting the bare repository (or hidden `.git` dir) and multiple sibling directories for worktrees.
* **Structure**:

    ```text
    my-project/                  # Container Folder
    ├── .git/                    # Git Data (or pointer to bare repo)
    ├── .repository/             # Repository Manager Metadata
    ├── .claude/                 # Global/Shared Tool Configs
    ├── main/                    # Primary Worktree (Main Branch)
    ├── feature-a/               # Feature Worktree (Feature Branch)
    └── venv/                    # Shared Environment (optional)
    ```

* **Config Strategy**: Tool configurations can exist at the container level (shared) or within specific worktrees. The manager handles syncing or referencing these configs.

### 2. The Repository Manager Crate (`repo-manager`)

Technical backend implemented as a separated Rust crate.

#### Responsibilities

1. **File Handling Abstraction**:
    * Unified path resolution regardless of mode.
    * "Project Root" resolution (container vs. git root).
2. **Git Integration**:
    * Wrapper around Git operations (using `git2` crate or CLI).
    * Worktree registration and pruning.
    * Branch management (creation, deletion, checkout).
3. **Mode Agnostic Logic**:
    * Traits/Interfaces for `RepositoryBackend`.
    * Implementations: `StandardBackend`, `WorktreeBackend`.
    * Calls like `manager.create_feature("feat-x")` automatically handle:
        * **Standard**: `git checkout -b feat-x`
        * **Worktree**: `git worktree add ../feat-x feat-x` + config bootstrap.

## Logic Flow

### Initialization

When `repo init` is called:

1. Detect existing git status.
2. If empty, initialize based on `--mode`.
3. Write `.repository/config.toml` (or similar) to store mode and active tools.

### Branch Operations

Operations are abstracted to handle mode differences:

| Operation | Standard Mode | Worktree Mode |
| :--- | :--- | :--- |
| **Add Branch** | `git checkout -b <name>` | `git worktree add <path>/<name> <name>` |
| **Remove Branch** | `git branch -d <name>` | `git worktree remove <path>/<name>` (+ dir cleanup) |
| **Pull/Push** | Standard git commands | Standard git commands inside the worktree |
| **Merge** | Standard merge | Merge in target worktree |

## Technical Components

* **`config` module**: Parses and serializes `.repository` metadata.
* **`git` module**: Handles low-level git interactions (optionally via `git2-rs`).
* **`fs` module**: Safe file operations, ensuring atomic writes for config updates.
* **`tooling` module**: Logic for detecting and configuring supported tools (VSCode, Claude, etc.).
