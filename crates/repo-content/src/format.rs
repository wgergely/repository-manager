//! Format detection and handling types.

use serde::{Deserialize, Serialize};

/// Comment style for a file format.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommentStyle {
    /// C-style comments: // and /* */
    CStyle,
    /// Hash comments: #
    Hash,
    /// HTML comments: <!-- -->
    Html,
    /// No comments supported.
    None,
}

/// File format identifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Format {
    /// Markdown format.
    Markdown,
    /// TOML format.
    Toml,
    /// JSON format.
    Json,
    /// Rust source code.
    Rust,
    /// TypeScript source code.
    TypeScript,
    /// Plain text with no semantic structure.
    PlainText,
    /// Unknown format.
    Unknown,
}

/// Trait for format-specific handling.
pub trait FormatHandler {
    /// Returns the comment style for this format.
    fn comment_style(&self) -> CommentStyle;

    /// Returns the file extensions associated with this format.
    fn extensions(&self) -> &[&str];
}
