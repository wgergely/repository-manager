//! Plain text config writer (full replacement)
//!
//! This writer completely replaces the file content.
//! Use when the tool owns its config file entirely.

use super::{ConfigWriter, SchemaKeys};
use crate::error::Result;
use crate::translator::TranslatedContent;
use repo_fs::{io, NormalizedPath};

/// Plain text writer that does full replacement.
///
/// Use this for tools that own their config file entirely,
/// where there's no need to preserve existing content.
pub struct TextWriter;

impl TextWriter {
    /// Create a new text writer.
    pub fn new() -> Self {
        Self
    }
}

impl Default for TextWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigWriter for TextWriter {
    fn write(
        &self,
        path: &NormalizedPath,
        content: &TranslatedContent,
        _: Option<&SchemaKeys>,
    ) -> Result<()> {
        io::write_text(path, content.instructions.as_deref().unwrap_or(""))?;
        Ok(())
    }

    fn can_handle(&self, path: &NormalizedPath) -> bool {
        let p = path.as_str();
        // Handles everything that's not JSON, YAML, TOML, or Markdown
        !p.ends_with(".json")
            && !p.ends_with(".yaml")
            && !p.ends_with(".yml")
            && !p.ends_with(".toml")
            && !p.ends_with(".md")
            && !p.ends_with(".markdown")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_meta::schema::ConfigType;
    use std::fs;
    use tempfile::TempDir;

    fn make_content(instructions: &str) -> TranslatedContent {
        TranslatedContent::with_instructions(ConfigType::Text, instructions.to_string())
    }

    #[test]
    fn test_write_new_file() {
        let temp = TempDir::new().unwrap();
        let path = NormalizedPath::new(temp.path()).join(".toolrules");
        let writer = TextWriter::new();

        let content = make_content("Rule content here");
        writer.write(&path, &content, None).unwrap();

        let written = fs::read_to_string(path.as_ref()).unwrap();
        assert_eq!(written, "Rule content here");
    }

    #[test]
    fn test_replaces_existing() {
        let temp = TempDir::new().unwrap();
        let path = NormalizedPath::new(temp.path()).join(".toolrules");

        // Create existing file
        fs::write(path.as_ref(), "Old content").unwrap();

        let writer = TextWriter::new();
        let content = make_content("New content");
        writer.write(&path, &content, None).unwrap();

        let written = fs::read_to_string(path.as_ref()).unwrap();
        assert_eq!(written, "New content");
        assert!(!written.contains("Old content"));
    }

    #[test]
    fn test_empty_content() {
        let temp = TempDir::new().unwrap();
        let path = NormalizedPath::new(temp.path()).join(".toolrules");
        let writer = TextWriter::new();

        let content = TranslatedContent::empty();
        writer.write(&path, &content, None).unwrap();

        let written = fs::read_to_string(path.as_ref()).unwrap();
        assert_eq!(written, "");
    }

    #[test]
    fn test_can_handle() {
        let writer = TextWriter::new();
        assert!(writer.can_handle(&NormalizedPath::new("/test/.cursorrules")));
        assert!(writer.can_handle(&NormalizedPath::new("/test/.clinerules")));
        assert!(writer.can_handle(&NormalizedPath::new("/test/rules.txt")));
        assert!(!writer.can_handle(&NormalizedPath::new("/test/config.json")));
        assert!(!writer.can_handle(&NormalizedPath::new("/test/rules.md")));
        assert!(!writer.can_handle(&NormalizedPath::new("/test/config.yaml")));
    }
}
