//! VSCode integration for Repository Manager.
//!
//! Manages `.vscode/settings.json` to configure Python interpreter paths
//! and other workspace settings.

use crate::error::Result;
use crate::integration::{ConfigLocation, ConfigType, Rule, SyncContext, ToolIntegration};
use repo_fs::{NormalizedPath, io};
use repo_meta::schema::{
    ConfigType as SchemaConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig,
    ToolMeta, ToolSchemaKeys,
};
use serde_json::{Value, json};

/// Returns the ToolDefinition for VS Code.
///
/// This provides the schema metadata for the registry while VSCodeIntegration
/// handles the actual sync logic.
pub fn vscode_definition() -> ToolDefinition {
    ToolDefinition {
        meta: ToolMeta {
            name: "VS Code".into(),
            slug: "vscode".into(),
            description: Some("Visual Studio Code IDE".into()),
        },
        integration: ToolIntegrationConfig {
            config_path: ".vscode/settings.json".into(),
            config_type: SchemaConfigType::Json,
            additional_paths: vec![],
        },
        capabilities: ToolCapabilities {
            // VSCode itself doesn't support custom instructions
            // but handles python path via schema_keys
            supports_custom_instructions: false,
            supports_mcp: true,
            supports_rules_directory: false,
        },
        schema_keys: Some(ToolSchemaKeys {
            instruction_key: None,
            mcp_key: None,
            python_path_key: Some("python.defaultInterpreterPath".into()),
        }),
    }
}

/// VSCode integration.
///
/// Syncs workspace settings to `.vscode/settings.json`, primarily for
/// Python interpreter configuration.
#[derive(Debug, Default)]
pub struct VSCodeIntegration;

impl VSCodeIntegration {
    /// Creates a new VSCode integration.
    pub fn new() -> Self {
        Self
    }

    /// Load existing settings.json or create empty object.
    fn load_settings(path: &NormalizedPath) -> Result<Value> {
        if path.exists() {
            let content = io::read_text(path)?;
            let settings: Value = serde_json::from_str(&content)?;
            Ok(settings)
        } else {
            Ok(json!({}))
        }
    }

    /// Save settings to JSON file with pretty formatting.
    fn save_settings(path: &NormalizedPath, settings: &Value) -> Result<()> {
        let content = serde_json::to_string_pretty(settings)?;
        io::write_text(path, &content)?;
        Ok(())
    }
}

impl ToolIntegration for VSCodeIntegration {
    fn name(&self) -> &str {
        "vscode"
    }

    fn config_locations(&self) -> Vec<ConfigLocation> {
        vec![ConfigLocation::file(
            ".vscode/settings.json",
            ConfigType::Json,
        )]
    }

    fn sync(&self, context: &SyncContext, _rules: &[Rule]) -> Result<()> {
        let settings_path = context.root.join(".vscode/settings.json");

        // Load existing settings or create empty
        let mut settings = Self::load_settings(&settings_path)?;

        // Ensure settings is an object
        if !settings.is_object() {
            settings = json!({});
        }

        // Set python interpreter path if provided
        if let Some(ref python_path) = context.python_path {
            settings["python.defaultInterpreterPath"] = json!(python_path.as_str());
        }

        // Save settings
        Self::save_settings(&settings_path, &settings)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_name() {
        let integration = VSCodeIntegration::new();
        assert_eq!(integration.name(), "vscode");
    }

    #[test]
    fn test_config_locations() {
        let integration = VSCodeIntegration::new();
        let locations = integration.config_locations();
        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].path, ".vscode/settings.json");
        assert_eq!(locations[0].config_type, ConfigType::Json);
        assert!(!locations[0].is_directory);
    }

    #[test]
    fn test_sync_creates_settings() {
        let temp_dir = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp_dir.path());
        let python_path = NormalizedPath::new("/usr/bin/python3");

        let context = SyncContext::new(root.clone()).with_python(python_path);
        let integration = VSCodeIntegration::new();

        integration.sync(&context, &[]).unwrap();

        let settings_path = temp_dir.path().join(".vscode/settings.json");
        assert!(settings_path.exists());

        let content = fs::read_to_string(&settings_path).unwrap();
        let settings: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(
            settings["python.defaultInterpreterPath"],
            "/usr/bin/python3"
        );
    }

    #[test]
    fn test_sync_preserves_existing() {
        let temp_dir = TempDir::new().unwrap();
        let vscode_dir = temp_dir.path().join(".vscode");
        fs::create_dir_all(&vscode_dir).unwrap();

        // Create existing settings
        let existing = json!({
            "editor.fontSize": 14,
            "files.autoSave": "afterDelay"
        });
        fs::write(
            vscode_dir.join("settings.json"),
            serde_json::to_string_pretty(&existing).unwrap(),
        )
        .unwrap();

        let root = NormalizedPath::new(temp_dir.path());
        let python_path = NormalizedPath::new("/my/python");
        let context = SyncContext::new(root).with_python(python_path);

        let integration = VSCodeIntegration::new();
        integration.sync(&context, &[]).unwrap();

        let content = fs::read_to_string(vscode_dir.join("settings.json")).unwrap();
        let settings: Value = serde_json::from_str(&content).unwrap();

        // Check existing settings preserved
        assert_eq!(settings["editor.fontSize"], 14);
        assert_eq!(settings["files.autoSave"], "afterDelay");

        // Check new setting added
        assert_eq!(settings["python.defaultInterpreterPath"], "/my/python");
    }
}
