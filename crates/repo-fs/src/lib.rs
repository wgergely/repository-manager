//! Filesystem abstraction for Repository Manager
//!
//! Provides layout-agnostic path resolution and safe I/O operations.

pub mod error;
pub mod path;
pub mod io;
pub mod config;
pub mod layout;

pub use error::{Error, Result};
pub use path::NormalizedPath;
pub use layout::{WorkspaceLayout, LayoutMode};
pub use config::ConfigStore;
