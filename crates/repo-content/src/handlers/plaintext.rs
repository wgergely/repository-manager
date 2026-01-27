//! Plain text format handler

use uuid::Uuid;

use super::html_comment;
use crate::block::{BlockLocation, ManagedBlock};
use crate::edit::Edit;
use crate::error::{Error, Result};
use crate::format::{Format, FormatHandler};

/// Handler for plain text files with HTML comment markers
#[derive(Debug, Default)]
pub struct PlainTextHandler;

impl PlainTextHandler {
    pub fn new() -> Self {
        Self
    }
}

impl FormatHandler for PlainTextHandler {
    fn format(&self) -> Format {
        Format::PlainText
    }

    fn parse(&self, source: &str) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        Ok(Box::new(source.to_string()))
    }

    fn find_blocks(&self, source: &str) -> Vec<ManagedBlock> {
        html_comment::find_blocks(source)
    }

    fn insert_block(
        &self,
        source: &str,
        uuid: Uuid,
        content: &str,
        location: BlockLocation,
    ) -> Result<(String, Edit)> {
        html_comment::insert_block(source, uuid, content, location)
    }

    fn update_block(&self, source: &str, uuid: Uuid, content: &str) -> Result<(String, Edit)> {
        html_comment::update_block(source, uuid, content)
    }

    fn remove_block(&self, source: &str, uuid: Uuid) -> Result<(String, Edit)> {
        html_comment::remove_block(source, uuid)
    }

    fn normalize(&self, source: &str) -> Result<serde_json::Value> {
        // For plain text, normalize whitespace by trimming line endings and overall content
        let normalized: String = source
            .lines()
            .map(|l| l.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();
        Ok(serde_json::Value::String(normalized))
    }

    fn render(&self, parsed: &dyn std::any::Any) -> Result<String> {
        parsed
            .downcast_ref::<String>()
            .cloned()
            .ok_or_else(|| Error::parse("plaintext", "invalid internal state"))
    }
}
