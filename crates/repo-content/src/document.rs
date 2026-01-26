//! Unified Document type

use crate::block::{BlockLocation, ManagedBlock};
use crate::diff::SemanticDiff;
use crate::edit::Edit;
use crate::error::Result;
use crate::format::{Format, FormatHandler};
use crate::handlers::{JsonHandler, PlainTextHandler, TomlHandler, YamlHandler};
use uuid::Uuid;

/// Unified document type wrapping format-specific backends
pub struct Document {
    source: String,
    format: Format,
    handler: Box<dyn FormatHandler>,
}

impl Document {
    /// Parse content with format auto-detection
    pub fn parse(source: &str) -> Result<Self> {
        let format = Format::from_content(source);
        Self::parse_as(source, format)
    }

    /// Parse with explicit format
    pub fn parse_as(source: &str, format: Format) -> Result<Self> {
        let handler: Box<dyn FormatHandler> = match format {
            Format::Toml => Box::new(TomlHandler::new()),
            Format::Json => Box::new(JsonHandler::new()),
            Format::PlainText | Format::Markdown => Box::new(PlainTextHandler::new()),
            Format::Yaml => Box::new(YamlHandler::new()),
        };

        // Verify it parses
        let _ = handler.parse(source)?;

        Ok(Self {
            source: source.to_string(),
            format,
            handler,
        })
    }

    /// Get the document format
    pub fn format(&self) -> Format {
        self.format
    }

    /// Get current source
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Find all managed blocks
    pub fn find_blocks(&self) -> Vec<ManagedBlock> {
        self.handler.find_blocks(&self.source)
    }

    /// Get block by UUID
    pub fn get_block(&self, uuid: Uuid) -> Option<ManagedBlock> {
        self.find_blocks().into_iter().find(|b| b.uuid == uuid)
    }

    /// Insert a new managed block
    pub fn insert_block(
        &mut self,
        uuid: Uuid,
        content: &str,
        location: BlockLocation,
    ) -> Result<Edit> {
        let (new_source, edit) =
            self.handler
                .insert_block(&self.source, uuid, content, location)?;
        self.source = new_source;
        Ok(edit)
    }

    /// Update existing block content
    pub fn update_block(&mut self, uuid: Uuid, content: &str) -> Result<Edit> {
        let (new_source, edit) = self.handler.update_block(&self.source, uuid, content)?;
        self.source = new_source;
        Ok(edit)
    }

    /// Remove block by UUID
    pub fn remove_block(&mut self, uuid: Uuid) -> Result<Edit> {
        let (new_source, edit) = self.handler.remove_block(&self.source, uuid)?;
        self.source = new_source;
        Ok(edit)
    }

    /// Check semantic equality
    pub fn semantic_eq(&self, other: &Document) -> bool {
        let Ok(norm1) = self.handler.normalize(&self.source) else {
            return false;
        };
        let Ok(norm2) = other.handler.normalize(&other.source) else {
            return false;
        };
        norm1 == norm2
    }

    /// Compute semantic diff between two documents.
    ///
    /// **Note:** This is a basic implementation that only reports whether
    /// documents are equivalent. Full diff computation with detailed changes
    /// is planned for Phase 4. The `similar` crate is available for this.
    ///
    /// Currently returns:
    /// - Empty changes list (no detailed diff)
    /// - Similarity of 1.0 if equivalent, 0.5 if not
    pub fn diff(&self, other: &Document) -> SemanticDiff {
        if self.semantic_eq(other) {
            SemanticDiff::equivalent()
        } else {
            SemanticDiff {
                is_equivalent: false,
                changes: Vec::new(),
                similarity: 0.5,
            }
        }
    }

    /// Render to string
    pub fn render(&self) -> String {
        if let Ok(parsed) = self.handler.parse(&self.source) {
            self.handler
                .render(parsed.as_ref())
                .unwrap_or_else(|_| self.source.clone())
        } else {
            self.source.clone()
        }
    }

    /// Check if document has been modified from its original source.
    ///
    /// **Note:** This method is not yet implemented and always returns `false`.
    /// Full implementation requires tracking original source state.
    /// See: Phase 5 in the implementation plan.
    pub fn is_modified(&self) -> bool {
        // TODO: Track original source to enable modification detection
        false
    }

    /// Get normalized representation for semantic comparison
    pub fn normalize(&self) -> Result<serde_json::Value> {
        self.handler.normalize(&self.source)
    }
}
