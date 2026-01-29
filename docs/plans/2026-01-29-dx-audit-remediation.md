# DX Audit Remediation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Close critical gaps identified in the 2026-01-29 Developer Experience Audit, making Repository Manager functional for real-world agentic development workflows.

**Architecture:** Phased approach - first make MCP minimally functional (critical), then add integration tests to verify tool sync works (high), then improve CLI usability (medium).

**Tech Stack:** Rust, rmcp crate (MCP SDK), tokio async runtime, insta for snapshot testing

---

## Phase 1: Minimal Functional MCP Server (CRITICAL)

### Task 1.1: Add MCP SDK Dependency

**Files:**
- Modify: `crates/repo-mcp/Cargo.toml`

**Step 1: Add rmcp dependency**

```toml
# Add after [dependencies] section, after existing deps
# MCP Protocol
rmcp = { version = "0.1", features = ["server", "transport-stdio"] }
```

**Step 2: Verify dependency resolves**

Run: `cargo check -p repo-mcp`
Expected: Compiles without errors (may have warnings)

**Step 3: Commit**

```bash
git add crates/repo-mcp/Cargo.toml
git commit -m "feat(repo-mcp): add rmcp MCP SDK dependency"
```

---

### Task 1.2: Implement MCP Server Protocol Handler

**Files:**
- Modify: `crates/repo-mcp/src/server.rs`
- Modify: `crates/repo-mcp/src/lib.rs`

**Step 1: Write test for server initialization**

Add to `crates/repo-mcp/src/server.rs` in the `#[cfg(test)]` module:

```rust
#[tokio::test]
async fn server_provides_server_info() {
    let server = RepoMcpServer::new(PathBuf::from("/tmp/test"));
    let info = server.server_info();
    assert_eq!(info.name, "repo-mcp");
    assert!(!info.version.is_empty());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p repo-mcp server_provides_server_info`
Expected: FAIL - `server_info` method not found

**Step 3: Implement server_info method**

In `crates/repo-mcp/src/server.rs`, add:

```rust
use rmcp::model::{ServerInfo, ServerCapabilities};

impl RepoMcpServer {
    /// Get MCP server information
    pub fn server_info(&self) -> ServerInfo {
        ServerInfo {
            name: "repo-mcp".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Get server capabilities
    pub fn capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            tools: Some(rmcp::model::ToolsCapability { list_changed: Some(false) }),
            resources: Some(rmcp::model::ResourcesCapability {
                subscribe: Some(false),
                list_changed: Some(false),
            }),
            ..Default::default()
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p repo-mcp server_provides_server_info`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/repo-mcp/src/server.rs
git commit -m "feat(repo-mcp): add server_info and capabilities methods"
```

---

### Task 1.3: Implement Tool Handler Dispatch

**Files:**
- Create: `crates/repo-mcp/src/handlers.rs`
- Modify: `crates/repo-mcp/src/lib.rs`

**Step 1: Write test for tool execution**

Create `crates/repo-mcp/src/handlers.rs`:

```rust
//! MCP tool handlers
//!
//! Dispatches tool calls to repo-core functionality.

use std::path::Path;
use serde_json::{json, Value};
use crate::{Error, Result};

/// Execute an MCP tool call
pub async fn execute_tool(
    root: &Path,
    tool_name: &str,
    arguments: Value,
) -> Result<Value> {
    match tool_name {
        "repo_check" => execute_repo_check(root).await,
        "repo_sync" => execute_repo_sync(root, arguments).await,
        _ => Err(Error::UnknownTool(tool_name.to_string())),
    }
}

