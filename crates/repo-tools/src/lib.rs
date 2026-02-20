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
pub mod claude_desktop;
pub mod cline;
pub mod copilot;
pub mod cursor;
pub mod dispatcher;
pub mod error;
pub mod gemini;
pub mod generic;
pub mod integration;
pub mod jetbrains;
pub mod mcp_installer;
pub mod mcp_registry;
pub mod mcp_translate;
pub mod registry;
pub mod roo;
pub mod syncer;
pub mod translator;
pub mod vscode;
pub mod windsurf;
pub mod writer;
pub mod zed;

pub use aider::aider_integration;
pub use amazonq::amazonq_integration;
pub use antigravity::{AntigravityIntegration, antigravity_integration};
pub use claude::{ClaudeIntegration, claude_integration};
pub use claude_desktop::claude_desktop_integration;
pub use cline::cline_integration;
pub use copilot::copilot_integration;
pub use cursor::{CursorIntegration, cursor_integration};
pub use dispatcher::ToolDispatcher;
pub use error::{Error, Result};
pub use gemini::{GeminiIntegration, gemini_integration};
pub use generic::GenericToolIntegration;
pub use integration::{ConfigLocation, ConfigType, Rule, SyncContext, ToolIntegration};
pub use jetbrains::jetbrains_integration;
pub use roo::roo_integration;
pub use vscode::{VSCodeIntegration, vscode_definition};
pub use windsurf::{WindsurfIntegration, windsurf_integration};
pub use zed::zed_integration;

// Registry types
pub use registry::{
    BUILTIN_COUNT, ToolCategory, ToolRegistration, ToolRegistry, builtin_registrations,
};

// Translator types
pub use translator::{CapabilityTranslator, RuleTranslator, TranslatedContent};

// Writer types
pub use writer::{
    ConfigWriter, JsonWriter, MarkdownWriter, SchemaKeys, TextWriter, WriterRegistry,
};

// MCP registry, translation, and installation
pub use mcp_installer::McpInstaller;
pub use mcp_registry::{MCP_CAPABLE_TOOLS, mcp_config_spec};
pub use mcp_translate::{from_tool_json, to_tool_json};

// Syncer
pub use syncer::ToolCapabilitySyncer;
