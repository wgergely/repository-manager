//! Writer registry for selecting writers by config type

use super::{ConfigWriter, JsonWriter, MarkdownWriter, TextWriter};
use repo_meta::schema::ConfigType;

/// Registry that selects the appropriate writer for a config type.
pub struct WriterRegistry {
    json: JsonWriter,
    markdown: MarkdownWriter,
    text: TextWriter,
}

impl WriterRegistry {
    /// Create a new writer registry with all built-in writers.
    pub fn new() -> Self {
        Self {
            json: JsonWriter::new(),
            markdown: MarkdownWriter::new(),
            text: TextWriter::new(),
        }
    }

    /// Get the appropriate writer for a config type.
    pub fn get_writer(&self, config_type: ConfigType) -> &dyn ConfigWriter {
        match config_type {
            ConfigType::Json => &self.json,
            ConfigType::Markdown => &self.markdown,
            // YAML and TOML use text writer for now (full replacement)
            // Future: Add AST-aware writers
            ConfigType::Text | ConfigType::Yaml | ConfigType::Toml => &self.text,
        }
    }
}

impl Default for WriterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_fs::NormalizedPath;

    #[test]
    fn test_get_json_writer() {
        let registry = WriterRegistry::new();
        let writer = registry.get_writer(ConfigType::Json);
        assert!(writer.can_handle(&NormalizedPath::new("/test/config.json")));
    }

    #[test]
    fn test_get_markdown_writer() {
        let registry = WriterRegistry::new();
        let writer = registry.get_writer(ConfigType::Markdown);
        assert!(writer.can_handle(&NormalizedPath::new("/test/rules.md")));
    }

    #[test]
    fn test_get_text_writer() {
        let registry = WriterRegistry::new();
        let writer = registry.get_writer(ConfigType::Text);
        assert!(writer.can_handle(&NormalizedPath::new("/test/.cursorrules")));
    }

    #[test]
    fn test_yaml_uses_text_writer() {
        let registry = WriterRegistry::new();
        let writer = registry.get_writer(ConfigType::Yaml);
        // YAML uses text writer for now, which handles plain files
        assert!(writer.can_handle(&NormalizedPath::new("/test/.rules")));
    }

    #[test]
    fn test_toml_uses_text_writer() {
        let registry = WriterRegistry::new();
        let writer = registry.get_writer(ConfigType::Toml);
        assert!(writer.can_handle(&NormalizedPath::new("/test/.rules")));
    }
}
