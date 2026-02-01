//! Path constants for superpowers installation

/// Default git repository URL
pub const SUPERPOWERS_REPO: &str = "https://github.com/obra/superpowers";

/// Default version tag to install
pub const DEFAULT_VERSION: &str = "v4.1.1";

/// Marketplace name for tracking
pub const MARKETPLACE_NAME: &str = "git";

/// Plugin name
pub const PLUGIN_NAME: &str = "superpowers";

/// Get the Claude plugins cache directory
/// Returns: ~/.claude/plugins/cache/
pub fn claude_plugins_cache() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("plugins").join("cache"))
}

/// Get the superpowers install directory
/// Returns: ~/.claude/plugins/cache/git/superpowers/{version}/
pub fn superpowers_install_dir(version: &str) -> Option<std::path::PathBuf> {
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
    dirs::home_dir().map(|h| h.join(".claude").join("plugins").join("installed_plugins.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_superpowers_install_dir() {
        let path = superpowers_install_dir("v4.1.1");
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains("superpowers"));
        assert!(path.to_string_lossy().contains("v4.1.1"));
    }

    #[test]
    fn test_claude_settings_path() {
        let path = claude_settings_path();
        assert!(path.is_some());
        assert!(path.unwrap().to_string_lossy().contains(".claude"));
    }
}
