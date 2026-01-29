# MCP Server Completion Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Transform the MCP skeleton into a fully functional server that AI agents can use to manage repositories.

**Architecture:** Implement MCP protocol using rmcp SDK (or manual JSON-RPC if unavailable). Server runs over stdio, exposes tools that delegate to repo-core, and resources that read repository state.

**Tech Stack:** Rust, rmcp/mcp-server crate, tokio, serde_json, repo-core

---

## Prerequisites

Before starting, verify:
```bash
# Check current MCP state
cargo check -p repo-mcp

# Verify repo-core compiles
cargo check -p repo-core
```

---

## Task 1: Add MCP Protocol Dependency

**Files:**
- Modify: `crates/repo-mcp/Cargo.toml`

**Step 1: Research available MCP crates**

Run: `cargo search mcp --limit 10`

If `rmcp` or `mcp-server` exists, use it. Otherwise, we'll implement JSON-RPC manually.

**Step 2: Add dependency (option A - if rmcp exists)**

```toml
# Add to [dependencies] section
rmcp = { version = "0.1", features = ["server", "transport-stdio"] }
```

**Step 2 (alt): Add dependency (option B - manual JSON-RPC)**

```toml
# Add to [dependencies] section
jsonrpc-core = "18.0"
```

**Step 3: Verify compilation**

Run: `cargo check -p repo-mcp`
Expected: Compiles with new dependency

**Step 4: Commit**

```bash
git add crates/repo-mcp/Cargo.toml Cargo.lock
git commit -m "feat(repo-mcp): add MCP protocol dependency"
```

---

## Task 2: Define MCP Message Types

**Files:**
- Create: `crates/repo-mcp/src/protocol.rs`
- Modify: `crates/repo-mcp/src/lib.rs`

**Step 1: Write test for message serialization**

```rust
// Add to bottom of protocol.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_request_deserialize() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0"}
            }
        }"#;
        let request: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.method, "initialize");
    }

    #[test]
    fn test_tool_call_result_serialize() {
        let result = ToolCallResult {
            content: vec![Content::Text { text: "Success".to_string() }],
            is_error: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Success"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p repo-mcp test_initialize_request`
Expected: FAIL - module not found

**Step 3: Implement protocol types**

Create `crates/repo-mcp/src/protocol.rs`:

```rust
//! MCP Protocol message types
//!
//! JSON-RPC 2.0 message structures for MCP communication.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 Request
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError { code, message, data: None }),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Initialize request params
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    pub protocol_version: String,
    #[serde(default)]
    pub capabilities: ClientCapabilities,
    pub client_info: ClientInfo,
}

#[derive(Debug, Deserialize, Default)]
pub struct ClientCapabilities {}

#[derive(Debug, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// Initialize response result
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    pub server_info: ServerInfo,
}

#[derive(Debug, Serialize)]
pub struct ServerCapabilities {
    pub tools: Option<ToolsCapability>,
    pub resources: Option<ResourcesCapability>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsCapability {
    pub list_changed: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcesCapability {
    pub subscribe: Option<bool>,
    pub list_changed: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

/// Tool definition for tools/list response
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Value,
}

/// Tool call params
#[derive(Debug, Deserialize)]
pub struct ToolCallParams {
    pub name: String,
    #[serde(default)]
    pub arguments: Value,
}

/// Tool call result
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallResult {
    pub content: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// Content types
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Content {
    Text { text: String },
}

/// Resource definition
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

/// Resource read params
#[derive(Debug, Deserialize)]
pub struct ReadResourceParams {
    pub uri: String,
}

/// Resource read result
#[derive(Debug, Serialize)]
pub struct ReadResourceResult {
    pub contents: Vec<ResourceContents>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ResourceContents {
    Text {
        uri: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
        text: String,
    },
}
```

**Step 4: Export module**

In `crates/repo-mcp/src/lib.rs`:

```rust
pub mod protocol;
```

**Step 5: Run tests**

Run: `cargo test -p repo-mcp protocol`
Expected: All tests pass

**Step 6: Commit**

```bash
git add crates/repo-mcp/src/protocol.rs crates/repo-mcp/src/lib.rs
git commit -m "feat(repo-mcp): add MCP protocol message types"
```

---

## Task 3: Implement Tool Handlers

**Files:**
- Create: `crates/repo-mcp/src/handlers.rs`
- Modify: `crates/repo-mcp/src/lib.rs`
- Modify: `crates/repo-mcp/src/error.rs`

