//! MCP configuration registry — maps tool slugs to their native MCP config specs.
//!
//! This is the single source of truth for how each tool stores MCP server
//! definitions: file paths, JSON keys, field names, transport support, and
//! env var interpolation syntax.
//!
//! # Adding a new tool
//!
//! 1. Add a `fn <slug>_mcp_spec() -> McpConfigSpec` function below.
//! 2. Add the slug to the `match` in [`mcp_config_spec`].
//! 3. Add the slug to [`MCP_CAPABLE_TOOLS`].

use repo_meta::schema::{
    McpConfigEmbedding, McpConfigSpec, McpEnvSyntax, McpFieldMappings, McpTransport,
    McpTypeValues, McpUserPath,
};

/// All tool slugs that support MCP, in alphabetical order.
pub const MCP_CAPABLE_TOOLS: &[&str] = &[
    "amazonq",
    "antigravity",
    "claude",
    "claude_desktop",
    "cline",
    "copilot",
    "cursor",
    "gemini",
    "jetbrains",
    "roo",
    "vscode",
    "windsurf",
    "zed",
];

/// Look up the MCP configuration spec for a tool by slug.
///
/// Returns `None` for tools that don't support MCP (e.g., `"aider"`).
pub fn mcp_config_spec(slug: &str) -> Option<McpConfigSpec> {
    match slug {
        "claude" => Some(claude_mcp_spec()),
        "claude_desktop" => Some(claude_desktop_mcp_spec()),
        "gemini" => Some(gemini_mcp_spec()),
        "cursor" => Some(cursor_mcp_spec()),
        "windsurf" => Some(windsurf_mcp_spec()),
        "vscode" => Some(vscode_mcp_spec()),
        "copilot" => Some(copilot_mcp_spec()),
        "antigravity" => Some(antigravity_mcp_spec()),
        "jetbrains" => Some(jetbrains_mcp_spec()),
        "zed" => Some(zed_mcp_spec()),
        "cline" => Some(cline_mcp_spec()),
        "roo" => Some(roo_mcp_spec()),
        "amazonq" => Some(amazonq_mcp_spec()),
        _ => None,
    }
}

// ===========================================================================
// Per-tool MCP config specs
// ===========================================================================

// ---------------------------------------------------------------------------
// 1. Claude Code (CLI)
// ---------------------------------------------------------------------------

fn claude_mcp_spec() -> McpConfigSpec {
    McpConfigSpec {
        servers_key: "mcpServers",
        project_path: Some(".mcp.json"),
        user_path: Some(McpUserPath::HomeRelative(".claude.json")),
        embedding: McpConfigEmbedding::Dedicated,
        transports: &[McpTransport::Stdio, McpTransport::Http],
        field_mappings: McpFieldMappings {
            http_url_field: "url",
            sse_url_field: None,
            requires_type_field: true,
            type_values: McpTypeValues {
                stdio: Some("stdio"),
                http: Some("http"),
                sse: None, // SSE deprecated in Claude Code
            },
        },
        env_syntax: Some(McpEnvSyntax::DollarBrace),
    }
}

// ---------------------------------------------------------------------------
// 2. Claude Desktop (GUI app)
// ---------------------------------------------------------------------------

fn claude_desktop_mcp_spec() -> McpConfigSpec {
    McpConfigSpec {
        servers_key: "mcpServers",
        project_path: None, // Desktop app has no project-level config
        user_path: Some(McpUserPath::OsSpecific {
            macos: "Library/Application Support/Claude/claude_desktop_config.json",
            linux: ".config/Claude/claude_desktop_config.json",
            windows: "AppData/Roaming/Claude/claude_desktop_config.json",
        }),
        embedding: McpConfigEmbedding::Nested, // MCP is inside a larger config file
        transports: &[McpTransport::Stdio],
        field_mappings: McpFieldMappings {
            http_url_field: "url",
            sse_url_field: None,
            requires_type_field: false,
            type_values: McpTypeValues::default(),
        },
        env_syntax: None,
    }
}

// ---------------------------------------------------------------------------
// 3. Gemini CLI
// ---------------------------------------------------------------------------

fn gemini_mcp_spec() -> McpConfigSpec {
    McpConfigSpec {
        servers_key: "mcpServers",
        project_path: Some(".gemini/settings.json"),
        user_path: Some(McpUserPath::HomeRelative(".gemini/settings.json")),
        embedding: McpConfigEmbedding::Nested, // settings.json has other keys too
        transports: &[McpTransport::Stdio, McpTransport::Http, McpTransport::Sse],
        field_mappings: McpFieldMappings {
            http_url_field: "httpUrl", // Gemini uses "httpUrl" for Streamable HTTP
            sse_url_field: Some("url"), // and "url" for SSE
            requires_type_field: false,
            type_values: McpTypeValues::default(),
        },
        env_syntax: Some(McpEnvSyntax::DollarSign),
    }
}

