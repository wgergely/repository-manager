//! SuperpowersProvider implementation

use crate::context::Context;
use crate::error::Result;
use crate::provider::{ActionType, ApplyReport, CheckReport, PresetProvider, PresetStatus};
use async_trait::async_trait;

/// Provider for the superpowers Claude Code plugin.
///
/// Handles cloning from GitHub and installing to Claude's plugin cache.
pub struct SuperpowersProvider {
    /// Git repository URL
    pub repo_url: String,
    /// Version tag to install
    pub version: String,
}

impl SuperpowersProvider {
    /// Create a new SuperpowersProvider with default settings.
    pub fn new() -> Self {
        Self {
            repo_url: super::paths::SUPERPOWERS_REPO.to_string(),
            version: super::paths::DEFAULT_VERSION.to_string(),
        }
    }

    /// Create with a specific version.
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }
}

impl Default for SuperpowersProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PresetProvider for SuperpowersProvider {
    fn id(&self) -> &str {
        "claude:superpowers"
    }

    async fn check(&self, _context: &Context) -> Result<CheckReport> {
        // Check if superpowers is installed at the expected location
        let install_dir = match super::paths::superpowers_install_dir(&self.version) {
            Some(dir) => dir,
            None => {
                return Ok(CheckReport {
                    status: PresetStatus::Broken,
                    details: vec!["Cannot determine home directory".to_string()],
                    action: ActionType::None,
                });
            }
        };

        // Check if plugin.json exists (indicates valid installation)
        let plugin_json = install_dir.join(".claude-plugin").join("plugin.json");

        if !plugin_json.exists() {
            return Ok(CheckReport::missing(format!(
                "Superpowers {} not installed at {}",
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
                return Ok(CheckReport::drifted(format!("Cannot read plugin.json: {}", e)));
            }
        }

        // Check if enabled in Claude settings
        if let Some(settings_path) = super::paths::claude_settings_path() {
            if settings_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&settings_path) {
                    if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&content) {
                        let plugin_key = format!("{}@{}", super::paths::PLUGIN_NAME, super::paths::MARKETPLACE_NAME);
                        if let Some(enabled) = settings.get("enabledPlugins")
                            .and_then(|ep| ep.get(&plugin_key))
                            .and_then(|v| v.as_bool())
                        {
                            if !enabled {
                                return Ok(CheckReport {
                                    status: PresetStatus::Drifted,
                                    details: vec!["Superpowers is installed but disabled".to_string()],
                                    action: ActionType::Repair,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(CheckReport::healthy())
    }

    async fn apply(&self, _context: &Context) -> Result<ApplyReport> {
        // TODO: Implement in Task 7
        Ok(ApplyReport::failure(vec!["Not implemented".to_string()]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_id() {
        let provider = SuperpowersProvider::new();
        assert_eq!(provider.id(), "claude:superpowers");
    }

    #[test]
    fn test_provider_default() {
        let provider = SuperpowersProvider::default();
        assert_eq!(provider.repo_url, super::super::paths::SUPERPOWERS_REPO);
    }

    #[test]
    fn test_with_version() {
        let provider = SuperpowersProvider::new().with_version("v4.0.0");
        assert_eq!(provider.version, "v4.0.0");
    }

    #[tokio::test]
    async fn test_check_not_installed() {
        use tempfile::TempDir;
        use repo_fs::{NormalizedPath, WorkspaceLayout, LayoutMode};
        use std::collections::HashMap;

        let temp = TempDir::new().unwrap();
        let layout = WorkspaceLayout {
            root: NormalizedPath::new(temp.path()),
            active_context: NormalizedPath::new(temp.path()),
            mode: LayoutMode::Classic,
        };
        let context = Context::new(layout, HashMap::new());

        let provider = SuperpowersProvider::new();
        let report = provider.check(&context).await.unwrap();

        assert_eq!(report.status, PresetStatus::Missing);
        assert_eq!(report.action, ActionType::Install);
    }
}
