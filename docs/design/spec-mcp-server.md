# MCP Server Specification

## Overview

This document specifies the design for `repo-mcp`, a Rust crate that exposes the functionality of the Repository Manager as a Model Context Protocol (MCP) server.

The goal is to provide Agentic IDEs (like Claude Desktop, Windsurf, Cursor) with first-class primitives to manage the repository structure, configuration, and strictly controlled Git operations without relying on fragile shell commands.

## Architecture

The `repo-mcp` crate acts as a facade layer over the core `repo-manager` library.

```text
[ MCP Client (Claude/IDE) ] 
       | (JSON-RPC)
       v
[ repo-mcp (MCP Server) ]
       | (Rust API)
       v
[ repo-manager (Core Logic) ]
       |
       +--> [ .repository/ (Config Store) ]
       +--> [ git / worktrees (Filesystem) ]
```

## Tools Specification

The server exposes the following tools, categorized by domain.

### 1. Repository Lifecycle

Tools to manage the fundamental state of the workspace.

| Tool Name | Arguments | Description |
| :--- | :--- | :--- |
| `repo_init` | `path` (string), `tools` (array\<string\>), `mode` (string: "worktrees"\|"standard"), `presets` (array\<string\>) | Initializes a new repository configuration. |
| `repo_check` | *None* | Checks for valid configuration and consistency between metadata and filesystem. |
| `repo_fix` | *None* | Attempts to repair inconsistency (e.g., pruning dead worktrees). |
| `repo_sync` | *None* | Regenerates tool configurations (e.g., `.cursorrules`, `settings.json`) based on current state. |

### 2. Feature & Branch Management

Abstracted branch operations that handle the underlying complexity of Worktrees vs. Standard Git.

| Tool Name | Arguments | Description |
| :--- | :--- | :--- |
| `branch_create` | `name` (string), `base` (optional string) | Creates a new branch. In `worktrees` mode, also creates the sibling directory and initializes it. |
| `branch_delete` | `name` (string) | Safely removes a branch and its associated worktree/directory. |
| `branch_list` | *None* | Returns a JSON list of active branches and their worktree paths (if applicable). |
| `branch_checkout` | `name` (string) | **Standard Mode Only**: Checks out the branch. **Worktree Mode**: Returns the path to the existing worktree. |

### 3. Git Primitives

Safe wrappers around Git commands to ensure they run in the correct context.

| Tool Name | Arguments | Description |
| :--- | :--- | :--- |
| `git_push` | `remote` (optional string), `branch` (optional string) | Pushes the current context's branch. |
| `git_pull` | `remote` (optional string), `branch` (optional string) | Pulls updates. |
| `git_merge` | `target` (string) | Merges the target branch into the current context. |

### 4. Configuration Management

Tools to modify the agentic capabilities of the repository (`.repository` state).

| Tool Name | Arguments | Description |
| :--- | :--- | :--- |
| `tool_add` | `name` (string) | Enables a tool (e.g., "claude", "cursor") in `config.toml`. Triggers implicit sync. |
| `tool_remove` | `name` (string) | Disables a tool. |
| `preset_add` | `name` (string) | Applies a preset stack (e.g., "python-web"). |
| `preset_remove` | `name` (string) | Removes a preset. |
| `rule_add` | `id` (string), `instruction` (string), `tags` (array\<string\>), `files` (array\<string\>) | Adds a new custom rule to `rules/`. |
| `rule_modify` | `id` (string), `instruction` (string) | Modifies an existing rule's instruction. |
| `rule_remove` | `id` (string) | Deletes a rule definition. |

## Resources Specification

The server exposes read-only resources to allow agents to inspect the repository state efficiently.

| Resource URI | Description | content-type |
| :--- | :--- | :--- |
| `repo://config` | The contents of `.repository/config.toml` | `application/toml` |
| `repo://state` | The computed state from `.repository/ledger.toml` | `application/toml` |
| `repo://rules` | A aggregated view of all active rules | `text/markdown` |

## Rust Implementation Plan

The generic `mcp-rust-sdk` (or similar compliant library) will be used.

```rust
// Draft struct for implementation
pub struct RepoMcpServer {
    core: Arc<RepoManager>,
}

#[mcp_server::tool(name = "branch_create")]
pub async fn branch_create(state: State<RepoMcpServer>, args: BranchCreateArgs) -> Result<ToolResult, Error> {
    state.core.create_branch(&args.name, args.base.as_deref())?;
    Ok(ToolResult::text(format!("Branch {} created at {}", args.name, path)))
}
```

### Dependencies

- `repo-core`: The core logic crate.
- `mcp-sdk`: For protocol implementation.
- `tokio`: For async runtime.
