//! MCP Server implementation
//!
//! The main server struct that coordinates MCP protocol handling
//! with Repository Manager functionality.

use std::path::PathBuf;

use crate::resources::{get_resource_definitions, ResourceDefinition};
use crate::tools::{get_tool_definitions, ToolDefinition};
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
    pub async fn run(&self) -> Result<()> {
        if !self.initialized {
            return Err(Error::NotInitialized);
        }

        tracing::info!("Starting MCP server");

        // TODO: Implement MCP protocol handling
        // TODO: Set up JSON-RPC message loop

        Ok(())
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
}