async fn execute_repo_check(root: &Path) -> Result<Value> {
    use repo_core::{ConfigResolver, Mode, SyncEngine};
    use repo_fs::NormalizedPath;

    let normalized = NormalizedPath::new(root);
    let resolver = ConfigResolver::new(normalized.clone());

    let mode = if resolver.has_config() {
        let config = resolver.resolve().map_err(|e| Error::Core(e))?;
        config.mode.parse().unwrap_or(Mode::Worktrees)
    } else {
        Mode::Worktrees
    };

    let engine = SyncEngine::new(normalized, mode).map_err(|e| Error::Core(e))?;
    let report = engine.check().map_err(|e| Error::Core(e))?;

    Ok(json!({
        "status": format!("{:?}", report.status),
        "missing": report.missing.len(),
        "drifted": report.drifted.len(),
        "messages": report.messages,
    }))
}

async fn execute_repo_sync(root: &Path, arguments: Value) -> Result<Value> {
    use repo_core::{ConfigResolver, Mode, SyncEngine, SyncOptions};
    use repo_fs::NormalizedPath;

    let dry_run = arguments.get("dry_run")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let normalized = NormalizedPath::new(root);
    let resolver = ConfigResolver::new(normalized.clone());

    let mode = if resolver.has_config() {
        let config = resolver.resolve().map_err(|e| Error::Core(e))?;
        config.mode.parse().unwrap_or(Mode::Worktrees)
    } else {
        Mode::Worktrees
    };

    let engine = SyncEngine::new(normalized, mode).map_err(|e| Error::Core(e))?;
    let options = SyncOptions { dry_run };
    let report = engine.sync_with_options(options).map_err(|e| Error::Core(e))?;

    Ok(json!({
        "success": report.success,
        "actions": report.actions,
        "errors": report.errors,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_test_repo(dir: &Path) {
        fs::create_dir_all(dir.join(".git")).unwrap();
        fs::create_dir_all(dir.join(".repository")).unwrap();
        fs::write(
            dir.join(".repository/config.toml"),
            "tools = []\n\n[core]\nmode = \"standard\"\n",
        ).unwrap();
    }

    #[tokio::test]
    async fn test_execute_repo_check() {
        let temp = TempDir::new().unwrap();
        create_test_repo(temp.path());

        let result = execute_tool(temp.path(), "repo_check", json!({})).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.get("status").is_some());
    }

    #[tokio::test]
    async fn test_execute_unknown_tool() {
        let temp = TempDir::new().unwrap();
        let result = execute_tool(temp.path(), "unknown_tool", json!({})).await;
        assert!(result.is_err());
    }
}
```

**Step 2: Add UnknownTool error variant**

In `crates/repo-mcp/src/error.rs`, add:

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Server not initialized")]
    NotInitialized,

    #[error("Unknown tool: {0}")]
    UnknownTool(String),

    #[error(transparent)]
    Core(#[from] repo_core::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

**Step 3: Export handlers module**

In `crates/repo-mcp/src/lib.rs`, add:

```rust
pub mod handlers;
pub use handlers::execute_tool;
```

**Step 4: Run tests**

Run: `cargo test -p repo-mcp`
Expected: All tests pass

**Step 5: Commit**

```bash
git add crates/repo-mcp/src/handlers.rs crates/repo-mcp/src/error.rs crates/repo-mcp/src/lib.rs
git commit -m "feat(repo-mcp): implement tool handler dispatch for repo_check and repo_sync"
```

---

### Task 1.4: Implement Resource Handlers

**Files:**
- Create: `crates/repo-mcp/src/resource_handlers.rs`
- Modify: `crates/repo-mcp/src/lib.rs`

**Step 1: Create resource handler module**

Create `crates/repo-mcp/src/resource_handlers.rs`:

```rust
//! MCP resource handlers
//!
//! Provides read-only access to repository state.

use std::path::Path;
use crate::{Error, Result};

/// Read an MCP resource
pub async fn read_resource(root: &Path, uri: &str) -> Result<ResourceContent> {
    match uri {
        "repo://config" => read_config(root).await,
        "repo://state" => read_state(root).await,
        "repo://rules" => read_rules(root).await,
        _ => Err(Error::UnknownResource(uri.to_string())),
    }
}

pub struct ResourceContent {
    pub uri: String,
    pub mime_type: String,
    pub text: String,
}

async fn read_config(root: &Path) -> Result<ResourceContent> {
    let config_path = root.join(".repository/config.toml");
    let text = std::fs::read_to_string(&config_path)
        .unwrap_or_else(|_| "# No configuration found\n".to_string());

    Ok(ResourceContent {
        uri: "repo://config".to_string(),
        mime_type: "application/toml".to_string(),
        text,
    })
}

async fn read_state(root: &Path) -> Result<ResourceContent> {
    let ledger_path = root.join(".repository/ledger.toml");
    let text = std::fs::read_to_string(&ledger_path)
        .unwrap_or_else(|_| "# No ledger found\n".to_string());

    Ok(ResourceContent {
        uri: "repo://state".to_string(),
        mime_type: "application/toml".to_string(),
        text,
    })
}

async fn read_rules(root: &Path) -> Result<ResourceContent> {
    let rules_dir = root.join(".repository/rules");
    let mut content = String::from("# Active Rules\n\n");

    if rules_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&rules_dir) {
            for entry in entries.flatten() {
                if entry.path().extension().map_or(false, |e| e == "md") {
                    if let Ok(rule_content) = std::fs::read_to_string(entry.path()) {
                        content.push_str(&format!("## {}\n\n", entry.file_name().to_string_lossy()));
                        content.push_str(&rule_content);
                        content.push_str("\n\n---\n\n");
                    }
                }
            }
        }
    }

    if content == "# Active Rules\n\n" {
        content.push_str("_No rules defined._\n");
    }

    Ok(ResourceContent {
        uri: "repo://rules".to_string(),
        mime_type: "text/markdown".to_string(),
        text: content,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_read_config_exists() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(".repository")).unwrap();
        fs::write(
            temp.path().join(".repository/config.toml"),
            "tools = [\"cursor\"]\n",
        ).unwrap();

        let result = read_resource(temp.path(), "repo://config").await.unwrap();
        assert_eq!(result.mime_type, "application/toml");
        assert!(result.text.contains("cursor"));
    }

    #[tokio::test]
    async fn test_read_config_missing() {
        let temp = TempDir::new().unwrap();
        let result = read_resource(temp.path(), "repo://config").await.unwrap();
        assert!(result.text.contains("No configuration"));
    }

    #[tokio::test]
    async fn test_unknown_resource() {
        let temp = TempDir::new().unwrap();
        let result = read_resource(temp.path(), "repo://unknown").await;
        assert!(result.is_err());
    }
}
```

**Step 2: Add UnknownResource error variant**

In `crates/repo-mcp/src/error.rs`:

```rust
#[error("Unknown resource: {0}")]
UnknownResource(String),
```

**Step 3: Export module**

In `crates/repo-mcp/src/lib.rs`:

```rust
pub mod resource_handlers;
pub use resource_handlers::{read_resource, ResourceContent};
```

**Step 4: Run tests**

Run: `cargo test -p repo-mcp`
Expected: All tests pass

**Step 5: Commit**

```bash
git add crates/repo-mcp/src/resource_handlers.rs crates/repo-mcp/src/error.rs crates/repo-mcp/src/lib.rs
git commit -m "feat(repo-mcp): implement resource handlers for config, state, and rules"
```

---

### Task 1.5: Wire Up MCP Server Main Loop

**Files:**
- Modify: `crates/repo-mcp/src/server.rs`

**Step 1: Implement the run method with actual protocol handling**

Replace the `run` method in `crates/repo-mcp/src/server.rs`:

```rust
use rmcp::{ServiceExt, transport::stdio};
use crate::handlers::execute_tool;
use crate::resource_handlers::read_resource;

