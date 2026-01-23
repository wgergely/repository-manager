//! Loader for tool, rule, and preset definitions from .repository/
//!
//! This module provides the `DefinitionLoader` which loads TOML definitions
//! from the `.repository/` directory structure:
//!
//! ```text
//! .repository/
//!   tools/
//!     cursor.toml
//!     vscode.toml
//!   rules/
//!     python-snake-case.toml
//!     no-api-keys.toml
//!   presets/
//!     python-agentic.toml
//! ```

use crate::schema::{PresetDefinition, RuleDefinition, ToolDefinition};
use crate::{Error, Result};
use repo_fs::{ConfigStore, NormalizedPath};
use std::collections::HashMap;
use std::fs;

/// Loads all definitions from .repository/ directory
pub struct DefinitionLoader {
    store: ConfigStore,
}

impl DefinitionLoader {
    /// Create a new DefinitionLoader
    pub fn new() -> Self {
        Self {
            store: ConfigStore::new(),
        }
    }

    /// Load all tool definitions from .repository/tools/
    ///
    /// # Arguments
    ///
    /// * `root` - Repository root path
    ///
    /// # Returns
    ///
    /// A map of tool slug to tool definition
    pub fn load_tools(&self, root: &NormalizedPath) -> Result<HashMap<String, ToolDefinition>> {
        let tools_dir = root.join(".repository").join("tools");
        self.load_definitions(&tools_dir)
    }

    /// Load all rule definitions from .repository/rules/
    ///
    /// # Arguments
    ///
    /// * `root` - Repository root path
    ///
    /// # Returns
    ///
    /// A map of rule ID to rule definition
    pub fn load_rules(&self, root: &NormalizedPath) -> Result<HashMap<String, RuleDefinition>> {
        let rules_dir = root.join(".repository").join("rules");
        self.load_definitions(&rules_dir)
    }

    /// Load all preset definitions from .repository/presets/
    ///
    /// # Arguments
    ///
    /// * `root` - Repository root path
    ///
    /// # Returns
    ///
    /// A map of preset ID to preset definition
    pub fn load_presets(&self, root: &NormalizedPath) -> Result<HashMap<String, PresetDefinition>> {
        let presets_dir = root.join(".repository").join("presets");
        self.load_definitions(&presets_dir)
    }

    /// Generic loader for definitions from a directory
    fn load_definitions<T>(&self, dir: &NormalizedPath) -> Result<HashMap<String, T>>
    where
        T: serde::de::DeserializeOwned + HasId,
    {
        let mut definitions = HashMap::new();

        if !dir.exists() {
            return Ok(definitions);
        }

        let entries = fs::read_dir(dir.to_native())
            .map_err(|e| Error::Fs(repo_fs::Error::io(dir.to_native(), e)))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "toml") {
                let norm_path = NormalizedPath::new(&path);
                match self.store.load::<T>(&norm_path) {
                    Ok(def) => {
                        definitions.insert(def.id().to_string(), def);
                    }
                    Err(e) => {
                        // Log warning but continue loading other files
                        tracing::warn!("Failed to load {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(definitions)
    }
}

impl Default for DefinitionLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for types that have an ID
///
/// This is implemented by all definition types to allow the generic
/// loader to extract the ID for building the hashmap.
pub trait HasId {
    /// Returns the unique identifier for this definition
    fn id(&self) -> &str;
}

impl HasId for ToolDefinition {
    fn id(&self) -> &str {
        &self.meta.slug
    }
}

impl HasId for RuleDefinition {
    fn id(&self) -> &str {
        &self.meta.id
    }
}

impl HasId for PresetDefinition {
    fn id(&self) -> &str {
        &self.meta.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loader_new() {
        let loader = DefinitionLoader::new();
        // Just verify it creates without error
        assert!(std::mem::size_of_val(&loader) > 0);
    }

    #[test]
    fn test_loader_default() {
        let loader = DefinitionLoader::default();
        assert!(std::mem::size_of_val(&loader) > 0);
    }

    #[test]
    fn test_load_from_nonexistent_dir() {
        let loader = DefinitionLoader::new();
        let root = NormalizedPath::new("/nonexistent/path");

        // Should return empty maps, not errors
        let tools = loader.load_tools(&root).unwrap();
        assert!(tools.is_empty());

        let rules = loader.load_rules(&root).unwrap();
        assert!(rules.is_empty());

        let presets = loader.load_presets(&root).unwrap();
        assert!(presets.is_empty());
    }
}
