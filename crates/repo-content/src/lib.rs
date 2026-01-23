//! Content parsing, editing, and diffing for Repository Manager
//!
//! Provides robust operations for reading, writing, editing, matching,
//! and diffing files with semantic understanding.

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
pub use handlers::PlainTextHandler;
