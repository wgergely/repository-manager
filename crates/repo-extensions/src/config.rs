//! Extension configuration from `config.toml`'s `[extensions."name"]` table.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Per-extension configuration stored in the repository's `config.toml`.
///
/// Represents a single `[extensions."<name>"]` table entry.
///
/// The TOML key for the pinned ref is `ref` (not `ref_pin`), matching what
/// `generate_config()` writes during `repo init`.  Since `ref` is a Rust
/// keyword the field is named `ref_pin` with a serde rename.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ExtensionConfig {
    /// Source URL or path for the extension (e.g., a git repository URL).
    pub source: String,
    /// Optional pinned ref (branch, tag, or commit hash).
    ///
    /// Serialized as `ref` in TOML to match the canonical config format.
    #[serde(default, rename = "ref", alias = "ref_pin")]
    pub ref_pin: Option<String>,
    /// Arbitrary extension-specific configuration values.
    #[serde(default, flatten)]
    pub config: HashMap<String, toml::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_ref_key() {
        // This is the canonical format written by generate_config()
        let toml_str = r#"
source = "https://github.com/vaultspec/vaultspec.git"
ref = "main"
"#;
        let config: ExtensionConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.source, "https://github.com/vaultspec/vaultspec.git");
        assert_eq!(config.ref_pin.as_deref(), Some("main"));
        assert!(
            config.config.is_empty(),
            "ref should NOT leak into the flattened config map, got: {:?}",
            config.config
        );
    }

    #[test]
    fn test_parse_with_ref_pin_alias() {
        // Backwards-compatible: ref_pin is accepted via serde alias
        let toml_str = r#"
source = "https://github.com/user/vaultspec.git"
ref_pin = "v0.1.0"
custom_key = "custom_value"
"#;
        let config: ExtensionConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.source, "https://github.com/user/vaultspec.git");
        assert_eq!(config.ref_pin.as_deref(), Some("v0.1.0"));
        assert_eq!(
            config.config.get("custom_key"),
            Some(&toml::Value::String("custom_value".to_string()))
        );
    }

    #[test]
    fn test_parse_extension_config_minimal() {
        let toml_str = r#"
source = "https://github.com/user/vaultspec.git"
"#;
        let config: ExtensionConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.source, "https://github.com/user/vaultspec.git");
        assert!(config.ref_pin.is_none());
        assert!(config.config.is_empty());
    }

    #[test]
    fn test_serialize_uses_ref_key() {
        let config = ExtensionConfig {
            source: "https://example.com/ext.git".to_string(),
            ref_pin: Some("v1.0.0".to_string()),
            config: HashMap::new(),
        };
        let toml_str = toml::to_string(&config).unwrap();
        assert!(
            toml_str.contains("ref = "),
            "serialized TOML should use 'ref', not 'ref_pin': {}",
            toml_str
        );
        assert!(
            !toml_str.contains("ref_pin"),
            "serialized TOML should not contain 'ref_pin': {}",
            toml_str
        );
    }

    #[test]
    fn test_round_trip_with_ref() {
        let config = ExtensionConfig {
            source: "https://example.com/ext.git".to_string(),
            ref_pin: Some("v2.0.0".to_string()),
            config: HashMap::new(),
        };
        let toml_str = toml::to_string(&config).unwrap();
        let reparsed: ExtensionConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(config, reparsed);
    }
}
