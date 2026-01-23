//! Integration tests for ToolDispatcher and GenericToolIntegration

use repo_fs::NormalizedPath;
use repo_meta::schema::{
    ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta, ToolSchemaKeys,
};
use repo_tools::{GenericToolIntegration, Rule, SyncContext, ToolDispatcher, ToolIntegration};
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

fn create_windsurf_definition() -> ToolDefinition {
    ToolDefinition {
        meta: ToolMeta {
            name: "Windsurf".to_string(),
            slug: "windsurf".to_string(),
            description: Some("Codeium's AI IDE".to_string()),
        },
        integration: ToolIntegrationConfig {
            config_path: ".windsurfrules".to_string(),
            config_type: ConfigType::Text,
            additional_paths: vec![],
        },
        capabilities: ToolCapabilities {
            supports_custom_instructions: true,
            supports_mcp: true,
            supports_rules_directory: false,
        },
        schema_keys: None,
    }
}

#[test]
fn test_dispatcher_builtin_tools() {
    let dispatcher = ToolDispatcher::new();

    assert!(dispatcher.get_integration("vscode").is_some());
    assert!(dispatcher.get_integration("cursor").is_some());
    assert!(dispatcher.get_integration("claude").is_some());
}

#[test]
fn test_dispatcher_schema_tool() {
    let mut dispatcher = ToolDispatcher::new();
    dispatcher.register(create_windsurf_definition());

    let integration = dispatcher.get_integration("windsurf");
    assert!(integration.is_some());
    assert_eq!(integration.unwrap().name(), "windsurf");
}

#[test]
fn test_dispatcher_list_available() {
    let mut dispatcher = ToolDispatcher::new();
    dispatcher.register(create_windsurf_definition());

    let available = dispatcher.list_available();
    assert!(available.contains(&"vscode".to_string()));
    assert!(available.contains(&"cursor".to_string()));
    assert!(available.contains(&"claude".to_string()));
    assert!(available.contains(&"windsurf".to_string()));
}

#[test]
fn test_dispatcher_with_definitions() {
    let mut definitions = HashMap::new();
    definitions.insert("windsurf".to_string(), create_windsurf_definition());
    definitions.insert(
        "zed".to_string(),
        ToolDefinition {
            meta: ToolMeta {
                name: "Zed".to_string(),
                slug: "zed".to_string(),
                description: None,
            },
            integration: ToolIntegrationConfig {
                config_path: ".zed/settings.json".to_string(),
                config_type: ConfigType::Json,
                additional_paths: vec![],
            },
            capabilities: ToolCapabilities::default(),
            schema_keys: None,
        },
    );

    let dispatcher = ToolDispatcher::with_definitions(definitions);

    assert!(dispatcher.get_integration("windsurf").is_some());
    assert!(dispatcher.get_integration("zed").is_some());
    assert_eq!(dispatcher.schema_tool_count(), 2);
}

#[test]
fn test_generic_integration_text() {
    let temp = TempDir::new().unwrap();
    let definition = create_windsurf_definition();
    let integration = GenericToolIntegration::new(definition);

    let context = SyncContext::new(NormalizedPath::new(temp.path()));
    let rules = vec![Rule {
        id: "test-rule".to_string(),
        content: "Test content".to_string(),
    }];

    integration.sync(&context, &rules).unwrap();

    let content = fs::read_to_string(temp.path().join(".windsurfrules")).unwrap();
    assert!(content.contains("test-rule"));
    assert!(content.contains("Test content"));
}

