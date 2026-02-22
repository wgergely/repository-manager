//! MCP (Model Context Protocol) configuration specifications for tools.
//!
//! Each tool that supports MCP has a unique way of storing server definitions:
//! different file paths, JSON key names, transport field names, and env var
//! interpolation syntax. This module provides types to describe those differences
//! so that a single canonical MCP server definition can be translated into any
//! tool's native format.
//!
//! # Two layers
//!
//! - **Spec types** (`McpConfigSpec`, `McpFieldMappings`, …) are compile-time
//!   constants that describe a tool's native MCP format. They use `&'static str`
//!   and do **not** derive `Serialize`/`Deserialize`.
//!
//! - **Config types** (`McpServerConfig`, `McpTransportConfig`) represent the
//!   canonical, tool-agnostic MCP server definition that users write and that
//!   gets translated per-tool. These **do** derive `Serialize`/`Deserialize`.

use serde::{Deserialize, Serialize};

// ===========================================================================
// Spec types — compile-time descriptions of each tool's MCP format
// ===========================================================================

/// Complete description of how a tool stores MCP server configuration.
#[derive(Debug, Clone)]
pub struct McpConfigSpec {
    /// Top-level JSON key for the servers map.
    ///
    /// Most tools use `"mcpServers"`, VS Code/Copilot uses `"servers"`,
    /// and Zed uses `"context_servers"`.
    pub servers_key: &'static str,

    /// Project-scope config path relative to repo root.
    /// `None` if the tool does not support project-scoped MCP.
    pub project_path: Option<&'static str>,

    /// User-scope config path (OS-dependent, resolved at runtime).
    /// `None` if the tool only supports project scope.
    pub user_path: Option<McpUserPath>,

    /// Whether the MCP config lives in a dedicated file or is nested
    /// inside a larger settings file.
    pub embedding: McpConfigEmbedding,

    /// Supported MCP transport types.
    pub transports: &'static [McpTransport],

    /// How the tool names transport-related JSON fields.
    pub field_mappings: McpFieldMappings,

    /// Environment variable interpolation syntax used in config values.
    /// `None` if the tool does not support env var interpolation.
    pub env_syntax: Option<McpEnvSyntax>,
}

// ---------------------------------------------------------------------------
// User-scope path resolution
// ---------------------------------------------------------------------------