impl RepoMcpServer {
    /// Run the MCP server over stdio
    ///
    /// This starts the JSON-RPC message loop and handles
    /// tool calls and resource reads.
    pub async fn run(self) -> Result<()> {
        if !self.initialized {
            return Err(Error::NotInitialized);
        }

        tracing::info!(root = ?self.root, "Starting MCP server");

        // Create the MCP service
        let service = McpService::new(self.root.clone(), self.tools.clone(), self.resources.clone());

        // Run over stdio transport
        let transport = stdio::StdioTransport::new();

        service.serve(transport).await.map_err(|e| {
            tracing::error!("MCP server error: {}", e);
            Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;

        Ok(())
    }
}

/// MCP Service implementation
struct McpService {
    root: PathBuf,
    tools: Vec<ToolDefinition>,
    resources: Vec<crate::resources::ResourceDefinition>,
}

impl McpService {
    fn new(
        root: PathBuf,
        tools: Vec<ToolDefinition>,
        resources: Vec<crate::resources::ResourceDefinition>,
    ) -> Self {
        Self { root, tools, resources }
    }
}

#[rmcp::async_trait]
impl rmcp::Service for McpService {
    async fn initialize(&self, _params: rmcp::model::InitializeParams) -> rmcp::Result<rmcp::model::InitializeResult> {
        Ok(rmcp::model::InitializeResult {
            server_info: rmcp::model::ServerInfo {
                name: "repo-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            capabilities: rmcp::model::ServerCapabilities {
                tools: Some(rmcp::model::ToolsCapability { list_changed: Some(false) }),
                resources: Some(rmcp::model::ResourcesCapability {
                    subscribe: Some(false),
                    list_changed: Some(false),
                }),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn list_tools(&self) -> rmcp::Result<Vec<rmcp::model::Tool>> {
        Ok(self.tools.iter().map(|t| rmcp::model::Tool {
            name: t.name.clone(),
            description: Some(t.description.clone()),
            input_schema: t.input_schema.clone(),
        }).collect())
    }

    async fn call_tool(&self, params: rmcp::model::CallToolParams) -> rmcp::Result<rmcp::model::CallToolResult> {
        let result = execute_tool(&self.root, &params.name, params.arguments.unwrap_or_default()).await;

        match result {
            Ok(value) => Ok(rmcp::model::CallToolResult {
                content: vec![rmcp::model::Content::Text {
                    text: serde_json::to_string_pretty(&value).unwrap_or_default()
                }],
                is_error: None,
            }),
            Err(e) => Ok(rmcp::model::CallToolResult {
                content: vec![rmcp::model::Content::Text {
                    text: format!("Error: {}", e)
                }],
                is_error: Some(true),
            }),
        }
    }

    async fn list_resources(&self) -> rmcp::Result<Vec<rmcp::model::Resource>> {
        Ok(self.resources.iter().map(|r| rmcp::model::Resource {
            uri: r.uri.clone(),
            name: r.name.clone(),
            description: Some(r.description.clone()),
            mime_type: Some(r.mime_type.clone()),
        }).collect())
    }

    async fn read_resource(&self, params: rmcp::model::ReadResourceParams) -> rmcp::Result<rmcp::model::ReadResourceResult> {
        let content = read_resource(&self.root, &params.uri).await
            .map_err(|e| rmcp::Error::internal(e.to_string()))?;

        Ok(rmcp::model::ReadResourceResult {
            contents: vec![rmcp::model::ResourceContents::Text {
                uri: content.uri,
                mime_type: Some(content.mime_type),
                text: content.text,
            }],
        })
    }
}
```

**Step 2: Run compilation check**

Run: `cargo check -p repo-mcp`
Expected: Compiles (with possible warnings about unused)

**Step 3: Commit**

```bash
git add crates/repo-mcp/src/server.rs
git commit -m "feat(repo-mcp): implement MCP server main loop with stdio transport"
```

---

## Phase 2: Integration Tests for Tool Sync (HIGH)

### Task 2.1: Test Claude CLAUDE.md Output

**Files:**
- Create: `crates/repo-tools/tests/integration_claude.rs`

**Step 1: Write integration test**

```rust
//! Integration tests for Claude tool sync

use std::fs;
use tempfile::TempDir;
use repo_tools::{ToolDispatcher, ToolId};
use repo_meta::Rule;

#[test]
fn test_claude_sync_creates_claude_md() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    // Setup: create .repository structure
    fs::create_dir_all(root.join(".repository/rules")).unwrap();
    fs::write(
        root.join(".repository/config.toml"),
        "tools = [\"claude\"]\n\n[core]\nmode = \"standard\"\n",
    ).unwrap();

    // Create a rule
    let rule = Rule {
        id: "python-style".to_string(),
        instruction: "Use snake_case for all variable names.".to_string(),
        tags: vec!["python".to_string()],
        files: vec![],
    };

    // Execute sync
    let dispatcher = ToolDispatcher::new();
    let result = dispatcher.sync_tool(root, ToolId::Claude, &[rule]);

    assert!(result.is_ok(), "Sync should succeed: {:?}", result.err());

    // Verify CLAUDE.md was created
    let claude_md = root.join("CLAUDE.md");
    assert!(claude_md.exists(), "CLAUDE.md should be created");

    // Verify content
    let content = fs::read_to_string(&claude_md).unwrap();
    assert!(content.contains("snake_case"), "Should contain rule content");
    assert!(content.contains("repo:block"), "Should have managed block markers");
}

#[test]
fn test_claude_sync_preserves_user_content() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    // Setup: create existing CLAUDE.md with user content
    fs::write(
        root.join("CLAUDE.md"),
        "# My Project\n\nThis is my custom content.\n",
    ).unwrap();

    fs::create_dir_all(root.join(".repository")).unwrap();
    fs::write(
        root.join(".repository/config.toml"),
        "tools = [\"claude\"]\n\n[core]\nmode = \"standard\"\n",
    ).unwrap();

    let rule = Rule {
        id: "test-rule".to_string(),
        instruction: "Test instruction.".to_string(),
        tags: vec![],
        files: vec![],
    };

    let dispatcher = ToolDispatcher::new();
    let _ = dispatcher.sync_tool(root, ToolId::Claude, &[rule]);

    let content = fs::read_to_string(root.join("CLAUDE.md")).unwrap();
    assert!(content.contains("My Project"), "User content should be preserved");
    assert!(content.contains("Test instruction"), "Rule should be added");
}
```

**Step 2: Run test**

Run: `cargo test -p repo-tools --test integration_claude`
Expected: Tests reveal actual behavior (may fail, exposing real gaps)

**Step 3: Document findings and commit**

```bash
git add crates/repo-tools/tests/integration_claude.rs
git commit -m "test(repo-tools): add integration tests for Claude CLAUDE.md sync"
```

---

### Task 2.2: Test Cursor .mdc Output

**Files:**
- Create: `crates/repo-tools/tests/integration_cursor.rs`

**Step 1: Write integration test**

```rust
//! Integration tests for Cursor tool sync

use std::fs;
use tempfile::TempDir;
use repo_tools::{ToolDispatcher, ToolId};
use repo_meta::Rule;

#[test]
fn test_cursor_sync_creates_mdc_files() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    fs::create_dir_all(root.join(".repository")).unwrap();
    fs::write(
        root.join(".repository/config.toml"),
        "tools = [\"cursor\"]\n\n[core]\nmode = \"standard\"\n",
    ).unwrap();

    let rule = Rule {
        id: "python-style".to_string(),
        instruction: "Use type hints for all function parameters.".to_string(),
        tags: vec!["python".to_string()],
        files: vec!["*.py".to_string()],
    };

    let dispatcher = ToolDispatcher::new();
    let result = dispatcher.sync_tool(root, ToolId::Cursor, &[rule]);

    assert!(result.is_ok());

    // Verify .cursor/rules/ directory was created
    let rules_dir = root.join(".cursor/rules");
    assert!(rules_dir.exists(), ".cursor/rules/ should exist");

    // Verify .mdc file was created
    let mdc_file = rules_dir.join("python-style.mdc");
    assert!(mdc_file.exists(), "python-style.mdc should be created");

    // Verify .mdc format
    let content = fs::read_to_string(&mdc_file).unwrap();
    assert!(content.starts_with("---"), "Should have frontmatter");
    assert!(content.contains("globs:"), "Should have globs in frontmatter");
    assert!(content.contains("*.py"), "Should include file patterns");
    assert!(content.contains("type hints"), "Should contain rule instruction");
}

#[test]
fn test_cursor_mdc_frontmatter_format() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    fs::create_dir_all(root.join(".repository")).unwrap();
    fs::write(
        root.join(".repository/config.toml"),
        "tools = [\"cursor\"]\n\n[core]\nmode = \"standard\"\n",
    ).unwrap();

    let rule = Rule {
        id: "always-active".to_string(),
        instruction: "Always use this rule.".to_string(),
        tags: vec!["global".to_string()],
        files: vec![], // Empty = always apply
    };

    let dispatcher = ToolDispatcher::new();
    let _ = dispatcher.sync_tool(root, ToolId::Cursor, &[rule]);

    let content = fs::read_to_string(root.join(".cursor/rules/always-active.mdc")).unwrap();

    // Parse frontmatter
    assert!(content.contains("alwaysApply: true") || content.contains("alwaysApply:true"),
        "Empty files array should result in alwaysApply: true");
}
```

**Step 2: Run test**

Run: `cargo test -p repo-tools --test integration_cursor`
Expected: Tests reveal actual behavior

**Step 3: Commit**

```bash
git add crates/repo-tools/tests/integration_cursor.rs
git commit -m "test(repo-tools): add integration tests for Cursor .mdc sync"
```

---

### Task 2.3: Test Copilot instructions.md Output

**Files:**
- Create: `crates/repo-tools/tests/integration_copilot.rs`

**Step 1: Write integration test**

```rust
//! Integration tests for GitHub Copilot tool sync

use std::fs;
use tempfile::TempDir;
use repo_tools::{ToolDispatcher, ToolId};
use repo_meta::Rule;

#[test]
fn test_copilot_sync_creates_instructions_file() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    fs::create_dir_all(root.join(".repository")).unwrap();
    fs::write(
        root.join(".repository/config.toml"),
        "tools = [\"copilot\"]\n\n[core]\nmode = \"standard\"\n",
    ).unwrap();

    let rule = Rule {
        id: "code-style".to_string(),
        instruction: "Follow the Google style guide.".to_string(),
        tags: vec![],
        files: vec![],
    };

    let dispatcher = ToolDispatcher::new();
    let result = dispatcher.sync_tool(root, ToolId::Copilot, &[rule]);

    assert!(result.is_ok());

    // Verify .github/copilot-instructions.md was created
    let instructions = root.join(".github/copilot-instructions.md");
    assert!(instructions.exists(), ".github/copilot-instructions.md should exist");

    let content = fs::read_to_string(&instructions).unwrap();
    assert!(content.contains("Google style guide"), "Should contain rule content");
}
```

**Step 2: Run and commit**

Run: `cargo test -p repo-tools --test integration_copilot`

```bash
git add crates/repo-tools/tests/integration_copilot.rs
git commit -m "test(repo-tools): add integration tests for Copilot instructions sync"
```

---

## Phase 3: CLI Usability Improvements (MEDIUM)

### Task 3.1: Add `repo status` Command

**Files:**
- Modify: `crates/repo-cli/src/cli.rs`
- Create: `crates/repo-cli/src/commands/status.rs`
- Modify: `crates/repo-cli/src/commands/mod.rs`
- Modify: `crates/repo-cli/src/main.rs`

**Step 1: Add Status command to CLI enum**

In `crates/repo-cli/src/cli.rs`, add to `Commands` enum:

```rust
/// Show repository status overview
Status,
```

**Step 2: Create status command implementation**

Create `crates/repo-cli/src/commands/status.rs`:

```rust
//! Status command implementation
//!
//! Shows an overview of repository configuration state.

use std::path::Path;
use colored::Colorize;
use repo_core::{ConfigResolver, Mode, SyncEngine};
use repo_fs::NormalizedPath;
use crate::context::{detect_context, RepoContext};
use crate::error::{CliError, Result};

/// Run the status command
pub fn run_status(path: &Path) -> Result<()> {
    let context = detect_context(path);

    match context {
        RepoContext::NotARepo => {
            println!("{} Not in a repository", "Status:".yellow().bold());
            println!("  Run {} to initialize", "repo init".cyan());
            return Ok(());
        }
        RepoContext::ContainerRoot { path: root } |
        RepoContext::Worktree { container: root, .. } |
        RepoContext::StandardRepo { path: root } => {
            print_status(&root)?;
        }
    }

    Ok(())
}

fn print_status(root: &Path) -> Result<()> {
    let normalized = NormalizedPath::new(root);
    let resolver = ConfigResolver::new(normalized.clone());

    println!("{}", "Repository Status".green().bold());
    println!();

    // Mode
    let mode = if resolver.has_config() {
        let config = resolver.resolve()?;
        config.mode.parse().unwrap_or(Mode::Worktrees)
    } else {
        Mode::Worktrees
    };
    println!("  {}: {}", "Mode".bold(), format!("{:?}", mode).cyan());

    // Root path
    println!("  {}: {}", "Root".bold(), root.display());

    // Config file
    let config_path = root.join(".repository/config.toml");
    if config_path.exists() {
        println!("  {}: {}", "Config".bold(), "exists".green());

        // Parse and show tools
        if let Ok(config) = resolver.resolve() {
            if !config.tools.is_empty() {
                println!("  {}: {}", "Tools".bold(), config.tools.join(", ").yellow());
            } else {
                println!("  {}: {}", "Tools".bold(), "none".dimmed());
            }
        }
    } else {
        println!("  {}: {}", "Config".bold(), "missing".red());
    }

    // Rules
    let rules_dir = root.join(".repository/rules");
    if rules_dir.exists() {
        let count = std::fs::read_dir(&rules_dir)
            .map(|entries| entries.filter_map(|e| e.ok()).count())
            .unwrap_or(0);
        println!("  {}: {}", "Rules".bold(), format!("{} defined", count).yellow());
    } else {
        println!("  {}: {}", "Rules".bold(), "none".dimmed());
    }

    // Sync status
    println!();
    let engine = SyncEngine::new(normalized, mode)?;
    let report = engine.check()?;

    match report.status {
        repo_core::CheckStatus::Healthy => {
            println!("  {}: {}", "Sync".bold(), "healthy".green());
        }
        repo_core::CheckStatus::Missing => {
            println!("  {}: {} ({} missing)", "Sync".bold(), "incomplete".yellow(), report.missing.len());
        }
        repo_core::CheckStatus::Drifted => {
            println!("  {}: {} ({} drifted)", "Sync".bold(), "drifted".red(), report.drifted.len());
        }
        repo_core::CheckStatus::Broken => {
            println!("  {}: {}", "Sync".bold(), "broken".red().bold());
        }
    }

    Ok(())
}
```

**Step 3: Export from mod.rs**

In `crates/repo-cli/src/commands/mod.rs`:

```rust
pub mod status;
pub use status::run_status;
```

**Step 4: Wire up in main.rs**

In `crates/repo-cli/src/main.rs`, add to `execute_command`:

```rust
Commands::Status => cmd_status(),
```

And add the function:

```rust
fn cmd_status() -> Result<()> {
    let cwd = std::env::current_dir()?;
    commands::run_status(&cwd)
}
```

**Step 5: Test and commit**

Run: `cargo build -p repo-cli && ./target/debug/repo status`

```bash
git add crates/repo-cli/src/cli.rs crates/repo-cli/src/commands/status.rs crates/repo-cli/src/commands/mod.rs crates/repo-cli/src/main.rs
git commit -m "feat(repo-cli): add status command for repository overview"
```

---

### Task 3.2: Add `repo diff` Command

**Files:**
- Modify: `crates/repo-cli/src/cli.rs`
- Create: `crates/repo-cli/src/commands/diff.rs`
- Modify: `crates/repo-cli/src/commands/mod.rs`
- Modify: `crates/repo-cli/src/main.rs`

**Step 1: Add Diff command**

In `crates/repo-cli/src/cli.rs`:

```rust
/// Preview what sync would change
Diff,
```

**Step 2: Implement diff command**

Create `crates/repo-cli/src/commands/diff.rs`:

```rust
//! Diff command implementation
//!
//! Shows what changes sync would make without applying them.

use std::path::Path;
use colored::Colorize;
use repo_core::{ConfigResolver, Mode, SyncEngine, SyncOptions};
use repo_fs::NormalizedPath;
use crate::commands::sync::resolve_root;
use crate::error::Result;

/// Run the diff command
pub fn run_diff(path: &Path) -> Result<()> {
    println!("{} Previewing sync changes...", "=>".blue().bold());

    let root = resolve_root(path)?;
    let resolver = ConfigResolver::new(root.clone());

    let mode = if resolver.has_config() {
        let config = resolver.resolve()?;
        config.mode.parse().unwrap_or(Mode::Worktrees)
    } else {
        Mode::Worktrees
    };

    let engine = SyncEngine::new(root, mode)?;
    let options = SyncOptions { dry_run: true };
    let report = engine.sync_with_options(options)?;

    if report.actions.is_empty() {
        println!("{} No changes needed", "OK".green().bold());
        return Ok(());
    }

    println!();
    println!("{}", "Changes that would be made:".bold());
    for action in &report.actions {
        // Parse action to determine type
        if action.contains("create") || action.contains("Create") {
            println!("  {} {}", "+".green(), action);
        } else if action.contains("update") || action.contains("Update") {
            println!("  {} {}", "~".yellow(), action);
        } else if action.contains("delete") || action.contains("Delete") {
            println!("  {} {}", "-".red(), action);
        } else {
            println!("  {} {}", "*".cyan(), action);
        }
    }

    println!();
    println!("Run {} to apply these changes.", "repo sync".cyan());

    Ok(())
}
```

**Step 3: Wire up and test**

Follow same pattern as status command.

```bash
git add crates/repo-cli/src/cli.rs crates/repo-cli/src/commands/diff.rs crates/repo-cli/src/commands/mod.rs crates/repo-cli/src/main.rs
git commit -m "feat(repo-cli): add diff command for sync preview"
```

---

## Completion Checklist

After implementing all tasks:

- [ ] MCP server starts and responds to initialize request
- [ ] `repo_check` tool callable via MCP
- [ ] `repo_sync` tool callable via MCP
- [ ] Resources readable via MCP
- [ ] Claude CLAUDE.md sync verified by integration test
- [ ] Cursor .mdc sync verified by integration test
- [ ] Copilot instructions sync verified by integration test
- [ ] `repo status` shows repository overview
- [ ] `repo diff` previews sync changes

---

## Notes for Implementer

1. **rmcp crate**: If `rmcp` is not available, alternatives are `mcp-server` or implementing JSON-RPC manually. Check crates.io for current MCP implementations.

2. **Test failures are expected**: Integration tests may fail initially - that's the point. They expose real gaps in tool sync.

3. **ToolDispatcher API**: The tests assume a `sync_tool(root, tool_id, rules)` method. Adjust to actual API.

4. **Worktree awareness**: All commands use `resolve_root()` to handle both standard and worktree modes.

---

*Plan created: 2026-01-29*
*Based on: docs/audits/2026-01-29-developer-experience-audit.md*
