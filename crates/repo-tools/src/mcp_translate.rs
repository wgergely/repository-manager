//! Translation between canonical MCP server configs and tool-native JSON formats.
//!
//! Each tool has its own conventions for field names, type values, and env var syntax.
//! This module converts a tool-agnostic `McpServerConfig` into the JSON structure
//! that each tool expects, and vice versa.

use repo_meta::schema::{McpConfigSpec, McpServerConfig, McpTransportConfig};
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

/// Convert a canonical `McpServerConfig` into the JSON format expected by a specific tool.
///
/// The translation handles:
/// - Field naming: `url` vs `serverUrl` vs `httpUrl`
/// - Type field: present/absent, and values like `"stdio"` vs `"command"`
/// - Auto-approve: mapped to tool-specific fields
///
/// `auto_approve` is intentionally **not** emitted here because each tool
/// uses a different field name (`alwaysAllow`, `autoApprove`, etc.).
pub fn to_tool_json(config: &McpServerConfig, spec: &McpConfigSpec) -> Value {
    let mut obj = Map::new();
    let fm = &spec.field_mappings;

    match &config.transport {
        McpTransportConfig::Stdio { command, args, cwd } => {
            if fm.requires_type_field
                && let Some(type_val) = fm.type_values.stdio {
                    obj.insert("type".into(), json!(type_val));
                }
            obj.insert("command".into(), json!(command));
            if !args.is_empty() {
                obj.insert("args".into(), json!(args));
            }
            if let Some(cwd) = cwd {
                obj.insert("cwd".into(), json!(cwd));
            }
        }
        McpTransportConfig::Http { url, headers } => {
            if fm.requires_type_field
                && let Some(type_val) = fm.type_values.http {
                    obj.insert("type".into(), json!(type_val));
                }
            obj.insert(fm.http_url_field.into(), json!(url));
            if let Some(headers) = headers {
                obj.insert("headers".into(), json!(headers));
            }
        }
        McpTransportConfig::Sse { url, headers } => {
            if fm.requires_type_field
                && let Some(type_val) = fm.type_values.sse {
                    obj.insert("type".into(), json!(type_val));
                }
            let url_field = fm.sse_url_field.unwrap_or(fm.http_url_field);
            obj.insert(url_field.into(), json!(url));
            if let Some(headers) = headers {
                obj.insert("headers".into(), json!(headers));
            }
        }
    }

    // Add env if present and non-empty.
    if let Some(env) = &config.env
        && !env.is_empty() {
            obj.insert("env".into(), json!(env));
        }

    // NOTE: auto_approve is intentionally omitted — it is tool-specific.

    Value::Object(obj)
}

/// Parse a tool-native JSON server entry back into a canonical `McpServerConfig`.
///
/// Returns `None` if the JSON cannot be parsed into a valid config
/// (e.g., it lacks both a `"command"` field and a recognizable URL field).
///
/// `auto_approve` is always set to `false` because each tool stores it
/// under a different key, and parsing those is the caller's responsibility.
pub fn from_tool_json(value: &Value, spec: &McpConfigSpec) -> Option<McpServerConfig> {
    let obj = value.as_object()?;
    let fm = &spec.field_mappings;

    // Determine transport type.
    let transport = if obj.contains_key("command") {
        // Stdio transport: presence of "command" is the distinguishing signal.
        let command = obj.get("command")?.as_str()?.to_string();
        let args = obj
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        let cwd = obj.get("cwd").and_then(|v| v.as_str()).map(String::from);
        McpTransportConfig::Stdio { command, args, cwd }
    } else if obj.contains_key(fm.http_url_field) {
        // Check whether we should treat this as SSE based on an explicit type
        // field, but only when the http and sse URL fields are the same
        // (otherwise the field name itself disambiguates).
        let is_sse_by_type = obj
            .get("type")
            .and_then(|v| v.as_str())
            .map(|t| fm.type_values.sse == Some(t))
            .unwrap_or(false);

        // If the SSE URL field is the same as (or None, falling back to) the
        // HTTP URL field, we need the type discriminator to tell them apart.
        let sse_field_same = fm.sse_url_field.is_none()
            || fm.sse_url_field == Some(fm.http_url_field);

        if is_sse_by_type && sse_field_same {
            let url = obj.get(fm.http_url_field)?.as_str()?.to_string();
            let headers = extract_headers(obj);
            McpTransportConfig::Sse { url, headers }
        } else {
            let url = obj.get(fm.http_url_field)?.as_str()?.to_string();
            let headers = extract_headers(obj);
            McpTransportConfig::Http { url, headers }
        }
    } else if let Some(sse_field) = fm.sse_url_field {
        // SSE URL field is distinct from HTTP URL field and the entry has it.
        if obj.contains_key(sse_field) {
            let url = obj.get(sse_field)?.as_str()?.to_string();
            let headers = extract_headers(obj);
            McpTransportConfig::Sse { url, headers }
        } else {
            // Try type-based detection as a last resort.
            detect_transport_by_type(obj, fm)?
        }
    } else {
        // Last resort: try to detect from "type" field.
        detect_transport_by_type(obj, fm)?
    };

    // Extract env map.
    let env = obj.get("env").and_then(|v| {
        v.as_object().map(|map| {
            map.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect::<BTreeMap<String, String>>()
        })
    });
    // Normalize empty env to None.
    let env = env.filter(|m| !m.is_empty());

    Some(McpServerConfig {
        transport,
        env,
        auto_approve: false,
    })
}

