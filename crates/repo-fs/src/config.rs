//! Format-agnostic configuration loading and saving

use crate::{Error, NormalizedPath, Result, io};
use serde::{Serialize, de::DeserializeOwned};

/// Format-agnostic configuration store.
///
/// Automatically detects format from file extension and handles
/// serialization/deserialization transparently.
#[derive(Debug, Default)]
pub struct ConfigStore;

impl ConfigStore {
    /// Create a new ConfigStore.
    pub fn new() -> Self {
        Self
    }

    /// Load configuration from a file.
    ///
    /// Format is detected from file extension:
    /// - `.toml` -> TOML
    /// - `.json` -> JSON
    /// - `.yaml`, `.yml` -> YAML
    pub fn load<T: DeserializeOwned>(&self, path: &NormalizedPath) -> Result<T> {
        let content = io::read_text(path)?;
        let extension = path.extension().unwrap_or("");

        match extension.to_lowercase().as_str() {
            "toml" => toml::from_str(&content).map_err(|e| Error::ConfigParse {
                path: path.to_native(),
                format: "TOML".into(),
                message: e.to_string(),
            }),
            "json" => serde_json::from_str(&content).map_err(|e| Error::ConfigParse {
                path: path.to_native(),
                format: "JSON".into(),
                message: e.to_string(),
            }),
            "yaml" | "yml" => serde_yaml::from_str(&content).map_err(|e| Error::ConfigParse {
                path: path.to_native(),
                format: "YAML".into(),
                message: e.to_string(),
            }),
            _ => Err(Error::UnsupportedFormat {
                extension: extension.to_string(),
            }),
        }
    }

    /// Save configuration to a file.
    ///
    /// Format is determined from file extension.
    /// Uses atomic write to prevent corruption.
    pub fn save<T: Serialize>(&self, path: &NormalizedPath, value: &T) -> Result<()> {
        let extension = path.extension().unwrap_or("");

        let content = match extension.to_lowercase().as_str() {
            "toml" => toml::to_string_pretty(value).map_err(|e| Error::ConfigParse {
                path: path.to_native(),
                format: "TOML".into(),
                message: e.to_string(),
            })?,
            "json" => serde_json::to_string_pretty(value).map_err(|e| Error::ConfigParse {
                path: path.to_native(),
                format: "JSON".into(),
                message: e.to_string(),
            })?,
            "yaml" | "yml" => serde_yaml::to_string(value).map_err(|e| Error::ConfigParse {
                path: path.to_native(),
                format: "YAML".into(),
                message: e.to_string(),
            })?,
            _ => {
                return Err(Error::UnsupportedFormat {
                    extension: extension.to_string(),
                });
            }
        };

        io::write_text(path, &content)
    }
}
