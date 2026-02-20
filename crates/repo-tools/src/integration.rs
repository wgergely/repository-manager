//! ToolIntegration trait for syncing to external tools

use crate::error::Result;
use repo_fs::NormalizedPath;

// Re-export ConfigType for convenience
pub use repo_meta::schema::ConfigType;

/// Rule to be synced to tools
#[derive(Debug, Clone)]
pub struct Rule {
    pub id: String,
    pub content: String,
}

/// Context for tool sync operations
#[derive(Debug, Clone)]
pub struct SyncContext {
    pub root: NormalizedPath,
    pub python_path: Option<NormalizedPath>,
    /// Resolved MCP server configuration from extensions.
    ///
    /// This is a JSON object where keys are server names and values are
    /// their full configuration (command, args, env, etc.).
    pub mcp_servers: Option<serde_json::Value>,
}

impl SyncContext {
    pub fn new(root: NormalizedPath) -> Self {
        Self {
            root,
            python_path: None,
            mcp_servers: None,
        }
    }

    pub fn with_python(mut self, path: NormalizedPath) -> Self {
        self.python_path = Some(path);
        self
    }

    pub fn with_mcp_servers(mut self, servers: serde_json::Value) -> Self {
        self.mcp_servers = Some(servers);
        self
    }
}

/// Describes a configuration location for a tool.
///
/// This provides richer type information than a plain path string,
/// enabling the dispatcher to understand how to interact with different
/// configuration files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigLocation {
    /// Path relative to repository root (e.g., ".cursorrules", ".vscode/settings.json")
    pub path: String,
    /// The format of this configuration file
    pub config_type: ConfigType,
    /// Whether this path is a directory (e.g., ".claude/rules/")
    pub is_directory: bool,
}

impl ConfigLocation {
    /// Create a new config location for a file.
    pub fn file(path: impl Into<String>, config_type: ConfigType) -> Self {
        Self {
            path: path.into(),
            config_type,
            is_directory: false,
        }
    }

    /// Create a new config location for a directory.
    pub fn directory(path: impl Into<String>, config_type: ConfigType) -> Self {
        Self {
            path: path.into(),
            config_type,
            is_directory: true,
        }
    }
}

/// Trait for tool integrations
pub trait ToolIntegration {
    /// Returns the tool's slug identifier (e.g., "vscode", "cursor", "claude")
    fn name(&self) -> &str;

    /// Returns the configuration locations this tool uses.
    ///
    /// Includes the primary config file and any additional paths (like rules directories).
    fn config_locations(&self) -> Vec<ConfigLocation>;

    /// Sync rules to this tool's configuration files.
    fn sync(&self, context: &SyncContext, rules: &[Rule]) -> Result<()>;
}
