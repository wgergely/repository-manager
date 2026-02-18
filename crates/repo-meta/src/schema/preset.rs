//! Preset definition schema - loaded from .repository/presets/*.toml
//!
//! Presets bundle together tools, rules, and configuration to provide
//! ready-to-use development environments.
//!
//! # Example TOML
//!
//! ```toml
//! [meta]
//! id = "python-agentic"
//! description = "Python development with agentic AI tools"
//!
//! [requires]
//! tools = ["cursor", "claude"]
//! presets = ["env:python"]
//!
//! [rules]
//! include = ["python-snake-case", "no-api-keys"]
//!
//! [config]
//! python_version = "3.11"
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete preset definition loaded from TOML
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PresetDefinition {
    /// Preset metadata
    pub meta: PresetMeta,
    /// Dependencies on other tools and presets
    #[serde(default)]
    pub requires: PresetRequires,
    /// Rules included in this preset
    #[serde(default)]
    pub rules: PresetRules,
    /// Preset-specific configuration overrides
    #[serde(default)]
    pub config: HashMap<String, toml::Value>,
}

/// Preset metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PresetMeta {
    /// Unique preset identifier (e.g., "python-agentic")
    pub id: String,
    /// Optional description of what this preset provides
    #[serde(default)]
    pub description: Option<String>,
}

/// Dependencies required by this preset
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PresetRequires {
    /// Tool slugs that must be available (e.g., ["cursor", "claude"])
    #[serde(default)]
    pub tools: Vec<String>,
    /// Other preset IDs that must be activated first
    #[serde(default)]
    pub presets: Vec<String>,
}

/// Rules configuration for this preset
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PresetRules {
    /// Rule IDs to include when this preset is active
    #[serde(default)]
    pub include: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_requires_default() {
        let requires = PresetRequires::default();
        assert!(requires.tools.is_empty());
        assert!(requires.presets.is_empty());
    }

    #[test]
    fn test_rules_default() {
        let rules = PresetRules::default();
        assert!(rules.include.is_empty());
    }

    #[test]
    fn test_config_is_flexible() {
        let toml_str = r#"
[meta]
id = "test"

[config]
string_value = "hello"
number_value = 42
bool_value = true
array_value = [1, 2, 3]

[config.nested]
key = "value"
"#;

        let def: PresetDefinition = toml::from_str(toml_str).unwrap();
        assert_eq!(def.meta.id, "test");
        assert!(def.config.contains_key("string_value"));
        assert!(def.config.contains_key("nested"));
    }

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
}
