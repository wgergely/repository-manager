//! Generic tool integration that uses ToolDefinition schema
//!
//! This module provides a generic implementation of `ToolIntegration` that is
//! driven by `ToolDefinition` schemas loaded from TOML files. This allows new
//! tools to be added without writing Rust code.

use crate::error::Result;
use crate::integration::{ConfigLocation, ConfigType, Rule, SyncContext, ToolIntegration};
use repo_blocks::upsert_block;
use repo_fs::{NormalizedPath, io};
use repo_meta::schema::ToolDefinition;
use serde_json::{Value, json};

/// Sanitize a string for use as a filename.
fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

/// Generic tool integration driven by ToolDefinition schema.
///
/// This implementation uses the schema to determine:
/// - Where to write configuration (config_path)
/// - How to format the output (config_type)
/// - Where to place values in JSON structures (schema_keys)
#[derive(Debug, Clone)]
pub struct GenericToolIntegration {
    definition: ToolDefinition,
    /// If true, insert rule content directly without adding headers
    raw_content: bool,
}

impl GenericToolIntegration {
    /// Create a new generic integration from a tool definition.
    pub fn new(definition: ToolDefinition) -> Self {
        Self {
            definition,
            raw_content: false,
        }
    }

    /// Create from a definition (alias for new).
    pub fn from_definition(definition: ToolDefinition) -> Self {
        Self::new(definition)
    }

    /// Set raw content mode (no headers added around rule content).
    ///
    /// When true, rule content is inserted directly into managed blocks
    /// without adding `## {rule_id}` headers.
    pub fn with_raw_content(mut self, raw: bool) -> Self {
        self.raw_content = raw;
        self
    }

    /// Get the underlying tool definition.
    pub fn definition(&self) -> &ToolDefinition {
        &self.definition
    }

    /// Check if the primary config path is a directory (ends with /).
    fn is_directory_config(&self) -> bool {
        self.definition.integration.config_path.ends_with('/')
    }

    /// Get the config file path for this tool.
    /// For directory configs, this returns the directory path.
    fn config_path(&self, root: &NormalizedPath) -> NormalizedPath {
        root.join(&self.definition.integration.config_path)
    }

    /// Sync rules to a text-based config file using managed blocks.
    fn sync_text(&self, context: &SyncContext, rules: &[Rule]) -> Result<()> {
        // If config_path is a directory, write each rule to a separate file
        if self.is_directory_config() {
            return self.sync_to_directory(context, rules);
        }

        let path = self.config_path(&context.root);
        self.sync_text_to_path(&path, rules)
    }

    /// Write rules as text with managed blocks to an explicit path.
    fn sync_text_to_path(&self, path: &NormalizedPath, rules: &[Rule]) -> Result<()> {
        // Load existing content or start empty
        let mut content = if path.exists() {
            io::read_text(path).map_err(|e| {
                tracing::warn!("Failed to read existing config at {}: {}", path.as_str(), e);
                e
            })?
        } else {
            String::new()
        };

        // Insert/update each rule as a managed block
        for rule in rules {
            let block_content = if self.raw_content {
                rule.content.clone()
            } else {
                format!("## {}\n\n{}", rule.id, rule.content)
            };
            content = upsert_block(&content, &rule.id, &block_content)?;
        }

        io::write_text(path, &content)?;

        Ok(())
    }

    /// Sync rules to a directory, creating one file per rule.
    fn sync_to_directory(&self, context: &SyncContext, rules: &[Rule]) -> Result<()> {
        let dir_path = self.config_path(&context.root);
        self.sync_to_directory_at_path(&dir_path, rules)
    }

    /// Write rules as individual files to an explicit directory path.
    fn sync_to_directory_at_path(&self, dir_path: &NormalizedPath, rules: &[Rule]) -> Result<()> {
        let native = dir_path.to_native();

        // If a regular file exists at this path, remove it first so we can
        // create a directory (e.g., `.clinerules` file -> `.clinerules/` dir).
        if native.is_file() {
            std::fs::remove_file(&native).map_err(|e| crate::Error::SyncFailed {
                tool: self.definition.meta.slug.clone(),
                message: format!(
                    "Failed to remove existing file at {} to create directory: {}",
                    dir_path.as_str(),
                    e
                ),
            })?;
        }

        // Create directory if it doesn't exist
        if !dir_path.exists() {
            std::fs::create_dir_all(dir_path.as_ref()).map_err(|e| crate::Error::SyncFailed {
                tool: self.definition.meta.slug.clone(),
                message: format!("Failed to create directory: {}", e),
            })?;
        }

        // Write each rule to a separate file
        for (i, rule) in rules.iter().enumerate() {
            let filename = format!("{:02}-{}.md", i + 1, sanitize_filename(&rule.id));
            let file_path = dir_path.join(&filename);

            let content = if self.raw_content {
                rule.content.clone()
            } else {
                format!("# {}\n\n{}", rule.id, rule.content)
            };

            io::write_text(&file_path, &content)?;
        }

        Ok(())
    }

