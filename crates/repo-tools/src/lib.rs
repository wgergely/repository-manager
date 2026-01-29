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

pub mod aider;
pub mod amazonq;
pub mod antigravity;
pub mod claude;
pub mod cline;
pub mod copilot;
pub mod cursor;
pub mod dispatcher;
pub mod error;
pub mod gemini;
pub mod generic;
pub mod integration;
pub mod jetbrains;
pub mod registry;
pub mod logging;
pub mod roo;
pub mod vscode;
pub mod windsurf;
pub mod zed;

pub use aider::aider_integration;
pub use amazonq::amazonq_integration;
pub use antigravity::{antigravity_integration, AntigravityIntegration};
pub use claude::{claude_integration, ClaudeIntegration};
pub use cline::cline_integration;
pub use copilot::copilot_integration;
pub use cursor::{cursor_integration, CursorIntegration};
pub use dispatcher::ToolDispatcher;
pub use error::{Error, Result};
pub use gemini::{gemini_integration, GeminiIntegration};
pub use generic::GenericToolIntegration;
pub use integration::{ConfigLocation, ConfigType, Rule, SyncContext, ToolIntegration};
pub use jetbrains::jetbrains_integration;
pub use roo::roo_integration;
pub use vscode::VSCodeIntegration;
pub use windsurf::{windsurf_integration, WindsurfIntegration};
pub use zed::zed_integration;

// Registry types
pub use registry::{ToolCategory, ToolRegistration, ToolRegistry};
