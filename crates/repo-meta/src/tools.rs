//! Known tools registry
//!
//! Provides a registry of recognized tool names for validation.

use std::collections::HashSet;

/// Registry of known tool names for validation
#[derive(Debug, Clone)]
pub struct ToolRegistry {
    known_tools: HashSet<&'static str>,
}

impl ToolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            known_tools: HashSet::new(),
        }
    }

    /// Create a registry with built-in known tools
    ///
    /// Includes: claude, claude-desktop, cursor, vscode, windsurf, gemini-cli, antigravity
    pub fn with_builtins() -> Self {
        let known_tools = HashSet::from([
            "claude",
            "claude-desktop",
            "cursor",
            "vscode",
            "windsurf",
            "gemini-cli",
            "antigravity",
        ]);
        Self { known_tools }
    }

    /// Check if a tool name is known
    pub fn is_known(&self, name: &str) -> bool {
        self.known_tools.contains(name)
    }

    /// List all known tools, sorted alphabetically
    pub fn list_known(&self) -> Vec<&'static str> {
        let mut tools: Vec<_> = self.known_tools.iter().copied().collect();
        tools.sort();
        tools
    }

    /// Get the number of known tools
    pub fn len(&self) -> usize {
        self.known_tools.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.known_tools.is_empty()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::with_builtins()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_registry_is_empty() {
        let registry = ToolRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_with_builtins_has_known_tools() {
        let registry = ToolRegistry::with_builtins();
        assert!(!registry.is_empty());
        assert!(registry.is_known("vscode"));
        assert!(registry.is_known("claude"));
        assert!(registry.is_known("cursor"));
    }

    #[test]
    fn test_unknown_tool() {
        let registry = ToolRegistry::with_builtins();
        assert!(!registry.is_known("unknown-tool"));
        assert!(!registry.is_known("vim"));
    }

    #[test]
    fn test_list_known_is_sorted() {
        let registry = ToolRegistry::with_builtins();
        let list = registry.list_known();

        assert!(list.len() >= 7);

        // Verify sorted
        let mut sorted = list.clone();
        sorted.sort();
        assert_eq!(list, sorted);
    }

    #[test]
    fn test_default_uses_builtins() {
        let registry = ToolRegistry::default();
        assert!(registry.is_known("vscode"));
    }
}
