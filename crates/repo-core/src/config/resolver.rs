//! Configuration resolution with hierarchical merge
//!
//! The `ConfigResolver` loads and merges configuration from multiple sources
//! in a defined hierarchy, with later sources overriding earlier ones.

use crate::Result;
use repo_fs::NormalizedPath;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

use super::manifest::Manifest;

/// The final resolved configuration after merging all sources
///
/// This is the output of the configuration resolution process and
/// represents the effective configuration for a repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedConfig {
    /// Repository mode: "standard" or "worktree"
    pub mode: String,

    /// Merged preset configurations
    pub presets: HashMap<String, Value>,

    /// Combined list of tools (unique)
    pub tools: Vec<String>,

    /// Combined list of rules (unique)
    pub rules: Vec<String>,
}

impl Default for ResolvedConfig {
    fn default() -> Self {
        Self {
            mode: "standard".to_string(),
            presets: HashMap::new(),
            tools: Vec::new(),
            rules: Vec::new(),
        }
    }
}

impl From<Manifest> for ResolvedConfig {
    fn from(manifest: Manifest) -> Self {
        Self {
            mode: manifest.core.mode,
            presets: manifest.presets,
            tools: manifest.tools,
            rules: manifest.rules,
        }
    }
}

/// Resolves configuration by merging multiple sources
///
/// Configuration is loaded from a hierarchy of sources:
/// 1. Global defaults (~/.config/repo-manager/config.toml) - TODO
/// 2. Organization config - TODO
/// 3. Repository config (.repository/config.toml)
/// 4. Local overrides (.repository/config.local.toml) - git-ignored
///
/// Later sources override earlier ones, with deep merging for preset objects.
pub struct ConfigResolver {
    /// Repository root directory
    root: NormalizedPath,
}

impl ConfigResolver {
    /// Create a new configuration resolver for the given repository root
    ///
    /// # Arguments
    ///
    /// * `root` - The repository root directory containing `.repository/`
    pub fn new(root: NormalizedPath) -> Self {
        Self { root }
    }

    /// Resolve the configuration by merging all sources
    ///
    /// # Returns
    ///
    /// The final merged configuration, or an error if parsing fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// use repo_core::config::ConfigResolver;
    /// use repo_fs::NormalizedPath;
    ///
    /// let resolver = ConfigResolver::new(NormalizedPath::new("/path/to/repo"));
    /// let config = resolver.resolve()?;
    /// println!("Mode: {}", config.mode);
    /// ```
    pub fn resolve(&self) -> Result<ResolvedConfig> {
        let mut manifest = Manifest::empty();

        // TODO: Layer 1 - Global defaults (~/.config/repo-manager/config.toml)
        // This would load user-global settings

        // TODO: Layer 2 - Organization config
        // This would load organization-level settings

        // Layer 3 - Repository config (.repository/config.toml)
        let repo_config_path = self.root.join(".repository/config.toml");
        if repo_config_path.is_file() {
            let content = fs::read_to_string(repo_config_path.to_native())?;
            let repo_manifest = Manifest::parse(&content)?;
            manifest.merge(&repo_manifest);
        }

        // Layer 4 - Local overrides (.repository/config.local.toml)
        let local_config_path = self.root.join(".repository/config.local.toml");
        if local_config_path.is_file() {
            let content = fs::read_to_string(local_config_path.to_native())?;
            let local_manifest = Manifest::parse(&content)?;
            manifest.merge(&local_manifest);
        }

        Ok(ResolvedConfig::from(manifest))
    }

    /// Get the repository root path
    pub fn root(&self) -> &NormalizedPath {
        &self.root
    }

    /// Check if a repository configuration exists
    pub fn has_config(&self) -> bool {
        self.root.join(".repository/config.toml").is_file()
    }

    /// Check if local overrides exist
    pub fn has_local_overrides(&self) -> bool {
        self.root.join(".repository/config.local.toml").is_file()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn resolve_returns_defaults_when_no_config_exists() {
        let temp_dir = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp_dir.path());
        let resolver = ConfigResolver::new(root);

        assert!(!resolver.has_config());
        assert!(!resolver.has_local_overrides());

        let config = resolver.resolve().unwrap();
        // With no config files, resolve should return defaults
        assert!(config.tools.is_empty());
        assert!(config.rules.is_empty());
        assert!(config.presets.is_empty());
    }

    #[test]
    fn resolve_loads_repo_config_toml() {
        let temp_dir = TempDir::new().unwrap();
        let repo_dir = temp_dir.path().join(".repository");
        std::fs::create_dir_all(&repo_dir).unwrap();

        let config_content = r#"
tools = ["cursor", "vscode"]
rules = ["no-unsafe"]

[core]
mode = "standard"

[presets."env:python"]
version = "3.12"
"#;
        std::fs::write(repo_dir.join("config.toml"), config_content).unwrap();

        let root = NormalizedPath::new(temp_dir.path());
        let resolver = ConfigResolver::new(root);

        assert!(resolver.has_config());

        let config = resolver.resolve().unwrap();
        assert_eq!(config.mode, "standard");
        assert_eq!(config.tools, vec!["cursor", "vscode"]);
        assert_eq!(config.rules, vec!["no-unsafe"]);
        assert_eq!(config.presets["env:python"]["version"], "3.12");
    }

    #[test]
    fn resolve_merges_local_overrides_on_top_of_repo_config() {
        let temp_dir = TempDir::new().unwrap();
        let repo_dir = temp_dir.path().join(".repository");
        std::fs::create_dir_all(&repo_dir).unwrap();

        // Base config
        let base_content = r#"
tools = ["cursor"]

[core]
mode = "standard"

[presets."env:python"]
version = "3.11"
debug = false
"#;
        std::fs::write(repo_dir.join("config.toml"), base_content).unwrap();

        // Local overrides: override python version, add a tool
        let local_content = r#"
tools = ["vscode"]

[presets."env:python"]
version = "3.12"
"#;
        std::fs::write(repo_dir.join("config.local.toml"), local_content).unwrap();

        let root = NormalizedPath::new(temp_dir.path());
        let resolver = ConfigResolver::new(root);

        assert!(resolver.has_config());
        assert!(resolver.has_local_overrides());

        let config = resolver.resolve().unwrap();

        // Local override should override python version
        assert_eq!(config.presets["env:python"]["version"], "3.12");
        // But base-only fields should be preserved (deep merge)
        assert_eq!(config.presets["env:python"]["debug"], false);
        // Tools should be merged (both cursor and vscode)
        assert!(config.tools.contains(&"cursor".to_string()));
        assert!(config.tools.contains(&"vscode".to_string()));
    }
}
