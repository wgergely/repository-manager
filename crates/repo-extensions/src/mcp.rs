//! MCP configuration resolver for extensions.
//!
//! Reads an `mcp.json` file shipped by an extension, resolves template variables
//! (runtime paths, repo root), and produces a `serde_json::Value` that can be
//! merged into each tool's MCP configuration.
//!
//! # Template Variables
//!
//! The resolver supports the following placeholders inside `mcp.json` values:
//!
//! | Variable               | Resolved to                                      |
//! |------------------------|--------------------------------------------------|
//! | `{{runtime.python}}`   | Absolute path to the extension's Python venv      |
//! | `{{root}}`             | Absolute path to the repository / container root   |
//! | `{{extension.source}}` | Absolute path to the extension's source directory  |

use std::path::Path;

use serde_json::Value;

use crate::error::{Error, Result};
use crate::manifest::ExtensionManifest;

/// Variables that can be substituted in `mcp.json` templates.
#[derive(Debug, Clone)]
pub struct ResolveContext {
    /// Absolute path to the repository root (or container root for worktrees).
    pub root: String,
    /// Absolute path to the extension source directory.
    pub extension_source: String,
    /// Absolute path to the Python interpreter in the extension's venv.
    /// `None` if the extension does not use Python.
    pub python_path: Option<String>,
}

/// Read an `mcp.json` from an extension and resolve template variables.
///
/// # Arguments
///
/// * `manifest`  – The parsed `extension.toml` manifest.
/// * `source_dir` – Absolute path to the extension's source directory on disk.
/// * `ctx`        – Template variable values to substitute.
///
/// # Returns
///
/// A JSON object whose keys are MCP server names and values are their
/// resolved configurations, ready to be merged into tool configs.
///
/// Returns `Ok(None)` if the extension does not declare `provides.mcp_config`.
pub fn resolve_mcp_config(
    manifest: &ExtensionManifest,
    source_dir: &Path,
    ctx: &ResolveContext,
) -> Result<Option<Value>> {
    // Check if the extension declares an mcp_config path
    let mcp_config_path = match manifest
        .provides
        .as_ref()
        .and_then(|p| p.mcp_config.as_deref())
    {
        Some(p) => p,
        None => return Ok(None),
    };

    // Resolve the path relative to the extension source directory
    let full_path = source_dir.join(mcp_config_path);
    if !full_path.exists() {
        return Err(Error::McpConfigNotFound {
            path: full_path.clone(),
            extension: manifest.extension.name.clone(),
        });
    }

    let content = std::fs::read_to_string(&full_path).map_err(|e| Error::Io(e))?;
    let mut json: Value =
        serde_json::from_str(&content).map_err(|e| Error::McpConfigParse {
            path: full_path.clone(),
            reason: e.to_string(),
        })?;

    // The mcp.json must be an object at the top level
    if !json.is_object() {
        return Err(Error::McpConfigParse {
            path: full_path,
            reason: "mcp.json must be a JSON object at the top level".to_string(),
        });
    }

    // Resolve template variables throughout the JSON tree
    resolve_templates(&mut json, ctx);

    Ok(Some(json))
}

/// Collect MCP configs from all extensions into a single merged object.
///
/// Each extension contributes its own MCP server entries. If two extensions
/// define the same server name, the later one wins (last-write-wins).
pub fn merge_mcp_configs(configs: &[Value]) -> Value {
    let mut merged = serde_json::Map::new();

    for config in configs {
        if let Some(obj) = config.as_object() {
            for (key, value) in obj {
                merged.insert(key.clone(), value.clone());
            }
        }
    }

    Value::Object(merged)
}

/// Recursively resolve `{{...}}` template variables in all JSON string values.
fn resolve_templates(value: &mut Value, ctx: &ResolveContext) {
    match value {
        Value::String(s) => {
            *s = resolve_string(s, ctx);
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                resolve_templates(item, ctx);
            }
        }
        Value::Object(map) => {
            for v in map.values_mut() {
                resolve_templates(v, ctx);
            }
        }
        _ => {}
    }
}

