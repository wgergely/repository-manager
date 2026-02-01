# Superpowers Preset Provider Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a `SuperpowersProvider` preset that clones and installs the superpowers Claude Code plugin from GitHub with robust install/uninstall tracking.

**Architecture:** Implement as a new `PresetProvider` in `repo-presets` crate. Uses git clone to fetch superpowers from `github.com/obra/superpowers`, installs to `~/.claude/plugins/cache/git/superpowers/{version}/`, tracks installation state in a local manifest, and updates Claude's `settings.json` to enable the plugin.

**Tech Stack:** Rust, git2 crate (for cloning), serde_json (for settings.json), tokio (async), tempfile (tests)

---

## Research Summary

### Claude Code Plugin Architecture
- Plugins cached at: `~/.claude/plugins/cache/{marketplace}/{plugin}/{version}/`
- Installation tracking: `~/.claude/plugins/installed_plugins.json`
- Enable/disable: `~/.claude/settings.json` → `enabledPlugins: { "plugin@marketplace": true }`
- **Known bug**: Uninstall doesn't clear cache - must manually delete

### Superpowers Plugin Structure
- Repository: `https://github.com/obra/superpowers`
- Manifest: `.claude-plugin/plugin.json` with version field
- Skills in `skills/` directories with `SKILL.md` files
- Version tags: `v4.1.1`, `v4.0.3`, etc.

### Existing Provider Pattern
- Trait: `PresetProvider` with `id()`, `check()`, `apply()` methods
- Context: `Context { layout, root, config, venv_tag }`
- Status: `PresetStatus::Healthy | Missing | Drifted | Broken`
- Actions: `ActionType::None | Install | Repair | Update`

---

## Task 1: Add git2 Dependency to repo-presets

**Files:**
- Modify: `crates/repo-presets/Cargo.toml`

**Step 1: Add git2 to dependencies**

```toml
[dependencies]
# ... existing deps ...
git2 = "0.19"
```

**Step 2: Verify compilation**

Run: `cargo check -p repo-presets`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add crates/repo-presets/Cargo.toml
git commit -m "feat(repo-presets): add git2 dependency for superpowers provider

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 2: Define SuperpowersProvider Error Types

**Files:**
- Modify: `crates/repo-presets/src/error.rs`

**Step 1: Write the failing test**

```rust
// In crates/repo-presets/src/error.rs, add test at bottom
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_clone_error_display() {
        let err = Error::GitClone {
            url: "https://github.com/obra/superpowers".to_string(),
            message: "network error".to_string(),
        };
        assert!(err.to_string().contains("superpowers"));
        assert!(err.to_string().contains("network error"));
    }

    #[test]
    fn test_plugin_manifest_error_display() {
        let err = Error::PluginManifest {
            path: "/path/to/plugin.json".to_string(),
            message: "invalid JSON".to_string(),
        };
        assert!(err.to_string().contains("plugin.json"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p repo-presets test_git_clone_error`
Expected: FAIL with "GitClone not found"

**Step 3: Add error variants**

```rust
// Add to Error enum in crates/repo-presets/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // ... existing variants ...

    #[error("Failed to clone git repository {url}: {message}")]
    GitClone { url: String, message: String },

    #[error("Failed to read plugin manifest at {path}: {message}")]
    PluginManifest { path: String, message: String },

    #[error("Failed to update Claude settings: {0}")]
    ClaudeSettings(String),

    #[error("Superpowers not installed")]
    SuperpowersNotInstalled,
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p repo-presets test_git_clone_error test_plugin_manifest_error`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/repo-presets/src/error.rs
git commit -m "feat(repo-presets): add error types for superpowers provider

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Create SuperpowersProvider Module Structure

**Files:**
- Create: `crates/repo-presets/src/superpowers/mod.rs`
- Create: `crates/repo-presets/src/superpowers/provider.rs`
- Create: `crates/repo-presets/src/superpowers/paths.rs`
- Modify: `crates/repo-presets/src/lib.rs`

**Step 1: Create module structure**

Create `crates/repo-presets/src/superpowers/mod.rs`:
```rust
//! Superpowers Claude Code plugin provider

mod paths;
mod provider;

pub use provider::SuperpowersProvider;
```

**Step 2: Create paths module with constants**

Create `crates/repo-presets/src/superpowers/paths.rs`:
```rust
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
```

**Step 3: Create provider stub**

