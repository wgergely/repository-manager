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
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
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

        // Load existing content or start empty
        let mut content = if path.exists() {
            io::read_text(&path).unwrap_or_default()
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

        io::write_text(&path, &content)?;

        Ok(())
    }

    /// Sync rules to a directory, creating one file per rule.
    fn sync_to_directory(&self, context: &SyncContext, rules: &[Rule]) -> Result<()> {
        let dir_path = self.config_path(&context.root);

        // Create directory if it doesn't exist
        if !dir_path.exists() {
            std::fs::create_dir_all(dir_path.as_ref())
                .map_err(|e| crate::Error::SyncFailed {
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

        io::write_text(&path, &content)?;

        Ok(())
    }

    /// Sync rules to a JSON config file using schema keys.
    fn sync_json(&self, context: &SyncContext, rules: &[Rule]) -> Result<()> {
        let path = self.config_path(&context.root);

        // Load existing or create new
        let mut settings: Value = if path.exists() {
            let content = io::read_text(&path)?;
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
        }

        let content = serde_json::to_string_pretty(&settings)?;
        io::write_text(&path, &content)?;

        Ok(())
    }

    /// Sync rules to a markdown config file using managed blocks.
    fn sync_markdown(&self, context: &SyncContext, rules: &[Rule]) -> Result<()> {
        // Markdown uses the same approach as text with managed blocks
        self.sync_text(context, rules)
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
            ConfigType::Text => self.sync_text(context, rules),
            ConfigType::Json => self.sync_json(context, rules),
            ConfigType::Markdown => self.sync_markdown(context, rules),
            ConfigType::Yaml => self.sync_yaml(context, rules),
            ConfigType::Toml => {
                // TOML uses # comments like YAML
                self.sync_yaml(context, rules)
            }
        }
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
}