**Step 1: Write test for repo_check handler**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_test_repo(dir: &std::path::Path) {
        fs::create_dir_all(dir.join(".git")).unwrap();
        fs::create_dir_all(dir.join(".repository")).unwrap();
        fs::write(
            dir.join(".repository/config.toml"),
            "tools = []\n\n[core]\nmode = \"standard\"\n",
        ).unwrap();
    }

    #[tokio::test]
    async fn test_handle_repo_check() {
        let temp = TempDir::new().unwrap();
        create_test_repo(temp.path());

        let result = handle_tool_call(temp.path(), "repo_check", serde_json::json!({})).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.get("status").is_some());
    }

    #[tokio::test]
    async fn test_handle_unknown_tool() {
        let temp = TempDir::new().unwrap();
        let result = handle_tool_call(temp.path(), "unknown", serde_json::json!({})).await;
        assert!(result.is_err());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p repo-mcp test_handle_repo_check`
Expected: FAIL - function not found

**Step 3: Implement handlers**

Create `crates/repo-mcp/src/handlers.rs`:

```rust
//! MCP Tool Handlers
//!
//! Executes tool calls by delegating to repo-core functionality.

use std::path::Path;
use serde_json::{json, Value};
use crate::{Error, Result};

/// Handle a tool call
pub async fn handle_tool_call(
    root: &Path,
    tool_name: &str,
    arguments: Value,
) -> Result<Value> {
    match tool_name {
        "repo_check" => handle_repo_check(root).await,
        "repo_sync" => handle_repo_sync(root, arguments).await,
        "repo_fix" => handle_repo_fix(root, arguments).await,
        "repo_init" => handle_repo_init(root, arguments).await,
        "branch_list" => handle_branch_list(root).await,
        "branch_create" => handle_branch_create(root, arguments).await,
        "branch_delete" => handle_branch_delete(root, arguments).await,
        "tool_add" => handle_tool_add(root, arguments).await,
        "tool_remove" => handle_tool_remove(root, arguments).await,
        "rule_add" => handle_rule_add(root, arguments).await,
        "rule_remove" => handle_rule_remove(root, arguments).await,
        _ => Err(Error::UnknownTool(tool_name.to_string())),
    }
}

async fn handle_repo_check(root: &Path) -> Result<Value> {
    use repo_core::{ConfigResolver, Mode, SyncEngine};
    use repo_fs::NormalizedPath;

    let normalized = NormalizedPath::new(root);
    let mode = resolve_mode(&normalized)?;

    let engine = SyncEngine::new(normalized, mode)?;
    let report = engine.check()?;

    Ok(json!({
        "status": format!("{:?}", report.status),
        "missing_count": report.missing.len(),
        "drifted_count": report.drifted.len(),
        "missing": report.missing.iter().map(|d| json!({
            "file": d.file,
            "tool": d.tool,
            "description": d.description,
        })).collect::<Vec<_>>(),
        "drifted": report.drifted.iter().map(|d| json!({
            "file": d.file,
            "tool": d.tool,
            "description": d.description,
        })).collect::<Vec<_>>(),
        "messages": report.messages,
    }))
}

async fn handle_repo_sync(root: &Path, arguments: Value) -> Result<Value> {
    use repo_core::{SyncEngine, SyncOptions};
    use repo_fs::NormalizedPath;

    let dry_run = arguments.get("dry_run")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let normalized = NormalizedPath::new(root);
    let mode = resolve_mode(&normalized)?;

    let engine = SyncEngine::new(normalized, mode)?;
    let options = SyncOptions { dry_run };
    let report = engine.sync_with_options(options)?;

    Ok(json!({
        "success": report.success,
        "dry_run": dry_run,
        "actions": report.actions,
        "errors": report.errors,
    }))
}

async fn handle_repo_fix(root: &Path, arguments: Value) -> Result<Value> {
    use repo_core::{SyncEngine, SyncOptions};
    use repo_fs::NormalizedPath;

    let dry_run = arguments.get("dry_run")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let normalized = NormalizedPath::new(root);
    let mode = resolve_mode(&normalized)?;

    let engine = SyncEngine::new(normalized, mode)?;
    let options = SyncOptions { dry_run };
    let report = engine.fix_with_options(options)?;

    Ok(json!({
        "success": report.success,
        "dry_run": dry_run,
        "actions": report.actions,
        "errors": report.errors,
    }))
}

async fn handle_repo_init(root: &Path, arguments: Value) -> Result<Value> {
    let name = arguments.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(".");

    let mode = arguments.get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("worktrees");

    let tools: Vec<String> = arguments.get("tools")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    // Create .repository directory
    let repo_dir = root.join(".repository");
    std::fs::create_dir_all(&repo_dir)?;

    // Generate config
    let tools_str: Vec<String> = tools.iter().map(|t| format!("\"{}\"", t)).collect();
    let config = format!(
        "tools = [{}]\n\n[core]\nmode = \"{}\"\n",
        tools_str.join(", "),
        mode
    );
    std::fs::write(repo_dir.join("config.toml"), &config)?;

    Ok(json!({
        "success": true,
        "path": root.display().to_string(),
        "mode": mode,
        "tools": tools,
    }))
}

async fn handle_branch_list(root: &Path) -> Result<Value> {
    use repo_core::{ModeBackend, StandardBackend, WorktreeBackend};
    use repo_fs::NormalizedPath;

    let normalized = NormalizedPath::new(root);
    let mode = resolve_mode(&normalized)?;

    let branches = match mode {
        repo_core::Mode::Standard => {
            let backend = StandardBackend::new(normalized)?;
            backend.list_branches()?
        }
        repo_core::Mode::Worktrees => {
            let backend = WorktreeBackend::new(normalized)?;
            backend.list_branches()?
        }
    };

    Ok(json!({
        "branches": branches.iter().map(|b| json!({
            "name": b.name,
            "is_current": b.is_current,
            "path": b.path.as_ref().map(|p| p.display().to_string()),
        })).collect::<Vec<_>>(),
    }))
}

async fn handle_branch_create(root: &Path, arguments: Value) -> Result<Value> {
    use repo_core::{ModeBackend, StandardBackend, WorktreeBackend};
    use repo_fs::NormalizedPath;

    let name = arguments.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::InvalidArgument("name is required".to_string()))?;

    let base = arguments.get("base")
        .and_then(|v| v.as_str());

    let normalized = NormalizedPath::new(root);
    let mode = resolve_mode(&normalized)?;

    match mode {
        repo_core::Mode::Standard => {
            let backend = StandardBackend::new(normalized)?;
            backend.create_branch(name, base)?;
        }
        repo_core::Mode::Worktrees => {
            let backend = WorktreeBackend::new(normalized)?;
            backend.create_branch(name, base)?;
        }
    };

    Ok(json!({
        "success": true,
        "branch": name,
        "base": base,
    }))
}

