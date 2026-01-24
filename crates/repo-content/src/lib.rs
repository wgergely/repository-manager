//! # repo-content
//!
//! Content parsing, editing, and diffing for Repository Manager.
//!
//! This crate provides robust operations for reading, writing, editing,
//! matching, and diffing files with semantic understanding. It supports
//! managed blocks across multiple formats and provides format-preserving
//! editing where possible.
//!
//! ## Supported Formats
//!
//! - **TOML** - Using toml_edit for format preservation
//! - **JSON** - With `_repo_managed` key for blocks
//! - **Markdown** - HTML comment markers
//! - **Plain Text** - HTML comment markers
//!
//! ## Quick Start
//!
//! ```rust
//! use repo_content::{Document, Format, BlockLocation};
//! use uuid::Uuid;
//!
//! // Parse a TOML document with explicit format
//! let mut doc = Document::parse_as("[package]\nname = \"test\"", Format::Toml).unwrap();
//!
//! // Insert a managed block
//! let uuid = Uuid::new_v4();
//! doc.insert_block(uuid, "[managed]\nkey = \"value\"", BlockLocation::End).unwrap();
//!
//! // Find blocks
//! for block in doc.find_blocks() {
//!     println!("Block {}: {}", block.uuid, block.content);
//! }
//!
//! // Render output
//! println!("{}", doc.render());
//! ```
//!
//! ## Semantic Comparison
//!
//! Documents can be compared semantically, ignoring formatting differences:
//!
//! ```rust
//! use repo_content::Document;
//!
//! let doc1 = Document::parse(r#"{"a": 1, "b": 2}"#).unwrap();
//! let doc2 = Document::parse(r#"{"b": 2, "a": 1}"#).unwrap();
//!
//! assert!(doc1.semantic_eq(&doc2));
//! ```

pub mod block;
pub mod diff;
pub mod document;
pub mod edit;
pub mod error;
pub mod format;
pub mod handlers;

pub use block::{BlockLocation, ManagedBlock};
pub use diff::{SemanticChange, SemanticDiff};
pub use document::Document;
pub use edit::{Edit, EditKind};
pub use error::{Error, Result};
pub use format::{CommentStyle, Format, FormatHandler};
pub use handlers::{JsonHandler, PlainTextHandler, TomlHandler};