    /// Sync rules to a YAML config file using proper YAML comments.
    fn sync_yaml(&self, context: &SyncContext, rules: &[Rule]) -> Result<()> {
        // If config_path is a directory, write each rule to a separate file
        if self.is_directory_config() {
            return self.sync_to_directory(context, rules);
        }

        let path = self.config_path(&context.root);
        self.sync_yaml_to_path(&path, rules)
    }

    /// Write rules as YAML comments to an explicit path.
    fn sync_yaml_to_path(&self, path: &NormalizedPath, rules: &[Rule]) -> Result<()> {
        // For YAML, we use # comments instead of HTML-style managed blocks
        let mut content = String::new();

        // Add header comment
        content.push_str("# Configuration managed by Repository Manager\n");
        content.push_str("# Do not edit the sections between repo:block markers\n\n");

        for rule in rules {
            // Use YAML comment style for block markers
            content.push_str(&format!("# repo:block:{}\n", rule.id));
            if self.raw_content {
                content.push_str(&rule.content);
            } else {
                content.push_str(&format!("# {}\n", rule.id));
                // Indent content as YAML comment
                for line in rule.content.lines() {
                    content.push_str(&format!("# {}\n", line));
                }
            }
            content.push_str(&format!("# /repo:block:{}\n\n", rule.id));
        }

        io::write_text(path, &content)?;

        Ok(())
    }

    /// Sync rules to a JSON config file using schema keys.
    fn sync_json(&self, context: &SyncContext, rules: &[Rule]) -> Result<()> {
        let path = self.config_path(&context.root);
        self.sync_json_to_path(&path, context, rules)
    }

    /// Write rules as JSON to an explicit path using schema keys.
    fn sync_json_to_path(
        &self,
        path: &NormalizedPath,
        context: &SyncContext,
        rules: &[Rule],
    ) -> Result<()> {
        // Load existing or create new
        let mut settings: Value = if path.exists() {
            let content = io::read_text(path)?;
            serde_json::from_str(&content)?
        } else {
            json!({})
        };

        // Ensure we have an object
        if !settings.is_object() {
            settings = json!({});
        }

        // Apply schema-driven keys
        if let Some(ref schema_keys) = self.definition.schema_keys {
            // Python path
            if let (Some(key), Some(python_path)) =
                (&schema_keys.python_path_key, &context.python_path)
            {
                settings[key] = json!(python_path.as_str());
            }

            // Custom instructions (concatenate all rules)
            if let Some(ref key) = schema_keys.instruction_key
                && !rules.is_empty()
            {
                let instructions: String = if self.raw_content {
                    rules
                        .iter()
                        .map(|r| r.content.as_str())
                        .collect::<Vec<_>>()
                        .join("\n\n")
                } else {
                    rules
                        .iter()
                        .map(|r| format!("## {}\n{}", r.id, r.content))
                        .collect::<Vec<_>>()
                        .join("\n\n")
                };
                settings[key] = json!(instructions);
            }

            // MCP servers
            if let (Some(key), Some(mcp_servers)) = (&schema_keys.mcp_key, &context.mcp_servers) {
                settings[key] = mcp_servers.clone();
            }
        }

        let content = serde_json::to_string_pretty(&settings)?;
        io::write_text(path, &content)?;

        Ok(())
    }

    /// Sync rules to a markdown config file using managed blocks.
    fn sync_markdown(&self, context: &SyncContext, rules: &[Rule]) -> Result<()> {
        // Markdown uses the same approach as text with managed blocks
        self.sync_text(context, rules)
    }

