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
use tracing::warn;

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

    /// Validate that a server name is acceptable as a JSON key in MCP configs.
    fn validate_server_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(Error::McpInvalidServerName {
                message: "server name must not be empty".into(),
            });
        }
        if name.len() > 255 {
            return Err(Error::McpInvalidServerName {
                message: format!(
                    "server name exceeds maximum length of 255 characters (got {})",
                    name.len()
                ),
            });
        }
        if name.chars().any(|c| c.is_control()) {
            return Err(Error::McpInvalidServerName {
                message: "server name must not contain control characters".into(),
            });
        }
        Ok(())
    }

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
                let parsed: Value =
                    serde_json::from_str(&content).map_err(|e| Error::McpConfig {
                        tool: self.slug.clone(),
                        message: format!("Failed to parse {}: {e}", path.display()),
                    })?;
                if !parsed.is_object() {
                    return Err(Error::McpConfig {
                        tool: self.slug.clone(),
                        message: format!(
                            "Config file {} contains a JSON {}, expected an object",
                            path.display(),
                            json_type_name(&parsed),
                        ),
                    });
                }
                parsed
            }
        } else {
            json!({})
        };
        Ok((path, value))
    }

    /// Write JSON to the config file, creating parent directories as needed.
    ///
    /// Uses atomic write-to-temp-then-rename to prevent config corruption if
    /// the process is interrupted mid-write.
    fn write_config(&self, path: &PathBuf, value: &Value) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::McpConfig {
                tool: self.slug.clone(),
                message: format!("Failed to create directory {}: {e}", parent.display()),
            })?;
        }
        let mut content = serde_json::to_string_pretty(value)?;
        content.push('\n');

        // Atomic write: write to a sibling temp file, then rename.
        let tmp_path = path.with_extension("tmp");
        std::fs::write(&tmp_path, content.as_bytes()).map_err(|e| Error::McpConfig {
            tool: self.slug.clone(),
            message: format!("Failed to write {}: {e}", tmp_path.display()),
        })?;
        std::fs::rename(&tmp_path, path).map_err(|e| Error::McpConfig {
            tool: self.slug.clone(),
            message: format!("Failed to rename {} -> {}: {e}", tmp_path.display(), path.display()),
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
    ///
    /// Callers must ensure `config` is a JSON object (enforced by `read_config`).
    fn get_or_create_servers<'a>(&self, config: &'a mut Value) -> &'a mut Map<String, Value> {
        let obj = config
            .as_object_mut()
            .expect("invariant: config must be a JSON object (enforced by read_config)");
        if !obj.contains_key(self.spec.servers_key) {
            obj.insert(self.spec.servers_key.to_string(), json!({}));
        }
        obj[self.spec.servers_key]
            .as_object_mut()
            .expect("invariant: servers_key value is always inserted as json!({})")
    }

    // -----------------------------------------------------------------------
    // Public API
    // -----------------------------------------------------------------------

    /// Install an MCP server into the tool's config at the given scope.
    ///
    /// If a server with the same name already exists, it is overwritten and a
    /// warning is logged.
    pub fn install(
        &self,
        scope: McpScope,
        server_name: &str,
        config: &McpServerConfig,
    ) -> Result<()> {
        Self::validate_server_name(server_name)?;
        let (path, mut root_value) = self.read_config(scope)?;
        let tool_json = to_tool_json(config, &self.spec);
        let servers = self.get_or_create_servers(&mut root_value);
        if servers.contains_key(server_name) {
            warn!(
                tool = %self.slug,
                server = server_name,
                "overwriting existing server entry with the same name"
            );
        }
        servers.insert(server_name.to_string(), tool_json);
        self.write_config(&path, &root_value)
    }

    /// Remove an MCP server from the tool's config.
    ///
    /// Returns `Ok(true)` if the server was found and removed, `Ok(false)` if
    /// the server was not present.
    pub fn remove(&self, scope: McpScope, server_name: &str) -> Result<bool> {
        Self::validate_server_name(server_name)?;
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
        Self::validate_server_name(server_name)?;
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
            if let Some(obj) = json.as_object() {
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
            } else {
                issues.push("Server entry is not a JSON object".into());
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
    /// `managed_servers` is the authoritative set of servers that should be
    /// managed by the repo-manager. The sync applies the following policy:
    ///
    /// - **Name collision with existing entry**: the managed definition wins
    ///   (overwrite) and a warning is logged so the user knows.
    /// - **Managed server no longer in the set**: it is removed from the config
    ///   and a warning is logged.
    /// - **Unknown servers** (not in `managed_servers` and not in
    ///   `previously_managed`): preserved untouched — they belong to the user
    ///   or another extension.
    ///
    /// `previously_managed` is the set of server names that were managed by
    /// the repo-manager in a prior sync. This is how we tell "ours, now
    /// removed" apart from "user-added". Pass an empty slice on the first sync.
    pub fn sync(
        &self,
        scope: McpScope,
        managed_servers: &std::collections::BTreeMap<String, McpServerConfig>,
        previously_managed: &[String],
    ) -> Result<McpSyncResult> {
        for name in managed_servers.keys() {
            Self::validate_server_name(name)?;
        }

        let (path, mut root_value) = self.read_config(scope)?;
        let servers = self.get_or_create_servers(&mut root_value);

        let mut added = Vec::new();
        let mut updated = Vec::new();
        let mut removed = Vec::new();
        let mut unchanged = Vec::new();

        // Build the set of previously managed names for quick lookup.
        let prev_set: std::collections::HashSet<&str> =
            previously_managed.iter().map(|s| s.as_str()).collect();

        // Compute the desired state for every managed server.
        let mut desired: std::collections::BTreeMap<String, Value> = managed_servers
            .iter()
            .map(|(name, config)| (name.clone(), to_tool_json(config, &self.spec)))
            .collect();

        // Walk existing servers and reconcile with the desired state.
        let existing_names: Vec<String> = servers.keys().cloned().collect();

        for name in &existing_names {
            if let Some(new_value) = desired.remove(name) {
                // Server exists in both current and desired state.
                if servers[name] == new_value {
                    unchanged.push(name.clone());
                } else {
                    warn!(
                        tool = %self.slug,
                        server = %name,
                        "overwriting existing server entry during sync"
                    );
                    servers.insert(name.clone(), new_value);
                    updated.push(name.clone());
                }
            } else if prev_set.contains(name.as_str()) {
                // Was previously managed but is no longer in the desired set —
                // remove it.
                warn!(
                    tool = %self.slug,
                    server = %name,
                    "removing previously-managed server that is no longer in the managed set"
                );
                servers.remove(name);
                removed.push(name.clone());
            }
            // Otherwise it is a user-managed server — preserve it.
        }

        // Add servers that are in desired but not yet in the config.
        for (name, value) in desired {
            if servers.contains_key(&name) {
                // Should not happen (we removed from desired above), but guard.
                warn!(
                    tool = %self.slug,
                    server = %name,
                    "overwriting unexpected existing entry during sync add"
                );
            }
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

/// Return a human-readable name for the JSON value type.
fn json_type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
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

    // -- Server name validation ----------------------------------------------

    #[test]
    fn test_validate_server_name_empty() {
        let temp = TempDir::new().unwrap();
        let installer = McpInstaller::new("cursor", NormalizedPath::new(temp.path())).unwrap();
        let result = installer.install(McpScope::Project, "", &stdio_config("cmd"));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("empty"), "expected 'empty' in: {err}");
    }

    #[test]
    fn test_validate_server_name_too_long() {
        let temp = TempDir::new().unwrap();
        let installer = McpInstaller::new("cursor", NormalizedPath::new(temp.path())).unwrap();
        let long_name = "a".repeat(256);
        let result = installer.install(McpScope::Project, &long_name, &stdio_config("cmd"));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("maximum length"),
            "expected 'maximum length' in: {err}"
        );
    }

    #[test]
    fn test_validate_server_name_control_chars() {
        let temp = TempDir::new().unwrap();
        let installer = McpInstaller::new("cursor", NormalizedPath::new(temp.path())).unwrap();
        let result = installer.install(McpScope::Project, "bad\0name", &stdio_config("cmd"));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("control characters"),
            "expected 'control characters' in: {err}"
        );
    }

    #[test]
    fn test_validate_server_name_accepts_normal_names() {
        let temp = TempDir::new().unwrap();
        let installer = McpInstaller::new("cursor", NormalizedPath::new(temp.path())).unwrap();
        // Various valid server names (spaces, dots, hyphens, unicode are OK)
        for name in &[
            "my-server",
            "my.server",
            "my server",
            "server_1",
            "MCP Server (v2)",
            "unicode-日本語",
        ] {
            installer
                .install(McpScope::Project, name, &stdio_config("cmd"))
                .unwrap();
        }
    }

    #[test]
    fn test_validate_server_name_max_length_boundary() {
        let temp = TempDir::new().unwrap();
        let installer = McpInstaller::new("cursor", NormalizedPath::new(temp.path())).unwrap();
        let name_255 = "a".repeat(255);
        installer
            .install(McpScope::Project, &name_255, &stdio_config("cmd"))
            .unwrap();
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

    #[test]
    fn test_install_http_config() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let installer = McpInstaller::new("windsurf", root).unwrap();

        installer
            .install(
                McpScope::Project,
                "remote",
                &http_config("https://example.com/mcp"),
            )
            .unwrap();

        let servers = installer.list(McpScope::Project).unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].1["serverUrl"], "https://example.com/mcp");
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

    #[test]
    fn test_verify_non_object_server_entry() {
        let temp = TempDir::new().unwrap();
        let cursor_dir = temp.path().join(".cursor");
        std::fs::create_dir_all(&cursor_dir).unwrap();
        std::fs::write(
            cursor_dir.join("mcp.json"),
            r#"{"mcpServers":{"bad": "not-an-object"}}"#,
        )
        .unwrap();

        let installer = McpInstaller::new("cursor", NormalizedPath::new(temp.path())).unwrap();
        let result = installer.verify(McpScope::Project, "bad").unwrap();
        assert!(result.exists);
        assert!(!result.issues.is_empty());
        assert!(result.issues[0].contains("not a JSON object"));
    }

    #[test]
    fn test_verify_server_missing_command_and_url() {
        let temp = TempDir::new().unwrap();
        let cursor_dir = temp.path().join(".cursor");
        std::fs::create_dir_all(&cursor_dir).unwrap();
        std::fs::write(
            cursor_dir.join("mcp.json"),
            r#"{"mcpServers":{"empty": {}}}"#,
        )
        .unwrap();

        let installer = McpInstaller::new("cursor", NormalizedPath::new(temp.path())).unwrap();
        let result = installer.verify(McpScope::Project, "empty").unwrap();
        assert!(result.exists);
        assert!(!result.issues.is_empty());
        assert!(result.issues[0].contains("neither"));
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

        let result = installer
            .sync(McpScope::Project, &managed, &[])
            .unwrap();
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
        let result = installer
            .sync(McpScope::Project, &managed, &[])
            .unwrap();

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
    fn test_sync_removes_previously_managed() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let installer = McpInstaller::new("cursor", root).unwrap();

        // Install two managed servers initially
        installer
            .install(McpScope::Project, "keep-me", &stdio_config("keep"))
            .unwrap();
        installer
            .install(McpScope::Project, "remove-me", &stdio_config("bye"))
            .unwrap();

        // Sync with only "keep-me" in managed set, but both in previously_managed
        let mut managed = BTreeMap::new();
        managed.insert("keep-me".into(), stdio_config("keep"));
        let previously = vec!["keep-me".into(), "remove-me".into()];

        let result = installer
            .sync(McpScope::Project, &managed, &previously)
            .unwrap();

        assert!(result.added.is_empty());
        assert_eq!(result.removed, vec!["remove-me"]);
        assert_eq!(result.unchanged, vec!["keep-me"]);

        // Verify only keep-me remains
        let all = installer.list(McpScope::Project).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].0, "keep-me");
    }

    #[test]
    fn test_sync_does_not_remove_user_servers_when_not_previously_managed() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let installer = McpInstaller::new("cursor", root).unwrap();

        // User adds their own server
        installer
            .install(McpScope::Project, "user-added", &stdio_config("user"))
            .unwrap();
        // We also installed a managed server
        installer
            .install(McpScope::Project, "managed", &stdio_config("ours"))
            .unwrap();

        // Sync with empty managed set but only "managed" was previously ours
        let managed = BTreeMap::new();
        let previously = vec!["managed".into()];

        let result = installer
            .sync(McpScope::Project, &managed, &previously)
            .unwrap();

        assert_eq!(result.removed, vec!["managed"]);

        // user-added should be preserved
        let all = installer.list(McpScope::Project).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].0, "user-added");
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
        let result = installer
            .sync(McpScope::Project, &managed, &[])
            .unwrap();

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
        let result = installer
            .sync(McpScope::Project, &managed, &[])
            .unwrap();

        assert!(result.is_empty());
        assert_eq!(result.unchanged, vec!["s1"]);
    }

    #[test]
    fn test_sync_empty_managed_set() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let installer = McpInstaller::new("cursor", root).unwrap();

        // User has some servers
        installer
            .install(McpScope::Project, "user-s1", &stdio_config("user"))
            .unwrap();

        // Sync with empty set — nothing should change
        let managed = BTreeMap::new();
        let result = installer
            .sync(McpScope::Project, &managed, &[])
            .unwrap();
        assert!(result.is_empty());
        assert!(result.added.is_empty());
        assert!(result.removed.is_empty());

        let all = installer.list(McpScope::Project).unwrap();
        assert_eq!(all.len(), 1);
    }

    // -- Scope not supported -------------------------------------------------

    #[test]
    fn test_scope_not_supported() {
        let temp = TempDir::new().unwrap();
        let installer = McpInstaller::new("antigravity", NormalizedPath::new(temp.path())).unwrap();

        let result = installer.install(McpScope::Project, "s1", &stdio_config("test"));
        assert!(result.is_err());
    }

    // -- Non-object JSON root ------------------------------------------------

    #[test]
    fn test_read_config_rejects_json_array() {
        let temp = TempDir::new().unwrap();
        let cursor_dir = temp.path().join(".cursor");
        std::fs::create_dir_all(&cursor_dir).unwrap();
        std::fs::write(cursor_dir.join("mcp.json"), "[1, 2, 3]").unwrap();

        let installer = McpInstaller::new("cursor", NormalizedPath::new(temp.path())).unwrap();
        let result = installer.list(McpScope::Project);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("array"), "expected 'array' in: {err}");
    }

    #[test]
    fn test_read_config_rejects_json_string() {
        let temp = TempDir::new().unwrap();
        let cursor_dir = temp.path().join(".cursor");
        std::fs::create_dir_all(&cursor_dir).unwrap();
        std::fs::write(cursor_dir.join("mcp.json"), r#""hello""#).unwrap();

        let installer = McpInstaller::new("cursor", NormalizedPath::new(temp.path())).unwrap();
        let result = installer.list(McpScope::Project);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("string"), "expected 'string' in: {err}");
    }

    // -- Trailing newline ----------------------------------------------------

    #[test]
    fn test_written_json_has_trailing_newline() {
        let temp = TempDir::new().unwrap();
        let installer = McpInstaller::new("cursor", NormalizedPath::new(temp.path())).unwrap();
        installer
            .install(McpScope::Project, "s1", &stdio_config("test"))
            .unwrap();

        let path = temp.path().join(".cursor").join("mcp.json");
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(
            content.ends_with('\n'),
            "config file should end with a newline"
        );
    }

    // -- Atomic write --------------------------------------------------------

    #[test]
    fn test_no_leftover_tmp_file() {
        let temp = TempDir::new().unwrap();
        let installer = McpInstaller::new("cursor", NormalizedPath::new(temp.path())).unwrap();
        installer
            .install(McpScope::Project, "s1", &stdio_config("test"))
            .unwrap();

        let tmp_path = temp.path().join(".cursor").join("mcp.tmp");
        assert!(
            !tmp_path.exists(),
            "temp file should be cleaned up after atomic write"
        );
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
