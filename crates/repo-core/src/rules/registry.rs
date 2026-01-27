//! Rule Registry for central rule management
//!
//! The registry is the single source of truth for all rules.
//! It persists to `.repository/rules/registry.toml`.

use super::rule::Rule;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Central registry of all rules
///
/// The registry stores rules in TOML format and provides CRUD operations.
/// Rule UUIDs are used as managed block markers in tool config files.
#[derive(Debug, Serialize, Deserialize)]
pub struct RuleRegistry {
    /// Registry format version
    version: String,
    /// All registered rules
    #[serde(default)]
    rules: Vec<Rule>,
    /// Path to registry file (not serialized)
    #[serde(skip)]
    path: PathBuf,
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            rules: Vec::new(),
            path: PathBuf::new(),
        }
    }
}

impl RuleRegistry {
    /// Create a new empty registry at the given path
    pub fn new(path: PathBuf) -> Self {
        Self {
            version: "1.0".to_string(),
            rules: Vec::new(),
            path,
        }
    }

    /// Load registry from TOML file
    pub fn load(path: PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        let mut registry: Self = toml::from_str(&content)?;
        registry.path = path;
        Ok(registry)
    }

    /// Load registry or create new if doesn't exist
    pub fn load_or_create(path: PathBuf) -> Result<Self> {
        if path.exists() {
            Self::load(path)
        } else {
            Ok(Self::new(path))
        }
    }

    /// Save registry to TOML file
    pub fn save(&self) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&self.path, content)?;
        Ok(())
    }

    /// Add a new rule to the registry
    ///
    /// Generates a UUID and saves the registry.
    pub fn add_rule(
        &mut self,
        id: &str,
        content: &str,
        tags: Vec<String>,
    ) -> Result<&Rule> {
        let rule = Rule::new(id, content, tags);
        self.rules.push(rule);
        self.save()?;
        Ok(self.rules.last().unwrap())
    }

    /// Get a rule by UUID
    pub fn get_rule(&self, uuid: Uuid) -> Option<&Rule> {
        self.rules.iter().find(|r| r.uuid == uuid)
    }

    /// Get a mutable reference to a rule by UUID
    pub fn get_rule_mut(&mut self, uuid: Uuid) -> Option<&mut Rule> {
        self.rules.iter_mut().find(|r| r.uuid == uuid)
    }

    /// Get a rule by human-readable ID
    ///
    /// If multiple rules have the same ID, returns the first one.
    pub fn get_rule_by_id(&self, id: &str) -> Option<&Rule> {
        self.rules.iter().find(|r| r.id == id)
    }

    /// Update a rule's content
    pub fn update_rule(&mut self, uuid: Uuid, new_content: &str) -> Result<()> {
        if let Some(rule) = self.get_rule_mut(uuid) {
            rule.update_content(new_content);
            self.save()?;
            Ok(())
        } else {
            Err(crate::Error::NotFound(format!("Rule with UUID {} not found", uuid)))
        }
    }

    /// Remove a rule by UUID
    ///
    /// Returns the removed rule if found.
    pub fn remove_rule(&mut self, uuid: Uuid) -> Option<Rule> {
        let pos = self.rules.iter().position(|r| r.uuid == uuid)?;
        let rule = self.rules.remove(pos);
        self.save().ok()?;
        Some(rule)
    }

    /// Get all rules
    pub fn all_rules(&self) -> &[Rule] {
        &self.rules
    }

    /// Get rules by tag
    pub fn rules_by_tag(&self, tag: &str) -> Vec<&Rule> {
        self.rules
            .iter()
            .filter(|r| r.tags.iter().any(|t| t == tag))
            .collect()
    }

    /// Get the registry file path
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Check if a rule ID already exists
    pub fn has_rule_id(&self, id: &str) -> bool {
        self.rules.iter().any(|r| r.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_new_registry() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("registry.toml");
        let registry = RuleRegistry::new(path.clone());

        assert_eq!(registry.version, "1.0");
        assert!(registry.rules.is_empty());
        assert_eq!(registry.path, path);
    }

    #[test]
    fn test_save_and_load() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("registry.toml");

        // Create and save
        {
            let mut registry = RuleRegistry::new(path.clone());
            registry.rules.push(Rule::new("test", "content", vec![]));
            registry.save().unwrap();
        }

        // Load and verify
        let loaded = RuleRegistry::load(path).unwrap();
        assert_eq!(loaded.rules.len(), 1);
        assert_eq!(loaded.rules[0].id, "test");
    }

    #[test]
    fn test_load_or_create_new() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("nonexistent.toml");

        let registry = RuleRegistry::load_or_create(path).unwrap();
        assert!(registry.rules.is_empty());
    }

    #[test]
    fn test_load_or_create_existing() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("existing.toml");

        // Create first
        {
            let mut registry = RuleRegistry::new(path.clone());
            registry.rules.push(Rule::new("existing", "content", vec![]));
            registry.save().unwrap();
        }

        // Load existing
        let registry = RuleRegistry::load_or_create(path).unwrap();
        assert_eq!(registry.rules.len(), 1);
    }
}