/// Resolve template placeholders in a single string.
fn resolve_string(s: &str, ctx: &ResolveContext) -> String {
    let mut result = s.to_string();
    result = result.replace("{{root}}", &ctx.root);
    result = result.replace("{{extension.source}}", &ctx.extension_source);
    if let Some(ref python) = ctx.python_path {
        result = result.replace("{{runtime.python}}", python);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::ExtensionManifest;
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    fn make_manifest(mcp_config: Option<&str>) -> ExtensionManifest {
        let provides = if let Some(path) = mcp_config {
            format!(
                r#"
[provides]
mcp = ["test-server"]
mcp_config = "{}"
content_types = []
"#,
                path
            )
        } else {
            String::new()
        };

        let toml = format!(
            r#"
[extension]
name = "test-ext"
version = "1.0.0"

{}
"#,
            provides
        );

        ExtensionManifest::from_toml(&toml).unwrap()
    }

    fn make_ctx() -> ResolveContext {
        ResolveContext {
            root: "/repo".to_string(),
            extension_source: "/repo/.repository/extensions/test-ext".to_string(),
            python_path: Some("/repo/.repository/extensions/test-ext/.venv/bin/python".to_string()),
        }
    }

    #[test]
    fn test_no_mcp_config_returns_none() {
        let manifest = make_manifest(None);
        let ctx = make_ctx();
        let result = resolve_mcp_config(&manifest, Path::new("/tmp"), &ctx).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_reads_and_resolves_mcp_json() {
        let tmp = TempDir::new().unwrap();
        let mcp_json = json!({
            "test-server": {
                "command": "{{runtime.python}}",
                "args": ["{{extension.source}}/scripts/serve.py", "--root", "{{root}}"],
                "env": {}
            }
        });
        fs::write(
            tmp.path().join("mcp.json"),
            serde_json::to_string_pretty(&mcp_json).unwrap(),
        )
        .unwrap();

        let manifest = make_manifest(Some("mcp.json"));
        let ctx = make_ctx();
        let result = resolve_mcp_config(&manifest, tmp.path(), &ctx)
            .unwrap()
            .unwrap();

        let server = &result["test-server"];
        assert_eq!(
            server["command"],
            "/repo/.repository/extensions/test-ext/.venv/bin/python"
        );
        assert_eq!(
            server["args"][0],
            "/repo/.repository/extensions/test-ext/scripts/serve.py"
        );
        assert_eq!(server["args"][1], "--root");
        assert_eq!(server["args"][2], "/repo");
    }

    #[test]
    fn test_missing_mcp_json_returns_error() {
        let tmp = TempDir::new().unwrap();
        let manifest = make_manifest(Some("nonexistent.json"));
        let ctx = make_ctx();
        let result = resolve_mcp_config(&manifest, tmp.path(), &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_json_returns_error() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("mcp.json"), "not json").unwrap();

        let manifest = make_manifest(Some("mcp.json"));
        let ctx = make_ctx();
        let result = resolve_mcp_config(&manifest, tmp.path(), &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_non_object_json_returns_error() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("mcp.json"), "[1,2,3]").unwrap();

        let manifest = make_manifest(Some("mcp.json"));
        let ctx = make_ctx();
        let result = resolve_mcp_config(&manifest, tmp.path(), &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_mcp_configs() {
        let a = json!({"server-a": {"command": "a"}});
        let b = json!({"server-b": {"command": "b"}});
        let merged = merge_mcp_configs(&[a, b]);

        assert!(merged["server-a"].is_object());
        assert!(merged["server-b"].is_object());
    }

    #[test]
    fn test_merge_mcp_configs_last_wins() {
        let a = json!({"server": {"command": "old"}});
        let b = json!({"server": {"command": "new"}});
        let merged = merge_mcp_configs(&[a, b]);

        assert_eq!(merged["server"]["command"], "new");
    }

    #[test]
    fn test_resolve_templates_nested() {
        let ctx = make_ctx();
        let mut value = json!({
            "outer": {
                "inner": "{{root}}/subdir",
                "list": ["{{runtime.python}}", "plain"]
            }
        });

        resolve_templates(&mut value, &ctx);

        assert_eq!(value["outer"]["inner"], "/repo/subdir");
        assert_eq!(
            value["outer"]["list"][0],
            "/repo/.repository/extensions/test-ext/.venv/bin/python"
        );
        assert_eq!(value["outer"]["list"][1], "plain");
    }

    #[test]
    fn test_resolve_without_python() {
        let ctx = ResolveContext {
            root: "/repo".to_string(),
            extension_source: "/ext".to_string(),
            python_path: None,
        };
        let mut value = json!({"cmd": "{{runtime.python}}"});
        resolve_templates(&mut value, &ctx);
        // Unresolved template stays as-is when python_path is None
        assert_eq!(value["cmd"], "{{runtime.python}}");
    }
}
