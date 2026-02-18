//! Manifest parsing for config.toml files
//!
//! The manifest represents the parsed configuration from a single config.toml file.
//! Multiple manifests can be merged together to create a resolved configuration.

use crate::Result;
use crate::hooks::HookConfig;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

fn default_mode() -> String {
    "worktrees".to_string()
}

/// Core configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreSection {
    /// Repository mode: "standard" or "worktree"
    #[serde(default = "default_mode")]
    pub mode: String,
}

impl Default for CoreSection {
    fn default() -> Self {
        Self {
            mode: default_mode(),
        }
    }
}

/// Repository configuration manifest parsed from config.toml
///
/// This struct represents a single configuration file. Multiple manifests
/// from different sources (global, org, repo, local) are merged together
/// to create the final resolved configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Manifest {
    /// Core settings
    #[serde(default)]
    pub core: CoreSection,

    /// Preset configurations keyed by type and name
    ///
    /// Keys follow the pattern "type:name", e.g.:
    /// - "env:python" - Python environment configuration
    /// - "tool:linter" - Linter tool configuration
    /// - "config:editor" - Editor configuration
    #[serde(default)]
    pub presets: HashMap<String, Value>,

    /// List of tools to configure
    #[serde(default)]
    pub tools: Vec<String>,

    /// List of rules to apply
    #[serde(default)]
    pub rules: Vec<String>,

    /// Lifecycle hooks
    #[serde(default)]
    pub hooks: Vec<HookConfig>,
}

impl Manifest {
    /// Parse a manifest from TOML content
    ///
    /// # Arguments
    ///
    /// * `content` - TOML string to parse
    ///
    /// # Returns
    ///
    /// The parsed manifest, or an error if parsing fails
    ///
    /// # Example
    ///
    /// ```
    /// use repo_core::config::Manifest;
    ///
    /// let manifest = Manifest::parse(r#"
    /// [core]
    /// mode = "standard"
    ///
    /// [presets."env:python"]
    /// version = "3.12"
    ///
    /// tools = ["cargo", "python"]
    /// "#).unwrap();
    ///
    /// assert_eq!(manifest.core.mode, "standard");
    /// ```
    pub fn parse(content: &str) -> Result<Self> {
        let manifest: Manifest = toml::from_str(content)?;
        Ok(manifest)
    }

    /// Create an empty manifest with default values
    ///
    /// This is equivalent to parsing an empty TOML file.
    pub fn empty() -> Self {
        Self {
            core: CoreSection {
                mode: default_mode(),
            },
            presets: HashMap::new(),
            tools: Vec::new(),
            rules: Vec::new(),
            hooks: Vec::new(),
        }
    }

    /// Serialize this manifest to a clean TOML string
    ///
    /// Produces a readable TOML representation with tools and rules before
    /// the `[core]` section (required for correct parsing as top-level keys).
    pub fn to_toml(&self) -> String {
        let mut content = String::new();

        // tools array - must be BEFORE [core] section to be top-level
        if !self.tools.is_empty() {
            content.push_str("tools = [");
            let tools_str: Vec<String> = self.tools.iter().map(|t| format!("\"{}\"", t)).collect();
            content.push_str(&tools_str.join(", "));
            content.push_str("]\n");
        }

        // rules array - must be BEFORE [core] section to be top-level
        if !self.rules.is_empty() {
            content.push_str("rules = [");
            let rules_str: Vec<String> = self.rules.iter().map(|r| format!("\"{}\"", r)).collect();
            content.push_str(&rules_str.join(", "));
            content.push_str("]\n");
        }

        // Add blank line before [core] if we had top-level keys
        if !self.tools.is_empty() || !self.rules.is_empty() {
            content.push('\n');
        }

        // [core] section
        content.push_str("[core]\n");
        content.push_str(&format!("mode = \"{}\"\n", self.core.mode));

        // [presets] section
        if !self.presets.is_empty() {
            content.push('\n');
            content.push_str("[presets]\n");
            for (name, value) in &self.presets {
                if value.is_object()
                    && value.as_object().is_some_and(|o| o.is_empty())
                {
                    content.push_str(&format!("\"{}\" = {{}}\n", name));
                } else {
                    let toml_value = json_to_toml_value(value);
                    content.push_str(&format!("\"{}\" = {}\n", name, toml_value));
                }
            }
        }

        // [[hooks]] array of tables
        for hook in &self.hooks {
            content.push_str("\n[[hooks]]\n");
            content.push_str(&format!("event = \"{}\"\n", hook.event));
            content.push_str(&format!("command = \"{}\"\n", hook.command));
            if !hook.args.is_empty() {
                let args_str: Vec<String> =
                    hook.args.iter().map(|a| format!("\"{}\"", a)).collect();
                content.push_str(&format!("args = [{}]\n", args_str.join(", ")));
            }
            if let Some(ref dir) = hook.working_dir {
                content.push_str(&format!("working_dir = \"{}\"\n", dir.display()));
            }
        }

        content
    }

