//! Markdown config writer with section-based merge
//!
//! This writer preserves user content while updating a managed section.

use super::{ConfigWriter, SchemaKeys};
use crate::error::Result;
use crate::translator::TranslatedContent;
use repo_fs::{NormalizedPath, io};

/// Markers for the managed section.
const MANAGED_START: &str = "<!-- repo:managed:start -->";
const MANAGED_END: &str = "<!-- repo:managed:end -->";

/// Markdown config writer that uses section-based merge.
///
/// Features:
/// - Preserves all user content outside the managed section
/// - Creates managed section if it doesn't exist
/// - Updates only the content between markers
pub struct MarkdownWriter;

impl MarkdownWriter {
    /// Create a new Markdown writer.
    pub fn new() -> Self {
        Self
    }

    /// Parse existing file, returning (user_content, managed_content).
    fn parse_existing(path: &NormalizedPath) -> (String, String) {
        if !path.exists() {
            return (String::new(), String::new());
        }

        let content = match io::read_text(path) {
            Ok(c) => c,
            Err(_) => return (String::new(), String::new()),
        };

        if let (Some(start), Some(end)) = (content.find(MANAGED_START), content.find(MANAGED_END)) {
            let before = content[..start].trim_end();
            let after = content[end + MANAGED_END.len()..].trim_start();

            let user = if after.is_empty() {
                before.to_string()
            } else {
                format!("{}\n\n{}", before, after)
            };

            let managed_start = start + MANAGED_START.len();
            let managed = content[managed_start..end].trim().to_string();

            (user, managed)
        } else {
            // No markers, entire content is user content
            (content, String::new())
        }
    }

    /// Combine user content and managed content.
    fn combine(user: &str, managed: &str) -> String {
        let mut out = String::new();

        if !user.is_empty() {
            out.push_str(user);
            out.push_str("\n\n");
        }

        out.push_str(MANAGED_START);
        out.push('\n');
        out.push_str(managed);
        out.push('\n');
        out.push_str(MANAGED_END);
        out.push('\n');

        out
    }
}

impl Default for MarkdownWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigWriter for MarkdownWriter {
    fn write(
        &self,
        path: &NormalizedPath,
        content: &TranslatedContent,
        _: Option<&SchemaKeys>,
    ) -> Result<()> {
        let (user, _) = Self::parse_existing(path);
        let managed = content.instructions.as_deref().unwrap_or("");
        io::write_text(path, &Self::combine(&user, managed))?;
        Ok(())
    }

    fn can_handle(&self, path: &NormalizedPath) -> bool {
        let p = path.as_str();
        p.ends_with(".md") || p.ends_with(".markdown")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_meta::schema::ConfigType;
    use std::fs;
    use tempfile::TempDir;

    fn make_content(instructions: &str) -> TranslatedContent {
        TranslatedContent::with_instructions(ConfigType::Markdown, instructions.to_string())
    }

    #[test]
    fn test_write_new_file() {
        let temp = TempDir::new().unwrap();
        let path = NormalizedPath::new(temp.path()).join("rules.md");
        let writer = MarkdownWriter::new();

        let content = make_content("Test instructions");
        writer.write(&path, &content, None).unwrap();

        let written = fs::read_to_string(path.as_ref()).unwrap();
        assert!(written.contains(MANAGED_START));
        assert!(written.contains("Test instructions"));
        assert!(written.contains(MANAGED_END));
    }

    #[test]
    fn test_preserves_user_content() {
        let temp = TempDir::new().unwrap();
        let path = NormalizedPath::new(temp.path()).join("rules.md");

        // Create file with user content
        fs::write(path.as_ref(), "# My Rules\n\nThese are my custom rules.\n").unwrap();

        let writer = MarkdownWriter::new();
        let content = make_content("Managed content");
        writer.write(&path, &content, None).unwrap();

        let written = fs::read_to_string(path.as_ref()).unwrap();

        // User content preserved
        assert!(written.contains("# My Rules"));
        assert!(written.contains("These are my custom rules."));

        // Managed section added
        assert!(written.contains(MANAGED_START));
        assert!(written.contains("Managed content"));
        assert!(written.contains(MANAGED_END));
    }

    #[test]
    fn test_updates_existing_managed_section() {
        let temp = TempDir::new().unwrap();
        let path = NormalizedPath::new(temp.path()).join("rules.md");

        // Create file with existing managed section
        let existing = format!(
            "# User Content\n\n{}\nOld managed content\n{}\n",
            MANAGED_START, MANAGED_END
        );
        fs::write(path.as_ref(), existing).unwrap();

        let writer = MarkdownWriter::new();
        let content = make_content("New managed content");
        writer.write(&path, &content, None).unwrap();

        let written = fs::read_to_string(path.as_ref()).unwrap();

        // User content preserved
        assert!(written.contains("# User Content"));

        // Managed section updated
        assert!(written.contains("New managed content"));
        assert!(!written.contains("Old managed content"));

        // Only one managed section
        assert_eq!(written.matches(MANAGED_START).count(), 1);
        assert_eq!(written.matches(MANAGED_END).count(), 1);
    }

    #[test]
    fn test_preserves_content_after_managed_section() {
        let temp = TempDir::new().unwrap();
        let path = NormalizedPath::new(temp.path()).join("rules.md");

        // Create file with content before and after managed section
        let existing = format!(
            "# Before\n\n{}\nManaged\n{}\n\n# After\n",
            MANAGED_START, MANAGED_END
        );
        fs::write(path.as_ref(), existing).unwrap();

        let writer = MarkdownWriter::new();
        let content = make_content("Updated");
        writer.write(&path, &content, None).unwrap();

        let written = fs::read_to_string(path.as_ref()).unwrap();

        // Both before and after content preserved
        assert!(written.contains("# Before"));
        assert!(written.contains("# After"));
        assert!(written.contains("Updated"));
    }

    #[test]
    fn test_can_handle() {
        let writer = MarkdownWriter::new();
        assert!(writer.can_handle(&NormalizedPath::new("/test/rules.md")));
        assert!(writer.can_handle(&NormalizedPath::new("/test/doc.markdown")));
        assert!(!writer.can_handle(&NormalizedPath::new("/test/config.json")));
    }
}
