//! Rust environment detection provider

use crate::context::Context;
use crate::error::Result;
use crate::provider::{ActionType, ApplyReport, CheckReport, PresetProvider, PresetStatus};
use async_trait::async_trait;
use std::process::Stdio;
use tokio::process::Command;

/// Provider for detecting Rust development environments.
///
/// This provider is detection-only - it checks whether a project uses Rust
/// (has Cargo.toml) and whether rustc is available on the system PATH.
/// It does not perform any installation or modification actions.
pub struct RustProvider;

impl RustProvider {
    /// Create a new RustProvider instance.
    pub fn new() -> Self {
        Self
    }

    /// Check if rustc is available on the system PATH.
    async fn check_rustc_available(&self) -> bool {
        Command::new("rustc")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Check if rustc is available (synchronous version for testing).
    pub fn check_rustc_available_sync(&self) -> bool {
        std::process::Command::new("rustc")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Check if Cargo.toml exists in the project root.
    fn check_cargo_toml_exists(&self, context: &Context) -> bool {
        context.root.join("Cargo.toml").exists()
    }
}

impl Default for RustProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PresetProvider for RustProvider {
    fn id(&self) -> &str {
        "env:rust"
    }

    async fn check(&self, context: &Context) -> Result<CheckReport> {
        // Check if Cargo.toml exists
        if !self.check_cargo_toml_exists(context) {
            return Ok(CheckReport {
                status: PresetStatus::Missing,
                details: vec!["Cargo.toml not found. This may not be a Rust project.".to_string()],
                action: ActionType::None,
            });
        }

        // Check if rustc is available
        if !self.check_rustc_available().await {
            return Ok(CheckReport {
                status: PresetStatus::Broken,
                details: vec![
                    "Cargo.toml found but rustc not available on PATH.".to_string(),
                    "Install Rust via https://rustup.rs to use this project.".to_string(),
                ],
                action: ActionType::Install,
            });
        }

        Ok(CheckReport::healthy())
    }

    async fn apply(&self, _context: &Context) -> Result<ApplyReport> {
        // This is a detection-only provider
        Ok(ApplyReport::success(vec![
            "Rust environment provider is detection-only.".to_string(),
            "No actions taken. Use rustup to manage Rust installations.".to_string(),
        ]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_fs::{LayoutMode, NormalizedPath, WorkspaceLayout};
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn make_test_context(temp: &TempDir) -> Context {
        let root = NormalizedPath::new(temp.path());
        let layout = WorkspaceLayout {
            root: root.clone(),
            active_context: root.clone(),
            mode: LayoutMode::Classic,
        };
        Context::new(layout, HashMap::new())
    }

    #[test]
    fn test_rust_provider_id() {
        let provider = RustProvider::new();
        assert_eq!(provider.id(), "env:rust");
    }

    #[test]
    fn test_rust_provider_default() {
        let provider = RustProvider::default();
        assert_eq!(provider.id(), "env:rust");
    }

    #[test]
    fn test_rust_provider_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RustProvider>();
    }

    #[test]
    fn test_check_cargo_toml_exists_false() {
        let temp = TempDir::new().unwrap();
        let context = make_test_context(&temp);
        let provider = RustProvider::new();

        assert!(!provider.check_cargo_toml_exists(&context));
    }

    #[test]
    fn test_check_cargo_toml_exists_true() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

        let context = make_test_context(&temp);
        let provider = RustProvider::new();

        assert!(provider.check_cargo_toml_exists(&context));
    }

    #[test]
    fn test_check_rustc_available_sync() {
        let provider = RustProvider::new();
        // This test verifies the method runs without panicking
        // The result depends on whether rustc is installed
        let _available = provider.check_rustc_available_sync();
    }

    #[tokio::test]
    async fn test_check_no_cargo_toml() {
        let temp = TempDir::new().unwrap();
        let context = make_test_context(&temp);
        let provider = RustProvider::new();

        let report = provider.check(&context).await.unwrap();
        assert_eq!(report.status, PresetStatus::Missing);
        assert_eq!(report.action, ActionType::None);
        assert!(report.details[0].contains("Cargo.toml not found"));
    }

    #[tokio::test]
    async fn test_check_with_cargo_toml_and_rustc() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

        let context = make_test_context(&temp);
        let provider = RustProvider::new();

        // Skip if rustc is not available
        if !provider.check_rustc_available_sync() {
            eprintln!("Skipping test: rustc not available");
            return;
        }

        let report = provider.check(&context).await.unwrap();
        assert_eq!(report.status, PresetStatus::Healthy);
        assert_eq!(report.action, ActionType::None);
    }

    #[tokio::test]
    async fn test_apply_returns_detection_only_message() {
        let temp = TempDir::new().unwrap();
        let context = make_test_context(&temp);
        let provider = RustProvider::new();

        let report = provider.apply(&context).await.unwrap();
        assert!(report.success);
        assert!(report.actions_taken[0].contains("detection-only"));
    }
}