/// Extract an optional `"headers"` map from a JSON object.
fn extract_headers(obj: &Map<String, Value>) -> Option<BTreeMap<String, String>> {
    obj.get("headers").and_then(|v| {
        v.as_object().map(|map| {
            map.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect::<BTreeMap<String, String>>()
        })
    })
}

/// Attempt to determine transport from an explicit `"type"` JSON field.
fn detect_transport_by_type(
    obj: &Map<String, Value>,
    fm: &repo_meta::schema::McpFieldMappings,
) -> Option<McpTransportConfig> {
    let type_str = obj.get("type")?.as_str()?;

    // Check stdio type values.
    if fm.type_values.stdio == Some(type_str) {
        let command = obj.get("command")?.as_str()?.to_string();
        let args = obj
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        let cwd = obj.get("cwd").and_then(|v| v.as_str()).map(String::from);
        return Some(McpTransportConfig::Stdio { command, args, cwd });
    }

    // Check HTTP type values.
    if fm.type_values.http == Some(type_str) {
        let url = obj.get(fm.http_url_field)?.as_str()?.to_string();
        let headers = extract_headers(obj);
        return Some(McpTransportConfig::Http { url, headers });
    }

    // Check SSE type values.
    if fm.type_values.sse == Some(type_str) {
        let url_field = fm.sse_url_field.unwrap_or(fm.http_url_field);
        let url = obj.get(url_field)?.as_str()?.to_string();
        let headers = extract_headers(obj);
        return Some(McpTransportConfig::Sse { url, headers });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp_registry::{MCP_CAPABLE_TOOLS, mcp_config_spec};

    // -----------------------------------------------------------------------
    // to_tool_json tests — verify per-tool field naming and type values
    // -----------------------------------------------------------------------

    // Test stdio translation for Claude Code (requires type field)
    #[test]
    fn test_to_tool_json_claude_stdio() {
        let spec = mcp_config_spec("claude").unwrap();
        let config = McpServerConfig {
            transport: McpTransportConfig::Stdio {
                command: "npx".into(),
                args: vec!["-y".into(), "some-server".into()],
                cwd: None,
            },
            env: Some(BTreeMap::from([("KEY".into(), "value".into())])),
            auto_approve: false,
        };
        let json = to_tool_json(&config, &spec);
        assert_eq!(json["type"], "stdio");
        assert_eq!(json["command"], "npx");
        assert_eq!(json["args"][0], "-y");
        assert_eq!(json["env"]["KEY"], "value");
    }

    // Test HTTP translation for Windsurf (uses "serverUrl")
    #[test]
    fn test_to_tool_json_windsurf_http() {
        let spec = mcp_config_spec("windsurf").unwrap();
        let config = McpServerConfig {
            transport: McpTransportConfig::Http {
                url: "https://example.com/mcp".into(),
                headers: None,
            },
            env: None,
            auto_approve: false,
        };
        let json = to_tool_json(&config, &spec);
        assert_eq!(json["serverUrl"], "https://example.com/mcp");
        assert!(json.get("url").is_none());
    }

    // Test HTTP for Gemini (uses "httpUrl")
    #[test]
    fn test_to_tool_json_gemini_http() {
        let spec = mcp_config_spec("gemini").unwrap();
        let config = McpServerConfig {
            transport: McpTransportConfig::Http {
                url: "https://example.com/mcp".into(),
                headers: None,
            },
            env: None,
            auto_approve: false,
        };
        let json = to_tool_json(&config, &spec);
        assert_eq!(json["httpUrl"], "https://example.com/mcp");
    }

    // Test SSE for Gemini (uses "url" for SSE, different from HTTP)
    #[test]
    fn test_to_tool_json_gemini_sse() {
        let spec = mcp_config_spec("gemini").unwrap();
        let config = McpServerConfig {
            transport: McpTransportConfig::Sse {
                url: "https://example.com/sse".into(),
                headers: None,
            },
            env: None,
            auto_approve: false,
        };
        let json = to_tool_json(&config, &spec);
        assert_eq!(json["url"], "https://example.com/sse");
    }

    // Test JetBrains uses "command" for stdio type and "url" for HTTP type
    #[test]
    fn test_to_tool_json_jetbrains_stdio() {
        let spec = mcp_config_spec("jetbrains").unwrap();
        let config = McpServerConfig {
            transport: McpTransportConfig::Stdio {
                command: "npx".into(),
                args: vec!["server".into()],
                cwd: None,
            },
            env: None,
            auto_approve: false,
        };
        let json = to_tool_json(&config, &spec);
        assert_eq!(json["type"], "command"); // JetBrains uses "command" not "stdio"
    }

    // Test Roo uses "streamable-http" type
    #[test]
    fn test_to_tool_json_roo_http() {
        let spec = mcp_config_spec("roo").unwrap();
        let config = McpServerConfig {
            transport: McpTransportConfig::Http {
                url: "https://example.com/mcp".into(),
                headers: Some(BTreeMap::from([(
                    "Authorization".into(),
                    "Bearer token".into(),
                )])),
            },
            env: None,
            auto_approve: false,
        };
        let json = to_tool_json(&config, &spec);
        assert_eq!(json["type"], "streamable-http");
        assert_eq!(json["url"], "https://example.com/mcp");
        assert_eq!(json["headers"]["Authorization"], "Bearer token");
    }

    // Test Cursor does NOT add type field (auto-inferred)
    #[test]
    fn test_to_tool_json_cursor_no_type_field() {
        let spec = mcp_config_spec("cursor").unwrap();
        let config = McpServerConfig {
            transport: McpTransportConfig::Stdio {
                command: "npx".into(),
                args: vec![],
                cwd: None,
            },
            env: None,
            auto_approve: false,
        };
        let json = to_tool_json(&config, &spec);
        assert!(json.get("type").is_none());
        assert_eq!(json["command"], "npx");
    }

    // Test VS Code uses "servers" key and requires type field
    #[test]
    fn test_to_tool_json_vscode_http() {
        let spec = mcp_config_spec("vscode").unwrap();
        let config = McpServerConfig {
            transport: McpTransportConfig::Http {
                url: "https://example.com/mcp".into(),
                headers: None,
            },
            env: None,
            auto_approve: false,
        };
        let json = to_tool_json(&config, &spec);
        assert_eq!(json["type"], "http");
        assert_eq!(json["url"], "https://example.com/mcp");
    }

    // Test VS Code stdio requires type field
    #[test]
    fn test_to_tool_json_vscode_stdio() {
        let spec = mcp_config_spec("vscode").unwrap();
        let config = McpServerConfig {
            transport: McpTransportConfig::Stdio {
                command: "node".into(),
                args: vec!["server.js".into()],
                cwd: Some("/home/user".into()),
            },
            env: None,
            auto_approve: false,
        };
        let json = to_tool_json(&config, &spec);
        assert_eq!(json["type"], "stdio");
        assert_eq!(json["command"], "node");
        assert_eq!(json["cwd"], "/home/user");
    }

    // Test Gemini HTTP with headers
    #[test]
    fn test_to_tool_json_gemini_http_with_headers() {
        let spec = mcp_config_spec("gemini").unwrap();
        let config = McpServerConfig {
            transport: McpTransportConfig::Http {
                url: "https://example.com/mcp".into(),
                headers: Some(BTreeMap::from([(
                    "Authorization".into(),
                    "Bearer xxx".into(),
                )])),
            },
            env: None,
            auto_approve: false,
        };
        let json = to_tool_json(&config, &spec);
        assert_eq!(json["httpUrl"], "https://example.com/mcp");
        assert_eq!(json["headers"]["Authorization"], "Bearer xxx");
    }

    // Test SSE for Roo requires type "sse"
    #[test]
    fn test_to_tool_json_roo_sse() {
        let spec = mcp_config_spec("roo").unwrap();
        let config = McpServerConfig {
            transport: McpTransportConfig::Sse {
                url: "https://example.com/events".into(),
                headers: None,
            },
            env: None,
            auto_approve: false,
        };
        let json = to_tool_json(&config, &spec);
        assert_eq!(json["type"], "sse");
        assert_eq!(json["url"], "https://example.com/events");
    }

    // Test Roo stdio does not emit type (type_values.stdio is None)
    #[test]
    fn test_to_tool_json_roo_stdio_no_type() {
        let spec = mcp_config_spec("roo").unwrap();
        let config = McpServerConfig {
            transport: McpTransportConfig::Stdio {
                command: "npx".into(),
                args: vec!["-y".into(), "server".into()],
                cwd: None,
            },
            env: None,
            auto_approve: false,
        };
        let json = to_tool_json(&config, &spec);
        assert!(json.get("type").is_none());
        assert_eq!(json["command"], "npx");
    }

    // Test Amazon Q stdio does not emit type (type_values.stdio is None)
    #[test]
    fn test_to_tool_json_amazonq_stdio_no_type() {
        let spec = mcp_config_spec("amazonq").unwrap();
        let config = McpServerConfig {
            transport: McpTransportConfig::Stdio {
                command: "node".into(),
                args: vec!["index.js".into()],
                cwd: None,
            },
            env: None,
            auto_approve: false,
        };
        let json = to_tool_json(&config, &spec);
        assert!(json.get("type").is_none());
        assert_eq!(json["command"], "node");
        assert_eq!(json["args"][0], "index.js");
    }

    // Test Amazon Q HTTP gets type "http"
    #[test]
    fn test_to_tool_json_amazonq_http() {
        let spec = mcp_config_spec("amazonq").unwrap();
        let config = McpServerConfig {
            transport: McpTransportConfig::Http {
                url: "https://example.com/mcp".into(),
                headers: None,
            },
            env: None,
            auto_approve: false,
        };
        let json = to_tool_json(&config, &spec);
        assert_eq!(json["type"], "http");
        assert_eq!(json["url"], "https://example.com/mcp");
    }

    // Test cwd field on stdio
    #[test]
    fn test_to_tool_json_with_cwd() {
        let spec = mcp_config_spec("cursor").unwrap();
        let config = McpServerConfig {
            transport: McpTransportConfig::Stdio {
                command: "node".into(),
                args: vec!["server.js".into()],
                cwd: Some("/home/user/project".into()),
            },
            env: None,
            auto_approve: false,
        };
        let json = to_tool_json(&config, &spec);
        assert_eq!(json["cwd"], "/home/user/project");
    }

    // Test empty args are omitted from output
    #[test]
    fn test_to_tool_json_empty_args_omitted() {
        let spec = mcp_config_spec("jetbrains").unwrap();
        let config = McpServerConfig {
            transport: McpTransportConfig::Stdio {
                command: "npx".into(),
                args: vec![],
                cwd: None,
            },
            env: None,
            auto_approve: false,
        };
        let json = to_tool_json(&config, &spec);
        assert_eq!(json["type"], "command");
        assert!(json.get("args").is_none());
    }

    // -----------------------------------------------------------------------
    // Env handling tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_env_not_added_when_none() {
        let spec = mcp_config_spec("cursor").unwrap();
        let config = McpServerConfig {
            transport: McpTransportConfig::Stdio {
                command: "cmd".into(),
                args: vec![],
                cwd: None,
            },
            env: None,
            auto_approve: false,
        };
        let json = to_tool_json(&config, &spec);
        assert!(json.get("env").is_none());
    }

    #[test]
    fn test_env_not_added_when_empty() {
        let spec = mcp_config_spec("cursor").unwrap();
        let config = McpServerConfig {
            transport: McpTransportConfig::Stdio {
                command: "cmd".into(),
                args: vec![],
                cwd: None,
            },
            env: Some(BTreeMap::new()),
            auto_approve: false,
        };
        let json = to_tool_json(&config, &spec);
        assert!(json.get("env").is_none());
    }

    // -----------------------------------------------------------------------
    // Auto-approve tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_auto_approve_not_emitted() {
        let spec = mcp_config_spec("roo").unwrap();
        let config = McpServerConfig {
            transport: McpTransportConfig::Stdio {
                command: "cmd".into(),
                args: vec![],
                cwd: None,
            },
            env: None,
            auto_approve: true,
        };
        let json = to_tool_json(&config, &spec);
        assert!(json.get("auto_approve").is_none());
        assert!(json.get("autoApprove").is_none());
        assert!(json.get("alwaysAllow").is_none());
    }

    #[test]
    fn test_from_tool_json_auto_approve_always_false() {
        let spec = mcp_config_spec("cursor").unwrap();
        let json = json!({
            "command": "npx",
            "args": ["-y", "server"],
            "alwaysAllow": ["read", "write"]
        });
        let config = from_tool_json(&json, &spec).unwrap();
        assert!(!config.auto_approve);
    }

    // -----------------------------------------------------------------------
    // Roundtrip tests — to_tool_json -> from_tool_json
    // -----------------------------------------------------------------------

    // Test roundtrip: to_tool_json -> from_tool_json for stdio
    #[test]
    fn test_roundtrip_stdio() {
        let spec = mcp_config_spec("claude").unwrap();
        let original = McpServerConfig {
            transport: McpTransportConfig::Stdio {
                command: "npx".into(),
                args: vec!["-y".into(), "server".into()],
                cwd: Some("/tmp".into()),
            },
            env: Some(BTreeMap::from([("KEY".into(), "val".into())])),
            auto_approve: false,
        };
        let json = to_tool_json(&original, &spec);
        let roundtripped = from_tool_json(&json, &spec).unwrap();
        match roundtripped.transport {
            McpTransportConfig::Stdio {
                ref command,
                ref args,
                ref cwd,
            } => {
                assert_eq!(command, "npx");
                assert_eq!(args, &vec!["-y".to_string(), "server".to_string()]);
                assert_eq!(cwd.as_deref(), Some("/tmp"));
            }
            _ => panic!("Expected Stdio transport"),
        }
        assert_eq!(roundtripped.env.unwrap()["KEY"], "val");
    }

    // Test roundtrip for HTTP
    #[test]
    fn test_roundtrip_http() {
        let spec = mcp_config_spec("windsurf").unwrap();
        let original = McpServerConfig {
            transport: McpTransportConfig::Http {
                url: "https://example.com".into(),
                headers: Some(BTreeMap::from([("Auth".into(), "token".into())])),
            },
            env: None,
            auto_approve: false,
        };
        let json = to_tool_json(&original, &spec);
        let roundtripped = from_tool_json(&json, &spec).unwrap();
        match roundtripped.transport {
            McpTransportConfig::Http {
                ref url,
                ref headers,
            } => {
                assert_eq!(url, "https://example.com");
                assert_eq!(headers.as_ref().unwrap()["Auth"], "token");
            }
            _ => panic!("Expected Http transport"),
        }
    }

    // Test roundtrip SSE for Gemini (distinct url fields)
    #[test]
    fn test_roundtrip_sse_gemini() {
        let spec = mcp_config_spec("gemini").unwrap();
        let original = McpServerConfig {
            transport: McpTransportConfig::Sse {
                url: "https://example.com/sse".into(),
                headers: Some(BTreeMap::from([("X-Key".into(), "abc".into())])),
            },
            env: None,
            auto_approve: false,
        };
        let json = to_tool_json(&original, &spec);
        assert_eq!(json["url"], "https://example.com/sse");
        let recovered = from_tool_json(&json, &spec).unwrap();
        match recovered.transport {
            McpTransportConfig::Sse { url, headers } => {
                assert_eq!(url, "https://example.com/sse");
                assert_eq!(headers.unwrap()["X-Key"], "abc");
            }
            _ => panic!("expected Sse"),
        }
    }

    // Test roundtrip HTTP for Gemini (httpUrl field)
    #[test]
    fn test_roundtrip_http_gemini() {
        let spec = mcp_config_spec("gemini").unwrap();
        let original = McpServerConfig {
            transport: McpTransportConfig::Http {
                url: "https://example.com/http".into(),
                headers: None,
            },
            env: None,
            auto_approve: false,
        };
        let json = to_tool_json(&original, &spec);
        assert_eq!(json["httpUrl"], "https://example.com/http");
        let recovered = from_tool_json(&json, &spec).unwrap();
        match recovered.transport {
            McpTransportConfig::Http { url, .. } => {
                assert_eq!(url, "https://example.com/http");
            }
            _ => panic!("expected Http"),
        }
    }

    // Test roundtrip stdio with cwd and env
    #[test]
    fn test_roundtrip_stdio_with_cwd() {
        let spec = mcp_config_spec("vscode").unwrap();
        let original = McpServerConfig {
            transport: McpTransportConfig::Stdio {
                command: "python".into(),
                args: vec!["-m".into(), "server".into()],
                cwd: Some("/workspace/project".into()),
            },
            env: Some(BTreeMap::from([
                ("FOO".into(), "bar".into()),
                ("BAZ".into(), "qux".into()),
            ])),
            auto_approve: false,
        };
        let json = to_tool_json(&original, &spec);
        assert_eq!(json["type"], "stdio");
        assert_eq!(json["command"], "python");
        assert_eq!(json["cwd"], "/workspace/project");
        assert_eq!(json["env"]["FOO"], "bar");
        assert_eq!(json["env"]["BAZ"], "qux");

        let recovered = from_tool_json(&json, &spec).unwrap();
        match recovered.transport {
            McpTransportConfig::Stdio { command, args, cwd } => {
                assert_eq!(command, "python");
                assert_eq!(args, vec!["-m", "server"]);
                assert_eq!(cwd.unwrap(), "/workspace/project");
            }
            _ => panic!("expected Stdio"),
        }
        let env = recovered.env.unwrap();
        assert_eq!(env.len(), 2);
        assert_eq!(env["FOO"], "bar");
    }

    // -----------------------------------------------------------------------
    // from_tool_json error/edge-case tests
    // -----------------------------------------------------------------------

    // Test from_tool_json with invalid input
    #[test]
    fn test_from_tool_json_invalid() {
        let spec = mcp_config_spec("claude").unwrap();
        assert!(from_tool_json(&json!(42), &spec).is_none());
        assert!(from_tool_json(&json!({}), &spec).is_none());
    }

    #[test]
    fn test_from_tool_json_rejects_non_object() {
        let spec = mcp_config_spec("cursor").unwrap();
        assert!(from_tool_json(&json!("string"), &spec).is_none());
        assert!(from_tool_json(&json!(42), &spec).is_none());
        assert!(from_tool_json(&json!(null), &spec).is_none());
        assert!(from_tool_json(&json!([1, 2, 3]), &spec).is_none());
    }

    #[test]
    fn test_from_tool_json_unknown_format() {
        let spec = mcp_config_spec("cursor").unwrap();
        let json = json!({"unknown_field": "value"});
        assert!(from_tool_json(&json, &spec).is_none());
    }

    // -----------------------------------------------------------------------
    // Comprehensive coverage: all 13 MCP-capable tools
    // -----------------------------------------------------------------------

    // Test that all 13 MCP-capable tools can translate a basic stdio config
    #[test]
    fn test_all_tools_translate_stdio() {
        let config = McpServerConfig {
            transport: McpTransportConfig::Stdio {
                command: "test-server".into(),
                args: vec![],
                cwd: None,
            },
            env: None,
            auto_approve: false,
        };
        for slug in MCP_CAPABLE_TOOLS {
            let spec = mcp_config_spec(slug).unwrap();
            let json = to_tool_json(&config, &spec);
            assert!(
                json.is_object(),
                "to_tool_json for {slug} must return an object"
            );
            assert_eq!(
                json["command"], "test-server",
                "command field wrong for {slug}"
            );
        }
    }
}
