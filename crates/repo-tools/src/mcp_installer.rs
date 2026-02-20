//! MCP server installation management.
//!
//! Provides operations to install, remove, list, verify, and sync MCP server
//! definitions across different tool configurations. Uses [`McpConfigSpec`] to
//! adapt to each tool's native JSON format and file locations.

use crate::error::{Error, Result};
use crate::mcp_registry::mcp_config_spec;
use crate::mcp_translate::to_tool_json;
use repo_fs::NormalizedPath;
use repo_meta::schema::{
    McpConfigSpec, McpScope, McpServerConfig, McpSyncResult, McpVerifyResult,
};
use serde_json::{Map, Value, json};
use std::path::PathBuf;

/// Manages MCP server installations for a specific tool.
///
/// Each instance is bound to a tool (via its slug) and a repository root.
/// It uses the tool's [`McpConfigSpec`] to determine file paths, JSON keys,
/// and field naming conventions.
pub struct McpInstaller {
    slug: String,
    spec: McpConfigSpec,
    root: NormalizedPath,
}

impl McpInstaller {
    /// Create an installer for the given tool slug and repository root.
    ///
    /// Returns an error if the tool doesn't support MCP.
    pub fn new(slug: &str, root: NormalizedPath) -> Result<Self> {
        let spec = mcp_config_spec(slug).ok_or_else(|| Error::McpNotSupported {
            tool: slug.to_string(),
        })?;
        Ok(Self {
            slug: slug.to_string(),
            spec,
            root,
        })
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Resolve the config file path for the given scope.
    fn config_path(&self, scope: McpScope) -> Result<PathBuf> {
        match scope {
            McpScope::Project => {
                let rel =
                    self.spec
                        .project_path
                        .ok_or_else(|| Error::McpScopeNotSupported {
                            tool: self.slug.clone(),
                            scope: "project".into(),
                        })?;
                Ok(self.root.join(rel).to_native())
            }
            McpScope::User => {
                let user_path =
                    self.spec
                        .user_path
                        .as_ref()
                        .ok_or_else(|| Error::McpScopeNotSupported {
                            tool: self.slug.clone(),
                            scope: "user".into(),
                        })?;
                let home = home_dir()?;
                let rel = user_path.resolve().ok_or(Error::HomeDirNotFound)?;
                Ok(home.join(rel))
            }
        }
    }

    /// Read the config file and parse as JSON. Returns an empty object if the
    /// file doesn't exist.
    fn read_config(&self, scope: McpScope) -> Result<(PathBuf, Value)> {
        let path = self.config_path(scope)?;
        let value = if path.exists() {
            let content = std::fs::read_to_string(&path).map_err(|e| Error::McpConfig {
                tool: self.slug.clone(),
                message: format!("Failed to read {}: {e}", path.display()),
            })?;
            if content.trim().is_empty() {
                json!({})
            } else {
                serde_json::from_str(&content).map_err(|e| Error::McpConfig {
                    tool: self.slug.clone(),
                    message: format!("Failed to parse {}: {e}", path.display()),
                })?
            }
        } else {
            json!({})
        };
        Ok((path, value))
    }

    /// Write JSON to the config file, creating parent directories as needed.
    fn write_config(&self, path: &PathBuf, value: &Value) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::McpConfig {
                tool: self.slug.clone(),
                message: format!("Failed to create directory {}: {e}", parent.display()),
            })?;
        }
        let content = serde_json::to_string_pretty(value)?;
        std::fs::write(path, content.as_bytes()).map_err(|e| Error::McpConfig {
            tool: self.slug.clone(),
            message: format!("Failed to write {}: {e}", path.display()),
        })?;
        Ok(())
    }

    /// Get the servers map from a config value.
    ///
    /// For both `Dedicated` and `Nested` configs the `servers_key` lives at
    /// the top level of the JSON object.
    fn get_servers<'a>(&self, config: &'a Value) -> Option<&'a Map<String, Value>> {
        config.get(self.spec.servers_key)?.as_object()
    }

    /// Get or create a mutable servers map within the config.
    fn get_or_create_servers<'a>(&self, config: &'a mut Value) -> &'a mut Map<String, Value> {
        if !config.is_object() {
            *config = json!({});
        }
        let obj = config.as_object_mut().unwrap();
        if !obj.contains_key(self.spec.servers_key) {
            obj.insert(self.spec.servers_key.to_string(), json!({}));
        }
        obj[self.spec.servers_key].as_object_mut().unwrap()
    }

    // -----------------------------------------------------------------------
    // Public API
    // -----------------------------------------------------------------------

    /// Install an MCP server into the tool's config at the given scope.
    ///
    /// If a server with the same name already exists, it is overwritten.
    pub fn install(
        &self,
        scope: McpScope,
        server_name: &str,
        config: &McpServerConfig,
    ) -> Result<()> {
        let (path, mut root_value) = self.read_config(scope)?;
        let tool_json = to_tool_json(config, &self.spec);
        let servers = self.get_or_create_servers(&mut root_value);
        servers.insert(server_name.to_string(), tool_json);
        self.write_config(&path, &root_value)
    }

    /// Remove an MCP server from the tool's config.
    ///
    /// Returns `Ok(true)` if the server was found and removed, `Ok(false)` if
    /// the server was not present.
    pub fn remove(&self, scope: McpScope, server_name: &str) -> Result<bool> {
        let (path, mut root_value) = self.read_config(scope)?;
        let servers = self.get_or_create_servers(&mut root_value);
        let removed = servers.remove(server_name).is_some();
        if removed {
            self.write_config(&path, &root_value)?;
        }
        Ok(removed)
    }

    /// List all MCP servers installed in the tool's config at the given scope.
    ///
    /// Returns a list of `(server_name, server_json)` pairs.
    pub fn list(&self, scope: McpScope) -> Result<Vec<(String, Value)>> {
        let (_path, root_value) = self.read_config(scope)?;
        let servers = match self.get_servers(&root_value) {
            Some(s) => s,
            None => return Ok(vec![]),
        };
        Ok(servers
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect())
    }

    /// Verify that an MCP server is correctly installed.
    pub fn verify(&self, scope: McpScope, server_name: &str) -> Result<McpVerifyResult> {
        let path = self.config_path(scope)?;
        let config_exists = path.exists();

        if !config_exists {
            return Ok(McpVerifyResult {
                exists: false,
                config_exists: false,
                server_json: None,
                issues: vec![format!("Config file does not exist: {}", path.display())],
            });
        }

        let (_path, root_value) = self.read_config(scope)?;
        let mut issues = Vec::new();

        let server_json = self
            .get_servers(&root_value)
            .and_then(|s| s.get(server_name))
            .cloned();

        let exists = server_json.is_some();

        if !exists {
            issues.push(format!("Server '{server_name}' not found in config"));
        } else if let Some(ref json) = server_json {
            // Basic validation: the entry must be a JSON object.
            if !json.is_object() {
                issues.push("Server entry is not a JSON object".into());
            } else {
                let obj = json.as_object().unwrap();
                // Check for command (stdio) or a URL field (http/sse).
                let has_command = obj.contains_key("command");
                let has_url = obj.contains_key(self.spec.field_mappings.http_url_field)
                    || self
                        .spec
                        .field_mappings
                        .sse_url_field
                        .map_or(false, |f| obj.contains_key(f));
                if !has_command && !has_url {
                    issues.push("Server entry has neither 'command' nor a URL field".into());
                }
            }
        }

        Ok(McpVerifyResult {
            exists,
            config_exists,
            server_json,
            issues,
        })
    }

    /// Sync a set of servers to the tool's config, computing a diff.
    ///
    /// `managed_servers` is a map of `server_name -> McpServerConfig` for
    /// servers that should be managed by RepoManager. User-added servers
    /// (not in this map) are preserved.
    pub fn sync(
        &self,
        scope: McpScope,
        managed_servers: &std::collections::BTreeMap<String, McpServerConfig>,
    ) -> Result<McpSyncResult> {
        let (path, mut root_value) = self.read_config(scope)?;
        let servers = self.get_or_create_servers(&mut root_value);

        let mut added = Vec::new();
        let mut updated = Vec::new();
        let mut unchanged = Vec::new();

        // Compute the desired state for every managed server.
        let mut desired: std::collections::BTreeMap<String, Value> = managed_servers
            .iter()
            .map(|(name, config)| (name.clone(), to_tool_json(config, &self.spec)))
            .collect();

        // Walk existing servers and reconcile with the desired state.
        let existing_names: Vec<String> = servers.keys().cloned().collect();
        let removed = Vec::new();

        for name in &existing_names {
            if let Some(new_value) = desired.remove(name) {
                // Server exists in both current and desired state.
                if servers[name] == new_value {
                    unchanged.push(name.clone());
                } else {
                    servers.insert(name.clone(), new_value);
                    updated.push(name.clone());
                }
            }
            // If not in desired, it is a user-managed server — preserve it.
        }

        // Add servers that are in desired but not yet in the config.
        for (name, value) in desired {
            servers.insert(name.clone(), value);
            added.push(name);
        }

        let result = McpSyncResult {
            added,
            updated,
            removed,
            unchanged,
        };

        if !result.is_empty() {
            self.write_config(&path, &root_value)?;
        }

        Ok(result)
    }
}

