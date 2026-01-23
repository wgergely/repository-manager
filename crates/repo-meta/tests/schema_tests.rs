//! Tests for schema definitions and the DefinitionLoader

use repo_fs::NormalizedPath;
use repo_meta::DefinitionLoader;
use repo_meta::schema::{ConfigType, PresetDefinition, RuleDefinition, Severity, ToolDefinition};
use std::fs;
use tempfile::TempDir;

// ============================================================================
// Tool Definition Tests
// ============================================================================

#[test]
fn test_parse_tool_definition_minimal() {
    let toml = r#"
[meta]
name = "Cursor"
slug = "cursor"

[integration]
config_path = ".cursorrules"
type = "text"
"#;

    let def: ToolDefinition = toml::from_str(toml).unwrap();
    assert_eq!(def.meta.name, "Cursor");
    assert_eq!(def.meta.slug, "cursor");
    assert!(def.meta.description.is_none());
    assert_eq!(def.integration.config_path, ".cursorrules");
    assert_eq!(def.integration.config_type, ConfigType::Text);
    assert!(!def.capabilities.supports_custom_instructions);
    assert!(!def.capabilities.supports_mcp);
}

#[test]
fn test_parse_tool_definition_full() {
    let toml = r#"
[meta]
name = "Cursor"
slug = "cursor"
description = "AI-first code editor"

[integration]
config_path = ".cursorrules"
type = "text"
additional_paths = [".cursor/rules/"]

[capabilities]
supports_custom_instructions = true
supports_mcp = true
supports_rules_directory = true

[schema]
instruction_key = "global_instructions"
mcp_key = "mcpServers"
"#;

    let def: ToolDefinition = toml::from_str(toml).unwrap();
    assert_eq!(def.meta.slug, "cursor");
    assert_eq!(
        def.meta.description,
        Some("AI-first code editor".to_string())
    );
    assert_eq!(def.integration.config_type, ConfigType::Text);
    assert_eq!(def.integration.additional_paths, vec![".cursor/rules/"]);
    assert!(def.capabilities.supports_custom_instructions);
    assert!(def.capabilities.supports_mcp);
    assert!(def.capabilities.supports_rules_directory);

    let schema_keys = def.schema_keys.unwrap();
    assert_eq!(
        schema_keys.instruction_key,
        Some("global_instructions".to_string())
    );
    assert_eq!(schema_keys.mcp_key, Some("mcpServers".to_string()));
}

#[test]
fn test_parse_tool_definition_json_type() {
    let toml = r#"
[meta]
name = "VSCode"
slug = "vscode"

[integration]
config_path = ".vscode/settings.json"
type = "json"

[schema]
python_path_key = "python.defaultInterpreterPath"
"#;

    let def: ToolDefinition = toml::from_str(toml).unwrap();
    assert_eq!(def.integration.config_type, ConfigType::Json);
    assert_eq!(
        def.schema_keys.unwrap().python_path_key,
        Some("python.defaultInterpreterPath".to_string())
    );
}

#[test]
fn test_parse_tool_definition_markdown_type() {
    let toml = r#"
[meta]
name = "Claude"
slug = "claude"

[integration]
config_path = ".claude/CLAUDE.md"
type = "markdown"
additional_paths = [".claude/rules/"]

[capabilities]
supports_custom_instructions = true
supports_rules_directory = true
"#;

    let def: ToolDefinition = toml::from_str(toml).unwrap();
    assert_eq!(def.integration.config_type, ConfigType::Markdown);
    assert!(def.capabilities.supports_rules_directory);
}

// ============================================================================
// Rule Definition Tests
// ============================================================================

#[test]
fn test_parse_rule_definition_minimal() {
    let toml = r#"
[meta]
id = "no-api-keys"

[content]
instruction = "Never hardcode API keys in source code."
"#;

    let def: RuleDefinition = toml::from_str(toml).unwrap();
    assert_eq!(def.meta.id, "no-api-keys");
    assert_eq!(def.meta.severity, Severity::Suggestion);
    assert!(def.meta.tags.is_empty());
    assert!(def.content.instruction.contains("API keys"));
    assert!(def.examples.is_none());
    assert!(def.targets.is_none());
}

#[test]
fn test_parse_rule_definition_full() {
    let toml = r#"
[meta]
id = "python-snake-case"
severity = "mandatory"
tags = ["python", "style", "naming"]

[content]
instruction = "Use snake_case for all Python variables and function names."

[examples]
positive = ["my_variable = 1", "def calculate_total():"]
negative = ["myVariable = 1", "def calculateTotal():"]

[targets]
files = ["**/*.py", "**/*.pyi"]
"#;

    let def: RuleDefinition = toml::from_str(toml).unwrap();
    assert_eq!(def.meta.id, "python-snake-case");
    assert_eq!(def.meta.severity, Severity::Mandatory);
    assert_eq!(def.meta.tags, vec!["python", "style", "naming"]);
    assert!(def.content.instruction.contains("snake_case"));

    let examples = def.examples.unwrap();
    assert_eq!(examples.positive.len(), 2);
    assert_eq!(examples.negative.len(), 2);

    let targets = def.targets.unwrap();
    assert_eq!(targets.file_patterns, vec!["**/*.py", "**/*.pyi"]);
}

#[test]
fn test_parse_rule_severity_suggestion() {
    let toml = r#"
[meta]
id = "prefer-const"
severity = "suggestion"

[content]
instruction = "Prefer const over let for variables that are not reassigned."
"#;

    let def: RuleDefinition = toml::from_str(toml).unwrap();
    assert_eq!(def.meta.severity, Severity::Suggestion);
}