/// How to locate a tool's user-scope MCP config file.
#[derive(Debug, Clone)]
pub enum McpUserPath {
    /// Simple path relative to `$HOME`.
    HomeRelative(&'static str),

    /// Different paths per operating system, each relative to `$HOME`.
    OsSpecific {
        macos: &'static str,
        linux: &'static str,
        windows: &'static str,
    },

    /// Config lives in VS Code extension globalStorage.
    VsCodeExtStorage {
        extension_id: &'static str,
        filename: &'static str,
    },
}

impl McpUserPath {
    /// Resolve to a concrete path relative to `$HOME`.
    pub fn resolve(&self) -> Option<String> {
        match self {
            McpUserPath::HomeRelative(p) => Some((*p).to_string()),
            McpUserPath::OsSpecific {
                macos,
                linux,
                windows,
            } => {
                if cfg!(target_os = "macos") {
                    Some((*macos).to_string())
                } else if cfg!(target_os = "windows") {
                    Some((*windows).to_string())
                } else {
                    Some((*linux).to_string())
                }
            }
            McpUserPath::VsCodeExtStorage {
                extension_id,
                filename,
            } => {
                // Safety: extension_id and filename must not contain path separators
                // or path traversal sequences. Currently enforced by &'static str
                // (compile-time constants), but assert defensively.
                debug_assert!(
                    !extension_id.contains('/')
                        && !extension_id.contains('\\')
                        && !extension_id.contains(".."),
                    "extension_id must not contain path separators: {extension_id}"
                );
                debug_assert!(
                    !filename.contains('/') && !filename.contains('\\') && !filename.contains(".."),
                    "filename must not contain path separators: {filename}"
                );
                let base = if cfg!(target_os = "macos") {
                    "Library/Application Support/Code/User/globalStorage"
                } else if cfg!(target_os = "windows") {
                    "AppData/Roaming/Code/User/globalStorage"
                } else {
                    ".config/Code/User/globalStorage"
                };
                Some(format!("{base}/{extension_id}/settings/{filename}"))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Config embedding
// ---------------------------------------------------------------------------

/// Whether MCP config is a standalone file or embedded in a larger config.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpConfigEmbedding {
    /// File contains only MCP configuration (e.g., `.cursor/mcp.json`).
    Dedicated,
    /// MCP config is nested inside a larger settings file (e.g., Zed's `settings.json`).
    Nested,
}

// ---------------------------------------------------------------------------
// Transport types
// ---------------------------------------------------------------------------

/// MCP transport protocols.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpTransport {
    /// Standard I/O — local process communication.
    Stdio,
    /// Streamable HTTP — modern remote transport.
    Http,
    /// Server-Sent Events — legacy remote transport.
    Sse,
}

// ---------------------------------------------------------------------------
// Field mappings (handles naming differences across tools)
// ---------------------------------------------------------------------------

/// How a tool names JSON fields for MCP server entries.
///
/// Different tools use different field names for the same concept:
/// - HTTP URL: `"url"` vs `"serverUrl"` vs `"httpUrl"`
/// - Type field: `"type": "stdio"` vs `"type": "command"` vs auto-inferred
#[derive(Debug, Clone)]
pub struct McpFieldMappings {
    /// Field name for the HTTP/Streamable HTTP URL.
    /// Common values: `"url"`, `"serverUrl"`, `"httpUrl"`.
    pub http_url_field: &'static str,

    /// Field name for the SSE URL, if different from `http_url_field`.
    /// Most tools use the same field; Gemini uses `"url"` for SSE and
    /// `"httpUrl"` for Streamable HTTP.
    pub sse_url_field: Option<&'static str>,

    /// Whether the tool requires an explicit `"type"` field on server entries.
    pub requires_type_field: bool,

    /// Values for the `"type"` field per transport, if `requires_type_field` is true.
    pub type_values: McpTypeValues,
}

/// Values used in the `"type"` field for each transport.
#[derive(Debug, Clone)]
pub struct McpTypeValues {
    /// Value for stdio transport (e.g., `"stdio"`, `"command"`, or `None` if inferred).
    pub stdio: Option<&'static str>,
    /// Value for HTTP transport (e.g., `"http"`, `"streamable-http"`).
    pub http: Option<&'static str>,
    /// Value for SSE transport (e.g., `"sse"`).
    pub sse: Option<&'static str>,
}

impl Default for McpTypeValues {
    fn default() -> Self {
        Self {
            stdio: Some("stdio"),
            http: Some("http"),
            sse: Some("sse"),
        }
    }
}

impl Default for McpFieldMappings {
    fn default() -> Self {
        Self {
            http_url_field: "url",
            sse_url_field: None,
            requires_type_field: false,
            type_values: McpTypeValues::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Environment variable interpolation
// ---------------------------------------------------------------------------

/// The syntax a tool uses for environment variable references in config values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpEnvSyntax {
    /// `${VAR}` and `${VAR:-default}` — used by Claude Code
    DollarBrace,
    /// `$VAR` and `${VAR}` — used by Gemini CLI
    DollarSign,
    /// `${env:VAR}` — used by Cursor, Windsurf, Roo Code
    DollarEnvColon,
    /// `${input:id}` — VS Code / Copilot input variable system
    VsCodeInput,
}

// ===========================================================================
// Config types — user-facing, tool-agnostic MCP server definitions
// ===========================================================================

/// A tool-agnostic MCP server configuration.
///
/// This is the canonical representation that gets translated into each
/// tool's native format via [`McpConfigSpec`] and [`McpFieldMappings`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Transport configuration.
    pub transport: McpTransportConfig,
    /// Environment variables to pass to the server process.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<std::collections::BTreeMap<String, String>>,
    /// Whether to auto-approve all tools from this server.
    /// Used by Roo Code (`alwaysAllow`), Cline (`alwaysAllow`), and Amazon Q (`autoApprove`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub auto_approve: bool,
}

/// Transport-specific configuration for an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum McpTransportConfig {
    /// Local process communication via stdin/stdout.
    Stdio {
        command: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cwd: Option<String>,
    },
    /// Streamable HTTP remote transport.
    Http {
        url: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        headers: Option<std::collections::BTreeMap<String, String>>,
    },
    /// Server-Sent Events remote transport (legacy).
    Sse {
        url: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        headers: Option<std::collections::BTreeMap<String, String>>,
    },
}

// ===========================================================================
// Scope
// ===========================================================================

/// Where an MCP server definition should be installed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpScope {
    /// Project-level: config stored in repo, can be committed to VCS.
    Project,
    /// User-level: config stored in user's home dir, available across projects.
    User,
}

// ===========================================================================
// Operation result types
// ===========================================================================

/// Result of verifying an MCP server installation.
#[derive(Debug, Clone)]
pub struct McpVerifyResult {
    /// Whether the server entry exists in the config file.
    pub exists: bool,
    /// Whether the config file itself exists on disk.
    pub config_exists: bool,
    /// The raw JSON value of the server entry, if found.
    pub server_json: Option<serde_json::Value>,
    /// Any issues found during verification.
    pub issues: Vec<String>,
}

/// Result of syncing MCP servers to a tool's config.
#[derive(Debug, Clone)]
pub struct McpSyncResult {
    /// Servers that were newly added.
    pub added: Vec<String>,
    /// Servers that were updated (already existed, value changed).
    pub updated: Vec<String>,
    /// Servers that were removed.
    pub removed: Vec<String>,
    /// Servers that were unchanged.
    pub unchanged: Vec<String>,
}

impl McpSyncResult {
    /// Returns true if no changes were made.
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.updated.is_empty() && self.removed.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_user_path_home_relative() {
        let path = McpUserPath::HomeRelative(".cursor/mcp.json");
        assert_eq!(path.resolve().unwrap(), ".cursor/mcp.json");
    }