// ---------------------------------------------------------------------------
// 4. Cursor
// ---------------------------------------------------------------------------

fn cursor_mcp_spec() -> McpConfigSpec {
    McpConfigSpec {
        servers_key: "mcpServers",
        project_path: Some(".cursor/mcp.json"),
        user_path: Some(McpUserPath::HomeRelative(".cursor/mcp.json")),
        embedding: McpConfigEmbedding::Dedicated,
        transports: &[McpTransport::Stdio, McpTransport::Http],
        field_mappings: McpFieldMappings {
            http_url_field: "url",
            sse_url_field: None,
            requires_type_field: false, // Cursor auto-infers transport from fields
            type_values: McpTypeValues::default(),
        },
        env_syntax: Some(McpEnvSyntax::DollarEnvColon),
    }
}

// ---------------------------------------------------------------------------
// 5. Windsurf
// ---------------------------------------------------------------------------

fn windsurf_mcp_spec() -> McpConfigSpec {
    McpConfigSpec {
        servers_key: "mcpServers",
        project_path: Some(".windsurf/mcp.json"),
        user_path: Some(McpUserPath::HomeRelative(
            ".codeium/windsurf/mcp_config.json",
        )),
        embedding: McpConfigEmbedding::Dedicated,
        transports: &[McpTransport::Stdio, McpTransport::Http, McpTransport::Sse],
        field_mappings: McpFieldMappings {
            http_url_field: "serverUrl", // Windsurf uses "serverUrl"
            sse_url_field: None,
            requires_type_field: false,
            type_values: McpTypeValues::default(),
        },
        env_syntax: Some(McpEnvSyntax::DollarEnvColon),
    }
}

// ---------------------------------------------------------------------------
// 6. VS Code
// ---------------------------------------------------------------------------

fn vscode_mcp_spec() -> McpConfigSpec {
    McpConfigSpec {
        servers_key: "servers", // VS Code uses "servers", NOT "mcpServers"
        project_path: Some(".vscode/mcp.json"),
        user_path: Some(McpUserPath::OsSpecific {
            macos: "Library/Application Support/Code/User/mcp.json",
            linux: ".config/Code/User/mcp.json",
            windows: "AppData/Roaming/Code/User/mcp.json",
        }),
        embedding: McpConfigEmbedding::Dedicated,
        transports: &[McpTransport::Stdio, McpTransport::Http],
        field_mappings: McpFieldMappings {
            http_url_field: "url",
            sse_url_field: None,
            requires_type_field: true,
            type_values: McpTypeValues {
                stdio: Some("stdio"),
                http: Some("http"),
                sse: None, // SSE deprecated in VS Code
            },
        },
        env_syntax: Some(McpEnvSyntax::VsCodeInput),
    }
}

// ---------------------------------------------------------------------------
// 7. GitHub Copilot (shares VS Code MCP config)
// ---------------------------------------------------------------------------

fn copilot_mcp_spec() -> McpConfigSpec {
    // Copilot uses the same config files as VS Code
    vscode_mcp_spec()
}

// ---------------------------------------------------------------------------
// 8. Antigravity
// ---------------------------------------------------------------------------

fn antigravity_mcp_spec() -> McpConfigSpec {
    McpConfigSpec {
        servers_key: "mcpServers",
        project_path: None, // User-scope only
        user_path: Some(McpUserPath::HomeRelative(
            ".gemini/antigravity/mcp_config.json",
        )),
        embedding: McpConfigEmbedding::Dedicated,
        transports: &[McpTransport::Stdio, McpTransport::Http, McpTransport::Sse],
        field_mappings: McpFieldMappings {
            http_url_field: "serverUrl", // Antigravity uses "serverUrl"
            sse_url_field: None,
            requires_type_field: false,
            type_values: McpTypeValues::default(),
        },
        env_syntax: None, // Antigravity does not support env var interpolation
    }
}

// ---------------------------------------------------------------------------
// 9. JetBrains (Junie)
// ---------------------------------------------------------------------------

