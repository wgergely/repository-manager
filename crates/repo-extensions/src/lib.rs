//! Extension system for Repository Manager.
//!
//! This crate provides manifest parsing, configuration, MCP resolution,
//! dependency graph construction, version constraint checking, lock file
//! management, and a registry for repository-manager extensions.

pub mod config;
pub mod dependency;
pub mod error;
pub mod installer;
pub mod lock;
pub mod manifest;
pub mod mcp;
pub mod registry;
pub mod version;

/// The canonical filename for extension manifest files.
///
/// Client extensions must place a file with this name at the root of their
/// repository so the repo manager can discover and validate them.
pub const MANIFEST_FILENAME: &str = "repo_extension.toml";

pub use config::ExtensionConfig;
pub use dependency::{DependencyGraph, DependencyNode, NodeKind};
pub use error::Error;
pub use installer::{check_binary_on_path, query_python_version, run_install, synthesize_install_command};
pub use lock::{LOCK_FILENAME, LockFile, LockedExtension};
pub use manifest::{EntryPoints, ExtensionManifest, Provides, ResolvedCommand, ResolvedEntryPoints};
pub use mcp::{ResolveContext, merge_mcp_configs, resolve_mcp_config};
pub use registry::{ExtensionEntry, ExtensionRegistry};
pub use version::VersionConstraint;
