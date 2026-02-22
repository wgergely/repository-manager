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
use std::path::PathBuf;

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

    /// Merged extension configurations
    pub extensions: HashMap<String, Value>,
}

impl Default for ResolvedConfig {
    fn default() -> Self {
        Self {
            mode: "standard".to_string(),
            presets: HashMap::new(),
            tools: Vec::new(),
            rules: Vec::new(),
            extensions: HashMap::new(),
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
            extensions: manifest.extensions,
        }
    }
}

/// Resolves configuration by merging multiple sources
///
/// Configuration is loaded from a hierarchy of sources:
/// 1. Global defaults (~/.config/repo-manager/config.toml)
/// 2. Organization config (~/.config/repo-manager/org/config.toml)
/// 3. Repository config (.repository/config.toml)
/// 4. Local overrides (.repository/config.local.toml) - git-ignored
///
/// Later sources override earlier ones, with deep merging for preset objects.
pub struct ConfigResolver {
    /// Repository root directory
    root: NormalizedPath,

    /// Override for the global config directory (used for testing).
    /// When `None`, the platform-appropriate directory is used via `dirs::config_dir()`.
    global_config_dir_override: Option<PathBuf>,
}

impl ConfigResolver {
    /// Create a new configuration resolver for the given repository root
    ///
    /// Uses the platform-appropriate global config directory:
    /// - Linux: `~/.config/repo-manager/`
    /// - macOS: `~/Library/Application Support/repo-manager/`
    /// - Windows: `%APPDATA%\repo-manager\`
    ///
    /// # Arguments
    ///
    /// * `root` - The repository root directory containing `.repository/`
    pub fn new(root: NormalizedPath) -> Self {
        Self {
            root,
            global_config_dir_override: None,
        }
    }

    /// Create a resolver with a custom global config directory.
    ///
    /// This is primarily useful for testing, where you need to control
    /// the global config path without affecting the real user config.
    ///
    /// # Arguments
    ///
    /// * `root` - The repository root directory containing `.repository/`
    /// * `global_config_dir` - Custom path to use as the global config directory
    pub fn with_global_config_dir(root: NormalizedPath, global_config_dir: PathBuf) -> Self {
        Self {
            root,
            global_config_dir_override: Some(global_config_dir),
        }
    }

    /// Determine the global config directory path.
    ///
    /// Returns the override if set, otherwise falls back to the
    /// platform-appropriate config directory via `dirs::config_dir()`.
    fn global_config_dir(&self) -> Option<PathBuf> {
        if let Some(ref override_dir) = self.global_config_dir_override {
            return Some(override_dir.clone());
        }
        dirs::config_dir().map(|d| d.join("repo-manager"))
    }

    /// Resolve the configuration by merging all sources
    ///
    /// Loads and merges configuration from 4 layers in order:
    /// 1. Global defaults (`<config_dir>/repo-manager/config.toml`)
    /// 2. Organization config (`<config_dir>/repo-manager/org/config.toml`)
    /// 3. Repository config (`.repository/config.toml`)
    /// 4. Local overrides (`.repository/config.local.toml`)
    ///
    /// Each layer overrides values from the previous layer. Missing layers
    /// are silently skipped. Invalid TOML in any layer produces an error.
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

        // Layer 1 - Global defaults (~/.config/repo-manager/config.toml)
        if let Some(global_dir) = self.global_config_dir() {
            let global_config_path = global_dir.join("config.toml");
            if global_config_path.is_file() {
                tracing::debug!(?global_config_path, "Loading global config (layer 1)");
                let content = fs::read_to_string(&global_config_path)?;
                let global_manifest = Manifest::parse(&content)?;
                manifest.merge(&global_manifest);
            } else {
                tracing::debug!(
                    ?global_config_path,
                    "No global config found (layer 1) — skipping"
                );
            }
        }

        // Layer 2 - Organization config (~/.config/repo-manager/org/config.toml)
        //
        // The org config provides a second layer of shared defaults that sit between
        // user-global settings and per-repository config. This is useful for teams
        // that want to share tool/preset/rule defaults across repositories.
        //
        // The org config lives in a subdirectory of the global config dir:
        //   <config_dir>/repo-manager/org/config.toml
        //
        // Future enhancements could support multiple named orgs:
        //   <config_dir>/repo-manager/org/<org-name>/config.toml
        // with the org name derived from the git remote URL.
        if let Some(global_dir) = self.global_config_dir() {
            let org_config_path = global_dir.join("org").join("config.toml");
            if org_config_path.is_file() {
                tracing::debug!(?org_config_path, "Loading org config (layer 2)");
                let content = fs::read_to_string(&org_config_path)?;
                let org_manifest = Manifest::parse(&content)?;
                manifest.merge(&org_manifest);
            } else {
                tracing::debug!(?org_config_path, "No org config found (layer 2) — skipping");
            }
        }

        // Layer 3 - Repository config (.repository/config.toml)
        let repo_config_path = self.root.join(".repository/config.toml");
        if repo_config_path.is_file() {
            tracing::debug!(?repo_config_path, "Loading repo config (layer 3)");
            let content = fs::read_to_string(repo_config_path.to_native())?;
            let repo_manifest = Manifest::parse(&content)?;
            manifest.merge(&repo_manifest);
        }

        // Layer 4 - Local overrides (.repository/config.local.toml)
        let local_config_path = self.root.join(".repository/config.local.toml");
        if local_config_path.is_file() {
            tracing::debug!(?local_config_path, "Loading local config (layer 4)");
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
