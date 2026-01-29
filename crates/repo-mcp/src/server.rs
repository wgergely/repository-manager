//! MCP Server implementation
//!
//! The main server struct that coordinates MCP protocol handling
//! with Repository Manager functionality.

use std::io::{BufRead, Write};
use std::path::PathBuf;

use serde_json::{json, Value};

use crate::handlers::handle_tool_call;
use crate::protocol::{
    InitializeResult, JsonRpcRequest, JsonRpcResponse, ReadResourceParams, ResourcesCapability,
    ServerCapabilities, ServerInfo, ToolCallParams, ToolsCapability,
};
use crate::resource_handlers::read_resource;
use crate::resources::{get_resource_definitions, ResourceDefinition};
use crate::tools::{get_tool_definitions, ToolDefinition, ToolResult};
use crate::{Error, Result};

/// MCP Server for Repository Manager
///
/// This server exposes repository management functionality via the
/// Model Context Protocol, allowing agentic IDEs to interact with
/// the repository structure, configuration, and Git operations.
///
/// # Example
///
/// ```ignore
/// use repo_mcp::RepoMcpServer;
/// use std::path::PathBuf;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let server = RepoMcpServer::new(PathBuf::from("."));
///     server.run().await?;
///     Ok(())
/// }
/// ```
pub struct RepoMcpServer {
    /// Root path of the repository
    root: PathBuf,

    /// Whether the server has been initialized
    initialized: bool,

    /// Available MCP tools
    tools: Vec<ToolDefinition>,

    /// Available MCP resources
    resources: Vec<ResourceDefinition>,
}

