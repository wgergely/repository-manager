//! Filesystem abstraction for Repository Manager
//!
//! Provides layout-agnostic path resolution and safe I/O operations.

pub mod config;
pub mod constants;
pub mod error;
pub mod io;
pub mod layout;
pub mod path;

pub use config::ConfigStore;
pub use constants::RepoPath;
pub use error::{Error, Result};
pub use layout::{LayoutMode, WorkspaceLayout};
pub use path::NormalizedPath;
