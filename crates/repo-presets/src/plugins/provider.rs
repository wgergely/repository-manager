//! PluginsProvider implementation

use crate::context::Context;
use crate::error::Result;
use crate::provider::{ActionType, ApplyReport, CheckReport, PresetProvider, PresetStatus};
use async_trait::async_trait;

/// Provider for Claude Code plugins.
///
/// Handles cloning from GitHub and installing to Claude's plugin cache.
pub struct PluginsProvider {
    /// Git repository URL
    pub repo_url: String,
    /// Version tag to install
    pub version: String,
}

impl PluginsProvider {
    /// Create a new PluginsProvider with default settings.
    pub fn new() -> Self {
        Self {
            repo_url: super::paths::PLUGINS_REPO.to_string(),
            version: super::paths::DEFAULT_VERSION.to_string(),
        }
    }

    /// Create with a specific version.
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Uninstall the plugin.
    pub async fn uninstall(&self, _context: &Context) -> Result<ApplyReport> {
        let mut actions = Vec::new();

        // Disable in Claude settings first
        if let Some(settings_path) = super::paths::claude_settings_path() {
            let plugin_key = format!(
                "{}@{}",
                super::paths::PLUGIN_NAME,
                super::paths::MARKETPLACE_NAME
            );

            if super::settings::is_enabled(&settings_path, &plugin_key) {
                super::settings::disable_plugin(&settings_path, &plugin_key)?;
                actions.push("Disabled plugin in Claude settings".to_string());
            }
        }

        // Remove install directory
        if let Some(install_dir) =
            super::paths::plugin_install_dir(&self.version).filter(|d| d.exists())
        {
            std::fs::remove_dir_all(&install_dir).map_err(|e| {
                crate::error::Error::ClaudeSettings(format!(
                    "Failed to remove {}: {}",
                    install_dir.display(),
                    e
                ))
            })?;
            actions.push(format!("Removed {}", install_dir.display()));
        }

        Ok(ApplyReport::success(actions))
    }
}

impl Default for PluginsProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PresetProvider for PluginsProvider {
    fn id(&self) -> &str {
        "claude:plugins"
    }

    async fn check(&self, _context: &Context) -> Result<CheckReport> {
        // Check if plugin is installed at the expected location
        let install_dir = match super::paths::plugin_install_dir(&self.version) {
            Some(dir) => dir,
            None => {
                return Ok(CheckReport::broken("Cannot determine home directory"));
            }
        };

        // Check if plugin.json exists (indicates valid installation)
        let plugin_json = install_dir.join(".claude-plugin").join("plugin.json");

        if !plugin_json.exists() {
            return Ok(CheckReport::missing(format!(
                "Plugin {} not installed at {}",
                self.version,
                install_dir.display()
            )));
        }

        // Verify plugin.json is valid
        match std::fs::read_to_string(&plugin_json) {
            Ok(content) => {
                if serde_json::from_str::<serde_json::Value>(&content).is_err() {
                    return Ok(CheckReport::drifted("plugin.json is corrupted"));
                }
            }
            Err(e) => {
                return Ok(CheckReport::drifted(format!(
                    "Cannot read plugin.json: {}",
                    e
                )));
            }
        }

        // Check if enabled in Claude settings
        let settings_path = match super::paths::claude_settings_path() {
            Some(path) if path.exists() => path,
            _ => return Ok(CheckReport::healthy()),
        };

        let content = match std::fs::read_to_string(&settings_path) {
            Ok(c) => c,
            Err(_) => return Ok(CheckReport::healthy()),
        };

        let settings: serde_json::Value = match serde_json::from_str(&content) {
            Ok(s) => s,
            Err(_) => return Ok(CheckReport::healthy()),
        };

        let plugin_key = format!(
            "{}@{}",
            super::paths::PLUGIN_NAME,
            super::paths::MARKETPLACE_NAME
        );
        let is_disabled = settings
            .get("enabledPlugins")
            .and_then(|ep| ep.get(&plugin_key))
            .and_then(|v| v.as_bool())
            .is_some_and(|enabled| !enabled);

        if is_disabled {
            return Ok(CheckReport {
                status: PresetStatus::Drifted,
                details: vec!["Plugin is installed but disabled".to_string()],
                action: ActionType::Repair,
            });
        }

        Ok(CheckReport::healthy())
    }

    async fn apply(&self, _context: &Context) -> Result<ApplyReport> {
        let mut actions = Vec::new();

        // Determine install directory
        let install_dir = match super::paths::plugin_install_dir(&self.version) {
            Some(dir) => dir,
            None => {
                return Ok(ApplyReport::failure(vec![
                    "Cannot determine home directory".to_string(),
                ]));
            }
        };

        // Clone if not present
        if !install_dir.exists() {
            actions.push(format!(
                "Cloning plugin {} from {}",
                self.version, self.repo_url
            ));

            super::git::clone_repo(&self.repo_url, &install_dir, Some(&self.version))?;

            actions.push(format!("Installed to {}", install_dir.display()));
        } else {
            actions.push(format!("Plugin {} already installed", self.version));
        }

        // Enable in Claude settings
        if let Some(settings_path) = super::paths::claude_settings_path() {
            let plugin_key = format!(
                "{}@{}",
                super::paths::PLUGIN_NAME,
                super::paths::MARKETPLACE_NAME
            );

            if !super::settings::is_enabled(&settings_path, &plugin_key) {
                super::settings::enable_plugin(&settings_path, &plugin_key)?;
                actions.push("Enabled plugin in Claude settings".to_string());
            }
        }

        Ok(ApplyReport::success(actions))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_id() {
        let provider = PluginsProvider::new();
        assert_eq!(provider.id(), "claude:plugins");
    }

    #[test]
    fn test_provider_default() {
        let provider = PluginsProvider::default();
        assert_eq!(provider.repo_url, super::super::paths::PLUGINS_REPO);
    }

    #[test]
    fn test_with_version() {
        let provider = PluginsProvider::new().with_version("v4.0.0");
        assert_eq!(provider.version, "v4.0.0");
    }

    #[tokio::test]
    async fn test_check_not_installed() {
        use repo_fs::{LayoutMode, NormalizedPath, WorkspaceLayout};
        use std::collections::HashMap;
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        let layout = WorkspaceLayout {
            root: NormalizedPath::new(temp.path()),
            active_context: NormalizedPath::new(temp.path()),
            mode: LayoutMode::Classic,
        };
        let context = Context::new(layout, HashMap::new());

        let provider = PluginsProvider::new();
        let report = provider.check(&context).await.unwrap();

        assert_eq!(report.status, PresetStatus::Missing);
        assert_eq!(report.action, ActionType::Install);
    }

    #[tokio::test]
    async fn test_uninstall() {
        use repo_fs::{LayoutMode, NormalizedPath, WorkspaceLayout};
        use std::collections::HashMap;
        use tempfile::TempDir;

        // This is a unit test - doesn't actually install
        let temp = TempDir::new().unwrap();
        let layout = WorkspaceLayout {
            root: NormalizedPath::new(temp.path()),
            active_context: NormalizedPath::new(temp.path()),
            mode: LayoutMode::Classic,
        };
        let context = Context::new(layout, HashMap::new());

        let provider = PluginsProvider::new();
        let report = provider.uninstall(&context).await.unwrap();

        // Should succeed even if nothing to uninstall
        assert!(report.success);
    }
}
