//! Format detection and handler trait

use crate::block::{BlockLocation, ManagedBlock};
use crate::edit::Edit;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Supported document formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Format {
    Toml,
    Yaml,
    Json,
    Markdown,
    PlainText,
}

impl Format {
    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "toml" => Some(Self::Toml),
            "yaml" | "yml" => Some(Self::Yaml),
            "json" => Some(Self::Json),
            "md" | "markdown" => Some(Self::Markdown),
            "txt" | "text" => Some(Self::PlainText),
            _ => None,
        }
    }

    /// Detect format from content heuristics
    pub fn from_content(content: &str) -> Self {
        let trimmed = content.trim_start();

        // JSON starts with { or [
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            return Self::Json;
        }

        // TOML has [section] headers or key = value
        if trimmed.contains("\n[") || trimmed.starts_with('[') {
            // Could be TOML or Markdown
            if trimmed.lines().any(|l| l.contains(" = ")) {
                return Self::Toml;
            }
        }

        // YAML often has key: value at start
        if trimmed
            .lines()
            .next()
            .map_or(false, |l| l.contains(": ") && !l.starts_with('#'))
        {
            return Self::Yaml;
        }

        // Markdown has headers, lists, or code blocks
        if trimmed.starts_with('#')
            || trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || trimmed.starts_with("```")
        {
            return Self::Markdown;
        }

        Self::PlainText
    }

    /// Get the comment style for this format
    pub fn comment_style(&self) -> CommentStyle {
        match self {
            Self::Toml => CommentStyle::Hash,
            Self::Yaml => CommentStyle::Hash,
            Self::Json => CommentStyle::None,
            Self::Markdown => CommentStyle::Html,
            Self::PlainText => CommentStyle::Html,
        }
    }

    /// Get default file extensions for this format
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Self::Toml => &["toml"],
            Self::Yaml => &["yaml", "yml"],
            Self::Json => &["json"],
            Self::Markdown => &["md", "markdown"],
            Self::PlainText => &["txt", "text"],
        }
    }
}

/// Comment syntax styles for managed block markers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentStyle {
    /// HTML-style: `<!-- comment -->`
    Html,
    /// Hash: `# comment`
    Hash,
    /// No comment support (embed in data structure)
    None,
}

impl CommentStyle {
    /// Format a block start marker
    pub fn format_start(&self, uuid: Uuid) -> String {
        match self {
            Self::Html => format!("<!-- repo:block:{uuid} -->"),
            Self::Hash => format!("# repo:block:{uuid}"),
            Self::None => String::new(), // JSON uses _repo_managed key
        }
    }

    /// Format a block end marker
    pub fn format_end(&self, uuid: Uuid) -> String {
        match self {
            Self::Html => format!("<!-- /repo:block:{uuid} -->"),
            Self::Hash => format!("# /repo:block:{uuid}"),
            Self::None => String::new(),
        }
    }
}

/// Trait for format-specific handlers
pub trait FormatHandler: Send + Sync {
    /// Format identifier
    fn format(&self) -> Format;

    /// Parse source into internal representation
    fn parse(&self, source: &str) -> Result<Box<dyn std::any::Any + Send + Sync>>;

    /// Find managed blocks in the document
    fn find_blocks(&self, source: &str) -> Vec<ManagedBlock>;

    /// Insert a managed block
    fn insert_block(
        &self,
        source: &str,
        uuid: Uuid,
        content: &str,
        location: BlockLocation,
    ) -> Result<(String, Edit)>;

    /// Update a managed block
    fn update_block(&self, source: &str, uuid: Uuid, content: &str) -> Result<(String, Edit)>;

    /// Remove a managed block
    fn remove_block(&self, source: &str, uuid: Uuid) -> Result<(String, Edit)>;

    /// Normalize content for semantic comparison
    fn normalize(&self, source: &str) -> Result<serde_json::Value>;

    /// Render back to string (may reformat)
    fn render(&self, parsed: &dyn std::any::Any) -> Result<String>;
}
