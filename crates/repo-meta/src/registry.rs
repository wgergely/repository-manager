//! Provider registry for preset management
//!
//! This module provides a registry that maps preset IDs to their
//! implementing provider names.

use std::collections::HashMap;

/// Registry mapping preset IDs to provider names.
///
/// The registry tracks which provider implements each preset,
/// allowing the system to look up the correct provider when
/// a preset needs to be applied.
///
/// # Example
///
/// ```
/// use repo_meta::Registry;
///
/// let mut registry = Registry::new();
/// registry.register("env:python", "uv");
/// assert_eq!(registry.get_provider("env:python"), Some(&"uv".to_string()));
/// ```
#[derive(Debug, Clone, Default)]
pub struct Registry {
    /// Maps preset ID to provider name
    providers: HashMap<String, String>,
}

impl Registry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Create a registry with built-in presets registered.
    ///
    /// Currently registers:
    /// - `env:python` -> `uv`
    /// - `env:node` -> `node`
    /// - `env:rust` -> `rust`
    /// - `claude:superpowers` -> `superpowers`
    pub fn with_builtins() -> Self {
        let mut registry = Self::new();
        registry.register("env:python", "uv");
        registry.register("env:node", "node");
        registry.register("env:rust", "rust");
        registry.register("claude:superpowers", "superpowers");
        registry
    }

    /// Register a provider for a preset ID.
    ///
    /// If the preset was already registered, the previous provider
    /// is replaced.
    ///
    /// # Arguments
    ///
    /// * `preset_id` - The preset identifier (e.g., "env:python")
    /// * `provider_name` - The provider name (e.g., "uv")
    pub fn register(&mut self, preset_id: impl Into<String>, provider_name: impl Into<String>) {
        self.providers
            .insert(preset_id.into(), provider_name.into());
    }

    /// Get the provider name for a preset ID.
    ///
    /// # Arguments
    ///
    /// * `preset_id` - The preset identifier to look up
    ///
    /// # Returns
    ///
    /// The provider name if registered, or None.
    pub fn get_provider(&self, preset_id: &str) -> Option<&String> {
        self.providers.get(preset_id)
    }

    /// List all registered preset IDs.
    ///
    /// # Returns
    ///
    /// A sorted vector of preset IDs.
    pub fn list_presets(&self) -> Vec<String> {
        let mut presets: Vec<String> = self.providers.keys().cloned().collect();
        presets.sort();
        presets
    }

    /// Check if a provider is registered for a preset ID.
    ///
    /// # Arguments
    ///
    /// * `preset_id` - The preset identifier to check
    ///
    /// # Returns
    ///
    /// True if a provider is registered for the preset.
    pub fn has_provider(&self, preset_id: &str) -> bool {
        self.providers.contains_key(preset_id)
    }

    /// Get the number of registered presets.
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_registry_is_empty() {
        let registry = Registry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_with_builtins() {
        let registry = Registry::with_builtins();
        assert!(!registry.is_empty());
        assert!(registry.has_provider("env:python"));
        assert_eq!(registry.get_provider("env:python"), Some(&"uv".to_string()));
        assert!(registry.has_provider("env:node"));
        assert_eq!(registry.get_provider("env:node"), Some(&"node".to_string()));
        assert!(registry.has_provider("env:rust"));
        assert_eq!(registry.get_provider("env:rust"), Some(&"rust".to_string()));
        assert!(registry.has_provider("claude:superpowers"));
        assert_eq!(
            registry.get_provider("claude:superpowers"),
            Some(&"superpowers".to_string())
        );
    }

    #[test]
    fn test_register_and_get() {
        let mut registry = Registry::new();
        registry.register("env:node", "nvm");

        assert_eq!(registry.get_provider("env:node"), Some(&"nvm".to_string()));
        assert!(registry.has_provider("env:node"));
    }

    #[test]
    fn test_unknown_preset_returns_none() {
        let registry = Registry::new();
        assert_eq!(registry.get_provider("unknown"), None);
        assert!(!registry.has_provider("unknown"));
    }

    #[test]
    fn test_list_presets() {
        let mut registry = Registry::new();
        registry.register("env:python", "uv");
        registry.register("env:node", "nvm");
        registry.register("env:ruby", "rbenv");

        let presets = registry.list_presets();
        assert_eq!(presets, vec!["env:node", "env:python", "env:ruby"]);
    }

    #[test]
    fn test_register_replaces_existing() {
        let mut registry = Registry::new();
        registry.register("env:python", "pyenv");
        registry.register("env:python", "uv");

        assert_eq!(registry.get_provider("env:python"), Some(&"uv".to_string()));
        assert_eq!(registry.len(), 1);
    }
}
