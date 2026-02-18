//! Block parsing and writing for Repository Manager.
//!
//! This crate provides functionality for parsing and writing structured blocks
//! in configuration files. Supports multiple file formats through the `formats` module.
//!
//! # Two Block-Marker Systems
//!
//! This crate contains **two independent block-marker systems** that serve different layers:
//!
//! ## 1. `parser` + `writer` modules (HTML comment markers)
//!
//! Used by `repo-tools` for embedding managed blocks in tool config files (e.g.,
//! `.cursorrules`, `CLAUDE.md`). These use HTML-comment markers:
//!
//! ```text
//! <!-- repo:block:UUID -->
//! content
//! <!-- /repo:block:UUID -->
//! ```
//!
//! UUIDs are short alphanumeric IDs (e.g., `abc-123`).
//!
//! ## 2. `formats` module (format-specific markers)
//!
//! Used by `repo-content` for format-aware block management. Markers vary by file type:
//!
//! - **TOML/YAML**: `# repo:block:<uuid-v4>` / `# /repo:block:<uuid-v4>`
//! - **JSON**: `_repo_managed` key in JSON objects
//! - **Markdown/PlainText**: HTML comment markers (same style as system 1)
//!
//! UUIDs are full UUID-v4 values (e.g., `550e8400-e29b-41d4-a716-446655440000`).
//!
//! The two systems are not interchangeable. Use `parser`/`writer` for tool integration
//! and `formats` for content-level document management.

pub mod error;
pub mod formats;
pub mod parser;
pub mod writer;

pub use error::{Error, Result};
pub use formats::{
    FormatHandler, JsonFormatHandler, ManagedBlock, TomlFormatHandler, YamlFormatHandler,
};
pub use parser::{Block, find_block, has_block, parse_blocks};
pub use writer::{insert_block, remove_block, update_block, upsert_block};