// ============================================================================
// Preset Definition Tests
// ============================================================================

#[test]
fn test_parse_preset_definition_minimal() {
    let toml = r#"
[meta]
id = "basic"
"#;

    let def: PresetDefinition = toml::from_str(toml).unwrap();
    assert_eq!(def.meta.id, "basic");
    assert!(def.meta.description.is_none());
    assert!(def.requires.tools.is_empty());
    assert!(def.requires.presets.is_empty());
    assert!(def.rules.include.is_empty());
    assert!(def.config.is_empty());
}

#[test]
fn test_parse_preset_definition_full() {
    let toml = r#"
[meta]
id = "python-agentic"
description = "Python development with agentic AI tools"

[requires]
tools = ["cursor", "claude"]
presets = ["env:python"]

[rules]
include = ["python-snake-case", "no-api-keys", "prefer-type-hints"]

[config]
python_version = "3.11"
use_strict_typing = true
max_line_length = 100
"#;

    let def: PresetDefinition = toml::from_str(toml).unwrap();
    assert_eq!(def.meta.id, "python-agentic");
    assert_eq!(
        def.meta.description,
        Some("Python development with agentic AI tools".to_string())
    );
    assert_eq!(def.requires.tools, vec!["cursor", "claude"]);
    assert_eq!(def.requires.presets, vec!["env:python"]);
    assert_eq!(
        def.rules.include,
        vec!["python-snake-case", "no-api-keys", "prefer-type-hints"]
    );

    // Check config values
    assert_eq!(
        def.config.get("python_version").unwrap().as_str(),
        Some("3.11")
    );
    assert_eq!(
        def.config.get("use_strict_typing").unwrap().as_bool(),
        Some(true)
    );
    assert_eq!(
        def.config.get("max_line_length").unwrap().as_integer(),
        Some(100)
    );
}

#[test]
fn test_parse_preset_with_nested_config() {
    let toml = r#"
[meta]
id = "custom"

[config.linting]
enabled = true
rules = ["E501", "E302"]

[config.formatting]
style = "black"
line_length = 88
"#;

    let def: PresetDefinition = toml::from_str(toml).unwrap();
    assert!(def.config.contains_key("linting"));
    assert!(def.config.contains_key("formatting"));
}

// ============================================================================
// DefinitionLoader Tests
// ============================================================================

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
    let tools = loader
        .load_tools(&NormalizedPath::new(temp.path()))
        .unwrap();

    assert_eq!(tools.len(), 2);
    assert!(tools.contains_key("cursor"));
    assert!(tools.contains_key("vscode"));

    let cursor = tools.get("cursor").unwrap();
    assert_eq!(cursor.meta.name, "Cursor");
    assert!(cursor.capabilities.supports_custom_instructions);

    let vscode = tools.get("vscode").unwrap();
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
    let rules = loader
        .load_rules(&NormalizedPath::new(temp.path()))
        .unwrap();

    assert_eq!(rules.len(), 2);
    assert!(rules.contains_key("python-snake-case"));
    assert!(rules.contains_key("no-api-keys"));

    let snake_case = rules.get("python-snake-case").unwrap();
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
    let presets = loader
        .load_presets(&NormalizedPath::new(temp.path()))
        .unwrap();

    assert_eq!(presets.len(), 1);
    let preset = presets.get("python-agentic").unwrap();
    assert_eq!(preset.requires.tools, vec!["cursor"]);
}

#[test]
fn test_loader_ignores_non_toml_files() {
    let temp = TempDir::new().unwrap();
    let tools_dir = temp.path().join(".repository").join("tools");
    fs::create_dir_all(&tools_dir).unwrap();

    // Valid TOML file
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
    let tools = loader
        .load_tools(&NormalizedPath::new(temp.path()))
        .unwrap();

    assert_eq!(tools.len(), 1);
    assert!(tools.contains_key("cursor"));
}

#[test]
fn test_loader_handles_invalid_toml_gracefully() {
    let temp = TempDir::new().unwrap();
    let tools_dir = temp.path().join(".repository").join("tools");
    fs::create_dir_all(&tools_dir).unwrap();

    // Valid TOML file
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
    let tools = loader
        .load_tools(&NormalizedPath::new(temp.path()))
        .unwrap();

    // Should still load the valid one
    assert_eq!(tools.len(), 1);
    assert!(tools.contains_key("valid"));
}

#[test]
fn test_loader_returns_empty_for_nonexistent_directory() {
    let temp = TempDir::new().unwrap();
    // Don't create any directories

    let loader = DefinitionLoader::new();

    let tools = loader
        .load_tools(&NormalizedPath::new(temp.path()))
        .unwrap();
    assert!(tools.is_empty());

    let rules = loader
        .load_rules(&NormalizedPath::new(temp.path()))
        .unwrap();
    assert!(rules.is_empty());

    let presets = loader
        .load_presets(&NormalizedPath::new(temp.path()))
        .unwrap();
    assert!(presets.is_empty());
}

// ============================================================================
// ConfigType Tests
// ============================================================================

#[test]
fn test_config_type_all_variants() {
    let variants = [
        ("text", ConfigType::Text),
        ("json", ConfigType::Json),
        ("toml", ConfigType::Toml),
        ("yaml", ConfigType::Yaml),
        ("markdown", ConfigType::Markdown),
    ];

    for (str_val, expected) in variants {
        let toml = format!(
            r#"
[meta]
name = "Test"
slug = "test"

[integration]
config_path = ".test"
type = "{}"
"#,
            str_val
        );

        let def: ToolDefinition = toml::from_str(&toml).unwrap();
        assert_eq!(def.integration.config_type, expected);
    }
}
