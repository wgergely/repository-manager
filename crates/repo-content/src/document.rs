//! Unified Document type

use crate::block::{BlockLocation, ManagedBlock};
use crate::diff::SemanticDiff;
use crate::edit::Edit;
use crate::error::{Error, Result};
use crate::format::{Format, FormatHandler};
use crate::handlers::{JsonHandler, MarkdownHandler, PlainTextHandler, TomlHandler, YamlHandler};
use crate::path::{get_at_path, parse_path, remove_at_path, set_at_path};
use serde_json::Value;
use uuid::Uuid;

/// Unified document type wrapping format-specific backends
pub struct Document {
    /// Original source as provided to parse/parse_as (for is_modified tracking)
    original_source: String,
    /// Current source (may differ from original after edits)
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
            Format::PlainText => Box::new(PlainTextHandler::new()),
            Format::Markdown => Box::new(MarkdownHandler::new()),
            Format::Yaml => Box::new(YamlHandler::new()),
        };

        // Verify it parses
        let _ = handler.parse(source)?;

        Ok(Self {
            original_source: source.to_string(),
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
    /// For structured formats (JSON, TOML, YAML), this performs a recursive
    /// comparison of the normalized JSON representations, tracking changes
    /// with their paths (e.g., "config.host" for nested keys).
    ///
    /// For text formats (Markdown, PlainText), this performs a line-by-line
    /// text diff using the `similar` crate.
    ///
    /// Returns:
    /// - `is_equivalent`: true if documents are semantically equal
    /// - `changes`: list of Added/Removed/Modified changes with paths
    /// - `similarity`: ratio from 0.0 to 1.0
    pub fn diff(&self, other: &Document) -> SemanticDiff {
        // First check semantic equality
        if self.semantic_eq(other) {
            return SemanticDiff::equivalent();
        }

        // For structured formats, use JSON diff
        match (self.format, other.format) {
            (Format::Json, _)
            | (Format::Toml, _)
            | (Format::Yaml, _)
            | (_, Format::Json)
            | (_, Format::Toml)
            | (_, Format::Yaml) => {
                // Normalize both to JSON and compute diff
                let Ok(old_norm) = self.handler.normalize(&self.source) else {
                    return SemanticDiff::with_changes(Vec::new(), 0.0);
                };
                let Ok(new_norm) = other.handler.normalize(&other.source) else {
                    return SemanticDiff::with_changes(Vec::new(), 0.0);
                };
                SemanticDiff::compute(&old_norm, &new_norm)
            }
            // For text formats, use text diff
            (Format::Markdown, Format::Markdown) | (Format::PlainText, Format::PlainText) => {
                SemanticDiff::compute_text(&self.source, &other.source)
            }
            // Mixed text formats - also use text diff
            _ => SemanticDiff::compute_text(&self.source, &other.source),
        }
    }

    /// Render to string.
    ///
    /// For text formats (PlainText, Markdown), returns the source as-is.
    /// For structured formats (TOML, JSON, YAML), re-parses and re-renders
    /// to produce canonical output.
    pub fn render(&self) -> String {
        match self.format {
            Format::PlainText | Format::Markdown => self.source.clone(),
            _ => {
                if let Ok(parsed) = self.handler.parse(&self.source) {
                    self.handler
                        .render(parsed.as_ref())
                        .unwrap_or_else(|_| self.source.clone())
                } else {
                    self.source.clone()
                }
            }
        }
    }

    /// Check if document has been modified from its original source.
    pub fn is_modified(&self) -> bool {
        self.source != self.original_source
    }

    /// Get normalized representation for semantic comparison
    pub fn normalize(&self) -> Result<serde_json::Value> {
        self.handler.normalize(&self.source)
    }

    /// Get a value at the given path from the normalized JSON representation.
    ///
    /// # Path Syntax
    ///
    /// - Dot-separated keys: `config.database.host`
    /// - Array indexing: `items[0].name`
    /// - Combined: `config.servers[0].host`
    ///
    /// # Examples
    ///
    /// ```
    /// use repo_content::Document;
    /// use serde_json::json;
    ///
    /// let doc = Document::parse(r#"{"config": {"host": "localhost"}}"#).unwrap();
    /// assert_eq!(doc.get_path("config.host"), Some(json!("localhost")));
    /// assert_eq!(doc.get_path("config.missing"), None);
    /// ```
    pub fn get_path(&self, path: &str) -> Option<Value> {
        let normalized = self.handler.normalize(&self.source).ok()?;
        let segments = parse_path(path);
        get_at_path(&normalized, &segments)
    }

    /// Set a value at the given path.
    ///
    /// Returns an `Edit` for rollback support. The document is re-rendered
    /// after the change, which may affect formatting.
    ///
    /// # Path Syntax
    ///
    /// - Dot-separated keys: `config.database.host`
    /// - Array indexing: `items[0].name`
    ///
    /// # Errors
    ///
    /// Returns `PathNotFound` if the path doesn't exist.
    /// Returns `PathSetFailed` if the value cannot be set (e.g., wrong type).
    ///
    /// # Examples
    ///
    /// ```
    /// use repo_content::Document;
    /// use serde_json::json;
    ///
    /// let mut doc = Document::parse(r#"{"name": "old"}"#).unwrap();
    /// let edit = doc.set_path("name", "new").unwrap();
    /// assert_eq!(doc.get_path("name"), Some(json!("new")));
    /// ```
    pub fn set_path(&mut self, path: &str, value: impl Into<Value>) -> Result<Edit> {
        let old_source = self.source.clone();
        let mut normalized = self.handler.normalize(&self.source)?;
        let segments = parse_path(path);

        // Check that the path exists (for set, we require existing path)
        if get_at_path(&normalized, &segments).is_none() {
            return Err(Error::PathNotFound {
                path: path.to_string(),
            });
        }

        let new_value = value.into();
        if !set_at_path(&mut normalized, &segments, new_value) {
            return Err(Error::PathSetFailed {
                format: format!("{:?}", self.format),
                path: path.to_string(),
                reason: "Failed to set value at path".to_string(),
            });
        }

        // Re-render the document from the normalized form
        let new_source = self.render_from_normalized(&normalized)?;
        self.source = new_source.clone();

        Ok(Edit::path_set(
            path,
            0..old_source.len(),
            old_source,
            new_source,
        ))
    }

    /// Remove a value at the given path.
    ///
    /// Returns an `Edit` for rollback support.
    ///
    /// # Errors
    ///
    /// Returns `PathNotFound` if the path doesn't exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use repo_content::Document;
    /// use serde_json::json;
    ///
    /// let mut doc = Document::parse(r#"{"name": "test", "version": "1.0"}"#).unwrap();
    /// let edit = doc.remove_path("version").unwrap();
    /// assert_eq!(doc.get_path("version"), None);
    /// ```
    pub fn remove_path(&mut self, path: &str) -> Result<Edit> {
        let old_source = self.source.clone();
        let mut normalized = self.handler.normalize(&self.source)?;
        let segments = parse_path(path);

        // Check that the path exists
        if get_at_path(&normalized, &segments).is_none() {
            return Err(Error::PathNotFound {
                path: path.to_string(),
            });
        }

        if remove_at_path(&mut normalized, &segments).is_none() {
            return Err(Error::PathNotFound {
                path: path.to_string(),
            });
        }

        // Re-render the document from the normalized form
        let new_source = self.render_from_normalized(&normalized)?;
        self.source = new_source;

        Ok(Edit::path_remove(path, 0..old_source.len(), old_source))
    }

    /// Render the document from a normalized JSON value.
    ///
    /// This is an internal helper for set_path and remove_path.
    fn render_from_normalized(&self, normalized: &Value) -> Result<String> {
        match self.format {
            Format::Json => {
                // Pretty print JSON
                Ok(serde_json::to_string_pretty(normalized)?)
            }
            Format::Toml => {
                // Convert JSON to TOML
                let toml_value: toml::Value = json_to_toml(normalized)?;
                Ok(toml::to_string_pretty(&toml_value)
                    .map_err(|e| Error::parse("TOML", e.to_string()))?)
            }
            Format::Yaml => {
                // Convert JSON to YAML
                Ok(serde_yaml::to_string(normalized)
                    .map_err(|e| Error::parse("YAML", e.to_string()))?)
            }
            Format::Markdown | Format::PlainText => {
                // For text formats, we can't really re-render from normalized
                // This would need format-specific handling
                Err(Error::PathSetFailed {
                    format: format!("{:?}", self.format),
                    path: String::new(),
                    reason: "Cannot modify structured data in text format".to_string(),
                })
            }
        }
    }
}

/// Convert a serde_json::Value to a toml::Value
fn json_to_toml(json: &Value) -> Result<toml::Value> {
    match json {
        Value::Null => Err(Error::parse("TOML", "TOML does not support null values")),
        Value::Bool(b) => Ok(toml::Value::Boolean(*b)),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(toml::Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(toml::Value::Float(f))
            } else {
                Err(Error::parse("TOML", "Invalid number"))
            }
        }
        Value::String(s) => Ok(toml::Value::String(s.clone())),
        Value::Array(arr) => {
            let toml_arr: Result<Vec<toml::Value>> = arr.iter().map(json_to_toml).collect();
            Ok(toml::Value::Array(toml_arr?))
        }
        Value::Object(obj) => {
            let mut toml_map = toml::map::Map::new();
            for (k, v) in obj {
                toml_map.insert(k.clone(), json_to_toml(v)?);
            }
            Ok(toml::Value::Table(toml_map))
        }
    }
}