fn jetbrains_mcp_spec() -> McpConfigSpec {
    McpConfigSpec {
        servers_key: "mcpServers",
        project_path: Some(".junie/mcp/mcp.json"),
        user_path: Some(McpUserPath::HomeRelative(".junie/mcp/mcp.json")),
        embedding: McpConfigEmbedding::Dedicated,
        transports: &[McpTransport::Stdio, McpTransport::Http, McpTransport::Sse],
        field_mappings: McpFieldMappings {
            http_url_field: "url",
            sse_url_field: None,
            requires_type_field: true,
            type_values: McpTypeValues {
                stdio: Some("command"), // JetBrains uses "command" not "stdio"
                http: Some("url"),      // JetBrains uses "url" not "http"
                sse: Some("sse"),
            },
        },
        env_syntax: None,
    }
}

// ---------------------------------------------------------------------------
// 10. Zed
// ---------------------------------------------------------------------------

fn zed_mcp_spec() -> McpConfigSpec {
    McpConfigSpec {
        servers_key: "context_servers", // Zed uses "context_servers"
        project_path: Some(".zed/settings.json"),
        user_path: Some(McpUserPath::OsSpecific {
            macos: ".zed/settings.json",
            linux: ".config/zed/settings.json",
            windows: ".config/zed/settings.json", // Zed on Windows uses same layout
        }),
        embedding: McpConfigEmbedding::Nested, // Part of larger settings.json
        transports: &[McpTransport::Stdio, McpTransport::Http],
        field_mappings: McpFieldMappings {
            http_url_field: "url",
            sse_url_field: None,
            requires_type_field: false,
            type_values: McpTypeValues::default(),
        },
        env_syntax: None,
    }
}

// ---------------------------------------------------------------------------
// 11. Cline
// ---------------------------------------------------------------------------

fn cline_mcp_spec() -> McpConfigSpec {
    McpConfigSpec {
        servers_key: "mcpServers",
        project_path: None, // Cline is global-only (no project-level MCP config)
        user_path: Some(McpUserPath::VsCodeExtStorage {
            extension_id: "saoudrizwan.claude-dev",
            filename: "cline_mcp_settings.json",
        }),
        embedding: McpConfigEmbedding::Dedicated,
        transports: &[McpTransport::Stdio, McpTransport::Http, McpTransport::Sse],
        field_mappings: McpFieldMappings {
            http_url_field: "url",
            sse_url_field: None,
            requires_type_field: false,
            type_values: McpTypeValues::default(),
        },
        env_syntax: Some(McpEnvSyntax::DollarEnvColon), // ${env:VAR} in args array
    }
}

// ---------------------------------------------------------------------------
// 12. Roo Code
// ---------------------------------------------------------------------------

fn roo_mcp_spec() -> McpConfigSpec {
    McpConfigSpec {
        servers_key: "mcpServers",
        project_path: Some(".roo/mcp.json"),
        user_path: Some(McpUserPath::VsCodeExtStorage {
            extension_id: "rooveterinaryinc.roo-cline",
            filename: "cline_mcp_settings.json",
        }),
        embedding: McpConfigEmbedding::Dedicated,
        transports: &[McpTransport::Stdio, McpTransport::Http, McpTransport::Sse],
        field_mappings: McpFieldMappings {
            http_url_field: "url",
            sse_url_field: None,
            requires_type_field: true, // Roo requires "type" for remote transports
            type_values: McpTypeValues {
                stdio: None, // stdio inferred from command/args
                http: Some("streamable-http"),
                sse: Some("sse"),
            },
        },
        env_syntax: Some(McpEnvSyntax::DollarEnvColon),
    }
}

// ---------------------------------------------------------------------------
// 13. Amazon Q Developer
// ---------------------------------------------------------------------------

