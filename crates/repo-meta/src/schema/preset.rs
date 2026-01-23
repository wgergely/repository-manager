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
}