async fn handle_branch_delete(root: &Path, arguments: Value) -> Result<Value> {
    use repo_core::{ModeBackend, StandardBackend, WorktreeBackend};
    use repo_fs::NormalizedPath;

    let name = arguments.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::InvalidArgument("name is required".to_string()))?;

    let normalized = NormalizedPath::new(root);
    let mode = resolve_mode(&normalized)?;

    match mode {
        repo_core::Mode::Standard => {
            let backend = StandardBackend::new(normalized)?;
            backend.delete_branch(name)?;
        }
        repo_core::Mode::Worktrees => {
            let backend = WorktreeBackend::new(normalized)?;
            backend.delete_branch(name)?;
        }
    };

    Ok(json!({
        "success": true,
        "branch": name,
    }))
}

async fn handle_tool_add(root: &Path, arguments: Value) -> Result<Value> {
    let name = arguments.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::InvalidArgument("name is required".to_string()))?;

    // Read current config
    let config_path = root.join(".repository/config.toml");
    let content = std::fs::read_to_string(&config_path)?;

    // Parse and add tool
    let mut manifest: toml::Value = toml::from_str(&content)?;
    if let Some(tools) = manifest.get_mut("tools").and_then(|v| v.as_array_mut()) {
        if !tools.iter().any(|t| t.as_str() == Some(name)) {
            tools.push(toml::Value::String(name.to_string()));
        }
    }

    // Write back
    let new_content = toml::to_string_pretty(&manifest)?;
    std::fs::write(&config_path, &new_content)?;

    Ok(json!({
        "success": true,
        "tool": name,
        "action": "added",
    }))
}

async fn handle_tool_remove(root: &Path, arguments: Value) -> Result<Value> {
    let name = arguments.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::InvalidArgument("name is required".to_string()))?;

    let config_path = root.join(".repository/config.toml");
    let content = std::fs::read_to_string(&config_path)?;

    let mut manifest: toml::Value = toml::from_str(&content)?;
    if let Some(tools) = manifest.get_mut("tools").and_then(|v| v.as_array_mut()) {
        tools.retain(|t| t.as_str() != Some(name));
    }

    let new_content = toml::to_string_pretty(&manifest)?;
    std::fs::write(&config_path, &new_content)?;

    Ok(json!({
        "success": true,
        "tool": name,
        "action": "removed",
    }))
}

