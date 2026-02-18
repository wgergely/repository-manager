//! Integration tests for the DefinitionLoader
//!
//! These tests exercise the loader against real filesystem directories
//! using tempfile. Serde deserialization tests for individual schema types
//! live in their respective source modules (schema/tool.rs, schema/rule.rs,
//! schema/preset.rs).

use repo_fs::NormalizedPath;
use repo_meta::DefinitionLoader;
use repo_meta::schema::{ConfigType, Severity};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_load_tools_from_directory() {
    let temp = TempDir::new().unwrap();
    let tools_dir = temp.path().join(".repository").join("tools");
    fs::create_dir_all(&tools_dir).unwrap();

    fs::write(
        tools_dir.join("cursor.toml"),
        r#"
[meta]
name = "Cursor"
slug = "cursor"

[integration]
config_path = ".cursorrules"
type = "text"

[capabilities]
supports_custom_instructions = true
"#,
    )
    .unwrap();

    fs::write(
        tools_dir.join("vscode.toml"),
        r#"
[meta]
name = "VSCode"
slug = "vscode"

[integration]
config_path = ".vscode/settings.json"
type = "json"

[schema]
python_path_key = "python.defaultInterpreterPath"
"#,
    )
    .unwrap();

    let loader = DefinitionLoader::new();
    let result = loader
        .load_tools(&NormalizedPath::new(temp.path()))
        .unwrap();

    assert!(result.warnings.is_empty());
    assert_eq!(result.definitions.len(), 2);
    assert!(result.definitions.contains_key("cursor"));
    assert!(result.definitions.contains_key("vscode"));

    let cursor = result.definitions.get("cursor").unwrap();
    assert_eq!(cursor.meta.name, "Cursor");
    assert!(cursor.capabilities.supports_custom_instructions);

    let vscode = result.definitions.get("vscode").unwrap();
    assert_eq!(vscode.integration.config_type, ConfigType::Json);
}

#[test]
fn test_load_rules_from_directory() {
    let temp = TempDir::new().unwrap();
    let rules_dir = temp.path().join(".repository").join("rules");
    fs::create_dir_all(&rules_dir).unwrap();

    fs::write(
        rules_dir.join("python-snake-case.toml"),
        r#"
[meta]
id = "python-snake-case"
severity = "mandatory"
tags = ["python"]

[content]
instruction = "Use snake_case for Python identifiers."
"#,
    )
    .unwrap();

    fs::write(
        rules_dir.join("no-api-keys.toml"),
        r#"
[meta]
id = "no-api-keys"
tags = ["security"]

[content]
instruction = "Never commit API keys."
"#,
    )
    .unwrap();

    let loader = DefinitionLoader::new();
    let result = loader
        .load_rules(&NormalizedPath::new(temp.path()))
        .unwrap();

    assert!(result.warnings.is_empty());
    assert_eq!(result.definitions.len(), 2);
    assert!(result.definitions.contains_key("python-snake-case"));
    assert!(result.definitions.contains_key("no-api-keys"));

    let snake_case = result.definitions.get("python-snake-case").unwrap();
    assert_eq!(snake_case.meta.severity, Severity::Mandatory);
}

#[test]
fn test_load_presets_from_directory() {
    let temp = TempDir::new().unwrap();
    let presets_dir = temp.path().join(".repository").join("presets");
    fs::create_dir_all(&presets_dir).unwrap();

    fs::write(
        presets_dir.join("python-agentic.toml"),
        r#"
[meta]
id = "python-agentic"
description = "Python with AI tools"

[requires]
tools = ["cursor"]

[rules]
include = ["python-snake-case"]
"#,
    )
    .unwrap();

    let loader = DefinitionLoader::new();
    let result = loader
        .load_presets(&NormalizedPath::new(temp.path()))
        .unwrap();

    assert!(result.warnings.is_empty());
    assert_eq!(result.definitions.len(), 1);
    let preset = result.definitions.get("python-agentic").unwrap();
    assert_eq!(preset.requires.tools, vec!["cursor"]);
}

#[test]
fn test_loader_ignores_non_toml_files() {
    let temp = TempDir::new().unwrap();
    let tools_dir = temp.path().join(".repository").join("tools");
    fs::create_dir_all(&tools_dir).unwrap();

    fs::write(
        tools_dir.join("cursor.toml"),
        r#"
[meta]
name = "Cursor"
slug = "cursor"

[integration]
config_path = ".cursorrules"
type = "text"
"#,
    )
    .unwrap();

    // Non-TOML files that should be ignored
    fs::write(tools_dir.join("readme.md"), "# Tools").unwrap();
    fs::write(tools_dir.join(".gitkeep"), "").unwrap();
    fs::write(tools_dir.join("backup.toml.bak"), "invalid").unwrap();

    let loader = DefinitionLoader::new();
    let result = loader
        .load_tools(&NormalizedPath::new(temp.path()))
        .unwrap();

    assert_eq!(result.definitions.len(), 1);
    assert!(result.definitions.contains_key("cursor"));
    assert!(result.warnings.is_empty());
}

#[test]
fn test_loader_handles_invalid_toml_gracefully() {
    let temp = TempDir::new().unwrap();
    let tools_dir = temp.path().join(".repository").join("tools");
    fs::create_dir_all(&tools_dir).unwrap();

    fs::write(
        tools_dir.join("valid.toml"),
        r#"
[meta]
name = "Valid"
slug = "valid"

[integration]
config_path = ".valid"
type = "text"
"#,
    )
    .unwrap();

    // Invalid TOML file (missing required fields)
    fs::write(
        tools_dir.join("invalid.toml"),
        r#"
[meta]
name = "Invalid"
# Missing slug and integration
"#,
    )
    .unwrap();

    let loader = DefinitionLoader::new();
    let result = loader
        .load_tools(&NormalizedPath::new(temp.path()))
        .unwrap();

    // Should still load the valid one
    assert_eq!(result.definitions.len(), 1);
    assert!(result.definitions.contains_key("valid"));

    // Should have a warning for the invalid one
    assert_eq!(result.warnings.len(), 1);
    assert!(result.warnings[0].contains("invalid.toml"));
}

#[test]
fn test_loader_returns_empty_for_nonexistent_directory() {
    let temp = TempDir::new().unwrap();

    let loader = DefinitionLoader::new();

    let result = loader
        .load_tools(&NormalizedPath::new(temp.path()))
        .unwrap();
    assert!(result.definitions.is_empty());
    assert!(result.warnings.is_empty());

    let result = loader
        .load_rules(&NormalizedPath::new(temp.path()))
        .unwrap();
    assert!(result.definitions.is_empty());

    let result = loader
        .load_presets(&NormalizedPath::new(temp.path()))
        .unwrap();
    assert!(result.definitions.is_empty());
}
