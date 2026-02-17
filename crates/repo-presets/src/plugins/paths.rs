//! Path constants for plugin installation

/// Default git repository URL
pub const PLUGINS_REPO: &str = "https://github.com/anthropics/claude-code-plugins";

/// Default version tag to install
pub const DEFAULT_VERSION: &str = "v4.1.1";

/// Marketplace name for tracking
pub const MARKETPLACE_NAME: &str = "git";

/// Plugin name
pub const PLUGIN_NAME: &str = "plugins";

/// Get the Claude plugins cache directory
/// Returns: ~/.claude/plugins/cache/
pub fn claude_plugins_cache() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("plugins").join("cache"))
}

/// Get the plugin install directory
/// Returns: ~/.claude/plugins/cache/git/plugins/{version}/
pub fn plugin_install_dir(version: &str) -> Option<std::path::PathBuf> {
    claude_plugins_cache().map(|c| c.join(MARKETPLACE_NAME).join(PLUGIN_NAME).join(version))
}

/// Get Claude's settings.json path
/// Returns: ~/.claude/settings.json
pub fn claude_settings_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("settings.json"))
}

/// Get Claude's installed_plugins.json path
/// Returns: ~/.claude/plugins/installed_plugins.json
#[allow(dead_code)] // Reserved for future use (plugin tracking)
pub fn installed_plugins_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| {
        h.join(".claude")
            .join("plugins")
            .join("installed_plugins.json")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_install_dir() {
        let path = plugin_install_dir("v4.1.1");
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains("plugins"));
        assert!(path.to_string_lossy().contains("v4.1.1"));
    }

    #[test]
    fn test_claude_settings_path() {
        let path = claude_settings_path();
        assert!(path.is_some());
        assert!(path.unwrap().to_string_lossy().contains(".claude"));
    }
}
