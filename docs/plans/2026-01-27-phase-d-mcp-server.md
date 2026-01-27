# Phase D: MCP Server Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Date:** 2026-01-27
**Priority:** Medium-High
**Estimated Tasks:** 10
**Dependencies:** Phase B (Core Completion)

---

## Goal

Create the `repo-mcp` crate that exposes Repository Manager functionality via the Model Context Protocol (MCP), enabling agentic tools like Claude Desktop, Windsurf, and Cursor to manage repositories programmatically.

---

## Prerequisites

- Phase B completed (working sync/fix operations)
- Understanding of MCP protocol (tools, resources, prompts)
- `rmcp` or `mcp-sdk` crate for Rust MCP implementation

---

## Architecture

```
MCP Client (Claude Desktop, Cursor, etc.)
       |
       | JSON-RPC over stdio
       v
+------------------+
|    repo-mcp      |
|  (MCP Server)    |
+------------------+
       |
       | Rust API calls
       v
+------------------+
|    repo-core     |
|  (Orchestration) |
+------------------+
       |
       +-- repo-fs
       +-- repo-git
       +-- repo-meta
       +-- repo-tools
       +-- repo-presets
```

---

## Specification Reference

From `docs/design/spec-mcp-server.md`:

### Tools to Implement

| Category | Tool | Arguments |
|----------|------|-----------|
| Lifecycle | `repo_init` | path, tools, mode, presets |
| Lifecycle | `repo_check` | (none) |
| Lifecycle | `repo_fix` | (none) |
| Lifecycle | `repo_sync` | (none) |
| Branch | `branch_create` | name, base |
| Branch | `branch_delete` | name |
| Branch | `branch_list` | (none) |
| Branch | `branch_checkout` | name |
| Git | `git_push` | remote, branch |
| Git | `git_pull` | remote, branch |
| Git | `git_merge` | target |
| Config | `tool_add` | name |
| Config | `tool_remove` | name |
| Config | `preset_add` | name |
| Config | `preset_remove` | name |
| Config | `rule_add` | id, instruction, tags |
| Config | `rule_modify` | id, instruction |
| Config | `rule_remove` | id |

### Resources to Implement

| URI | Description | MIME Type |
|-----|-------------|-----------|
| `repo://config` | Repository config | application/toml |
| `repo://state` | Ledger state | application/toml |
| `repo://rules` | Aggregated rules | text/markdown |

---

## Task D.1: Create Crate Structure

**Files:**
- Create: `crates/repo-mcp/Cargo.toml`
- Create: `crates/repo-mcp/src/lib.rs`
- Create: `crates/repo-mcp/src/main.rs`
- Modify: `Cargo.toml` (workspace)

**Step 1: Create Cargo.toml**

```toml
[package]
name = "repo-mcp"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "MCP server for Repository Manager"

[[bin]]
name = "repo-mcp"
path = "src/main.rs"

[dependencies]
repo-core = { path = "../repo-core" }
repo-fs = { path = "../repo-fs" }
repo-git = { path = "../repo-git" }
repo-meta = { path = "../repo-meta" }

# MCP SDK (adjust based on available crate)
rmcp = "0.1"  # or mcp-server-rs, etc.

# Async runtime
tokio = { workspace = true, features = ["full"] }
async-trait = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }
toml = { workspace = true }

# Error handling
thiserror = { workspace = true }
anyhow = "1"

# Logging
tracing = { workspace = true }
tracing-subscriber = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
```

**Step 2: Create lib.rs skeleton**

```rust
// crates/repo-mcp/src/lib.rs
//! MCP Server for Repository Manager
//!
//! Exposes repository management operations via the Model Context Protocol.

pub mod error;
pub mod server;
pub mod tools;
pub mod resources;

pub use error::{Error, Result};
pub use server::RepoMcpServer;
```

**Step 3: Create main.rs**