fn amazonq_mcp_spec() -> McpConfigSpec {
    // NOTE: Amazon Q has separate config files for CLI and IDE variants:
    //   CLI:  .amazonq/mcp.json       (project), ~/.aws/amazonq/mcp.json     (user)
    //   IDE:  .amazonq/default.json   (project), ~/.aws/amazonq/default.json (user)
    // We currently target the CLI variant only. IDE support is a future extension.
    McpConfigSpec {
        servers_key: "mcpServers",
        project_path: Some(".amazonq/mcp.json"),
        user_path: Some(McpUserPath::HomeRelative(".aws/amazonq/mcp.json")),
        embedding: McpConfigEmbedding::Dedicated,
        transports: &[McpTransport::Stdio, McpTransport::Http],
        field_mappings: McpFieldMappings {
            http_url_field: "url",
            sse_url_field: None,
            requires_type_field: true,
            type_values: McpTypeValues {
                stdio: None, // inferred from command field
                http: Some("http"),
                sse: None,
            },
        },
        env_syntax: None,
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_mcp_capable_tools_have_specs() {
        for slug in MCP_CAPABLE_TOOLS {
            assert!(
                mcp_config_spec(slug).is_some(),
                "Missing MCP spec for tool: {slug}"
            );
        }
    }

    #[test]
    fn test_aider_has_no_mcp() {
        assert!(mcp_config_spec("aider").is_none());
    }

    #[test]
    fn test_unknown_tool_returns_none() {
        assert!(mcp_config_spec("nonexistent").is_none());
    }

    #[test]
    fn test_vscode_uses_servers_key() {
        let spec = mcp_config_spec("vscode").unwrap();
        assert_eq!(spec.servers_key, "servers");
    }

    #[test]
    fn test_copilot_shares_vscode_config() {
        let vs = mcp_config_spec("vscode").unwrap();
        let cp = mcp_config_spec("copilot").unwrap();
        assert_eq!(vs.servers_key, cp.servers_key);
        assert_eq!(vs.project_path, cp.project_path);
    }

    #[test]
    fn test_zed_uses_context_servers_key() {
        let spec = mcp_config_spec("zed").unwrap();
        assert_eq!(spec.servers_key, "context_servers");
    }

    #[test]
    fn test_claude_desktop_is_separate_from_claude() {
        let code = mcp_config_spec("claude").unwrap();
        let desktop = mcp_config_spec("claude_desktop").unwrap();
        // Claude Code has project-level config, Desktop does not
        assert!(code.project_path.is_some());
        assert!(desktop.project_path.is_none());
    }

    #[test]
    fn test_cline_is_user_only() {
        let spec = mcp_config_spec("cline").unwrap();
        assert!(spec.project_path.is_none());
        assert!(spec.user_path.is_some());
    }

    #[test]
    fn test_antigravity_is_user_only() {
        let spec = mcp_config_spec("antigravity").unwrap();
        assert!(spec.project_path.is_none());
        assert!(spec.user_path.is_some());
    }

    #[test]
    fn test_windsurf_uses_server_url() {
        let spec = mcp_config_spec("windsurf").unwrap();
        assert_eq!(spec.field_mappings.http_url_field, "serverUrl");
    }

    #[test]
    fn test_gemini_uses_http_url_field() {
        let spec = mcp_config_spec("gemini").unwrap();
        assert_eq!(spec.field_mappings.http_url_field, "httpUrl");
        assert_eq!(spec.field_mappings.sse_url_field, Some("url"));
    }

    #[test]
    fn test_jetbrains_uses_command_type() {
        let spec = mcp_config_spec("jetbrains").unwrap();
        assert_eq!(spec.field_mappings.type_values.stdio, Some("command"));
    }

    #[test]
    fn test_roo_uses_streamable_http_type() {
        let spec = mcp_config_spec("roo").unwrap();
        assert_eq!(
            spec.field_mappings.type_values.http,
            Some("streamable-http")
        );
    }

    #[test]
    fn test_mcp_capable_tools_count() {
        // 12 original tools with MCP support + claude_desktop = 13
        // (copilot shares VS Code config but is a separate entry)
        assert_eq!(MCP_CAPABLE_TOOLS.len(), 13);
    }

    #[test]
    fn test_mcp_capable_tools_sorted() {
        let mut sorted = MCP_CAPABLE_TOOLS.to_vec();
        sorted.sort();
        assert_eq!(
            sorted,
            MCP_CAPABLE_TOOLS.to_vec(),
            "MCP_CAPABLE_TOOLS must be in alphabetical order"
        );
    }

    #[test]
    fn test_env_syntax_consistency() {
        // Cursor, Windsurf, Roo, Cline all use the same env syntax
        let tools_with_env_colon = ["cursor", "windsurf", "roo", "cline"];
        for slug in tools_with_env_colon {
            let spec = mcp_config_spec(slug).unwrap();
            assert_eq!(
                spec.env_syntax,
                Some(McpEnvSyntax::DollarEnvColon),
                "{slug} should use DollarEnvColon syntax"
            );
        }
    }

    #[test]
    fn test_all_specs_have_user_path() {
        // Every MCP-capable tool should have at least a user-scope path
        for slug in MCP_CAPABLE_TOOLS {
            let spec = mcp_config_spec(slug).unwrap();
            assert!(
                spec.user_path.is_some(),
                "{slug} should have a user-scope MCP path"
            );
        }
    }
}