async fn handle_rule_add(root: &Path, arguments: Value) -> Result<Value> {
    let id = arguments.get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::InvalidArgument("id is required".to_string()))?;

    let content = arguments.get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::InvalidArgument("content is required".to_string()))?;

    let rules_dir = root.join(".repository/rules");
    std::fs::create_dir_all(&rules_dir)?;

    let rule_path = rules_dir.join(format!("{}.md", id));
    std::fs::write(&rule_path, content)?;

    Ok(json!({
        "success": true,
        "rule_id": id,
        "path": rule_path.display().to_string(),
    }))
}

async fn handle_rule_remove(root: &Path, arguments: Value) -> Result<Value> {
    let id = arguments.get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::InvalidArgument("id is required".to_string()))?;

    let rule_path = root.join(".repository/rules").join(format!("{}.md", id));

    if rule_path.exists() {
        std::fs::remove_file(&rule_path)?;
    }

    Ok(json!({
        "success": true,
        "rule_id": id,
    }))
}

/// Resolve repository mode from config
fn resolve_mode(root: &repo_fs::NormalizedPath) -> Result<repo_core::Mode> {
    use repo_core::ConfigResolver;

    let resolver = ConfigResolver::new(root.clone());

    if !resolver.has_config() {
        return Ok(repo_core::Mode::Worktrees);
    }

    let config = resolver.resolve()?;
    config.mode.parse().map_err(|e| Error::Core(e))
}
```

**Step 4: Update error types**

In `crates/repo-mcp/src/error.rs`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Server not initialized")]
    NotInitialized,

    #[error("Unknown tool: {0}")]
    UnknownTool(String),

    #[error("Unknown resource: {0}")]
    UnknownResource(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error(transparent)]
    Core(#[from] repo_core::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
}
```

**Step 5: Export handlers**

In `crates/repo-mcp/src/lib.rs`:

```rust
pub mod handlers;
pub use handlers::handle_tool_call;
```

**Step 6: Run tests**

Run: `cargo test -p repo-mcp handlers`
Expected: All tests pass

**Step 7: Commit**

```bash
git add crates/repo-mcp/src/handlers.rs crates/repo-mcp/src/error.rs crates/repo-mcp/src/lib.rs
git commit -m "feat(repo-mcp): implement tool handlers for all MCP tools"
```

---

## Task 4: Implement Resource Handlers

**Files:**
- Create: `crates/repo-mcp/src/resource_handlers.rs`
- Modify: `crates/repo-mcp/src/lib.rs`

**Step 1: Write test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_read_config_resource() {
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
}
```

**Step 2: Implement resource handlers**

Create `crates/repo-mcp/src/resource_handlers.rs`:

```rust
//! MCP Resource Handlers
//!
//! Read-only access to repository state.

use std::path::Path;
use crate::{Error, Result};

pub struct ResourceContent {
    pub uri: String,
    pub mime_type: String,
    pub text: String,
}

pub async fn read_resource(root: &Path, uri: &str) -> Result<ResourceContent> {
    match uri {
        "repo://config" => read_config(root).await,
        "repo://state" => read_state(root).await,
        "repo://rules" => read_rules(root).await,
        _ => Err(Error::UnknownResource(uri.to_string())),
    }
}

async fn read_config(root: &Path) -> Result<ResourceContent> {
    let config_path = root.join(".repository/config.toml");
    let text = std::fs::read_to_string(&config_path)
        .unwrap_or_else(|_| "# No configuration found\ntools = []\n\n[core]\nmode = \"worktrees\"\n".to_string());

    Ok(ResourceContent {
        uri: "repo://config".to_string(),
        mime_type: "application/toml".to_string(),
        text,
    })
}

