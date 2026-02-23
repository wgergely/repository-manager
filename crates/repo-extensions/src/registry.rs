//! Extension registry for known/built-in extensions.

use std::collections::HashMap;

use crate::error::{Error, Result};

/// Metadata for a known extension in the registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionEntry {
    /// Extension name (must be non-empty, alphanumeric + hyphens/underscores).
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Default source URL (must be non-empty).
    pub source: String,
}

impl ExtensionEntry {
    /// Validate that this entry has well-formed fields.
    ///
    /// Returns `Ok(())` if the name follows the same rules as extension
    /// manifest names and the source is non-empty.
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(Error::InvalidName {
                name: self.name.clone(),
                reason: "extension name must not be empty".to_string(),
            });
        }
        if !self
            .name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(Error::InvalidName {
                name: self.name.clone(),
                reason: "extension name must contain only alphanumeric characters, hyphens, or underscores".to_string(),
            });
        }
        if self.source.is_empty() {
            return Err(Error::InvalidSource {
                name: self.name.clone(),
                reason: "extension source must not be empty".to_string(),
            });
        }
        Ok(())
    }
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
        // Safety: these entries are statically known to be valid.
        registry
            .register(ExtensionEntry {
                name: "vaultspec".to_string(),
                description: "A governed development framework for AI agents".to_string(),
                source: "https://github.com/vaultspec/vaultspec.git".to_string(),
            })
            .expect("built-in extension entries must be valid");
        registry
    }

    /// Register an extension entry after validation.
    ///
    /// Returns an error if the entry has an invalid name or empty source.
    /// If an entry with the same name already exists it is replaced.
    pub fn register(&mut self, entry: ExtensionEntry) -> Result<()> {
        entry.validate()?;
        self.entries.insert(entry.name.clone(), entry);
        Ok(())
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

    /// Iterate over all registry entries (in arbitrary order).
    pub fn iter(&self) -> impl Iterator<Item = &ExtensionEntry> {
        self.entries.values()
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
        registry
            .register(ExtensionEntry {
                name: "beta".to_string(),
                description: "Beta extension".to_string(),
                source: "https://example.com/beta.git".to_string(),
            })
            .unwrap();
        registry
            .register(ExtensionEntry {
                name: "alpha".to_string(),
                description: "Alpha extension".to_string(),
                source: "https://example.com/alpha.git".to_string(),
            })
            .unwrap();

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
        registry
            .register(ExtensionEntry {
                name: "test".to_string(),
                description: "First".to_string(),
                source: "https://example.com/first.git".to_string(),
            })
            .unwrap();
        registry
            .register(ExtensionEntry {
                name: "test".to_string(),
                description: "Second".to_string(),
                source: "https://example.com/second.git".to_string(),
            })
            .unwrap();

        assert_eq!(registry.len(), 1);
        assert_eq!(registry.get("test").unwrap().description, "Second");
    }

    // --- Validation tests ---

    #[test]
    fn test_register_rejects_empty_name() {
        let mut registry = ExtensionRegistry::new();
        let result = registry.register(ExtensionEntry {
            name: String::new(),
            description: "Bad".to_string(),
            source: "https://example.com/ext.git".to_string(),
        });
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::InvalidName { .. }
        ));
    }

    #[test]
    fn test_register_rejects_invalid_name() {
        let mut registry = ExtensionRegistry::new();
        let result = registry.register(ExtensionEntry {
            name: "bad name!".to_string(),
            description: "Bad".to_string(),
            source: "https://example.com/ext.git".to_string(),
        });
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::InvalidName { .. }
        ));
    }

    #[test]
    fn test_register_rejects_empty_source() {
        let mut registry = ExtensionRegistry::new();
        let result = registry.register(ExtensionEntry {
            name: "valid-name".to_string(),
            description: "Desc".to_string(),
            source: String::new(),
        });
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::InvalidSource { .. }
        ));
    }

    #[test]
    fn test_entry_equality() {
        let a = ExtensionEntry {
            name: "ext".to_string(),
            description: "Desc".to_string(),
            source: "https://example.com/ext.git".to_string(),
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn test_iter_returns_all_entries() {
        let registry = ExtensionRegistry::with_known();
        let entries: Vec<&ExtensionEntry> = registry.iter().collect();
        assert_eq!(entries.len(), registry.len());
        assert!(entries.iter().any(|e| e.name == "vaultspec"));
    }
}
