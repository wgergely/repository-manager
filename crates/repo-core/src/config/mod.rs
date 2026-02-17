//! Configuration resolution and runtime context
//!
//! This module provides hierarchical configuration resolution for Repository Manager,
//! supporting multiple configuration sources that are merged in a defined order.
//!
//! # Configuration Hierarchy
//!
//! Configuration is loaded and merged from these sources (later sources override earlier):
//!
//! 1. **Global defaults** - `~/.config/repo-manager/config.toml` (TODO)
//! 2. **Organization config** - Organization-level settings (TODO)
//! 3. **Repository config** - `.repository/config.toml`
//! 4. **Local overrides** - `.repository/config.local.toml` (git-ignored)
//!
//! # Presets
//!
//! Presets are keyed by type and name (e.g., `env:python`, `tool:linter`):
//!
//! - `env:*` - Environment configurations (Python, Node, etc.)
//! - `tool:*` - Tool-specific settings
//! - `config:*` - General configuration presets
//!
//! # Example
//!
//! ```ignore
//! use repo_core::config::{ConfigResolver, RuntimeContext};
//! use repo_fs::NormalizedPath;
//!
//! // Resolve configuration
//! let resolver = ConfigResolver::new(NormalizedPath::new("/path/to/repo"));
//! let config = resolver.resolve()?;
//!
//! // Generate runtime context for agents
//! let context = RuntimeContext::from_resolved(&config);
//! let json = context.to_json();
//! ```

mod manifest;
mod resolver;
mod runtime;

pub use manifest::{Manifest, json_to_toml_value};
pub use resolver::{ConfigResolver, ResolvedConfig};
pub use runtime::RuntimeContext;