/// Get the user's home directory.
fn home_dir() -> Result<PathBuf> {
    // Use $HOME on Unix, %USERPROFILE% on Windows.
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .map_err(|_| Error::HomeDirNotFound)
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use repo_meta::schema::McpTransportConfig;
    use std::collections::BTreeMap;
    use tempfile::TempDir;

    fn stdio_config(command: &str) -> McpServerConfig {
        McpServerConfig {
            transport: McpTransportConfig::Stdio {
                command: command.into(),
                args: vec![],
                cwd: None,
            },
            env: None,
            auto_approve: false,
        }
    }

    #[allow(dead_code)]
    fn http_config(url: &str) -> McpServerConfig {
        McpServerConfig {
            transport: McpTransportConfig::Http {
                url: url.into(),
                headers: None,
            },
            env: None,
            auto_approve: false,
        }
    }

    // -- Constructor tests ---------------------------------------------------

    #[test]
    fn test_new_unsupported_tool() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        assert!(McpInstaller::new("aider", root).is_err());
    }

    #[test]
    fn test_new_supported_tool() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        assert!(McpInstaller::new("claude", root).is_ok());
    }

    // -- Install + list roundtrip -------------------------------------------

    #[test]
    fn test_install_and_list_project_scope() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let installer = McpInstaller::new("cursor", root).unwrap();

        let config = stdio_config("my-server");
        installer
            .install(McpScope::Project, "test-server", &config)
            .unwrap();

        let servers = installer.list(McpScope::Project).unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].0, "test-server");
        assert_eq!(servers[0].1["command"], "my-server");
    }

    #[test]
    fn test_install_creates_dirs() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let installer = McpInstaller::new("cursor", root).unwrap();

        // .cursor/mcp.json parent dir doesn't exist yet
        let config = stdio_config("test");
        installer
            .install(McpScope::Project, "s1", &config)
            .unwrap();

        let path = temp.path().join(".cursor").join("mcp.json");
        assert!(path.exists());
    }

    #[test]
    fn test_install_overwrites() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let installer = McpInstaller::new("cursor", root).unwrap();

        installer
            .install(McpScope::Project, "s1", &stdio_config("old"))
            .unwrap();
        installer
            .install(McpScope::Project, "s1", &stdio_config("new"))
            .unwrap();

        let servers = installer.list(McpScope::Project).unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].1["command"], "new");
    }

    // -- Remove --------------------------------------------------------------

    #[test]
    fn test_remove() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let installer = McpInstaller::new("cursor", root).unwrap();

        installer
            .install(McpScope::Project, "s1", &stdio_config("test"))
            .unwrap();
        assert!(installer.remove(McpScope::Project, "s1").unwrap());
        assert!(!installer.remove(McpScope::Project, "s1").unwrap());

        let servers = installer.list(McpScope::Project).unwrap();
        assert!(servers.is_empty());
    }

    // -- Verify --------------------------------------------------------------

    #[test]
    fn test_verify_exists() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let installer = McpInstaller::new("cursor", root).unwrap();

        installer
            .install(McpScope::Project, "s1", &stdio_config("test"))
            .unwrap();
        let result = installer.verify(McpScope::Project, "s1").unwrap();
        assert!(result.exists);
        assert!(result.config_exists);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_verify_no_config() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let installer = McpInstaller::new("cursor", root).unwrap();

        let result = installer.verify(McpScope::Project, "s1").unwrap();
        assert!(!result.exists);
        assert!(!result.config_exists);
        assert!(!result.issues.is_empty());
    }

    #[test]
    fn test_verify_missing_server() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let installer = McpInstaller::new("cursor", root).unwrap();

        installer
            .install(McpScope::Project, "other", &stdio_config("test"))
            .unwrap();
        let result = installer.verify(McpScope::Project, "missing").unwrap();
        assert!(!result.exists);
        assert!(result.config_exists);
    }

    // -- Sync ----------------------------------------------------------------

    #[test]
    fn test_sync_adds() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let installer = McpInstaller::new("cursor", root).unwrap();

        let mut managed = BTreeMap::new();
        managed.insert("s1".into(), stdio_config("cmd1"));
        managed.insert("s2".into(), stdio_config("cmd2"));

        let result = installer.sync(McpScope::Project, &managed).unwrap();
        assert_eq!(result.added.len(), 2);
        assert!(result.updated.is_empty());
        assert!(result.removed.is_empty());
    }

    #[test]
    fn test_sync_preserves_user_servers() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let installer = McpInstaller::new("cursor", root).unwrap();

        // User manually installs a server
        installer
            .install(McpScope::Project, "user-server", &stdio_config("user"))
            .unwrap();

        // Sync managed servers (not including user-server)
        let mut managed = BTreeMap::new();
        managed.insert("managed-server".into(), stdio_config("managed"));
        let result = installer.sync(McpScope::Project, &managed).unwrap();

        // user-server should still be there
        let all = installer.list(McpScope::Project).unwrap();
        assert_eq!(all.len(), 2);
        let names: Vec<&str> = all.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&"user-server"));
        assert!(names.contains(&"managed-server"));

        // Only the managed server should be reported as added
        assert_eq!(result.added, vec!["managed-server"]);
    }

    #[test]
    fn test_sync_updates() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let installer = McpInstaller::new("cursor", root).unwrap();

        // Install initial version
        installer
            .install(McpScope::Project, "s1", &stdio_config("old"))
            .unwrap();

        // Sync with updated version
        let mut managed = BTreeMap::new();
        managed.insert("s1".into(), stdio_config("new"));
        let result = installer.sync(McpScope::Project, &managed).unwrap();

        assert!(result.added.is_empty());
        assert_eq!(result.updated, vec!["s1"]);

        let servers = installer.list(McpScope::Project).unwrap();
        assert_eq!(servers[0].1["command"], "new");
    }

    #[test]
    fn test_sync_unchanged() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let installer = McpInstaller::new("cursor", root).unwrap();

        installer
            .install(McpScope::Project, "s1", &stdio_config("test"))
            .unwrap();

        let mut managed = BTreeMap::new();
        managed.insert("s1".into(), stdio_config("test"));
        let result = installer.sync(McpScope::Project, &managed).unwrap();

        assert!(result.is_empty());
        assert_eq!(result.unchanged, vec!["s1"]);
    }

    // -- Scope not supported -------------------------------------------------

    #[test]
    fn test_scope_not_supported() {
        let temp = TempDir::new().unwrap();
        let installer = McpInstaller::new("antigravity", NormalizedPath::new(temp.path())).unwrap();

        let result = installer.install(McpScope::Project, "s1", &stdio_config("test"));
        assert!(result.is_err());
    }

    // -- Nested config preservation ------------------------------------------

    #[test]
    fn test_nested_config_preserves_other_keys() {
        let temp = TempDir::new().unwrap();

        // Pre-create a settings file with existing content (Gemini-like)
        let gemini_dir = temp.path().join(".gemini");
        std::fs::create_dir_all(&gemini_dir).unwrap();
        let settings = serde_json::json!({
            "otherSetting": true,
            "mcpServers": {
                "existing": {"command": "old"}
            }
        });
        std::fs::write(
            gemini_dir.join("settings.json"),
            serde_json::to_string_pretty(&settings).unwrap(),
        )
        .unwrap();

        let installer = McpInstaller::new("gemini", NormalizedPath::new(temp.path())).unwrap();
        installer
            .install(McpScope::Project, "new-server", &stdio_config("new"))
            .unwrap();

        // Read back and verify both servers exist AND other settings preserved
        let content = std::fs::read_to_string(gemini_dir.join("settings.json")).unwrap();
        let json: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(json["otherSetting"], true);
        assert!(json["mcpServers"]["existing"].is_object());
        assert!(json["mcpServers"]["new-server"].is_object());
    }

    // -- List on empty/nonexistent config ------------------------------------

    #[test]
    fn test_list_empty() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let installer = McpInstaller::new("cursor", root).unwrap();

        let servers = installer.list(McpScope::Project).unwrap();
        assert!(servers.is_empty());
    }

    // -- Tool-specific key verification --------------------------------------

    #[test]
    fn test_vscode_uses_servers_key() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let installer = McpInstaller::new("vscode", root).unwrap();

        installer
            .install(McpScope::Project, "s1", &stdio_config("test"))
            .unwrap();

        let path = temp.path().join(".vscode").join("mcp.json");
        let content = std::fs::read_to_string(&path).unwrap();
        let json: Value = serde_json::from_str(&content).unwrap();
        // VS Code uses "servers" not "mcpServers"
        assert!(json.get("servers").is_some());
        assert!(json.get("mcpServers").is_none());
    }

    #[test]
    fn test_zed_uses_context_servers_key() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let installer = McpInstaller::new("zed", root).unwrap();

        installer
            .install(McpScope::Project, "s1", &stdio_config("test"))
            .unwrap();

        let path = temp.path().join(".zed").join("settings.json");
        let content = std::fs::read_to_string(&path).unwrap();
        let json: Value = serde_json::from_str(&content).unwrap();
        assert!(json.get("context_servers").is_some());
    }
}