    /// Merge another manifest into this one
    ///
    /// The `other` manifest takes precedence for scalar values.
    /// For collections:
    /// - `presets`: Deep merge - overlay values override, but base-only values preserved
    /// - `tools`: Extend with unique values from other
    /// - `rules`: Extend with unique values from other
    ///
    /// # Arguments
    ///
    /// * `other` - The manifest to merge into this one (takes precedence)
    pub fn merge(&mut self, other: &Manifest) {
        // Core mode: other takes precedence if not default
        if other.core.mode != default_mode() {
            self.core.mode = other.core.mode.clone();
        }

        // Presets: deep merge
        for (key, other_value) in &other.presets {
            if let Some(base_value) = self.presets.get_mut(key) {
                // Deep merge the preset objects
                deep_merge_value(base_value, other_value);
            } else {
                // New preset from other
                self.presets.insert(key.clone(), other_value.clone());
            }
        }

        // Tools: extend with unique values
        for tool in &other.tools {
            if !self.tools.contains(tool) {
                self.tools.push(tool.clone());
            }
        }

        // Rules: extend with unique values
        for rule in &other.rules {
            if !self.rules.contains(rule) {
                self.rules.push(rule.clone());
            }
        }

        // Hooks: extend (append all from other)
        self.hooks.extend(other.hooks.iter().cloned());
    }
}

/// Convert a JSON value to a TOML-compatible string representation
pub fn json_to_toml_value(value: &Value) -> String {
    match value {
        Value::Null => "\"\"".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(json_to_toml_value).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Object(obj) => {
            if obj.is_empty() {
                "{}".to_string()
            } else {
                let pairs: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| format!("{} = {}", k, json_to_toml_value(v)))
                    .collect();
                format!("{{ {} }}", pairs.join(", "))
            }
        }
    }
}

/// Deep merge two JSON values
///
/// If both values are objects, merge them recursively with `other` taking precedence.
/// Otherwise, `other` replaces `base`.
fn deep_merge_value(base: &mut Value, other: &Value) {
    match (base, other) {
        (Value::Object(base_map), Value::Object(other_map)) => {
            for (key, other_val) in other_map {
                if let Some(base_val) = base_map.get_mut(key) {
                    deep_merge_value(base_val, other_val);
                } else {
                    base_map.insert(key.clone(), other_val.clone());
                }
            }
        }
        (base, other) => {
            *base = other.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mode() {
        assert_eq!(default_mode(), "worktrees");
    }

    #[test]
    fn test_empty_manifest() {
        let manifest = Manifest::empty();
        assert_eq!(manifest.core.mode, "worktrees");
    }

    #[test]
    fn test_parse_tools_and_rules() {
        // Note: tools and rules must be BEFORE [core] section to be top-level
        let toml_content = r#"
tools = ["cargo", "rustfmt"]
rules = ["no-unsafe", "no-unwrap"]

[core]
mode = "worktree"
"#;
        let manifest: Manifest = toml::from_str(toml_content).unwrap();
        assert_eq!(manifest.core.mode, "worktree");
        assert_eq!(manifest.tools, vec!["cargo", "rustfmt"]);
        assert_eq!(manifest.rules, vec!["no-unsafe", "no-unwrap"]);
    }

    #[test]
    fn test_deep_merge_objects() {
        let mut base = serde_json::json!({
            "a": 1,
            "b": { "x": 10, "y": 20 }
        });
        let other = serde_json::json!({
            "b": { "y": 25, "z": 30 },
            "c": 3
        });

        deep_merge_value(&mut base, &other);

        assert_eq!(base["a"], 1);
        assert_eq!(base["b"]["x"], 10);
        assert_eq!(base["b"]["y"], 25);
        assert_eq!(base["b"]["z"], 30);
        assert_eq!(base["c"], 3);
    }
}
