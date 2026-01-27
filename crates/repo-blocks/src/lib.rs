//! Block parsing and writing for Repository Manager.
//!
//! This crate provides functionality for parsing and writing structured blocks
//! in configuration files. Supports multiple file formats through the `formats` module.

pub mod error;
pub mod formats;
pub mod parser;
pub mod writer;

pub use error::{Error, Result};
pub use formats::{FormatHandler, JsonFormatHandler, ManagedBlock};
pub use parser::{Block, find_block, has_block, parse_blocks};
pub use writer::{insert_block, remove_block, update_block, upsert_block};