impl RepoMcpServer {
    /// Create a new MCP server instance
    ///
    /// # Arguments
    ///
    /// * `root` - Path to the repository root
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            initialized: false,
            tools: Vec::new(),
            resources: Vec::new(),
        }
    }

    /// Initialize the server
    ///
    /// This loads the repository configuration and prepares
    /// the server to handle requests.
    pub async fn initialize(&mut self) -> Result<()> {
        tracing::info!(root = ?self.root, "Initializing MCP server");

        // TODO: Load repository configuration
        // TODO: Validate repository structure

        // Load tool and resource definitions
        self.tools = get_tool_definitions();
        self.resources = get_resource_definitions();

        self.initialized = true;
        Ok(())
    }

    /// Run the MCP server
    ///
    /// This starts the server and begins processing MCP protocol
    /// messages over stdin/stdout.
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
                Ok(response) if !response.is_empty() => {
                    writeln!(stdout, "{}", response)?;
                    stdout.flush()?;
                }
                Ok(_) => {} // No response needed (notifications)
                Err(e) => {
                    let error_response = JsonRpcResponse::error(
                        None,
                        -32603,
                        format!("Internal error: {}", e),
                    );
                    let json_str = serde_json::to_string(&error_response)?;
                    writeln!(stdout, "{}", json_str)?;
                    stdout.flush()?;
                }
            }
        }

        Ok(())
    }

    /// Handle a single MCP message
    ///
    /// Parses the JSON-RPC request and dispatches to the appropriate handler.
    ///
    /// # Arguments
    ///
    /// * `message` - The raw JSON-RPC message string
    ///
    /// # Returns
    ///
    /// The JSON-RPC response as a string, or empty string for notifications.
    pub async fn handle_message(&self, message: &str) -> Result<String> {
        let request: JsonRpcRequest = serde_json::from_str(message)?;

        let response = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.id).await?,
            "initialized" => return Ok(String::new()), // Notification, no response
            "notifications/initialized" => return Ok(String::new()), // Notification, no response
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

        serde_json::to_string(&response).map_err(Error::from)
    }

    /// Handle the initialize request
    ///
    /// Returns server capabilities and info.
    async fn handle_initialize(&self, id: Option<Value>) -> Result<JsonRpcResponse> {
        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
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

    /// Handle tools/list request
    ///
    /// Returns the list of available tools.
    async fn handle_tools_list(&self, id: Option<Value>) -> Result<JsonRpcResponse> {
        let tools = get_tool_definitions();

        // Convert to the format expected by MCP protocol
        let tools_value: Vec<Value> = tools
            .iter()
            .map(|t| {
                json!({
                    "name": t.name,
                    "description": t.description,
                    "inputSchema": t.input_schema
                })
            })
            .collect();

        Ok(JsonRpcResponse::success(id, json!({ "tools": tools_value })))
    }

    /// Handle tools/call request
    ///
    /// Executes the requested tool and returns the result.
    async fn handle_tools_call(
        &self,
        id: Option<Value>,
        params: Value,
    ) -> Result<JsonRpcResponse> {
        let tool_params: ToolCallParams = serde_json::from_value(params)?;

        match handle_tool_call(&self.root, &tool_params.name, tool_params.arguments).await {
            Ok(result) => {
                // Convert Value result to ToolResult format
                let tool_result = ToolResult::text(serde_json::to_string_pretty(&result)?);
                Ok(JsonRpcResponse::success(id, serde_json::to_value(tool_result)?))
            }
            Err(e) => {
                let tool_result = ToolResult::error(format!("{}", e));
                Ok(JsonRpcResponse::success(id, serde_json::to_value(tool_result)?))
            }
        }
    }

    /// Handle resources/list request
    ///
    /// Returns the list of available resources.
    async fn handle_resources_list(&self, id: Option<Value>) -> Result<JsonRpcResponse> {
        let resources = get_resource_definitions();

        // Convert to the format expected by MCP protocol
        let resources_value: Vec<Value> = resources
            .iter()
            .map(|r| {
                json!({
                    "uri": r.uri,
                    "name": r.name,
                    "description": r.description,
                    "mimeType": r.mime_type
                })
            })
            .collect();

        Ok(JsonRpcResponse::success(
            id,
            json!({ "resources": resources_value }),
        ))
    }

    /// Handle resources/read request
    ///
    /// Reads and returns the content of the requested resource.
    async fn handle_resources_read(
        &self,
        id: Option<Value>,
        params: Value,
    ) -> Result<JsonRpcResponse> {
        let read_params: ReadResourceParams = serde_json::from_value(params)?;

        match read_resource(&self.root, &read_params.uri).await {
            Ok(content) => {
                let result = json!({
                    "contents": [{
                        "uri": content.uri,
                        "mimeType": content.mime_type,
                        "text": content.text
                    }]
                });
                Ok(JsonRpcResponse::success(id, result))
            }
            Err(e) => Ok(JsonRpcResponse::error(
                id,
                -32602,
                format!("Resource error: {}", e),
            )),
        }
    }

    /// Get the repository root path
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Check if the server is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get available tools
    pub fn tools(&self) -> &[ToolDefinition] {
        &self.tools
    }

    /// Get available resources
    pub fn resources(&self) -> &[ResourceDefinition] {
        &self.resources
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_creation() {
        let server = RepoMcpServer::new(PathBuf::from("/tmp/test"));
        assert_eq!(server.root(), &PathBuf::from("/tmp/test"));
        assert!(!server.is_initialized());
        // Tools and resources should be empty before initialization
        assert!(server.tools().is_empty());
        assert!(server.resources().is_empty());
    }

    #[tokio::test]
    async fn server_initialization() {
        let mut server = RepoMcpServer::new(PathBuf::from("/tmp/test"));
        let result = server.initialize().await;
        assert!(result.is_ok());
        assert!(server.is_initialized());
    }

    #[tokio::test]
    async fn server_loads_tools_on_initialize() {
        let mut server = RepoMcpServer::new(PathBuf::from("/tmp/test"));
        server.initialize().await.unwrap();

        // Should have loaded tools
        assert!(!server.tools().is_empty());
        assert_eq!(server.tools().len(), 14); // 4 repo + 3 branch + 3 git + 4 config

        // Verify some expected tools
        let tool_names: Vec<&str> = server.tools().iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"repo_init"));
        assert!(tool_names.contains(&"git_push"));
        assert!(tool_names.contains(&"branch_create"));
    }

    #[tokio::test]
    async fn server_loads_resources_on_initialize() {
        let mut server = RepoMcpServer::new(PathBuf::from("/tmp/test"));
        server.initialize().await.unwrap();

        // Should have loaded resources
        assert!(!server.resources().is_empty());
        assert_eq!(server.resources().len(), 3);

        // Verify expected resources
        let resource_uris: Vec<&str> = server.resources().iter().map(|r| r.uri.as_str()).collect();
        assert!(resource_uris.contains(&"repo://config"));
        assert!(resource_uris.contains(&"repo://state"));
        assert!(resource_uris.contains(&"repo://rules"));
    }

    #[tokio::test]
    async fn test_handle_initialize() {
        let mut server = RepoMcpServer::new(PathBuf::from("/tmp/test"));
        server.initialize().await.unwrap();

        let request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#;

        let response = server.handle_message(request).await.unwrap();
        assert!(response.contains("repo-mcp"));
        assert!(response.contains("capabilities"));
        assert!(response.contains("protocolVersion"));
    }

    #[tokio::test]
    async fn test_handle_initialized_notification() {
        let mut server = RepoMcpServer::new(PathBuf::from("/tmp/test"));
        server.initialize().await.unwrap();

        let request = r#"{"jsonrpc":"2.0","method":"initialized"}"#;

        let response = server.handle_message(request).await.unwrap();
        // Notification should return empty string
        assert!(response.is_empty());
    }

    #[tokio::test]
    async fn test_handle_notifications_initialized() {
        let mut server = RepoMcpServer::new(PathBuf::from("/tmp/test"));
        server.initialize().await.unwrap();

        let request = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;

        let response = server.handle_message(request).await.unwrap();
        // Notification should return empty string
        assert!(response.is_empty());
    }

    #[tokio::test]
    async fn test_handle_tools_list() {
        let mut server = RepoMcpServer::new(PathBuf::from("/tmp/test"));
        server.initialize().await.unwrap();

        let request = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;

        let response = server.handle_message(request).await.unwrap();
        assert!(response.contains("repo_check"));
        assert!(response.contains("repo_sync"));
        assert!(response.contains("branch_create"));
        assert!(response.contains("git_push"));
    }

    #[tokio::test]
    async fn test_handle_resources_list() {
        let mut server = RepoMcpServer::new(PathBuf::from("/tmp/test"));
        server.initialize().await.unwrap();

        let request = r#"{"jsonrpc":"2.0","id":3,"method":"resources/list","params":{}}"#;

        let response = server.handle_message(request).await.unwrap();
        assert!(response.contains("repo://config"));
        assert!(response.contains("repo://state"));
        assert!(response.contains("repo://rules"));
    }

    #[tokio::test]
    async fn test_handle_unknown_method() {
        let mut server = RepoMcpServer::new(PathBuf::from("/tmp/test"));
        server.initialize().await.unwrap();

        let request = r#"{"jsonrpc":"2.0","id":4,"method":"unknown/method","params":{}}"#;

        let response = server.handle_message(request).await.unwrap();
        assert!(response.contains("error"));
        assert!(response.contains("-32601"));
        assert!(response.contains("Method not found"));
    }

    #[tokio::test]
    async fn test_handle_tools_call_unknown_tool() {
        let mut server = RepoMcpServer::new(PathBuf::from("/tmp/test"));
        server.initialize().await.unwrap();

        let request =
            r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"unknown_tool","arguments":{}}}"#;

        let response = server.handle_message(request).await.unwrap();
        // Tool errors are returned as successful responses with is_error: true
        assert!(response.contains("result"));
        assert!(response.contains("is_error"));
        assert!(response.contains("unknown tool"));
    }

    #[tokio::test]
    async fn test_handle_resources_read() {
        let mut server = RepoMcpServer::new(PathBuf::from("/tmp/test"));
        server.initialize().await.unwrap();

        let request =
            r#"{"jsonrpc":"2.0","id":6,"method":"resources/read","params":{"uri":"repo://config"}}"#;

        let response = server.handle_message(request).await.unwrap();
        assert!(response.contains("contents"));
        assert!(response.contains("repo://config"));
        assert!(response.contains("mimeType"));
    }

    #[tokio::test]
    async fn test_handle_resources_read_unknown() {
        let mut server = RepoMcpServer::new(PathBuf::from("/tmp/test"));
        server.initialize().await.unwrap();

        let request =
            r#"{"jsonrpc":"2.0","id":7,"method":"resources/read","params":{"uri":"repo://unknown"}}"#;

        let response = server.handle_message(request).await.unwrap();
        assert!(response.contains("error"));
        assert!(response.contains("-32602"));
    }

    #[tokio::test]
    async fn test_handle_invalid_json() {
        let mut server = RepoMcpServer::new(PathBuf::from("/tmp/test"));
        server.initialize().await.unwrap();

        let request = r#"{"invalid json"#;

        let result = server.handle_message(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_response_format() {
        let mut server = RepoMcpServer::new(PathBuf::from("/tmp/test"));
        server.initialize().await.unwrap();

        let request = r#"{"jsonrpc":"2.0","id":10,"method":"initialize","params":{}}"#;

        let response = server.handle_message(request).await.unwrap();

        // Parse the response to verify JSON-RPC 2.0 format
        let parsed: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["id"], 10);
        assert!(parsed.get("result").is_some());
        assert!(parsed.get("error").is_none());
    }

    #[tokio::test]
    async fn test_error_response_format() {
        let mut server = RepoMcpServer::new(PathBuf::from("/tmp/test"));
        server.initialize().await.unwrap();

        let request = r#"{"jsonrpc":"2.0","id":11,"method":"unknown","params":{}}"#;

        let response = server.handle_message(request).await.unwrap();

        // Parse the response to verify error format
        let parsed: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["id"], 11);
        assert!(parsed.get("result").is_none());
        assert!(parsed.get("error").is_some());
        assert!(parsed["error"]["code"].is_i64());
        assert!(parsed["error"]["message"].is_string());
    }
}
