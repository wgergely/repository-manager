//! YAML format handler for managed blocks
//!
//! Uses comment-based markers to delimit managed blocks in YAML files.
//!
//! Example:
//! ```yaml
//! user_setting: true
//!
//! # repo:block:550e8400-e29b-41d4-a716-446655440000
//! managed_setting: value
//! # /repo:block:550e8400-e29b-41d4-a716-446655440000
//!
//! another_setting: false
//! ```

use super::{FormatHandler, FormatManagedBlock};
use regex::Regex;
use std::sync::LazyLock;
use uuid::Uuid;

/// Opening block marker regex
static OPEN_MARKER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"# repo:block:([0-9a-fA-F-]+)").expect("Invalid open marker regex")
});

/// YAML format handler using comment-based markers
#[derive(Debug, Default, Clone)]
pub struct YamlFormatHandler;

impl YamlFormatHandler {
    /// Create a new YAML format handler
    pub fn new() -> Self {
        Self
    }

    /// Build the opening marker for a block
    fn opening_marker(uuid: Uuid) -> String {
        format!("# repo:block:{}", uuid)
    }

    /// Build the closing marker for a block
    fn closing_marker(uuid: Uuid) -> String {
        format!("# /repo:block:{}", uuid)
    }
}

impl FormatHandler for YamlFormatHandler {
    fn parse_blocks(&self, content: &str) -> Vec<FormatManagedBlock> {
        let mut blocks = Vec::new();

        for caps in OPEN_MARKER.captures_iter(content) {
            let uuid_str = caps.get(1).unwrap().as_str();
            let Ok(uuid) = Uuid::parse_str(uuid_str) else {
                continue;
            };

            let open_match = caps.get(0).unwrap();
            let open_end = open_match.end();

            // Build the closing marker pattern
            let close_marker = format!("# /repo:block:{}", uuid);

            // Find the closing marker
            if let Some(close_pos) = content[open_end..].find(&close_marker) {
                let close_start = open_end + close_pos;

                // Extract content between markers
                let raw_content = &content[open_end..close_start];
                let trimmed = raw_content
                    .strip_prefix('\n')
                    .unwrap_or(raw_content)
                    .strip_suffix('\n')
                    .unwrap_or(raw_content)
                    .to_string();

                blocks.push(FormatManagedBlock {
                    uuid,
                    content: trimmed,
                });
            }
        }

        blocks
    }

    fn write_block(&self, content: &str, uuid: Uuid, block_content: &str) -> String {
        let open_marker = Self::opening_marker(uuid);
        let close_marker = Self::closing_marker(uuid);

        // Check if block already exists
        if self.has_block(content, uuid) {
            // Replace existing block
            let pattern = format!(
                r"(?s)# repo:block:{}\n.*?# /repo:block:{}",
                regex::escape(&uuid.to_string()),
                regex::escape(&uuid.to_string())
            );
            let re = Regex::new(&pattern).unwrap();
            let replacement = format!("{}\n{}\n{}", open_marker, block_content, close_marker);
            re.replace(content, replacement.as_str()).to_string()
        } else {
            // Append new block
            let block = format!("{}\n{}\n{}", open_marker, block_content, close_marker);
            if content.trim().is_empty() {
                block
            } else {
                format!("{}\n\n{}", content.trim_end(), block)
            }
        }
    }

    fn remove_block(&self, content: &str, uuid: Uuid) -> String {
        if !self.has_block(content, uuid) {
            return content.to_string();
        }

        // Match the block including surrounding newlines
        let pattern = format!(
            r"(?s)\n*# repo:block:{}\n.*?# /repo:block:{}\n*",
            regex::escape(&uuid.to_string()),
            regex::escape(&uuid.to_string())
        );
        let re = Regex::new(&pattern).unwrap();
        let result = re.replace(content, "\n").to_string();

        // Clean up extra newlines
        result
            .trim_start_matches('\n')
            .trim_end_matches('\n')
            .to_string()
            + "\n"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_content() {
        let handler = YamlFormatHandler::new();
        let blocks = handler.parse_blocks("");
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_parse_no_blocks() {
        let handler = YamlFormatHandler::new();
        let content = "user_setting: true\nanother: false";
        let blocks = handler.parse_blocks(content);
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_parse_single_block() {
        let handler = YamlFormatHandler::new();
        let content = r#"user_setting: true

# repo:block:550e8400-e29b-41d4-a716-446655440000
managed_setting: value
# /repo:block:550e8400-e29b-41d4-a716-446655440000

another: false"#;

        let blocks = handler.parse_blocks(content);
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0].uuid,
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
        assert_eq!(blocks[0].content, "managed_setting: value");
    }

    #[test]
    fn test_write_block_to_empty() {
        let handler = YamlFormatHandler::new();
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let result = handler.write_block("", uuid, "setting: value");

        assert!(result.contains("# repo:block:550e8400-e29b-41d4-a716-446655440000"));
        assert!(result.contains("setting: value"));
        assert!(result.contains("# /repo:block:550e8400-e29b-41d4-a716-446655440000"));
    }

    #[test]
    fn test_write_block_preserves_user_content() {
        let handler = YamlFormatHandler::new();
        let existing = "user_setting: true\nanother: false";
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let result = handler.write_block(existing, uuid, "managed: value");

        assert!(result.contains("user_setting: true"));
        assert!(result.contains("another: false"));
        assert!(result.contains("managed: value"));
    }

    #[test]
    fn test_write_block_updates_existing() {
        let handler = YamlFormatHandler::new();
        let existing = r#"user: true

# repo:block:550e8400-e29b-41d4-a716-446655440000
old: value
# /repo:block:550e8400-e29b-41d4-a716-446655440000"#;

        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let result = handler.write_block(existing, uuid, "new: value");

        assert!(result.contains("new: value"));
        assert!(!result.contains("old: value"));
        assert!(result.contains("user: true"));
    }

    #[test]
    fn test_remove_block() {
        let handler = YamlFormatHandler::new();
        let existing = r#"user: true

# repo:block:550e8400-e29b-41d4-a716-446655440000
managed: value
# /repo:block:550e8400-e29b-41d4-a716-446655440000

another: false"#;

        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let result = handler.remove_block(existing, uuid);

        assert!(result.contains("user: true"));
        assert!(result.contains("another: false"));
        assert!(!result.contains("managed: value"));
        assert!(!result.contains("repo:block"));
    }

    #[test]
    fn test_has_block() {
        let handler = YamlFormatHandler::new();
        let content = r#"# repo:block:550e8400-e29b-41d4-a716-446655440000
setting: value
# /repo:block:550e8400-e29b-41d4-a716-446655440000"#;

        let uuid1 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let uuid2 = Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap();

        assert!(handler.has_block(content, uuid1));
        assert!(!handler.has_block(content, uuid2));
    }

    #[test]
    fn test_multiple_blocks() {
        let handler = YamlFormatHandler::new();

        let uuid1 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let uuid2 = Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap();

        let result = handler.write_block("", uuid1, "first: 1");
        let result = handler.write_block(&result, uuid2, "second: 2");

        let blocks = handler.parse_blocks(&result);
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn test_multiline_block_content() {
        let handler = YamlFormatHandler::new();
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let block_content = "key1: value1\nkey2: value2\nkey3: value3";
        let result = handler.write_block("", uuid, block_content);

        let blocks = handler.parse_blocks(&result);
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].content.contains("key1: value1"));
        assert!(blocks[0].content.contains("key2: value2"));
        assert!(blocks[0].content.contains("key3: value3"));
    }
}