    /// Sync rules to all additional paths declared in the tool definition.
    ///
    /// For each additional path, infers the config type from the path extension:
    /// - Paths ending in `/` -> directory sync (one file per rule)
    /// - Paths ending in `.json` -> JSON sync
    /// - Paths ending in `.md` -> Markdown sync
    /// - Paths ending in `.yml` or `.yaml` -> YAML sync
    /// - Everything else -> Text sync
    fn sync_additional_paths(&self, context: &SyncContext, rules: &[Rule]) -> Result<()> {
        for additional_path in &self.definition.integration.additional_paths {
            let resolved = context.root.join(additional_path);

            if additional_path.ends_with('/') {
                // Directory sync: create directory, write one file per rule
                self.sync_to_directory_at_path(&resolved, rules)?;
            } else if additional_path.ends_with(".json") {
                // JSON sync
                self.sync_json_to_path(&resolved, context, rules)?;
            } else if additional_path.ends_with(".md") {
                // Markdown sync (same as text with managed blocks)
                self.sync_text_to_path(&resolved, rules)?;
            } else if additional_path.ends_with(".yml") || additional_path.ends_with(".yaml") {
                // YAML sync
                self.sync_yaml_to_path(&resolved, rules)?;
            } else {
                // Default: text sync with managed blocks
                self.sync_text_to_path(&resolved, rules)?;
            }
        }

        Ok(())
    }
}

impl ToolIntegration for GenericToolIntegration {
    /// Returns the tool's slug (machine identifier), not display name.
    /// This matches the ToolIntegration trait's intended usage where
    /// `name()` returns an identifier for dispatch and comparison.
    #[allow(clippy::misnamed_getters)]
    fn name(&self) -> &str {
        &self.definition.meta.slug
    }

    fn config_locations(&self) -> Vec<ConfigLocation> {
        let config_type = self.definition.integration.config_type;
        let primary_path = &self.definition.integration.config_path;

        // Check if primary path is a directory (ends with /)
        let mut locations = if primary_path.ends_with('/') {
            vec![ConfigLocation::directory(primary_path, config_type)]
        } else {
            vec![ConfigLocation::file(primary_path, config_type)]
        };

        for path in &self.definition.integration.additional_paths {
            // Paths ending with / are directories
            let is_dir = path.ends_with('/');
            if is_dir {
                locations.push(ConfigLocation::directory(path, config_type));
            } else {
                locations.push(ConfigLocation::file(path, config_type));
            }
        }

        locations
    }

