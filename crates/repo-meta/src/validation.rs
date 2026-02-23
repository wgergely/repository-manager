//! Validation for tool and preset names

use std::collections::HashSet;

/// Registry of known tools for validation
pub struct KnownToolSlugs {
    known: HashSet<&'static str>,
}

impl KnownToolSlugs {
    pub fn with_builtins() -> Self {
        let known = [
            "aider",
            "amazonq",
            "antigravity",
            "claude",
            "claude_desktop",
            "cline",
            "copilot",
            "cursor",
            "gemini",
            "jetbrains",
            "roo",
            "vscode",
            "windsurf",
            "zed",
        ]
        .into_iter()
        .collect();
        Self { known }
    }

    pub fn is_known(&self, name: &str) -> bool {
        self.known.contains(name)
    }

    pub fn list_known(&self) -> Vec<&'static str> {
        self.known.iter().copied().collect()
    }
}

/// Registry of known presets for validation
pub struct PresetRegistry {
    known: HashSet<&'static str>,
}

impl PresetRegistry {
    pub fn with_builtins() -> Self {
        let known = ["python", "python-uv", "python-conda", "node", "rust", "web"]
            .into_iter()
            .collect();
        Self { known }
    }

    pub fn is_known(&self, name: &str) -> bool {
        self.known.contains(name)
    }

    pub fn list_known(&self) -> Vec<&'static str> {
        self.known.iter().copied().collect()
    }
}

impl Default for KnownToolSlugs {
    fn default() -> Self {
        Self::with_builtins()
    }
}

impl Default for PresetRegistry {
    fn default() -> Self {
        Self::with_builtins()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry_knows_builtins() {
        let registry = KnownToolSlugs::with_builtins();
        assert!(registry.is_known("cursor"));
        assert!(registry.is_known("vscode"));
        assert!(registry.is_known("claude"));
        assert!(!registry.is_known("unknown-tool"));
    }

    #[test]
    fn test_preset_registry_knows_builtins() {
        let registry = PresetRegistry::with_builtins();
        assert!(registry.is_known("python"));
        assert!(registry.is_known("rust"));
        assert!(!registry.is_known("unknown-preset"));
    }
}
