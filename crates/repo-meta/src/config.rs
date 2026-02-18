//! Configuration types and loading for Repository Manager
//!
//! This module provides types for loading and working with
//! the `.repository/config.toml` configuration file.

use crate::error::{Error, Result};
use repo_fs::{NormalizedPath, RepoPath};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maximum configuration file size (1 MB should be plenty)
const MAX_CONFIG_SIZE: u64 = 1024 * 1024;

/// Repository operation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RepositoryMode {
    /// Standard single-worktree repository
    #[default]
    Standard,
    /// Container-style with multiple worktrees
    Worktrees,
}

/// Core repository configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoreConfig {
    /// Configuration schema version
    #[serde(default = "default_version")]
    pub version: String,
    /// Repository operation mode
    #[serde(default)]
    pub mode: RepositoryMode,
}

fn default_version() -> String {
    "1".to_string()
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            mode: RepositoryMode::default(),
        }
    }
}

/// Active tools and presets configuration
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ActiveConfig {
    /// List of active tool IDs
    #[serde(default)]
    pub tools: Vec<String>,
    /// List of active preset IDs
    #[serde(default)]
    pub presets: Vec<String>,
}

/// Synchronization configuration
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Sync strategy (e.g., "auto", "manual", "on-commit")
    #[serde(default = "default_strategy")]
    pub strategy: String,
}

fn default_strategy() -> String {
    "auto".to_string()
}

/// Complete repository configuration
///
/// Loaded from `.repository/config.toml`
///
/// **Deprecated**: Use `repo_core::Manifest` instead. This struct expects
/// `[active] tools = [...]` format while `Manifest` uses top-level `tools = [...]`.
/// New code should use `Manifest::parse()`. This will be removed in a future release.
#[deprecated(note = "Use repo_core::Manifest instead")]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RepositoryConfig {
    /// Core settings
    #[serde(default)]
    pub core: CoreConfig,
    /// Active tools and presets
    #[serde(default)]
    pub active: ActiveConfig,
    /// Synchronization settings
    #[serde(default)]
    pub sync: SyncConfig,
    /// Per-preset configuration sections
    /// Maps preset ID to its configuration table
    #[serde(flatten)]
    pub presets_config: HashMap<String, toml::Value>,
}

/// Load repository configuration from the standard location.
///
/// Looks for `.repository/config.toml` relative to the given root path.
///
/// **Deprecated**: Use `repo_core::Manifest::parse()` instead. Read the file
/// contents and call `Manifest::parse(&content)` directly.
///
/// # Arguments
///
/// * `root` - The repository root path
///
/// # Returns
///
/// The loaded configuration or an error if not found or invalid.
#[deprecated(note = "Use repo_core::Manifest::parse() instead")]
#[allow(deprecated)]
pub fn load_config(root: &NormalizedPath) -> Result<RepositoryConfig> {
    let config_path = root
        .join(RepoPath::RepositoryConfig.as_str())
        .join("config.toml");

    if !config_path.exists() {
        return Err(Error::ConfigNotFound {
            path: config_path.to_native(),
        });
    }

    // Check file size before reading to prevent OOM attacks
    let metadata = std::fs::metadata(config_path.to_native())
        .map_err(|e| Error::Fs(repo_fs::Error::io(config_path.to_native(), e)))?;

    if metadata.len() > MAX_CONFIG_SIZE {
        return Err(Error::ConfigTooLarge {
            path: config_path.to_native(),
            size: metadata.len(),
            max: MAX_CONFIG_SIZE,
        });
    }

    let content = std::fs::read_to_string(config_path.to_native())
        .map_err(|e| Error::Fs(repo_fs::Error::io(config_path.to_native(), e)))?;

    let config: RepositoryConfig = toml::from_str(&content).map_err(|e| Error::InvalidConfig {
        path: config_path.to_native(),
        message: e.to_string(),
    })?;

    Ok(config)
}

/// Get preset-specific configuration from the repository config.
///
/// Preset configurations are stored as top-level tables in config.toml
/// with the preset ID as the key.
///
/// # Arguments
///
/// * `config` - The repository configuration
/// * `preset_id` - The preset identifier (e.g., "env:python")
///
/// # Returns
///
/// The preset configuration if found, or None.
///
/// # Example
///
/// ```ignore
/// let config = load_config(&root)?;
/// if let Some(python_config) = get_preset_config(&config, "env:python") {
///     println!("Python config: {:?}", python_config);
/// }
/// ```
#[allow(deprecated)]
pub fn get_preset_config<'a>(
    config: &'a RepositoryConfig,
    preset_id: &str,
) -> Option<&'a toml::Value> {
    config.presets_config.get(preset_id)
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_mode_default() {
        assert_eq!(RepositoryMode::default(), RepositoryMode::Standard);
    }

    #[test]
    fn test_core_config_default() {
        let config = CoreConfig::default();
        assert_eq!(config.version, "1");
        assert_eq!(config.mode, RepositoryMode::Standard);
    }

    #[test]
    fn test_parse_minimal_config() {
        let toml_str = r#"
[core]
version = "1"
"#;
        let config: RepositoryConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.core.version, "1");
        assert_eq!(config.core.mode, RepositoryMode::Standard);
    }

    #[test]
    fn test_parse_worktrees_mode() {
        let toml_str = r#"
[core]
version = "1"
mode = "worktrees"
"#;
        let config: RepositoryConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.core.mode, RepositoryMode::Worktrees);
    }

    #[test]
    fn test_parse_config_with_presets() {
        let toml_str = r#"
[core]
version = "1"

[active]
tools = ["vscode", "cursor"]
presets = ["env:python", "env:node"]

["env:python"]
version = "3.11"

["env:node"]
version = "20"
"#;
        let config: RepositoryConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.active.tools, vec!["vscode", "cursor"]);
        assert_eq!(config.active.presets, vec!["env:python", "env:node"]);

        let python_config = get_preset_config(&config, "env:python").unwrap();
        assert_eq!(python_config.get("version").unwrap().as_str(), Some("3.11"));
    }

    #[test]
    fn test_load_rejects_oversized_config() {
        use std::io::Write;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());

        // Create .repository directory
        let config_dir = dir.path().join(".repository");
        std::fs::create_dir_all(&config_dir).unwrap();

        // Create oversized config file (2 MB, above 1 MB limit)
        let config_path = config_dir.join("config.toml");
        let mut file = std::fs::File::create(&config_path).unwrap();
        for _ in 0..2 {
            file.write_all(&[b'a'; 1024 * 1024]).unwrap();
        }

        // Should reject oversized file
        let result = load_config(&root);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_string = err.to_string();
        assert!(
            err_string.contains("too large") || err_string.contains("size"),
            "Error message should mention size: {}",
            err_string
        );
    }
}
