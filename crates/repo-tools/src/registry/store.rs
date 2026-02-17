//! Tool registry storage

use super::{ToolCategory, ToolRegistration};
use std::collections::HashMap;

/// Central registry for tool definitions.
///
/// Provides lookup by slug, filtering by category, and priority-based ordering.
pub struct ToolRegistry {
    tools: HashMap<String, ToolRegistration>,
}

impl ToolRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Create a registry pre-populated with all built-in tools.
    pub fn with_builtins() -> Self {
        let mut registry = Self::new();
        for reg in super::builtins::builtin_registrations() {
            registry.register(reg);
        }
        registry
    }

    /// Register a tool.
    pub fn register(&mut self, reg: ToolRegistration) {
        self.tools.insert(reg.slug.clone(), reg);
    }

    /// Get a registration by slug.
    pub fn get(&self, slug: &str) -> Option<&ToolRegistration> {
        self.tools.get(slug)
    }

    /// Check if a tool is registered.
    pub fn contains(&self, slug: &str) -> bool {
        self.tools.contains_key(slug)
    }

    /// Get the number of registered tools.
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// List all registered tool slugs (sorted).
    pub fn list(&self) -> Vec<&str> {
        let mut slugs: Vec<_> = self.tools.keys().map(|s| s.as_str()).collect();
        slugs.sort();
        slugs
    }

    /// List tools by category (sorted).
    pub fn by_category(&self, cat: ToolCategory) -> Vec<&str> {
        let mut slugs: Vec<_> = self
            .tools
            .iter()
            .filter(|(_, r)| r.category == cat)
            .map(|(s, _)| s.as_str())
            .collect();
        slugs.sort();
        slugs
    }

    /// Get all registrations sorted by priority (lower = higher priority).
    pub fn by_priority(&self) -> Vec<&ToolRegistration> {
        let mut tools: Vec<_> = self.tools.values().collect();
        tools.sort_by_key(|t| t.priority);
        tools
    }

    /// Iterate over all registrations.
    pub fn iter(&self) -> impl Iterator<Item = &ToolRegistration> {
        self.tools.values()
    }

    /// Get all registrations as a vec.
    pub fn all(&self) -> Vec<&ToolRegistration> {
        self.tools.values().collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_meta::schema::{
        ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta,
    };

    fn make_def(slug: &str) -> ToolDefinition {
        ToolDefinition {
            meta: ToolMeta {
                name: slug.to_uppercase(),
                slug: slug.into(),
                description: None,
            },
            integration: ToolIntegrationConfig {
                config_path: format!(".{}", slug),
                config_type: ConfigType::Text,
                additional_paths: vec![],
            },
            capabilities: ToolCapabilities::default(),
            schema_keys: None,
        }
    }

    fn make_reg(slug: &str, category: ToolCategory) -> ToolRegistration {
        ToolRegistration::new(slug, slug.to_uppercase(), category, make_def(slug))
    }

    #[test]
    fn test_empty_registry() {
        let registry = ToolRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_and_get() {
        let mut registry = ToolRegistry::new();
        registry.register(make_reg("test", ToolCategory::Ide));

        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
        assert!(registry.contains("test"));
        assert!(registry.get("test").is_some());
        assert!(!registry.contains("unknown"));
    }

    #[test]
    fn test_list() {
        let mut registry = ToolRegistry::new();
        registry.register(make_reg("zed", ToolCategory::Ide));
        registry.register(make_reg("aider", ToolCategory::CliAgent));
        registry.register(make_reg("claude", ToolCategory::CliAgent));

        let list = registry.list();
        assert_eq!(list, vec!["aider", "claude", "zed"]); // Sorted
    }

    #[test]
    fn test_by_category() {
        let mut registry = ToolRegistry::new();
        registry.register(make_reg("vscode", ToolCategory::Ide));
        registry.register(make_reg("cursor", ToolCategory::Ide));
        registry.register(make_reg("claude", ToolCategory::CliAgent));
        registry.register(make_reg("cline", ToolCategory::Autonomous));

        let ides = registry.by_category(ToolCategory::Ide);
        assert_eq!(ides, vec!["cursor", "vscode"]); // Sorted

        let agents = registry.by_category(ToolCategory::CliAgent);
        assert_eq!(agents, vec!["claude"]);

        let copilots = registry.by_category(ToolCategory::Copilot);
        assert!(copilots.is_empty());
    }

    #[test]
    fn test_by_priority() {
        let mut registry = ToolRegistry::new();
        registry.register(make_reg("low", ToolCategory::Ide).with_priority(100));
        registry.register(make_reg("high", ToolCategory::Ide).with_priority(10));
        registry.register(make_reg("mid", ToolCategory::Ide).with_priority(50));

        let ordered = registry.by_priority();
        assert_eq!(ordered[0].slug, "high");
        assert_eq!(ordered[1].slug, "mid");
        assert_eq!(ordered[2].slug, "low");
    }

    #[test]
    fn test_iter() {
        let mut registry = ToolRegistry::new();
        registry.register(make_reg("a", ToolCategory::Ide));
        registry.register(make_reg("b", ToolCategory::Ide));

        let count = registry.iter().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_with_builtins() {
        let registry = ToolRegistry::with_builtins();

        // Should have all 13 built-in tools
        assert_eq!(registry.len(), crate::registry::BUILTIN_COUNT);

        // Spot check a few tools
        assert!(registry.contains("vscode"));
        assert!(registry.contains("claude"));
        assert!(registry.contains("copilot"));
    }
}