    #[test]
    fn test_mcp_user_path_os_specific() {
        let path = McpUserPath::OsSpecific {
            macos: "Library/Application Support/Windsurf/mcp_config.json",
            linux: ".codeium/windsurf/mcp_config.json",
            windows: ".codeium/windsurf/mcp_config.json",
        };
        assert!(path.resolve().is_some());
    }

    #[test]
    fn test_mcp_user_path_vscode_ext() {
        let path = McpUserPath::VsCodeExtStorage {
            extension_id: "saoudrizwan.claude-dev",
            filename: "cline_mcp_settings.json",
        };
        let resolved = path.resolve().unwrap();
        assert!(resolved.contains("saoudrizwan.claude-dev"));
        assert!(resolved.ends_with("cline_mcp_settings.json"));
    }

    #[test]
    fn test_default_field_mappings() {
        let m = McpFieldMappings::default();
        assert_eq!(m.http_url_field, "url");
        assert!(!m.requires_type_field);
    }

    #[test]
    fn test_mcp_server_config_serde_stdio() {
        let config = McpServerConfig {
            transport: McpTransportConfig::Stdio {
                command: "npx".into(),
                args: vec!["-y".into(), "some-server".into()],
                cwd: None,
            },
            env: None,
            auto_approve: false,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"type\":\"stdio\""));
        assert!(json.contains("\"command\":\"npx\""));
    }

    #[test]
    fn test_mcp_server_config_serde_http() {
        let config = McpServerConfig {
            transport: McpTransportConfig::Http {
                url: "https://example.com/mcp".into(),
                headers: None,
            },
            env: None,
            auto_approve: false,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"type\":\"http\""));
        assert!(json.contains("https://example.com/mcp"));
    }

    #[test]
    fn test_mcp_transport_enum() {
        assert_ne!(McpTransport::Stdio, McpTransport::Http);
        assert_ne!(McpTransport::Http, McpTransport::Sse);
    }

    #[test]
    fn test_env_syntax_variants() {
        assert_ne!(McpEnvSyntax::DollarBrace, McpEnvSyntax::DollarEnvColon);
        assert_ne!(McpEnvSyntax::DollarSign, McpEnvSyntax::VsCodeInput);
    }

    #[test]
    fn test_mcp_scope() {
        assert_ne!(McpScope::Project, McpScope::User);
    }

    #[test]
    fn test_mcp_server_config_auto_approve() {
        let config = McpServerConfig {
            transport: McpTransportConfig::Stdio {
                command: "npx".into(),
                args: vec![],
                cwd: None,
            },
            env: None,
            auto_approve: true,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"auto_approve\":true"));

        // false should be skipped
        let config2 = McpServerConfig {
            transport: McpTransportConfig::Stdio {
                command: "npx".into(),
                args: vec![],
                cwd: None,
            },
            env: None,
            auto_approve: false,
        };
        let json2 = serde_json::to_string(&config2).unwrap();
        assert!(!json2.contains("auto_approve"));
    }

    #[test]
    fn test_mcp_verify_result() {
        let result = McpVerifyResult {
            exists: true,
            config_exists: true,
            server_json: Some(serde_json::json!({"command":"npx"})),
            issues: vec![],
        };
        assert!(result.exists);
        assert!(result.config_exists);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_mcp_sync_result_empty() {
        let result = McpSyncResult {
            added: vec![],
            updated: vec![],
            removed: vec![],
            unchanged: vec!["existing-server".to_string()],
        };
        assert!(result.added.is_empty());
        assert_eq!(result.unchanged.len(), 1);
    }

    #[test]
    fn test_mcp_server_config_auto_approve_default() {
        let json = r#"{"transport":{"type":"stdio","command":"test"}}"#;
        let config: McpServerConfig = serde_json::from_str(json).unwrap();
        assert!(!config.auto_approve);
    }

    #[test]
    fn test_mcp_server_config_auto_approve_true() {
        let config = McpServerConfig {
            transport: McpTransportConfig::Stdio {
                command: "test".into(),
                args: vec![],
                cwd: None,
            },
            env: None,
            auto_approve: true,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"auto_approve\":true"));
    }

    #[test]
    fn test_mcp_server_config_auto_approve_false_skipped() {
        let config = McpServerConfig {
            transport: McpTransportConfig::Stdio {
                command: "test".into(),
                args: vec![],
                cwd: None,
            },
            env: None,
            auto_approve: false,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(!json.contains("auto_approve"));
    }

    #[test]
    fn test_mcp_sync_result_is_empty() {
        let result = McpSyncResult {
            added: vec![],
            updated: vec![],
            removed: vec![],
            unchanged: vec!["server1".into()],
        };
        assert!(result.is_empty());
    }

    #[test]
    fn test_mcp_sync_result_not_empty() {
        let result = McpSyncResult {
            added: vec!["new-server".into()],
            updated: vec![],
            removed: vec![],
            unchanged: vec![],
        };
        assert!(!result.is_empty());
    }
}
