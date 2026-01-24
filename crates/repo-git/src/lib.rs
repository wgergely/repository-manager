//! Git abstraction for Repository Manager
//!
//! Supports multiple worktree layout styles through a unified interface.

pub mod classic;
pub mod container;
pub mod error;
pub mod in_repo_worktrees;
pub mod naming;
pub mod provider;

pub use classic::ClassicLayout;
pub use container::ContainerLayout;
pub use error::{Error, Result};
pub use in_repo_worktrees::InRepoWorktreesLayout;
pub use naming::NamingStrategy;
pub use provider::{LayoutProvider, WorktreeInfo};