async fn read_state(root: &Path) -> Result<ResourceContent> {
    let ledger_path = root.join(".repository/ledger.toml");
    let text = std::fs::read_to_string(&ledger_path)
        .unwrap_or_else(|_| "# No ledger found - run 'repo sync' to create\n".to_string());

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
        let mut entries: Vec<_> = std::fs::read_dir(&rules_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
            .collect();

        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let rule_name = entry.path()
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            if let Ok(rule_content) = std::fs::read_to_string(entry.path()) {
                content.push_str(&format!("## {}\n\n", rule_name));
                content.push_str(&rule_content);
                content.push_str("\n\n---\n\n");
            }
        }
    }

    if content == "# Active Rules\n\n" {
        content.push_str("_No rules defined. Add rules with `repo add-rule` or via MCP `rule_add` tool._\n");
    }

    Ok(ResourceContent {
        uri: "repo://rules".to_string(),
        mime_type: "text/markdown".to_string(),
        text: content,
    })
}
```

**Step 3: Export and test**

Run: `cargo test -p repo-mcp resource`

**Step 4: Commit**

```bash
git add crates/repo-mcp/src/resource_handlers.rs crates/repo-mcp/src/lib.rs
git commit -m "feat(repo-mcp): implement resource handlers"
```

---

## Task 5: Implement Server Main Loop

**Files:**
- Modify: `crates/repo-mcp/src/server.rs`

**Step 1: Write test for message handling**

Add to server tests:

```rust
#[tokio::test]
async fn test_handle_initialize() {
    let mut server = RepoMcpServer::new(PathBuf::from("/tmp/test"));
    server.initialize().await.unwrap();

    let request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#;

    let response = server.handle_message(request).await.unwrap();
    assert!(response.contains("repo-mcp"));
    assert!(response.contains("capabilities"));
}
```

**Step 2: Implement message handler**

Replace `crates/repo-mcp/src/server.rs`:

```rust
//! MCP Server implementation
//!
//! Handles JSON-RPC 2.0 messages for MCP protocol.

use std::path::PathBuf;
use std::io::{BufRead, Write};
use serde_json::{json, Value};

use crate::protocol::*;
use crate::handlers::handle_tool_call;
use crate::resource_handlers::read_resource;
use crate::tools::get_tool_definitions;
use crate::resources::get_resource_definitions;
use crate::{Error, Result};

pub struct RepoMcpServer {
    root: PathBuf,
    initialized: bool,
}

impl RepoMcpServer {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            initialized: false,
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        tracing::info!(root = ?self.root, "Initializing MCP server");
        self.initialized = true;
        Ok(())
    }

    /// Run the server over stdio
    pub async fn run(&mut self) -> Result<()> {
        self.initialize().await?;

        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();

        tracing::info!("MCP server ready, listening on stdio");

        for line in stdin.lock().lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }

            tracing::debug!(request = %line, "Received message");

            match self.handle_message(&line).await {
                Ok(response) => {
                    writeln!(stdout, "{}", response)?;
                    stdout.flush()?;
                    tracing::debug!(response = %response, "Sent response");
                }
                Err(e) => {
                    let error_response = JsonRpcResponse::error(
                        None,
                        -32603,
                        format!("Internal error: {}", e),
                    );
                    let json = serde_json::to_string(&error_response)?;
                    writeln!(stdout, "{}", json)?;
                    stdout.flush()?;
                    tracing::error!(error = %e, "Error handling message");
                }
            }
        }

        Ok(())
    }

    pub async fn handle_message(&self, message: &str) -> Result<String> {
        let request: JsonRpcRequest = serde_json::from_str(message)
            .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;

        let response = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.id, request.params).await?,
            "initialized" => return Ok(String::new()), // Notification, no response
            "tools/list" => self.handle_tools_list(request.id).await?,
            "tools/call" => self.handle_tools_call(request.id, request.params).await?,
            "resources/list" => self.handle_resources_list(request.id).await?,
            "resources/read" => self.handle_resources_read(request.id, request.params).await?,
            _ => JsonRpcResponse::error(
                request.id,
                -32601,
                format!("Method not found: {}", request.method),
            ),
        };

        serde_json::to_string(&response)
            .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
    }

    async fn handle_initialize(&self, id: Option<Value>, _params: Value) -> Result<JsonRpcResponse> {
        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability { list_changed: Some(false) }),
                resources: Some(ResourcesCapability {
                    subscribe: Some(false),
                    list_changed: Some(false),
                }),
            },
            server_info: ServerInfo {
                name: "repo-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        Ok(JsonRpcResponse::success(id, serde_json::to_value(result)?))
    }

    async fn handle_tools_list(&self, id: Option<Value>) -> Result<JsonRpcResponse> {
        let tools: Vec<Tool> = get_tool_definitions()
            .into_iter()
            .map(|t| Tool {
                name: t.name,
                description: Some(t.description),
                input_schema: t.input_schema,
            })
            .collect();

        Ok(JsonRpcResponse::success(id, json!({ "tools": tools })))
    }

    async fn handle_tools_call(&self, id: Option<Value>, params: Value) -> Result<JsonRpcResponse> {
        let call_params: ToolCallParams = serde_json::from_value(params)
            .map_err(|e| Error::InvalidArgument(e.to_string()))?;

        match handle_tool_call(&self.root, &call_params.name, call_params.arguments).await {
            Ok(result) => {
                let tool_result = ToolCallResult {
                    content: vec![Content::Text {
                        text: serde_json::to_string_pretty(&result)?,
                    }],
                    is_error: None,
                };
                Ok(JsonRpcResponse::success(id, serde_json::to_value(tool_result)?))
            }
            Err(e) => {
                let tool_result = ToolCallResult {
                    content: vec![Content::Text {
                        text: format!("Error: {}", e),
                    }],
                    is_error: Some(true),
                };
                Ok(JsonRpcResponse::success(id, serde_json::to_value(tool_result)?))
            }
        }
    }

    async fn handle_resources_list(&self, id: Option<Value>) -> Result<JsonRpcResponse> {
        let resources: Vec<Resource> = get_resource_definitions()
            .into_iter()
            .map(|r| Resource {
                uri: r.uri,
                name: r.name,
                description: Some(r.description),
                mime_type: Some(r.mime_type),
            })
            .collect();

        Ok(JsonRpcResponse::success(id, json!({ "resources": resources })))
    }

    async fn handle_resources_read(&self, id: Option<Value>, params: Value) -> Result<JsonRpcResponse> {
        let read_params: ReadResourceParams = serde_json::from_value(params)
            .map_err(|e| Error::InvalidArgument(e.to_string()))?;

        let content = read_resource(&self.root, &read_params.uri).await?;

        let result = ReadResourceResult {
            contents: vec![ResourceContents::Text {
                uri: content.uri,
                mime_type: Some(content.mime_type),
                text: content.text,
            }],
        };

        Ok(JsonRpcResponse::success(id, serde_json::to_value(result)?))
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}
```

**Step 3: Run all tests**

Run: `cargo test -p repo-mcp`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/repo-mcp/src/server.rs
git commit -m "feat(repo-mcp): implement MCP server main loop with stdio transport"
```

