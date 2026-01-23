//! Block parsing and writing for Repository Manager.
//!
//! This crate provides functionality for parsing and writing structured blocks
//! in configuration files.

pub mod error;
pub mod parser;
pub mod writer;

pub use error::{Error, Result};
pub use parser::{parse_blocks, find_block, has_block, Block};
pub use writer::{insert_block, update_block, remove_block, upsert_block};