Create `crates/repo-presets/src/superpowers/provider.rs`:
```rust
//! SuperpowersProvider implementation

use crate::context::Context;
use crate::error::Result;
use crate::provider::{ApplyReport, CheckReport, PresetProvider, PresetStatus, ActionType};
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
        // TODO: Implement in Task 4
        Ok(CheckReport {
            status: PresetStatus::Missing,
            details: vec!["Not implemented".to_string()],
            action: ActionType::Install,
        })
    }

    async fn apply(&self, _context: &Context) -> Result<ApplyReport> {
        // TODO: Implement in Task 5
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
}
```

**Step 4: Update lib.rs exports**

Modify `crates/repo-presets/src/lib.rs`:
```rust
//! Preset providers for Repository Manager.

pub mod context;
pub mod error;
pub mod node;
pub mod provider;
pub mod python;
pub mod rust;
pub mod superpowers;  // Add this line

pub use context::Context;
pub use error::{Error, Result};
pub use node::NodeProvider;
pub use provider::{ActionType, ApplyReport, CheckReport, PresetProvider, PresetStatus};
pub use python::{UvProvider, VenvProvider};
pub use rust::RustProvider;
pub use superpowers::SuperpowersProvider;  // Add this line
```

**Step 5: Add dirs dependency**

Add to `crates/repo-presets/Cargo.toml`:
```toml
[dependencies]
dirs = "5.0"
```

**Step 6: Verify compilation and tests**

Run: `cargo test -p repo-presets superpowers`
Expected: All tests pass

**Step 7: Commit**

```bash
git add crates/repo-presets/src/superpowers/ crates/repo-presets/src/lib.rs crates/repo-presets/Cargo.toml
git commit -m "feat(repo-presets): add superpowers module structure

- Add paths module with install directory constants
- Add provider stub implementing PresetProvider
- Export SuperpowersProvider from lib.rs

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Implement check() Method

**Files:**
- Modify: `crates/repo-presets/src/superpowers/provider.rs`

**Step 1: Write failing test for check when not installed**

```rust
// Add to tests module in provider.rs
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p repo-presets test_check_not_installed`
Expected: May pass (stub returns Missing) - that's ok

**Step 3: Implement check() method**

```rust
// Replace check() implementation in provider.rs
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
```

**Step 4: Add serde_json to dependencies**

Add to `crates/repo-presets/Cargo.toml`:
```toml
serde_json = "1.0"
```

**Step 5: Run tests**

Run: `cargo test -p repo-presets test_check`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/repo-presets/src/superpowers/provider.rs crates/repo-presets/Cargo.toml
git commit -m "feat(repo-presets): implement superpowers check() method

- Check if plugin is installed at expected location
- Verify plugin.json is valid
- Check if enabled in Claude settings

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Implement Git Clone Helper

**Files:**
- Create: `crates/repo-presets/src/superpowers/git.rs`
- Modify: `crates/repo-presets/src/superpowers/mod.rs`

**Step 1: Write failing test**

```rust
// In new file crates/repo-presets/src/superpowers/git.rs
//! Git operations for superpowers installation

use crate::error::{Error, Result};
use std::path::Path;

