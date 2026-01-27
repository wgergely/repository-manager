//! Tool integrations for Repository Manager.
//!
//! This crate provides integrations with various development tools
//! such as VSCode, Cursor, and Claude.
//!
//! # Architecture
//!
//! The crate provides two levels of tool integration:
//!
//! 1. **Built-in integrations** - Optimized implementations for common tools
//!    (VSCode, Cursor, Claude) that handle tool-specific edge cases.
//!
//! 2. **Generic integration** - A schema-driven implementation that uses
//!    `ToolDefinition` from `repo-meta` to integrate with any tool without
//!    writing new Rust code.
//!
//! The `ToolDispatcher` routes requests to the appropriate integration,
//! preferring built-in implementations when available.

pub mod claude;
pub mod cursor;
pub mod dispatcher;
pub mod error;
pub mod generic;
pub mod integration;
pub mod logging;
pub mod vscode;

pub use claude::{claude_integration, ClaudeIntegration};
pub use cursor::{cursor_integration, CursorIntegration};
pub use dispatcher::ToolDispatcher;
pub use error::{Error, Result};
pub use generic::GenericToolIntegration;
pub use integration::{Rule, SyncContext, ToolIntegration};
pub use vscode::VSCodeIntegration;
