//! Workspace layout detection and abstraction
//!
//! Handles different repository layouts (classic, container, in-repo worktrees).

// TODO: Implement in Task 1.3

/// Workspace layout abstraction.
pub struct WorkspaceLayout;

/// Layout mode enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    /// Classic single repository layout
    Classic,
    /// Container layout with multiple worktrees
    Container,
    /// In-repo worktrees layout
    InRepoWorktrees,
}
