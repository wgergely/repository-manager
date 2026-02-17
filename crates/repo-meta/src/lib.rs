//! Metadata and configuration management for Repository Manager.
//!
//! This crate provides configuration types, schema definitions, and a
//! provider registry for managing repository metadata.
//!
//! ## Schema-Driven Registration
//!
//! Tools, rules, and presets are defined in TOML files under `.repository/`:
//!
//! ```text
//! .repository/
//!   tools/      # Tool definitions (cursor.toml, vscode.toml, etc.)
//!   rules/      # Rule definitions (python-snake-case.toml, etc.)
//!   presets/    # Preset definitions (python-agentic.toml, etc.)
//! ```
//!
//! Use [`DefinitionLoader`] to load these definitions:
//!
//! ```ignore
//! use repo_meta::DefinitionLoader;
//! use repo_fs::NormalizedPath;
//!
//! let loader = DefinitionLoader::new();
//! let root = NormalizedPath::new("/path/to/repo");
//!
//! let tools = loader.load_tools(&root)?;
//! let rules = loader.load_rules(&root)?;
//! let presets = loader.load_presets(&root)?;
//! ```

pub mod config;
pub mod error;
pub mod loader;
pub mod registry;
pub mod schema;
pub mod validation;

// Note: tools.rs was removed - ToolRegistry is now only in validation.rs

pub use config::{
    ActiveConfig, CoreConfig, RepositoryConfig, RepositoryMode, SyncConfig, get_preset_config,
    load_config,
};
pub use error::{Error, Result};
pub use loader::DefinitionLoader;
pub use registry::Registry;
pub use schema::{PresetDefinition, RuleDefinition, ToolDefinition};
pub use validation::{PresetRegistry, ToolRegistry};