```rust
// crates/repo-mcp/src/main.rs
use repo_mcp::RepoMcpServer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Create and run server
    let server = RepoMcpServer::new()?;
    server.run_stdio().await?;

    Ok(())
}
```

**Step 4: Add to workspace**

In root `Cargo.toml`, add `"crates/repo-mcp"` to workspace members.

**Step 5: Verify compilation**

```bash
cargo build -p repo-mcp
```

**Step 6: Commit**

```bash
git add crates/repo-mcp/ Cargo.toml
git commit -m "feat(repo-mcp): create MCP server crate structure

Initial scaffolding for Repository Manager MCP server
with main binary and module structure.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task D.2: Implement Error Types

**Files:**
- Create: `crates/repo-mcp/src/error.rs`

```rust
// crates/repo-mcp/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Repository error: {0}")]
    Repo(#[from] repo_core::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("MCP protocol error: {0}")]
    Protocol(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Not a repository: {0}")]
    NotRepository(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<Error> for rmcp::ToolError {
    fn from(err: Error) -> Self {
        rmcp::ToolError::new(err.to_string())
    }
}
```

---

## Task D.3: Implement Server Core

**Files:**
- Create: `crates/repo-mcp/src/server.rs`

```rust
// crates/repo-mcp/src/server.rs
//! MCP Server implementation

use crate::tools::ToolRegistry;
use crate::resources::ResourceRegistry;
use crate::{Error, Result};
use repo_core::SyncEngine;
use repo_fs::NormalizedPath;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// The MCP server state
pub struct RepoMcpServer {
    /// Current working directory
    cwd: PathBuf,
    /// Tool registry
    tools: ToolRegistry,
    /// Resource registry
    resources: ResourceRegistry,
    /// Cached sync engine (if repository is initialized)
    engine: Arc<RwLock<Option<SyncEngine>>>,
}

impl RepoMcpServer {
    pub fn new() -> Result<Self> {
        let cwd = std::env::current_dir()?;

        Ok(Self {
            cwd: cwd.clone(),
            tools: ToolRegistry::new(),
            resources: ResourceRegistry::new(),
            engine: Arc::new(RwLock::new(None)),
        })
    }

    /// Initialize the sync engine for the current directory
    pub async fn init_engine(&self) -> Result<()> {
        let root = NormalizedPath::new(&self.cwd)?;
        let config_path = root.join(".repository/config.toml");

        if config_path.exists() {
            let config = repo_meta::RepositoryConfig::load(config_path.as_ref())?;
            let mode = repo_core::Mode::from_config(&config);
            let engine = SyncEngine::new(root, mode)?;

            let mut guard = self.engine.write().await;
            *guard = Some(engine);
        }

        Ok(())
    }

    /// Run the server using stdio transport
    pub async fn run_stdio(self) -> Result<()> {
        // Initialize engine if we're in a repository
        self.init_engine().await?;

        // Create MCP server
        let server = rmcp::Server::builder()
            .name("repo-mcp")
            .version(env!("CARGO_PKG_VERSION"))
            .build();

        // Register tools
        self.tools.register(&server)?;

        // Register resources
        self.resources.register(&server)?;

        // Run server
        server.run_stdio().await
            .map_err(|e| Error::Protocol(e.to_string()))?;

        Ok(())
    }
}
```

---

## Task D.4: Implement Lifecycle Tools

**Files:**
- Create: `crates/repo-mcp/src/tools/mod.rs`
- Create: `crates/repo-mcp/src/tools/lifecycle.rs`

```rust
// crates/repo-mcp/src/tools/lifecycle.rs
//! Repository lifecycle tools

use crate::{Error, Result};
use repo_core::{Mode, SyncEngine, SyncOptions};
use repo_fs::NormalizedPath;
use repo_meta::RepositoryConfig;
use rmcp::{tool, ToolResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct InitArgs {
    pub path: Option<String>,
    pub tools: Option<Vec<String>>,
    pub mode: Option<String>,
    pub presets: Option<Vec<String>>,
}

#[tool(name = "repo_init", description = "Initialize a new repository configuration")]
pub async fn repo_init(args: InitArgs) -> Result<ToolResult> {
    let path = args.path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let root = NormalizedPath::new(&path)?;
    let repo_dir = root.join(".repository");

    if repo_dir.exists() {
        return Err(Error::InvalidArgument(
            "Repository already initialized".to_string()
        ));
    }

    // Create .repository structure
    std::fs::create_dir_all(repo_dir.join("tools"))?;
    std::fs::create_dir_all(repo_dir.join("rules"))?;
    std::fs::create_dir_all(repo_dir.join("presets"))?;

    // Determine mode
    let mode = match args.mode.as_deref() {
        Some("standard") => Mode::Standard,
        _ => Mode::Worktrees,
    };

    // Create config
    let config = RepositoryConfig {
        mode,
        tools: args.tools.unwrap_or_default(),
        presets: args.presets.unwrap_or_default(),
        ..Default::default()
    };

    let config_str = toml::to_string_pretty(&config)?;
    std::fs::write(repo_dir.join("config.toml"), config_str)?;

    // Run initial sync
    let engine = SyncEngine::new(root, mode)?;
    let report = engine.sync()?;

    Ok(ToolResult::text(format!(
        "Repository initialized at {}.\nActions: {:?}",
        path.display(),
        report.actions
    )))
}

#[tool(name = "repo_check", description = "Check repository configuration for drift")]
pub async fn repo_check() -> Result<ToolResult> {
    let cwd = std::env::current_dir()?;
    let root = NormalizedPath::new(&cwd)?;

    let config_path = root.join(".repository/config.toml");
    if !config_path.exists() {
        return Err(Error::NotRepository(cwd.display().to_string()));
    }

    let config = RepositoryConfig::load(config_path.as_ref())?;
    let mode = Mode::from_config(&config);
    let engine = SyncEngine::new(root, mode)?;

    let report = engine.check()?;

    let status = match report.status {
        repo_core::CheckStatus::Healthy => "healthy",
        repo_core::CheckStatus::Drifted => "drifted",
        repo_core::CheckStatus::Missing => "missing",
        repo_core::CheckStatus::Broken => "broken",
    };

    let mut output = format!("Status: {}\n", status);

    if !report.drifted.is_empty() {
        output.push_str("\nDrifted:\n");
        for item in &report.drifted {
            output.push_str(&format!("  - {} ({}): {}\n", item.file, item.tool, item.description));
        }
    }

    if !report.missing.is_empty() {
        output.push_str("\nMissing:\n");
        for item in &report.missing {
            output.push_str(&format!("  - {} ({}): {}\n", item.file, item.tool, item.description));
        }
    }

    Ok(ToolResult::text(output))
}

#[tool(name = "repo_sync", description = "Synchronize tool configurations")]
pub async fn repo_sync() -> Result<ToolResult> {
    let cwd = std::env::current_dir()?;
    let root = NormalizedPath::new(&cwd)?;

    let config_path = root.join(".repository/config.toml");
    if !config_path.exists() {
        return Err(Error::NotRepository(cwd.display().to_string()));
    }

    let config = RepositoryConfig::load(config_path.as_ref())?;
    let mode = Mode::from_config(&config);
    let engine = SyncEngine::new(root, mode)?;

    let report = engine.sync()?;

    let mut output = if report.success {
        "Sync completed successfully.\n".to_string()
    } else {
        "Sync completed with errors.\n".to_string()
    };

    if !report.actions.is_empty() {
        output.push_str("\nActions:\n");
        for action in &report.actions {
            output.push_str(&format!("  - {}\n", action));
        }
    }

    if !report.errors.is_empty() {
        output.push_str("\nErrors:\n");
        for error in &report.errors {
            output.push_str(&format!("  - {}\n", error));
        }
    }

    Ok(ToolResult::text(output))
}

#[tool(name = "repo_fix", description = "Fix configuration drift automatically")]
pub async fn repo_fix() -> Result<ToolResult> {
    let cwd = std::env::current_dir()?;
    let root = NormalizedPath::new(&cwd)?;

    let config_path = root.join(".repository/config.toml");
    if !config_path.exists() {
        return Err(Error::NotRepository(cwd.display().to_string()));
    }

    let config = RepositoryConfig::load(config_path.as_ref())?;
    let mode = Mode::from_config(&config);
    let engine = SyncEngine::new(root, mode)?;

    let report = engine.fix()?;

    let mut output = if report.success {
        "Fix completed successfully.\n".to_string()
    } else {
        "Fix completed with errors.\n".to_string()
    };

    for action in &report.actions {
        output.push_str(&format!("  - {}\n", action));
    }

    Ok(ToolResult::text(output))
}
```

---

## Task D.5: Implement Branch Tools

**Files:**
- Create: `crates/repo-mcp/src/tools/branch.rs`

Implements `branch_create`, `branch_delete`, `branch_list`, `branch_checkout`.

---

## Task D.6: Implement Git Primitive Tools

**Files:**
- Create: `crates/repo-mcp/src/tools/git.rs`

Implements `git_push`, `git_pull`, `git_merge`.

---

## Task D.7: Implement Configuration Tools

**Files:**
- Create: `crates/repo-mcp/src/tools/config.rs`

Implements `tool_add`, `tool_remove`, `preset_add`, `preset_remove`, `rule_add`, `rule_modify`, `rule_remove`.

---

## Task D.8: Implement Resources

**Files:**
- Create: `crates/repo-mcp/src/resources/mod.rs`
- Create: `crates/repo-mcp/src/resources/config.rs`
- Create: `crates/repo-mcp/src/resources/state.rs`
- Create: `crates/repo-mcp/src/resources/rules.rs`

```rust
// crates/repo-mcp/src/resources/mod.rs
mod config;
mod state;
mod rules;

pub use config::ConfigResource;
pub use state::StateResource;
pub use rules::RulesResource;

pub struct ResourceRegistry;

impl ResourceRegistry {
    pub fn new() -> Self {
        Self
    }

    pub fn register(&self, server: &rmcp::Server) -> crate::Result<()> {
        // Register repo://config
        server.add_resource(ConfigResource::new())?;

        // Register repo://state
        server.add_resource(StateResource::new())?;

        // Register repo://rules
        server.add_resource(RulesResource::new())?;

        Ok(())
    }
}
```

---

## Task D.9: Integration Tests

**Files:**
- Create: `crates/repo-mcp/tests/integration_tests.rs`

Test the MCP server by simulating tool calls and verifying responses.

---

## Task D.10: Documentation and Claude Desktop Config

**Files:**
- Create: `docs/mcp-server-usage.md`

Document how to configure Claude Desktop to use the server:

```json
{
  "mcpServers": {
    "repo-mcp": {
      "command": "repo-mcp",
      "args": []
    }
  }
}
```

---

## Verification

```bash
# Build the server
cargo build -p repo-mcp --release

# Test manually with JSON-RPC
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | ./target/release/repo-mcp

# Run tests
cargo test -p repo-mcp
```

---

## Summary

| Task | Description | Risk | Effort |
|------|-------------|------|--------|
| D.1 | Crate structure | Low | Low |
| D.2 | Error types | Low | Low |
| D.3 | Server core | Medium | Medium |
| D.4 | Lifecycle tools | Medium | Medium |
| D.5 | Branch tools | Medium | Medium |
| D.6 | Git primitive tools | Medium | Medium |
| D.7 | Configuration tools | Medium | Medium |
| D.8 | Resources | Low | Medium |
| D.9 | Integration tests | Low | Medium |
| D.10 | Documentation | Low | Low |

**Total Effort:** ~2-3 days of focused work

**Note:** The actual MCP SDK crate name and API may differ. Adjust implementation based on the actual `rmcp` or alternative crate documentation.
