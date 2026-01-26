//! Projection types for tracking tool-specific configuration outputs
//!
//! A projection represents how an intent is manifested in a specific tool's
//! configuration format. Each projection tracks the tool, file, and the
//! specific format of the configuration data.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use uuid::Uuid;

/// A projection of an intent into a specific tool's configuration
///
/// Projections track how intents are rendered into tool-specific formats,
/// enabling the system to verify, update, or remove configuration fragments.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Projection {
    /// The tool this projection targets (e.g., "cursor", "vscode")
    pub tool: String,
    /// Path to the configuration file, relative to config root
    pub file: PathBuf,
    /// The kind of projection and its backend-specific data
    pub kind: ProjectionKind,
}

/// The specific format/backend of a projection
///
/// Different tools require different configuration formats. This enum
/// captures the necessary metadata for each format type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum ProjectionKind {
    /// A text block with marker delimiters in a file
    ///
    /// Used for tools that support marked sections in text files
    /// (e.g., Cursor .mdc files, .gitignore)
    TextBlock {
        /// UUID marker identifying this block in the file
        marker: Uuid,
        /// Checksum of the block content for integrity verification
        checksum: String,
    },

    /// A JSON key-value pair in a JSON configuration file
    ///
    /// Used for tools with JSON configurations (e.g., VSCode settings.json)
    JsonKey {
        /// JSON path to the key (e.g., "editor.fontSize")
        path: String,
        /// The value at this path
        value: Value,
    },

    /// A fully managed configuration file
    ///
    /// Used when the entire file is controlled by repository-manager
    FileManaged {
        /// Checksum of the entire file content
        checksum: String,
    },
}

impl Projection {
    /// Create a new text block projection
    ///
    /// # Arguments
    ///
    /// * `tool` - The tool identifier (e.g., "cursor")
    /// * `file` - Path to the configuration file
    /// * `marker` - UUID used as the block delimiter
    /// * `checksum` - Checksum of the block content
    pub fn text_block(tool: String, file: PathBuf, marker: Uuid, checksum: String) -> Self {
        Self {
            tool,
            file,
            kind: ProjectionKind::TextBlock { marker, checksum },
        }
    }

    /// Create a new JSON key projection
    ///
    /// # Arguments
    ///
    /// * `tool` - The tool identifier (e.g., "vscode")
    /// * `file` - Path to the JSON configuration file
    /// * `path` - JSON path to the key
    /// * `value` - The value to set at this path
    pub fn json_key(tool: String, file: PathBuf, path: String, value: Value) -> Self {
        Self {
            tool,
            file,
            kind: ProjectionKind::JsonKey { path, value },
        }
    }

    /// Create a new file-managed projection
    ///
    /// # Arguments
    ///
    /// * `tool` - The tool identifier
    /// * `file` - Path to the managed file
    /// * `checksum` - Checksum of the entire file content
    pub fn file_managed(tool: String, file: PathBuf, checksum: String) -> Self {
        Self {
            tool,
            file,
            kind: ProjectionKind::FileManaged { checksum },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_text_block_serializes_correctly() {
        let marker = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let proj = Projection::text_block(
            "cursor".to_string(),
            PathBuf::from(".cursor/rules/test.mdc"),
            marker,
            "abc123".to_string(),
        );

        let serialized = toml::to_string(&proj).unwrap();
        assert!(serialized.contains("backend = \"text_block\""));
        assert!(serialized.contains("cursor"));
    }

    #[test]
    fn projection_json_key_serializes_correctly() {
        let proj = Projection::json_key(
            "vscode".to_string(),
            PathBuf::from(".vscode/settings.json"),
            "editor.tabSize".to_string(),
            serde_json::json!(4),
        );

        let serialized = toml::to_string(&proj).unwrap();
        assert!(serialized.contains("backend = \"json_key\""));
        assert!(serialized.contains("vscode"));
    }
}
