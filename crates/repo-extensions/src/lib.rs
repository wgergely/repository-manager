//! Extension system for Repository Manager.
//!
//! This crate provides manifest parsing, configuration, MCP resolution,
//! and a registry for repository-manager extensions.

pub mod config;
pub mod error;
pub mod manifest;
pub mod mcp;
pub mod registry;

pub use config::ExtensionConfig;
pub use error::Error;
pub use manifest::{EntryPoints, ExtensionManifest, Provides, ResolvedCommand, ResolvedEntryPoints};
pub use mcp::{ResolveContext, merge_mcp_configs, resolve_mcp_config};
pub use registry::{ExtensionEntry, ExtensionRegistry};