/// Clone a git repository to a destination directory.
///
/// # Arguments
/// * `url` - Git repository URL
/// * `dest` - Destination directory
/// * `tag` - Optional tag/branch to checkout
pub fn clone_repo(url: &str, dest: &Path, tag: Option<&str>) -> Result<()> {
    use git2::{build::RepoBuilder, FetchOptions};

    // Create parent directories
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| Error::GitClone {
            url: url.to_string(),
            message: format!("Failed to create directory: {}", e),
        })?;
    }

    // Clone the repository
    let mut builder = RepoBuilder::new();

    if let Some(tag_name) = tag {
        // For tags, we clone then checkout
        let repo = builder.clone(url, dest).map_err(|e| Error::GitClone {
            url: url.to_string(),
            message: e.message().to_string(),
        })?;

        // Checkout the specific tag
        let (object, reference) = repo.revparse_ext(tag_name).map_err(|e| Error::GitClone {
            url: url.to_string(),
            message: format!("Tag {} not found: {}", tag_name, e),
        })?;

        repo.checkout_tree(&object, None).map_err(|e| Error::GitClone {
            url: url.to_string(),
            message: format!("Failed to checkout {}: {}", tag_name, e),
        })?;

        // Set HEAD to the tag
        if let Some(ref_name) = reference {
            repo.set_head(ref_name.name().unwrap_or(tag_name))
        } else {
            repo.set_head_detached(object.id())
        }.map_err(|e| Error::GitClone {
            url: url.to_string(),
            message: format!("Failed to set HEAD: {}", e),
        })?;
    } else {
        builder.clone(url, dest).map_err(|e| Error::GitClone {
            url: url.to_string(),
            message: e.message().to_string(),
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_clone_requires_valid_url() {
        let temp = TempDir::new().unwrap();
        let result = clone_repo("not-a-valid-url", temp.path(), None);
        assert!(result.is_err());
    }

    // Note: Integration test with real git clone should be in integration tests
    // to avoid network dependency in unit tests
}
```

**Step 2: Update mod.rs**

```rust
//! Superpowers Claude Code plugin provider

mod git;
mod paths;
mod provider;

pub use provider::SuperpowersProvider;
```

**Step 3: Run tests**

Run: `cargo test -p repo-presets test_clone`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/repo-presets/src/superpowers/git.rs crates/repo-presets/src/superpowers/mod.rs
git commit -m "feat(repo-presets): add git clone helper for superpowers

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 6: Implement Settings Update Helper

**Files:**
- Create: `crates/repo-presets/src/superpowers/settings.rs`
- Modify: `crates/repo-presets/src/superpowers/mod.rs`

**Step 1: Create settings helper**

```rust
//! Claude settings.json manipulation

use crate::error::{Error, Result};
use serde_json::{json, Value};
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
    if !settings.get("enabledPlugins").is_some() {
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

    // Remove or set to false
    if let Some(enabled_plugins) = settings.get_mut("enabledPlugins") {
        if let Some(obj) = enabled_plugins.as_object_mut() {
            obj.remove(plugin_key);
        }
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
        .and_then(|settings| {
            settings.get("enabledPlugins")?
                .get(plugin_key)?
                .as_bool()
        })
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
```

**Step 2: Update mod.rs**

```rust
//! Superpowers Claude Code plugin provider

mod git;
mod paths;
mod provider;
mod settings;

pub use provider::SuperpowersProvider;
```

**Step 3: Run tests**

Run: `cargo test -p repo-presets settings`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/repo-presets/src/superpowers/settings.rs crates/repo-presets/src/superpowers/mod.rs
git commit -m "feat(repo-presets): add Claude settings.json helper

- enable_superpowers() to add plugin to enabledPlugins
- disable_superpowers() to remove plugin
- is_enabled() to check current state

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 7: Implement apply() Method

**Files:**
- Modify: `crates/repo-presets/src/superpowers/provider.rs`

**Step 1: Implement apply()**

```rust
// Replace apply() implementation in provider.rs
async fn apply(&self, _context: &Context) -> Result<ApplyReport> {
    let mut actions = Vec::new();

    // Determine install directory
    let install_dir = match super::paths::superpowers_install_dir(&self.version) {
        Some(dir) => dir,
        None => {
            return Ok(ApplyReport::failure(vec![
                "Cannot determine home directory".to_string()
            ]));
        }
    };

    // Clone if not present
    if !install_dir.exists() {
        actions.push(format!("Cloning superpowers {} from {}", self.version, self.repo_url));

        super::git::clone_repo(&self.repo_url, &install_dir, Some(&self.version))?;

        actions.push(format!("Installed to {}", install_dir.display()));
    } else {
        actions.push(format!("Superpowers {} already installed", self.version));
    }

    // Enable in Claude settings
    if let Some(settings_path) = super::paths::claude_settings_path() {
        let plugin_key = format!("{}@{}", super::paths::PLUGIN_NAME, super::paths::MARKETPLACE_NAME);

        if !super::settings::is_enabled(&settings_path, &plugin_key) {
            super::settings::enable_superpowers(&settings_path, &plugin_key)?;
            actions.push("Enabled superpowers in Claude settings".to_string());
        }
    }

    Ok(ApplyReport::success(actions))
}
```

**Step 2: Run all superpowers tests**

Run: `cargo test -p repo-presets superpowers`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/repo-presets/src/superpowers/provider.rs
git commit -m "feat(repo-presets): implement superpowers apply() method

- Clone from GitHub if not present
- Enable in Claude settings.json

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 8: Add Uninstall Support

**Files:**
- Modify: `crates/repo-presets/src/superpowers/provider.rs`
- Modify: `crates/repo-presets/src/provider.rs` (add uninstall to trait)

**Step 1: Add uninstall to PresetProvider trait**

```rust
// In crates/repo-presets/src/provider.rs, add to trait
#[async_trait]
pub trait PresetProvider: Send + Sync {
    fn id(&self) -> &str;
    async fn check(&self, context: &Context) -> Result<CheckReport>;
    async fn apply(&self, context: &Context) -> Result<ApplyReport>;

    /// Uninstall/remove the preset (optional, default does nothing)
    async fn uninstall(&self, _context: &Context) -> Result<ApplyReport> {
        Ok(ApplyReport::success(vec!["Uninstall not supported".to_string()]))
    }
}
```

**Step 2: Implement uninstall for SuperpowersProvider**

```rust
// Add to SuperpowersProvider impl in provider.rs
async fn uninstall(&self, _context: &Context) -> Result<ApplyReport> {
    let mut actions = Vec::new();

    // Disable in Claude settings first
    if let Some(settings_path) = super::paths::claude_settings_path() {
        let plugin_key = format!("{}@{}", super::paths::PLUGIN_NAME, super::paths::MARKETPLACE_NAME);

        if super::settings::is_enabled(&settings_path, &plugin_key) {
            super::settings::disable_superpowers(&settings_path, &plugin_key)?;
            actions.push("Disabled superpowers in Claude settings".to_string());
        }
    }

    // Remove install directory
    if let Some(install_dir) = super::paths::superpowers_install_dir(&self.version) {
        if install_dir.exists() {
            std::fs::remove_dir_all(&install_dir).map_err(|e| {
                crate::error::Error::ClaudeSettings(format!(
                    "Failed to remove {}: {}", install_dir.display(), e
                ))
            })?;
            actions.push(format!("Removed {}", install_dir.display()));
        }
    }

    Ok(ApplyReport::success(actions))
}
```

**Step 3: Add test**

```rust
#[tokio::test]
async fn test_uninstall() {
    use tempfile::TempDir;
    use repo_fs::{NormalizedPath, WorkspaceLayout, LayoutMode};
    use std::collections::HashMap;

    // This is a unit test - doesn't actually install
    let temp = TempDir::new().unwrap();
    let layout = WorkspaceLayout {
        root: NormalizedPath::new(temp.path()),
        active_context: NormalizedPath::new(temp.path()),
        mode: LayoutMode::Classic,
    };
    let context = Context::new(layout, HashMap::new());

    let provider = SuperpowersProvider::new();
    let report = provider.uninstall(&context).await.unwrap();

    // Should succeed even if nothing to uninstall
    assert!(report.success);
}
```

**Step 4: Run tests**

Run: `cargo test -p repo-presets uninstall`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/repo-presets/src/superpowers/provider.rs crates/repo-presets/src/provider.rs
git commit -m "feat(repo-presets): add uninstall support for superpowers

- Add uninstall() to PresetProvider trait with default impl
- Implement uninstall for SuperpowersProvider
- Disable in settings, remove install directory

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 9: Add CLI Commands

**Files:**
- Modify: `crates/repo-cli/src/cli.rs`
- Create: `crates/repo-cli/src/commands/superpowers.rs`
- Modify: `crates/repo-cli/src/commands/mod.rs`
- Modify: `crates/repo-cli/src/main.rs`

**Step 1: Add CLI subcommands**

In `crates/repo-cli/src/cli.rs`, add to `Commands` enum:
```rust
/// Manage superpowers Claude Code plugin
Superpowers {
    #[command(subcommand)]
    action: SuperpowersAction,
},
```

Add the action enum:
```rust
/// Superpowers plugin actions
#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum SuperpowersAction {
    /// Install superpowers plugin
    Install {
        /// Version tag to install (e.g., v4.1.1)
        #[arg(short, long, default_value = "v4.1.1")]
        version: String,
    },
    /// Check superpowers installation status
    Status,
    /// Uninstall superpowers plugin
    Uninstall {
        /// Version to uninstall
        #[arg(short, long, default_value = "v4.1.1")]
        version: String,
    },
}
```

**Step 2: Create command handler**

Create `crates/repo-cli/src/commands/superpowers.rs`:
```rust
//! Superpowers plugin management commands

use crate::cli::SuperpowersAction;
use anyhow::Result;
use repo_fs::{LayoutMode, NormalizedPath, WorkspaceLayout};
use repo_presets::{Context, PresetProvider, SuperpowersProvider};
use std::collections::HashMap;

pub async fn handle_superpowers(action: SuperpowersAction) -> Result<()> {
    // Create a minimal context (superpowers doesn't use project root)
    let current_dir = std::env::current_dir()?;
    let layout = WorkspaceLayout {
        root: NormalizedPath::new(&current_dir),
        active_context: NormalizedPath::new(&current_dir),
        mode: LayoutMode::Classic,
    };
    let context = Context::new(layout, HashMap::new());

    match action {
        SuperpowersAction::Install { version } => {
            let provider = SuperpowersProvider::new().with_version(&version);

            println!("Checking superpowers status...");
            let check = provider.check(&context).await?;

            if check.status == repo_presets::PresetStatus::Healthy {
                println!("Superpowers {} is already installed and enabled.", version);
                return Ok(());
            }

            println!("Installing superpowers {}...", version);
            let report = provider.apply(&context).await?;

            for action in &report.actions_taken {
                println!("  {}", action);
            }

            if report.success {
                println!("Superpowers {} installed successfully!", version);
            } else {
                for err in &report.errors {
                    eprintln!("Error: {}", err);
                }
                anyhow::bail!("Installation failed");
            }
        }

        SuperpowersAction::Status => {
            let provider = SuperpowersProvider::new();
            let check = provider.check(&context).await?;

            println!("Superpowers status: {:?}", check.status);
            for detail in &check.details {
                println!("  {}", detail);
            }
        }

        SuperpowersAction::Uninstall { version } => {
            let provider = SuperpowersProvider::new().with_version(&version);

            println!("Uninstalling superpowers {}...", version);
            let report = provider.uninstall(&context).await?;

            for action in &report.actions_taken {
                println!("  {}", action);
            }

            if report.success {
                println!("Superpowers {} uninstalled.", version);
            } else {
                for err in &report.errors {
                    eprintln!("Error: {}", err);
                }
            }
        }
    }

    Ok(())
}
```

**Step 3: Update commands/mod.rs**

Add: `pub mod superpowers;`

**Step 4: Update main.rs**

Add to match on commands:
```rust
Some(Commands::Superpowers { action }) => {
    commands::superpowers::handle_superpowers(action).await?;
}
```

**Step 5: Run and verify**

Run: `cargo build -p repo-cli && ./target/debug/repo superpowers --help`
Expected: Shows install/status/uninstall subcommands

**Step 6: Commit**

```bash
git add crates/repo-cli/src/cli.rs crates/repo-cli/src/commands/superpowers.rs crates/repo-cli/src/commands/mod.rs crates/repo-cli/src/main.rs
git commit -m "feat(cli): add superpowers install/status/uninstall commands

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 10: Update Docker Configuration

**Files:**
- Modify: `docker/cli/claude/Dockerfile`

**Step 1: Update Dockerfile to support plugins directory**

```dockerfile
# repo-test/claude - Claude Code CLI
FROM repo-test/cli-base:latest

LABEL tool="claude"
LABEL tool.version="latest"

# Install Claude CLI globally
RUN npm install -g @anthropic-ai/claude-code

# Create Claude plugin directories
RUN mkdir -p /root/.claude/plugins/cache/git

# Verify installation
RUN claude --version || echo "Claude CLI installed (version check may require API key)"

# Default working directory
WORKDIR /workspace

# Entry point
ENTRYPOINT ["claude"]
CMD ["--help"]
```

**Step 2: Add volume mount for plugins in docker-compose.yml**

```yaml
claude:
  build:
    context: ./docker
    dockerfile: cli/claude/Dockerfile
  image: repo-test/claude:latest
  volumes:
    - *common-volumes
    - claude-plugins:/root/.claude/plugins  # Add this line
  # ... rest of config
```

Add to volumes section at bottom:
```yaml
volumes:
  claude-plugins:
```

**Step 3: Commit**

```bash
git add docker/cli/claude/Dockerfile docker-compose.yml
git commit -m "feat(docker): add plugin support to Claude container

- Create plugin directories in Dockerfile
- Add named volume for plugin persistence

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 11: Add Integration Tests

**Files:**
- Create: `crates/repo-presets/tests/superpowers_tests.rs`

**Step 1: Create integration test file**

```rust
//! Integration tests for SuperpowersProvider
//!
//! Note: Tests that require network access are marked with #[ignore]
//! Run with: cargo test -p repo-presets --test superpowers_tests -- --ignored

use repo_fs::{LayoutMode, NormalizedPath, WorkspaceLayout};
use repo_presets::{Context, PresetProvider, PresetStatus, SuperpowersProvider};
use std::collections::HashMap;
use tempfile::TempDir;

fn create_test_context(temp: &TempDir) -> Context {
    let layout = WorkspaceLayout {
        root: NormalizedPath::new(temp.path()),
        active_context: NormalizedPath::new(temp.path()),
        mode: LayoutMode::Classic,
    };
    Context::new(layout, HashMap::new())
}

#[tokio::test]
async fn test_provider_id() {
    let provider = SuperpowersProvider::new();
    assert_eq!(provider.id(), "claude:superpowers");
}

#[tokio::test]
async fn test_check_when_not_installed() {
    let temp = TempDir::new().unwrap();
    let context = create_test_context(&temp);
    let provider = SuperpowersProvider::new();

    let report = provider.check(&context).await.unwrap();

    // Should be Missing since we haven't installed
    assert_eq!(report.status, PresetStatus::Missing);
}

#[tokio::test]
#[ignore] // Requires network access
async fn test_install_and_uninstall() {
    let temp = TempDir::new().unwrap();
    let context = create_test_context(&temp);

    // Use a test version
    let provider = SuperpowersProvider::new().with_version("v4.1.1");

    // Install
    let report = provider.apply(&context).await.unwrap();
    assert!(report.success, "Install failed: {:?}", report.errors);

    // Check should show healthy
    let check = provider.check(&context).await.unwrap();
    assert_eq!(check.status, PresetStatus::Healthy);

    // Uninstall
    let report = provider.uninstall(&context).await.unwrap();
    assert!(report.success, "Uninstall failed: {:?}", report.errors);

    // Check should show missing
    let check = provider.check(&context).await.unwrap();
    assert_eq!(check.status, PresetStatus::Missing);
}
```

**Step 2: Run unit tests**

Run: `cargo test -p repo-presets --test superpowers_tests`
Expected: Non-ignored tests pass

**Step 3: Commit**

```bash
git add crates/repo-presets/tests/superpowers_tests.rs
git commit -m "test(repo-presets): add superpowers integration tests

- Unit tests run without network
- Integration tests marked #[ignore] for CI

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 12: Update Documentation

**Files:**
- Modify: `README.md` or create `docs/presets/superpowers.md`

**Step 1: Add documentation**

Create `docs/presets/superpowers.md`:
```markdown
# Superpowers Preset

The superpowers preset installs the [superpowers](https://github.com/obra/superpowers) Claude Code plugin, providing agentic skills for TDD, debugging, planning, and collaboration workflows.

## Installation

```bash
repo superpowers install
```

Or with a specific version:

```bash
repo superpowers install --version v4.1.1
```

## Usage

After installation, superpowers skills are available in Claude Code:

- `/superpowers:brainstorming` - Refine ideas through collaborative dialogue
- `/superpowers:writing-plans` - Create detailed implementation plans
- `/superpowers:test-driven-development` - Enforce TDD workflow
- `/superpowers:systematic-debugging` - Structured debugging methodology

## Status

Check installation status:

```bash
repo superpowers status
```

## Uninstall

```bash
repo superpowers uninstall
```

## How It Works

1. Clones superpowers from GitHub to `~/.claude/plugins/cache/git/superpowers/{version}/`
2. Enables the plugin in `~/.claude/settings.json`
3. Skills become available in Claude Code sessions

## Troubleshooting

### Plugin not showing in Claude Code

Ensure Claude Code is restarted after installation. The plugin is enabled via settings.json but requires a session restart.

### Network errors during install

The install requires network access to clone from GitHub. Ensure you have internet connectivity and can access github.com.
```

**Step 2: Commit**

```bash
git add docs/presets/superpowers.md
git commit -m "docs: add superpowers preset documentation

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Summary

This plan implements:

1. **SuperpowersProvider** - New preset provider that handles git clone and settings management
2. **Git clone helper** - Uses git2 crate for reliable cloning with tag support
3. **Settings management** - Properly enables/disables in Claude's settings.json
4. **CLI commands** - `repo superpowers install/status/uninstall`
5. **Docker support** - Plugin directory persistence
6. **Tests** - Unit tests and integration tests
7. **Documentation** - User guide for the preset

Total: 12 tasks, each 2-5 minutes for execution.
