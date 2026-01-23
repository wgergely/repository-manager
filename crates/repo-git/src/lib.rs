//! Git abstraction for Repository Manager
//!
//! Supports multiple worktree layout styles through a unified interface.

pub mod error;
pub mod provider;
pub mod naming;
pub mod container;
pub mod in_repo_worktrees;
pub mod classic;

pub use error::{Error, Result};
pub use provider::{LayoutProvider, WorktreeInfo};
pub use naming::NamingStrategy;