    fn sync(&self, context: &SyncContext, rules: &[Rule]) -> Result<()> {
        match self.definition.integration.config_type {
            ConfigType::Text => self.sync_text(context, rules)?,
            ConfigType::Json => self.sync_json(context, rules)?,
            ConfigType::Markdown => self.sync_markdown(context, rules)?,
            ConfigType::Yaml => self.sync_yaml(context, rules)?,
            ConfigType::Toml => {
                // TOML uses # comments like YAML
                self.sync_yaml(context, rules)?;
            }
        }

        // Sync additional paths (if any)
        self.sync_additional_paths(context, rules)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_meta::schema::{ToolCapabilities, ToolIntegrationConfig, ToolMeta, ToolSchemaKeys};
    use std::fs;
    use tempfile::TempDir;

    fn create_text_definition() -> ToolDefinition {
        ToolDefinition {
            meta: ToolMeta {
                name: "Test Tool".to_string(),
                slug: "test-tool".to_string(),
                description: Some("A test tool".to_string()),
            },
            integration: ToolIntegrationConfig {
                config_path: ".testrules".to_string(),
                config_type: ConfigType::Text,
                additional_paths: vec![],
            },
            capabilities: ToolCapabilities::default(),
            schema_keys: None,
        }
    }

    #[test]
    fn test_name() {
        let def = create_text_definition();
        let integration = GenericToolIntegration::new(def);
        assert_eq!(integration.name(), "test-tool");
    }

    #[test]
    fn test_config_locations() {
        let mut def = create_text_definition();
        def.integration.additional_paths = vec![".test/rules/".to_string()];

        let integration = GenericToolIntegration::new(def);
        let locations = integration.config_locations();

        assert_eq!(locations.len(), 2);
        assert_eq!(locations[0].path, ".testrules");
        assert_eq!(locations[0].config_type, ConfigType::Text);
        assert!(!locations[0].is_directory);

        assert_eq!(locations[1].path, ".test/rules/");
        assert!(locations[1].is_directory);
    }

    #[test]
    fn test_sync_text() {
        let temp = TempDir::new().unwrap();
        let def = create_text_definition();
        let integration = GenericToolIntegration::new(def);

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![Rule {
            id: "test-rule".to_string(),
            content: "Test content".to_string(),
        }];

        integration.sync(&context, &rules).unwrap();

        let content = fs::read_to_string(temp.path().join(".testrules")).unwrap();
        assert!(content.contains("test-rule"));
        assert!(content.contains("Test content"));
        assert!(content.contains("<!-- repo:block:test-rule -->"));
    }

    #[test]
    fn test_sync_json_with_schema_keys() {
        let temp = TempDir::new().unwrap();

        let definition = ToolDefinition {
            meta: ToolMeta {
                name: "JSON Tool".to_string(),
                slug: "json-tool".to_string(),
                description: None,
            },
            integration: ToolIntegrationConfig {
                config_path: "config.json".to_string(),
                config_type: ConfigType::Json,
                additional_paths: vec![],
            },
            capabilities: ToolCapabilities::default(),
            schema_keys: Some(ToolSchemaKeys {
                instruction_key: Some("customInstructions".to_string()),
                mcp_key: None,
                python_path_key: Some("pythonPath".to_string()),
            }),
        };

        let integration = GenericToolIntegration::new(definition);
        let context = SyncContext::new(NormalizedPath::new(temp.path()))
            .with_python(NormalizedPath::new("/usr/bin/python3"));

        let rules = vec![Rule {
            id: "rule1".to_string(),
            content: "Content 1".to_string(),
        }];

        integration.sync(&context, &rules).unwrap();

        let content = fs::read_to_string(temp.path().join("config.json")).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert!(json.get("customInstructions").is_some());
        assert_eq!(json["pythonPath"], "/usr/bin/python3");
    }

    #[test]
    fn test_sync_json_with_mcp_servers() {
        let temp = TempDir::new().unwrap();

        let definition = ToolDefinition {
            meta: ToolMeta {
                name: "MCP Tool".to_string(),
                slug: "mcp-tool".to_string(),
                description: None,
            },
            integration: ToolIntegrationConfig {
                config_path: "config.json".to_string(),
                config_type: ConfigType::Json,
                additional_paths: vec![],
            },
            capabilities: ToolCapabilities {
                supports_custom_instructions: false,
                supports_mcp: true,
                supports_rules_directory: false,
            },
            schema_keys: Some(ToolSchemaKeys {
                instruction_key: None,
                mcp_key: Some("mcpServers".to_string()),
                python_path_key: None,
            }),
        };

        let integration = GenericToolIntegration::new(definition);
        let mcp_data = serde_json::json!({
            "my-server": {
                "command": "/usr/bin/python3",
                "args": ["serve.py", "--port", "8080"]
            }
        });
        let context = SyncContext::new(NormalizedPath::new(temp.path())).with_mcp_servers(mcp_data);

        // No rules — just MCP config
        integration.sync(&context, &[]).unwrap();

        let content = fs::read_to_string(temp.path().join("config.json")).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert!(
            json["mcpServers"].is_object(),
            "mcpServers must exist as object"
        );
        assert_eq!(
            json["mcpServers"]["my-server"]["command"],
            "/usr/bin/python3"
        );
        assert_eq!(json["mcpServers"]["my-server"]["args"][0], "serve.py");
        assert_eq!(json["mcpServers"]["my-server"]["args"][1], "--port");
        assert_eq!(json["mcpServers"]["my-server"]["args"][2], "8080");
    }

    #[test]
    fn test_sync_json_mcp_without_mcp_key_is_noop() {
        let temp = TempDir::new().unwrap();

        let definition = ToolDefinition {
            meta: ToolMeta {
                name: "No MCP Key".to_string(),
                slug: "no-mcp-key".to_string(),
                description: None,
            },
            integration: ToolIntegrationConfig {
                config_path: "config.json".to_string(),
                config_type: ConfigType::Json,
                additional_paths: vec![],
            },
            capabilities: ToolCapabilities::default(),
            // No mcp_key in schema_keys
            schema_keys: Some(ToolSchemaKeys {
                instruction_key: None,
                mcp_key: None,
                python_path_key: None,
            }),
        };

        let integration = GenericToolIntegration::new(definition);
        let mcp_data = serde_json::json!({"server": {"command": "test"}});
        let context = SyncContext::new(NormalizedPath::new(temp.path())).with_mcp_servers(mcp_data);

        integration.sync(&context, &[]).unwrap();

        let content = fs::read_to_string(temp.path().join("config.json")).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();

        // MCP servers should NOT be written because no mcp_key is configured
        assert!(json.get("mcpServers").is_none());
        assert!(json.get("server").is_none());
    }

    #[test]
    fn test_sync_json_preserves_existing_with_mcp() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("config.json");

        // Create existing config
        let existing = serde_json::json!({
            "existingSetting": true,
            "mcpServers": {
                "old-server": {"command": "old"}
            }
        });
        fs::write(
            &config_path,
            serde_json::to_string_pretty(&existing).unwrap(),
        )
        .unwrap();

        let definition = ToolDefinition {
            meta: ToolMeta {
                name: "Test".to_string(),
                slug: "test".to_string(),
                description: None,
            },
            integration: ToolIntegrationConfig {
                config_path: "config.json".to_string(),
                config_type: ConfigType::Json,
                additional_paths: vec![],
            },
            capabilities: ToolCapabilities {
                supports_custom_instructions: false,
                supports_mcp: true,
                supports_rules_directory: false,
            },
            schema_keys: Some(ToolSchemaKeys {
                instruction_key: None,
                mcp_key: Some("mcpServers".to_string()),
                python_path_key: None,
            }),
        };

        let integration = GenericToolIntegration::new(definition);
        let mcp_data = serde_json::json!({"new-server": {"command": "new"}});
        let context = SyncContext::new(NormalizedPath::new(temp.path())).with_mcp_servers(mcp_data);

        integration.sync(&context, &[]).unwrap();

        let content = fs::read_to_string(&config_path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();

        // existingSetting must be preserved
        assert_eq!(json["existingSetting"], true);
        // MCP servers are replaced (not merged) — the whole key is overwritten
        assert_eq!(json["mcpServers"]["new-server"]["command"], "new");
    }

    // ---------------------------------------------------------------
    // Tests for additional_paths syncing (sync_additional_paths)
    // ---------------------------------------------------------------

    #[test]
    fn test_sync_writes_additional_text_path() {
        let temp = TempDir::new().unwrap();

        let definition = ToolDefinition {
            meta: ToolMeta {
                name: "Text Extra".to_string(),
                slug: "text-extra".to_string(),
                description: None,
            },
            integration: ToolIntegrationConfig {
                config_path: ".primary-rules".to_string(),
                config_type: ConfigType::Text,
                additional_paths: vec![".secondary-rules".to_string()],
            },
            capabilities: ToolCapabilities::default(),
            schema_keys: None,
        };

        let integration = GenericToolIntegration::new(definition);
        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![Rule {
            id: "my-rule".to_string(),
            content: "Do the thing.".to_string(),
        }];

        integration.sync(&context, &rules).unwrap();

        // Primary file must exist with managed block content
        let primary = temp.path().join(".primary-rules");
        assert!(primary.exists(), "Primary path must exist");
        let primary_content = fs::read_to_string(&primary).unwrap();
        assert!(
            primary_content.contains("<!-- repo:block:my-rule -->"),
            "Primary must have managed block"
        );
        assert!(
            primary_content.contains("Do the thing."),
            "Primary must contain rule content"
        );

        // Additional text path must also exist with managed block content
        let secondary = temp.path().join(".secondary-rules");
        assert!(secondary.exists(), "Additional text path must exist");
        let secondary_content = fs::read_to_string(&secondary).unwrap();
        assert!(
            secondary_content.contains("<!-- repo:block:my-rule -->"),
            "Secondary must have managed block"
        );
        assert!(
            secondary_content.contains("Do the thing."),
            "Secondary must contain rule content"
        );
    }

    #[test]
    fn test_sync_writes_additional_markdown_path() {
        let temp = TempDir::new().unwrap();

        let definition = ToolDefinition {
            meta: ToolMeta {
                name: "Md Extra".to_string(),
                slug: "md-extra".to_string(),
                description: None,
            },
            integration: ToolIntegrationConfig {
                config_path: ".primary.md".to_string(),
                config_type: ConfigType::Markdown,
                additional_paths: vec!["CONVENTIONS.md".to_string()],
            },
            capabilities: ToolCapabilities::default(),
            schema_keys: None,
        };

        let integration = GenericToolIntegration::new(definition);
        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![Rule {
            id: "conv-rule".to_string(),
            content: "Follow conventions.".to_string(),
        }];

        integration.sync(&context, &rules).unwrap();

        // Primary markdown file
        let primary = temp.path().join(".primary.md");
        assert!(primary.exists(), "Primary .md path must exist");
        let primary_content = fs::read_to_string(&primary).unwrap();
        assert!(
            primary_content.contains("<!-- repo:block:conv-rule -->"),
            "Primary .md must have managed block"
        );

        // Additional markdown file
        let secondary = temp.path().join("CONVENTIONS.md");
        assert!(secondary.exists(), "Additional .md path must exist");
        let secondary_content = fs::read_to_string(&secondary).unwrap();
        assert!(
            secondary_content.contains("<!-- repo:block:conv-rule -->"),
            "Secondary .md must have managed block"
        );
        assert!(
            secondary_content.contains("Follow conventions."),
            "Secondary .md must contain rule content"
        );
    }

    #[test]
    fn test_sync_writes_additional_json_path() {
        let temp = TempDir::new().unwrap();

        let definition = ToolDefinition {
            meta: ToolMeta {
                name: "Json Extra".to_string(),
                slug: "json-extra".to_string(),
                description: None,
            },
            integration: ToolIntegrationConfig {
                config_path: ".primary-rules".to_string(),
                config_type: ConfigType::Text,
                additional_paths: vec![".tool/settings.json".to_string()],
            },
            capabilities: ToolCapabilities::default(),
            schema_keys: Some(ToolSchemaKeys {
                instruction_key: Some("instructions".to_string()),
                mcp_key: None,
                python_path_key: None,
            }),
        };

        let integration = GenericToolIntegration::new(definition);
        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![Rule {
            id: "json-rule".to_string(),
            content: "JSON rule content.".to_string(),
        }];

        integration.sync(&context, &rules).unwrap();

        // Primary text file must exist
        let primary = temp.path().join(".primary-rules");
        assert!(primary.exists(), "Primary text path must exist");

        // Additional JSON file must exist and be valid JSON
        let json_path = temp.path().join(".tool/settings.json");
        assert!(json_path.exists(), "Additional .json path must exist");
        let json_content = fs::read_to_string(&json_path).unwrap();
        let json: serde_json::Value =
            serde_json::from_str(&json_content).expect("Additional .json must be valid JSON");
        assert!(json.is_object(), "JSON file must contain an object");

        // The JSON sync should use schema_keys to populate instruction_key
        assert!(
            json.get("instructions").is_some(),
            "JSON file must contain instructions key from schema_keys"
        );
        let instructions = json["instructions"].as_str().unwrap();
        assert!(
            instructions.contains("JSON rule content."),
            "JSON instructions must contain rule content"
        );
    }

    #[test]
    fn test_sync_writes_additional_directory_path() {
        let temp = TempDir::new().unwrap();

        let definition = ToolDefinition {
            meta: ToolMeta {
                name: "Dir Extra".to_string(),
                slug: "dir-extra".to_string(),
                description: None,
            },
            integration: ToolIntegrationConfig {
                config_path: "PRIMARY.md".to_string(),
                config_type: ConfigType::Markdown,
                additional_paths: vec![".tool/rules/".to_string()],
            },
            capabilities: ToolCapabilities::default(),
            schema_keys: None,
        };

        let integration = GenericToolIntegration::new(definition);
        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![
            Rule {
                id: "rule-alpha".to_string(),
                content: "Alpha content.".to_string(),
            },
            Rule {
                id: "rule-beta".to_string(),
                content: "Beta content.".to_string(),
            },
        ];

        integration.sync(&context, &rules).unwrap();

        // Primary markdown file
        let primary = temp.path().join("PRIMARY.md");
        assert!(primary.exists(), "Primary path must exist");

        // Additional directory must be created
        let dir = temp.path().join(".tool/rules");
        assert!(
            dir.is_dir(),
            "Additional directory path must be a directory"
        );

        // Per-rule files must exist
        let rule1 = dir.join("01-rule-alpha.md");
        let rule2 = dir.join("02-rule-beta.md");
        assert!(rule1.exists(), "Per-rule file for rule-alpha must exist");
        assert!(rule2.exists(), "Per-rule file for rule-beta must exist");

        let content1 = fs::read_to_string(&rule1).unwrap();
        assert!(
            content1.contains("Alpha content."),
            "Per-rule file must contain rule content"
        );
        assert!(
            content1.contains("# rule-alpha"),
            "Per-rule file must contain rule header"
        );

        let content2 = fs::read_to_string(&rule2).unwrap();
        assert!(
            content2.contains("Beta content."),
            "Second per-rule file must contain rule content"
        );
    }

    #[test]
    fn test_sync_additional_path_content_has_managed_blocks() {
        let temp = TempDir::new().unwrap();

        let definition = ToolDefinition {
            meta: ToolMeta {
                name: "Block Check".to_string(),
                slug: "block-check".to_string(),
                description: None,
            },
            integration: ToolIntegrationConfig {
                config_path: ".primary".to_string(),
                config_type: ConfigType::Text,
                additional_paths: vec![".secondary".to_string()],
            },
            capabilities: ToolCapabilities::default(),
            schema_keys: None,
        };

        let integration = GenericToolIntegration::new(definition);
        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![
            Rule {
                id: "block-a".to_string(),
                content: "Content for block A.".to_string(),
            },
            Rule {
                id: "block-b".to_string(),
                content: "Content for block B.".to_string(),
            },
        ];

        integration.sync(&context, &rules).unwrap();

        // Verify secondary file has actual managed block structure, not empty
        let secondary_content = fs::read_to_string(temp.path().join(".secondary")).unwrap();

        // Must have opening and closing markers for both blocks
        assert!(
            secondary_content.contains("<!-- repo:block:block-a -->"),
            "Secondary must have block-a opening marker"
        );
        assert!(
            secondary_content.contains("<!-- /repo:block:block-a -->"),
            "Secondary must have block-a closing marker"
        );
        assert!(
            secondary_content.contains("Content for block A."),
            "Secondary must have block-a content"
        );
        assert!(
            secondary_content.contains("<!-- repo:block:block-b -->"),
            "Secondary must have block-b opening marker"
        );
        assert!(
            secondary_content.contains("<!-- /repo:block:block-b -->"),
            "Secondary must have block-b closing marker"
        );
        assert!(
            secondary_content.contains("Content for block B."),
            "Secondary must have block-b content"
        );

        // Verify it's not an empty file
        assert!(
            secondary_content.len() > 50,
            "Secondary file must not be empty (got {} bytes)",
            secondary_content.len()
        );
    }

    #[test]
    fn test_empty_additional_paths_no_extra_files() {
        let temp = TempDir::new().unwrap();

        // Tool with empty additional_paths
        let definition = ToolDefinition {
            meta: ToolMeta {
                name: "No Extra".to_string(),
                slug: "no-extra".to_string(),
                description: None,
            },
            integration: ToolIntegrationConfig {
                config_path: ".only-file".to_string(),
                config_type: ConfigType::Text,
                additional_paths: vec![],
            },
            capabilities: ToolCapabilities::default(),
            schema_keys: None,
        };

        let integration = GenericToolIntegration::new(definition);
        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![Rule {
            id: "solo-rule".to_string(),
            content: "Solo content.".to_string(),
        }];

        integration.sync(&context, &rules).unwrap();

        // Primary must exist
        let primary = temp.path().join(".only-file");
        assert!(primary.exists(), "Primary path must exist");

        // Count files in temp dir - should be exactly 1 real file
        // (plus potential .lock files from atomic writes)
        let entries: Vec<_> = fs::read_dir(temp.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| !e.file_name().to_string_lossy().ends_with(".lock"))
            .collect();
        assert_eq!(
            entries.len(),
            1,
            "Only primary file should exist, found: {:?}",
            entries.iter().map(|e| e.file_name()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_antigravity_path_is_directory() {
        // Import the antigravity integration to verify its config_path
        let integration = crate::antigravity::antigravity_integration();
        let def = integration.definition();

        assert!(
            def.integration.config_path.ends_with('/'),
            "Antigravity config_path must end with '/' for directory mode, got: {}",
            def.integration.config_path
        );

        // Verify is_directory_config() returns true
        assert!(
            integration.is_directory_config(),
            "Antigravity is_directory_config() must return true"
        );

        // Verify config_locations reports it as a directory
        let locations = integration.config_locations();
        assert!(
            locations[0].is_directory,
            "Antigravity primary config location must be a directory"
        );
    }
}
