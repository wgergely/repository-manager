//! Schema definitions for tools, rules, and presets
//!
//! This module provides strongly-typed definitions that are loaded from
//! TOML files in the `.repository/` directory structure:
//!
//! - `.repository/tools/*.toml` - Tool definitions
//! - `.repository/rules/*.toml` - Rule definitions
//! - `.repository/presets/*.toml` - Preset definitions

pub mod preset;
pub mod rule;
pub mod tool;

pub use preset::{PresetDefinition, PresetMeta, PresetRequires, PresetRules};
pub use rule::{RuleContent, RuleDefinition, RuleExamples, RuleMeta, RuleTargets, Severity};
pub use tool::{
    ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta, ToolSchemaKeys,
};
