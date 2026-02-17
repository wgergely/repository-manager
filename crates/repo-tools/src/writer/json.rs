//! JSON config writer with semantic merge
//!
//! This writer preserves existing JSON keys while updating managed fields.

use super::{ConfigWriter, SchemaKeys};
use crate::error::Result;
use crate::translator::TranslatedContent;
use repo_fs::{io, NormalizedPath};
use serde_json::{json, Value};

/// JSON config writer that semantically merges content.
///
/// Features:
/// - Preserves existing keys in the JSON file
/// - Uses schema_keys to place instructions and MCP config
/// - Merges additional data from TranslatedContent
pub struct JsonWriter;

impl JsonWriter {
    /// Create a new JSON writer.
    pub fn new() -> Self {
        Self
    }

    /// Parse existing JSON file or return empty object.
    fn parse_existing(path: &NormalizedPath) -> Value {
        if !path.exists() {
            return json!({});
        }
        io::read_text(path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_else(|| json!({}))
    }

    /// Merge content into existing JSON.
    fn merge(existing: &mut Value, content: &TranslatedContent, keys: Option<&SchemaKeys>) {
        let obj = match existing.as_object_mut() {
            Some(o) => o,
            None => return,
        };

        // Merge instructions if key specified
        if let (Some(instructions), Some(k)) = (&content.instructions, keys)
            && let Some(ref key) = k.instruction_key
        {
            obj.insert(key.clone(), json!(instructions));
        }

        // Merge MCP servers if key specified
        if let (Some(mcp), Some(k)) = (&content.mcp_servers, keys)
            && let Some(ref key) = k.mcp_key
        {
            obj.insert(key.clone(), mcp.clone());
        }

        // Merge additional data
        for (key, value) in &content.data {
            obj.insert(key.clone(), value.clone());
        }
    }
}

impl Default for JsonWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigWriter for JsonWriter {
    fn write(
        &self,
        path: &NormalizedPath,
        content: &TranslatedContent,
        keys: Option<&SchemaKeys>,
    ) -> Result<()> {
        let mut existing = Self::parse_existing(path);

        // Ensure we have an object
        if !existing.is_object() {
            existing = json!({});
        }

        Self::merge(&mut existing, content, keys);

        // Write back with pretty formatting
        io::write_text(path, &serde_json::to_string_pretty(&existing)?)?;
        Ok(())
    }

    fn can_handle(&self, path: &NormalizedPath) -> bool {
        path.as_str().ends_with(".json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_meta::schema::ConfigType;
    use std::fs;
    use tempfile::TempDir;

    fn make_content(instructions: Option<&str>) -> TranslatedContent {
        if let Some(inst) = instructions {
            TranslatedContent::with_instructions(ConfigType::Json, inst.to_string())
        } else {
            TranslatedContent::empty()
        }
    }

    #[test]
    fn test_write_new_file() {
        let temp = TempDir::new().unwrap();
        let path = NormalizedPath::new(temp.path()).join("config.json");
        let writer = JsonWriter::new();

        let content = make_content(Some("Test instructions"));
        let keys = SchemaKeys {
            instruction_key: Some("customInstructions".into()),
            ..Default::default()
        };

        writer.write(&path, &content, Some(&keys)).unwrap();

        let written = fs::read_to_string(path.as_ref()).unwrap();
        let json: Value = serde_json::from_str(&written).unwrap();
        assert_eq!(json["customInstructions"], "Test instructions");
    }

    #[test]
    fn test_preserves_existing_keys() {
        let temp = TempDir::new().unwrap();
        let path = NormalizedPath::new(temp.path()).join("config.json");

        // Create existing file
        let existing = json!({
            "existing_key": "preserved value",
            "another": 42
        });
        fs::write(
            path.as_ref(),
            serde_json::to_string_pretty(&existing).unwrap(),
        )
        .unwrap();

        let writer = JsonWriter::new();
        let content = make_content(Some("New instructions"));
        let keys = SchemaKeys {
            instruction_key: Some("instructions".into()),
            ..Default::default()
        };

        writer.write(&path, &content, Some(&keys)).unwrap();

        let written = fs::read_to_string(path.as_ref()).unwrap();
        let json: Value = serde_json::from_str(&written).unwrap();

        // Existing keys preserved
        assert_eq!(json["existing_key"], "preserved value");
        assert_eq!(json["another"], 42);
        // New key added
        assert_eq!(json["instructions"], "New instructions");
    }

    #[test]
    fn test_merges_additional_data() {
        let temp = TempDir::new().unwrap();
        let path = NormalizedPath::new(temp.path()).join("config.json");
        let writer = JsonWriter::new();

        let content = TranslatedContent::empty()
            .with_data("key1", json!("value1"))
            .with_data("key2", json!(123));

        writer.write(&path, &content, None).unwrap();

        let written = fs::read_to_string(path.as_ref()).unwrap();
        let json: Value = serde_json::from_str(&written).unwrap();
        assert_eq!(json["key1"], "value1");
        assert_eq!(json["key2"], 123);
    }

    #[test]
    fn test_writes_mcp_servers() {
        let temp = TempDir::new().unwrap();
        let path = NormalizedPath::new(temp.path()).join("config.json");
        let writer = JsonWriter::new();

        let content =
            TranslatedContent::empty().with_mcp_servers(json!({"server1": {"command": "test"}}));
        let keys = SchemaKeys {
            mcp_key: Some("mcpServers".into()),
            ..Default::default()
        };

        writer.write(&path, &content, Some(&keys)).unwrap();

        let written = fs::read_to_string(path.as_ref()).unwrap();
        let json: Value = serde_json::from_str(&written).unwrap();
        assert!(json.is_object(), "Root must be a JSON object");
        assert!(json["mcpServers"].is_object(), "mcpServers must be a JSON object");
        assert!(json["mcpServers"]["server1"].is_object(), "server1 must be a JSON object");
        assert_eq!(
            json["mcpServers"]["server1"]["command"], "test",
            "server1 command must match"
        );
    }

    #[test]
    fn test_can_handle() {
        let writer = JsonWriter::new();
        assert!(writer.can_handle(&NormalizedPath::new("/test/config.json")));
        assert!(!writer.can_handle(&NormalizedPath::new("/test/config.md")));
    }
}
