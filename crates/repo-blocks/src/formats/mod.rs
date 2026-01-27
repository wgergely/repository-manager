//! Format-specific handlers for managed blocks
//!
//! This module provides handlers for different file formats, each implementing
//! the `FormatHandler` trait for parsing and writing managed blocks.

pub mod json;
pub mod toml;
pub mod yaml;

use uuid::Uuid;

/// A parsed managed block from a file
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedBlock {
    /// The rule UUID this block belongs to
    pub uuid: Uuid,
    /// The content inside the block
    pub content: String,
}

/// Handler for format-specific managed block operations
pub trait FormatHandler: Send + Sync {
    /// Parse all managed blocks from file content
    fn parse_blocks(&self, content: &str) -> Vec<ManagedBlock>;

    /// Write or update a managed block in the content
    /// Returns the new file content with the block added/updated
    fn write_block(&self, content: &str, uuid: Uuid, block_content: &str) -> String;

    /// Remove a managed block from the content
    /// Returns the new file content with the block removed
    fn remove_block(&self, content: &str, uuid: Uuid) -> String;

    /// Check if a block with this UUID exists
    fn has_block(&self, content: &str, uuid: Uuid) -> bool {
        self.parse_blocks(content).iter().any(|b| b.uuid == uuid)
    }

    /// Get block content by UUID
    fn get_block(&self, content: &str, uuid: Uuid) -> Option<String> {
        self.parse_blocks(content)
            .into_iter()
            .find(|b| b.uuid == uuid)
            .map(|b| b.content)
    }
}

pub use json::JsonFormatHandler;
pub use toml::TomlFormatHandler;
pub use yaml::YamlFormatHandler;
