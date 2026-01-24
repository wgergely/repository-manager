# Repository Manager CLI Specification

## Overview

CLI specification for Repository Manager. Primary entry point for workspace configuration, tool management, and version control.

## Command Structure

The binary usually runs as `repo` (or specific crate name).

### 1. Initialization

Initialize a new repository or reconfigure an existing one.

```bash
repo init [OPTIONS]
```

**Options:**

* `--tools <TOOLS...>`
  * List of specific tools/IDEs to enable.
  * *Values*: `claude`, `claude-desktop`, `antigravity`, `windsurf`, `cursor`, `vscode`, `gemini-cli`.
* `--mode <MODE>`
  * Use specific physical layout strategy.
  * *Values*: `default` (Standard Git), `worktrees` (Container folders).
  * *Default*: `worktrees` (as per user preference).
* `--presets <PRESETS...>`
  * Apply a collection of configurations for a specific language or stack.
  * *Examples*:
    * `python`: Sets up VSCode Python settings, creates `venv`.
    * `rust`: Sets up Rust-Analyzer, Cargo defaults.
    * `web`: Node/TS configurations.

**Example:**

```bash
repo init --mode worktrees --presets python --tools claude vscode
```

### 2. Tool & Preset Management

Modify the active configuration of the repository.

```bash
repo add-tool <TOOL_NAME>
repo remove-tool <TOOL_NAME>
```

```bash
repo add-preset <PRESET_NAME>
repo remove-preset <PRESET_NAME>
```

### 3. Repository Metadata

Manage the internal state and consistency of the repository metadata (`.repository` folder).

```bash
repo check
# Checks for inconsistencies (e.g., config mentions a worktree that was deleted manually).

repo fix
# Attempts to auto-repair inconsistencies (e.g., pruning dead worktree references).

repo sync
# Synchronizes central info with tool-specific config files (e.g., regenerating .vscode/settings.json based on active preset).
```

### 4. Branch & Workspace Management

Abstracted Git operations that respect the active Mode.

```bash
repo branch add <BRANCH_NAME> [BASE]
# Worktree Mode: Creates new folder {worktrees}/{branch_name}, inits worktree.
# Standard Mode: git checkout -b {branch_name}

repo branch remove <BRANCH_NAME>
# Safe cleanup of branch and associated worktree folder (if applicable).

repo branch list
# Lists available branches/worktrees.
```

### 5. Git Wrappers

Convenience wrappers that ensure operations happen in the correct context.

```bash
repo push [remote] [branch]
repo pull [remote] [branch]
repo merge <target>
```

## Detailed Behavior

### `repo init` implementation details

1. **Check Context**: Are we in an empty folder? An existing git repo?
2. **Create Metadata**: Initialize `.repository/state.json` (or similar).
3. **Apply Mode**:
    * If `worktrees`:
        * Ensure `.git` is bare or move it to `.git-root`.
        * Create `main` worktree if not exists.
4. **Apply Tools/Presets**:
    * Download/Generate config files for requested tools.
    * Run preset setup scripts (e.g., `python -m venv venv`).

### `repo sync` implementation details

* Iterates through all registered tools.
* Regenerates configuration files if the `.repository` source of truth has changed.
* Example: If `python` preset is added, `sync` ensures VSCode `settings.json` has `python.defaultInterpreterPath` set correctly.
