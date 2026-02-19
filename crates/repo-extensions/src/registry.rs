//! Extension registry for known/built-in extensions.

use std::collections::HashMap;

/// Metadata for a known extension in the registry.
#[derive(Debug, Clone)]
pub struct ExtensionEntry {
    /// Extension name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Default source URL.
    pub source: String,
}

/// Registry of known extensions.
///
/// Provides a catalog of built-in extensions that can be discovered
/// and installed without manually specifying a source URL.
#[derive(Debug, Clone, Default)]
pub struct ExtensionRegistry {
    entries: HashMap<String, ExtensionEntry>,
}

impl ExtensionRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Create a registry populated with built-in known extensions.
    pub fn with_known() -> Self {
        let mut registry = Self::new();
        registry.register(ExtensionEntry {
            name: "vaultspec".to_string(),
            description: "A governed development framework for AI agents".to_string(),
            source: "https://github.com/vaultspec/vaultspec.git".to_string(),
        });
        registry
    }

    /// Register an extension entry.
    pub fn register(&mut self, entry: ExtensionEntry) {
        self.entries.insert(entry.name.clone(), entry);
    }

    /// Look up an extension by name.
    pub fn get(&self, name: &str) -> Option<&ExtensionEntry> {
        self.entries.get(name)
    }

    /// List all known extension names (sorted).
    pub fn known_extensions(&self) -> Vec<String> {
        let mut names: Vec<String> = self.entries.keys().cloned().collect();
        names.sort();
        names
    }

    /// Check if an extension is known.
    pub fn contains(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }

    /// Number of registered extensions.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_registry_is_empty() {
        let registry = ExtensionRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_with_known_has_vaultspec() {
        let registry = ExtensionRegistry::with_known();
        assert!(!registry.is_empty());
        assert!(registry.contains("vaultspec"));

        let entry = registry.get("vaultspec").unwrap();
        assert_eq!(entry.name, "vaultspec");
        assert!(!entry.description.is_empty());
        assert!(!entry.source.is_empty());
    }

    #[test]
    fn test_known_extensions_sorted() {
        let mut registry = ExtensionRegistry::new();
        registry.register(ExtensionEntry {
            name: "beta".to_string(),
            description: "Beta extension".to_string(),
            source: "https://example.com/beta.git".to_string(),
        });
        registry.register(ExtensionEntry {
            name: "alpha".to_string(),
            description: "Alpha extension".to_string(),
            source: "https://example.com/alpha.git".to_string(),
        });

        assert_eq!(registry.known_extensions(), vec!["alpha", "beta"]);
    }

    #[test]
    fn test_unknown_extension_returns_none() {
        let registry = ExtensionRegistry::with_known();
        assert!(registry.get("nonexistent").is_none());
        assert!(!registry.contains("nonexistent"));
    }

    #[test]
    fn test_register_replaces_existing() {
        let mut registry = ExtensionRegistry::new();
        registry.register(ExtensionEntry {
            name: "test".to_string(),
            description: "First".to_string(),
            source: "https://example.com/first.git".to_string(),
        });
        registry.register(ExtensionEntry {
            name: "test".to_string(),
            description: "Second".to_string(),
            source: "https://example.com/second.git".to_string(),
        });

        assert_eq!(registry.len(), 1);
        assert_eq!(registry.get("test").unwrap().description, "Second");
    }
}