---

## Task 6: Create MCP Server Binary

**Files:**
- Create: `crates/repo-mcp/src/main.rs`
- Modify: `crates/repo-mcp/Cargo.toml`

**Step 1: Add binary target to Cargo.toml**

```toml
[[bin]]
name = "repo-mcp"
path = "src/main.rs"
```

**Step 2: Create main.rs**

```rust
//! Repository Manager MCP Server
//!
//! Run with: repo-mcp [--root <path>]

use std::path::PathBuf;
use clap::Parser;
use repo_mcp::RepoMcpServer;

#[derive(Parser)]
#[command(name = "repo-mcp")]
#[command(about = "MCP server for Repository Manager")]
struct Args {
    /// Repository root path
    #[arg(short, long, default_value = ".")]
    root: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("repo_mcp=debug".parse()?)
        )
        .with_writer(std::io::stderr)
        .init();

    let args = Args::parse();

    tracing::info!(root = ?args.root, "Starting repo-mcp server");

    let mut server = RepoMcpServer::new(args.root);
    server.run().await?;

    Ok(())
}
```

**Step 3: Add required dependencies**

```toml
[dependencies]
# ... existing deps ...
clap = { workspace = true }
tracing-subscriber = { workspace = true, features = ["env-filter"] }
```

**Step 4: Build and test**

Run: `cargo build -p repo-mcp --bin repo-mcp`
Run: `echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | ./target/debug/repo-mcp`

Expected: JSON response with server info

**Step 5: Commit**

```bash
git add crates/repo-mcp/src/main.rs crates/repo-mcp/Cargo.toml
git commit -m "feat(repo-mcp): add MCP server binary"
```

---

## Completion Checklist

- [ ] MCP protocol types implemented
- [ ] All 14 tool handlers implemented
- [ ] All 3 resource handlers implemented
- [ ] Server main loop handles JSON-RPC over stdio
- [ ] Binary can be run standalone
- [ ] All tests pass

---

## Verification

After completing all tasks:

```bash
# Run all MCP tests
cargo test -p repo-mcp

# Build the binary
cargo build -p repo-mcp --release

# Test with a sample request
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | ./target/release/repo-mcp --root /tmp/test-repo
```

---

*Plan created: 2026-01-29*
*Addresses: DX-001 (MCP server non-functional)*
