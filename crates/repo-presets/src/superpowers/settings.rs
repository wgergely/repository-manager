//! Claude settings.json manipulation

use crate::error::{Error, Result};
use serde_json::{Value, json};
use std::path::Path;

/// Enable superpowers in Claude's settings.json
pub fn enable_superpowers(settings_path: &Path, plugin_key: &str) -> Result<()> {
    let mut settings = if settings_path.exists() {
        let content = std::fs::read_to_string(settings_path)
            .map_err(|e| Error::ClaudeSettings(format!("Failed to read: {}", e)))?;
        serde_json::from_str(&content)
            .map_err(|e| Error::ClaudeSettings(format!("Invalid JSON: {}", e)))?
    } else {
        json!({})
    };

    // Ensure enabledPlugins exists
    if settings.get("enabledPlugins").is_none() {
        settings["enabledPlugins"] = json!({});
    }

    // Enable the plugin
    settings["enabledPlugins"][plugin_key] = json!(true);

    // Create parent directory if needed
    if let Some(parent) = settings_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::ClaudeSettings(format!("Failed to create directory: {}", e)))?;
    }

    // Write back
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| Error::ClaudeSettings(format!("Failed to serialize: {}", e)))?;
    std::fs::write(settings_path, content)
        .map_err(|e| Error::ClaudeSettings(format!("Failed to write: {}", e)))?;

    Ok(())
}

/// Disable superpowers in Claude's settings.json
pub fn disable_superpowers(settings_path: &Path, plugin_key: &str) -> Result<()> {
    if !settings_path.exists() {
        return Ok(()); // Nothing to disable
    }

    let content = std::fs::read_to_string(settings_path)
        .map_err(|e| Error::ClaudeSettings(format!("Failed to read: {}", e)))?;
    let mut settings: Value = serde_json::from_str(&content)
        .map_err(|e| Error::ClaudeSettings(format!("Invalid JSON: {}", e)))?;

    // Remove plugin key from enabledPlugins
    if let Some(obj) = settings
        .get_mut("enabledPlugins")
        .and_then(|ep| ep.as_object_mut())
    {
        obj.remove(plugin_key);
    }

    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| Error::ClaudeSettings(format!("Failed to serialize: {}", e)))?;
    std::fs::write(settings_path, content)
        .map_err(|e| Error::ClaudeSettings(format!("Failed to write: {}", e)))?;

    Ok(())
}

/// Check if superpowers is enabled in settings
pub fn is_enabled(settings_path: &Path, plugin_key: &str) -> bool {
    if !settings_path.exists() {
        return false;
    }

    std::fs::read_to_string(settings_path)
        .ok()
        .and_then(|content| serde_json::from_str::<Value>(&content).ok())
        .and_then(|settings| settings.get("enabledPlugins")?.get(plugin_key)?.as_bool())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_enable_creates_file() {
        let temp = TempDir::new().unwrap();
        let settings_path = temp.path().join("settings.json");

        enable_superpowers(&settings_path, "superpowers@git").unwrap();

        assert!(settings_path.exists());
        let content = std::fs::read_to_string(&settings_path).unwrap();
        let settings: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(settings["enabledPlugins"]["superpowers@git"], true);
    }

    #[test]
    fn test_enable_preserves_existing() {
        let temp = TempDir::new().unwrap();
        let settings_path = temp.path().join("settings.json");

        // Create existing settings
        std::fs::write(&settings_path, r#"{"other": "value"}"#).unwrap();

        enable_superpowers(&settings_path, "superpowers@git").unwrap();

        let content = std::fs::read_to_string(&settings_path).unwrap();
        let settings: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(settings["other"], "value");
        assert_eq!(settings["enabledPlugins"]["superpowers@git"], true);
    }

    #[test]
    fn test_disable_removes_key() {
        let temp = TempDir::new().unwrap();
        let settings_path = temp.path().join("settings.json");

        // Enable first
        enable_superpowers(&settings_path, "superpowers@git").unwrap();
        assert!(is_enabled(&settings_path, "superpowers@git"));

        // Then disable
        disable_superpowers(&settings_path, "superpowers@git").unwrap();
        assert!(!is_enabled(&settings_path, "superpowers@git"));
    }

    #[test]
    fn test_is_enabled_false_when_missing() {
        let temp = TempDir::new().unwrap();
        let settings_path = temp.path().join("nonexistent.json");
        assert!(!is_enabled(&settings_path, "superpowers@git"));
    }
}
