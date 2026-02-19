//! Extension configuration from `config.toml`'s `[extensions."name"]` table.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Per-extension configuration stored in the repository's `config.toml`.
///
/// Represents a single `[extensions."<name>"]` table entry.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExtensionConfig {
    /// Source URL or path for the extension (e.g., a git repository URL).
    pub source: String,
    /// Optional pinned ref (branch, tag, or commit hash).
    #[serde(default)]
    pub ref_pin: Option<String>,
    /// Arbitrary extension-specific configuration values.
    #[serde(default, flatten)]
    pub config: HashMap<String, toml::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_extension_config() {
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
}
