//! Node.js environment detection provider

use crate::context::Context;
use crate::error::Result;
use crate::provider::{ActionType, ApplyReport, CheckReport, PresetProvider, PresetStatus};
use async_trait::async_trait;
use std::process::Stdio;
use tokio::process::Command;

/// Provider for detecting Node.js environments.
///
/// This is a detection-only provider that checks for:
/// - `package.json` file exists
/// - `node_modules` directory exists
/// - `node` command available on PATH
///
/// Unlike Python providers, this does not create or manage environments,
/// it only detects their presence.
pub struct NodeProvider;

impl NodeProvider {
    /// Create a new NodeProvider instance.
    pub fn new() -> Self {
        Self
    }

    /// Check if Node.js is available on the system.
    async fn check_node_available(&self) -> bool {
        Command::new("node")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Check if Node.js is available (synchronous version for testing).
    pub fn check_node_available_sync(&self) -> bool {
        std::process::Command::new("node")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Check if package.json exists in the project root.
    fn check_package_json_exists(&self, context: &Context) -> bool {
        context.root.join("package.json").exists()
    }

    /// Check if node_modules directory exists in the project root.
    fn check_node_modules_exists(&self, context: &Context) -> bool {
        context.root.join("node_modules").exists()
    }
}

impl Default for NodeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PresetProvider for NodeProvider {
    fn id(&self) -> &str {
        "env:node"
    }

    async fn check(&self, context: &Context) -> Result<CheckReport> {
        let mut details = Vec::new();

        // Check if package.json exists
        let has_package_json = self.check_package_json_exists(context);
        if !has_package_json {
            details.push("package.json not found".to_string());
        }

        // Check if node_modules exists
        let has_node_modules = self.check_node_modules_exists(context);
        if !has_node_modules {
            details.push("node_modules not found".to_string());
        }

        // Check if node is available on PATH
        let node_available = self.check_node_available().await;
        if !node_available {
            details.push("node not found on PATH".to_string());
        }

        // Determine status based on what's present
        if !has_package_json {
            // Not a Node.js project
            return Ok(CheckReport {
                status: PresetStatus::Missing,
                details,
                action: ActionType::None,
            });
        }

        if !node_available {
            // Node.js project but node not installed
            return Ok(CheckReport::broken(details.join("; ")));
        }

        if !has_node_modules {
            // Node.js project with node, but dependencies not installed
            return Ok(CheckReport {
                status: PresetStatus::Missing,
                details: vec![
                    "Dependencies not installed. Run npm install or yarn install.".to_string(),
                ],
                action: ActionType::Install,
            });
        }

        // Everything is present
        Ok(CheckReport::healthy())
    }

    async fn apply(&self, _context: &Context) -> Result<ApplyReport> {
        // This is a detection-only provider
        Ok(ApplyReport::success(vec![
            "Node environment detection complete. This provider is detection-only.".to_string(),
        ]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_fs::{LayoutMode, NormalizedPath, WorkspaceLayout};
    use std::collections::HashMap;
    use std::fs;
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
    fn test_node_provider_id() {
        let provider = NodeProvider::new();
        assert_eq!(provider.id(), "env:node");
    }

    #[test]
    fn test_node_provider_default() {
        let provider = NodeProvider;
        assert_eq!(provider.id(), "env:node");
    }

    #[test]
    fn test_node_provider_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<NodeProvider>();
    }

    #[test]
    fn test_check_package_json_exists() {
        let provider = NodeProvider::new();
        let temp = TempDir::new().unwrap();
        let context = make_test_context(&temp);

        // No package.json initially
        assert!(!provider.check_package_json_exists(&context));

        // Create package.json
        fs::write(temp.path().join("package.json"), "{}").unwrap();
        assert!(provider.check_package_json_exists(&context));
    }

    #[test]
    fn test_check_node_modules_exists() {
        let provider = NodeProvider::new();
        let temp = TempDir::new().unwrap();
        let context = make_test_context(&temp);

        // No node_modules initially
        assert!(!provider.check_node_modules_exists(&context));

        // Create node_modules directory
        fs::create_dir(temp.path().join("node_modules")).unwrap();
        assert!(provider.check_node_modules_exists(&context));
    }

    #[tokio::test]
    async fn test_check_missing_package_json() {
        let provider = NodeProvider::new();
        let temp = TempDir::new().unwrap();
        let context = make_test_context(&temp);

        let report = provider.check(&context).await.unwrap();
        assert_eq!(report.status, PresetStatus::Missing);
        assert!(report.details.iter().any(|d| d.contains("package.json")));
    }

    #[tokio::test]
    async fn test_check_missing_node_modules() {
        let provider = NodeProvider::new();
        let temp = TempDir::new().unwrap();
        let context = make_test_context(&temp);

        // Create package.json but no node_modules
        fs::write(temp.path().join("package.json"), "{}").unwrap();

        // Skip if node is not available
        if !provider.check_node_available().await {
            eprintln!("Skipping test: Node not available");
            return;
        }

        let report = provider.check(&context).await.unwrap();
        assert_eq!(report.status, PresetStatus::Missing);
        assert!(
            report
                .details
                .iter()
                .any(|d| d.contains("Dependencies not installed"))
        );
    }

    #[tokio::test]
    async fn test_check_healthy() {
        let provider = NodeProvider::new();
        let temp = TempDir::new().unwrap();
        let context = make_test_context(&temp);

        // Skip if node is not available
        if !provider.check_node_available().await {
            eprintln!("Skipping test: Node not available");
            return;
        }

        // Create package.json and node_modules
        fs::write(temp.path().join("package.json"), "{}").unwrap();
        fs::create_dir(temp.path().join("node_modules")).unwrap();

        let report = provider.check(&context).await.unwrap();
        assert_eq!(report.status, PresetStatus::Healthy);
    }

    #[tokio::test]
    async fn test_apply_is_detection_only() {
        let provider = NodeProvider::new();
        let temp = TempDir::new().unwrap();
        let context = make_test_context(&temp);

        let report = provider.apply(&context).await.unwrap();
        assert!(report.success);
        assert!(
            report
                .actions_taken
                .iter()
                .any(|a| a.contains("detection-only"))
        );
    }
}
