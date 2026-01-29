//! Config Writers - Format-aware config file writers
//!
//! This module provides writers that understand different config formats
//! and can merge content appropriately:
//!
//! - **JsonWriter**: Semantic merge, preserves existing keys
//! - **MarkdownWriter**: Section-based merge with managed markers
//! - **TextWriter**: Full replacement (tool owns the file)

mod traits;

pub use traits::{ConfigWriter, SchemaKeys};