#[test]
fn test_generic_integration_json_with_schema_keys() {
    let temp = TempDir::new().unwrap();

    let definition = ToolDefinition {
        meta: ToolMeta {
            name: "Custom JSON Tool".to_string(),
            slug: "custom-json".to_string(),
            description: None,
        },
        integration: ToolIntegrationConfig {
            config_path: "custom.json".to_string(),
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

    let content = fs::read_to_string(temp.path().join("custom.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert!(json.get("customInstructions").is_some());
    assert_eq!(json["pythonPath"], "/usr/bin/python3");
}

#[test]
fn test_generic_integration_preserves_existing_json() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.json");

    // Create existing config
    let existing = serde_json::json!({
        "editor.fontSize": 14,
        "existingKey": "existingValue"
    });
    fs::write(&config_path, serde_json::to_string_pretty(&existing).unwrap()).unwrap();

    let definition = ToolDefinition {
        meta: ToolMeta {
            name: "Existing JSON Tool".to_string(),
            slug: "existing-json".to_string(),
            description: None,
        },
        integration: ToolIntegrationConfig {
            config_path: "config.json".to_string(),
            config_type: ConfigType::Json,
            additional_paths: vec![],
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
        id: "new-rule".to_string(),
        content: "New content".to_string(),
    }];

    integration.sync(&context, &rules).unwrap();

    let content = fs::read_to_string(&config_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Existing values preserved
    assert_eq!(json["editor.fontSize"], 14);
    assert_eq!(json["existingKey"], "existingValue");

    // New value added
    assert!(json.get("instructions").is_some());
}

#[test]
fn test_generic_integration_multiple_rules() {
    let temp = TempDir::new().unwrap();

    let definition = ToolDefinition {
        meta: ToolMeta {
            name: "Multi Rule Tool".to_string(),
            slug: "multi-rule".to_string(),
            description: None,
        },
        integration: ToolIntegrationConfig {
            config_path: ".rules".to_string(),
            config_type: ConfigType::Text,
            additional_paths: vec![],
        },
        capabilities: ToolCapabilities::default(),
        schema_keys: None,
    };

    let integration = GenericToolIntegration::new(definition);
    let context = SyncContext::new(NormalizedPath::new(temp.path()));

    let rules = vec![
        Rule {
            id: "rule-a".to_string(),
            content: "Content A".to_string(),
        },
        Rule {
            id: "rule-b".to_string(),
            content: "Content B".to_string(),
        },
        Rule {
            id: "rule-c".to_string(),
            content: "Content C".to_string(),
        },
    ];

    integration.sync(&context, &rules).unwrap();

    let content = fs::read_to_string(temp.path().join(".rules")).unwrap();

    // All rules present
    assert!(content.contains("rule-a"));
    assert!(content.contains("Content A"));
    assert!(content.contains("rule-b"));
    assert!(content.contains("Content B"));
    assert!(content.contains("rule-c"));
    assert!(content.contains("Content C"));

    // Each rule has its own block
    assert_eq!(content.matches("<!-- repo:block:rule-a -->").count(), 1);
    assert_eq!(content.matches("<!-- repo:block:rule-b -->").count(), 1);
    assert_eq!(content.matches("<!-- repo:block:rule-c -->").count(), 1);
}

#[test]
fn test_sync_all_tools() {
    let temp = TempDir::new().unwrap();
    let dispatcher = ToolDispatcher::new();

    let context = SyncContext::new(NormalizedPath::new(temp.path()));
    let rules = vec![Rule {
        id: "test".to_string(),
        content: "Test".to_string(),
    }];

    let synced = dispatcher
        .sync_all(
            &context,
            &["vscode".to_string(), "cursor".to_string()],
            &rules,
        )
        .unwrap();

    assert!(synced.contains(&"vscode".to_string()));
    assert!(synced.contains(&"cursor".to_string()));
}

#[test]
fn test_sync_all_with_schema_tools() {
    let temp = TempDir::new().unwrap();
    let mut dispatcher = ToolDispatcher::new();
    dispatcher.register(create_windsurf_definition());

    let context = SyncContext::new(NormalizedPath::new(temp.path()));
    let rules = vec![Rule {
        id: "test".to_string(),
        content: "Test rule content".to_string(),
    }];

    let synced = dispatcher
        .sync_all(
            &context,
            &["cursor".to_string(), "windsurf".to_string()],
            &rules,
        )
        .unwrap();

    assert!(synced.contains(&"cursor".to_string()));
    assert!(synced.contains(&"windsurf".to_string()));

    // Verify files were created
    assert!(temp.path().join(".cursorrules").exists());
    assert!(temp.path().join(".windsurfrules").exists());
}

#[test]
fn test_sync_all_skips_unknown() {
    let temp = TempDir::new().unwrap();
    let dispatcher = ToolDispatcher::new();

    let context = SyncContext::new(NormalizedPath::new(temp.path()));
    let rules = vec![Rule {
        id: "test".to_string(),
        content: "Test".to_string(),
    }];

    let synced = dispatcher
        .sync_all(
            &context,
            &[
                "vscode".to_string(),
                "unknown-tool".to_string(),
                "cursor".to_string(),
            ],
            &rules,
        )
        .unwrap();

    // Should only contain known tools
    assert_eq!(synced.len(), 2);
    assert!(synced.contains(&"vscode".to_string()));
    assert!(synced.contains(&"cursor".to_string()));
    assert!(!synced.contains(&"unknown-tool".to_string()));
}

#[test]
fn test_generic_integration_markdown_type() {
    let temp = TempDir::new().unwrap();

    let definition = ToolDefinition {
        meta: ToolMeta {
            name: "Markdown Tool".to_string(),
            slug: "markdown-tool".to_string(),
            description: None,
        },
        integration: ToolIntegrationConfig {
            config_path: "INSTRUCTIONS.md".to_string(),
            config_type: ConfigType::Markdown,
            additional_paths: vec![],
        },
        capabilities: ToolCapabilities::default(),
        schema_keys: None,
    };

    let integration = GenericToolIntegration::new(definition);
    let context = SyncContext::new(NormalizedPath::new(temp.path()));

    let rules = vec![Rule {
        id: "md-rule".to_string(),
        content: "Markdown content with **bold** and _italic_.".to_string(),
    }];

    integration.sync(&context, &rules).unwrap();

    let content = fs::read_to_string(temp.path().join("INSTRUCTIONS.md")).unwrap();
    assert!(content.contains("md-rule"));
    assert!(content.contains("**bold**"));
    assert!(content.contains("_italic_"));
}

#[test]
fn test_config_paths_with_additional() {
    let definition = ToolDefinition {
        meta: ToolMeta {
            name: "Multi Path Tool".to_string(),
            slug: "multi-path".to_string(),
            description: None,
        },
        integration: ToolIntegrationConfig {
            config_path: ".tool/config.json".to_string(),
            config_type: ConfigType::Json,
            additional_paths: vec![
                ".tool/rules/".to_string(),
                ".tool/presets/".to_string(),
            ],
        },
        capabilities: ToolCapabilities::default(),
        schema_keys: None,
    };

    let integration = GenericToolIntegration::new(definition);
    let paths = integration.config_paths();

    assert_eq!(paths.len(), 3);
    assert_eq!(paths[0], ".tool/config.json");
    assert_eq!(paths[1], ".tool/rules/");
    assert_eq!(paths[2], ".tool/presets/");
}
