//! Runtime context generation for agents
//!
//! The `RuntimeContext` transforms resolved configuration into a format
//! suitable for consumption by AI agents and external tools.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use super::resolver::ResolvedConfig;

/// Runtime context for agents and external tools
///
/// This struct transforms the resolved configuration into a format
/// optimized for consumption by AI agents:
///
/// - `runtime`: Environment presets (e.g., "env:python") become runtime information
/// - `capabilities`: Tool and config presets become capability declarations
///
/// # Example
///
/// Given presets:
/// ```toml
/// [presets."env:python"]
/// version = "3.12"
///
/// [presets."tool:linter"]
/// enabled = true
/// ```
///
/// The runtime context would be:
/// ```json
/// {
///   "runtime": {
///     "python": { "version": "3.12" }
///   },
///   "capabilities": ["tool:linter"]
/// }
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuntimeContext {
    /// Environment runtime information
    ///
    /// Keys are the environment names (e.g., "python", "node")
    /// Values contain the full preset configuration
    pub runtime: HashMap<String, Value>,

    /// Declared capabilities
    ///
    /// Tool and config presets are listed as capabilities
    pub capabilities: Vec<String>,
}

impl RuntimeContext {
    /// Create a runtime context from resolved configuration
    ///
    /// This transforms presets based on their type prefix:
    /// - `env:*` presets become entries in `runtime` (keyed by the name after `:`)
    /// - `tool:*` and `config:*` presets become entries in `capabilities`
    ///
    /// # Arguments
    ///
    /// * `config` - The resolved configuration to transform
    ///
    /// # Example
    ///
    /// ```
    /// use repo_core::config::{ResolvedConfig, RuntimeContext};
    /// use std::collections::HashMap;
    /// use serde_json::json;
    ///
    /// let mut presets = HashMap::new();
    /// presets.insert("env:python".to_string(), json!({"version": "3.12"}));
    /// presets.insert("tool:linter".to_string(), json!({"enabled": true}));
    ///
    /// let config = ResolvedConfig {
    ///     mode: "standard".to_string(),
    ///     presets,
    ///     tools: vec![],
    ///     rules: vec![],
    /// };
    ///
    /// let context = RuntimeContext::from_resolved(&config);
    /// assert!(context.runtime.contains_key("python"));
    /// assert!(context.capabilities.contains(&"tool:linter".to_string()));
    /// ```
    pub fn from_resolved(config: &ResolvedConfig) -> Self {
        let mut runtime = HashMap::new();
        let mut capabilities = Vec::new();

        for (key, value) in &config.presets {
            if let Some(name) = key.strip_prefix("env:") {
                // Environment preset -> runtime info
                runtime.insert(name.to_string(), value.clone());
            } else if key.starts_with("tool:") || key.starts_with("config:") {
                // Tool/config preset -> capability
                capabilities.push(key.clone());
            }
        }

        // Sort capabilities for deterministic output
        capabilities.sort();

        Self {
            runtime,
            capabilities,
        }
    }

    /// Convert the runtime context to a JSON value
    ///
    /// This is useful for serialization to agents or external tools.
    ///
    /// # Returns
    ///
    /// A JSON object with `runtime` and `capabilities` fields
    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or_else(|_| {
            serde_json::json!({
                "runtime": {},
                "capabilities": []
            })
        })
    }

    /// Check if the context has any runtime environments
    pub fn has_runtime(&self) -> bool {
        !self.runtime.is_empty()
    }

    /// Check if the context has any capabilities
    pub fn has_capabilities(&self) -> bool {
        !self.capabilities.is_empty()
    }

    /// Get a specific runtime environment by name
    pub fn get_runtime(&self, name: &str) -> Option<&Value> {
        self.runtime.get(name)
    }

    /// Check if a specific capability is declared
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.contains(&capability.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_context_default() {
        let context = RuntimeContext::default();
        assert!(context.runtime.is_empty());
        assert!(context.capabilities.is_empty());
    }

    #[test]
    fn test_runtime_context_has_methods() {
        let mut context = RuntimeContext::default();
        assert!(!context.has_runtime());
        assert!(!context.has_capabilities());

        context.runtime.insert(
            "python".to_string(),
            serde_json::json!({"version": "3.12"}),
        );
        context.capabilities.push("tool:linter".to_string());

        assert!(context.has_runtime());
        assert!(context.has_capabilities());
        assert!(context.has_capability("tool:linter"));
        assert!(!context.has_capability("tool:formatter"));
    }
}
