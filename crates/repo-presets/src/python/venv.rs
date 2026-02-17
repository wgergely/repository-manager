//! Python built-in venv module provider

use crate::context::Context;
use crate::error::{Error, Result};
use crate::provider::{ApplyReport, CheckReport, PresetProvider};
use async_trait::async_trait;
use repo_fs::NormalizedPath;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

/// Provider for Python virtual environments using Python's built-in venv module.
///
/// This provider handles creation and management of Python virtual environments
/// using `python -m venv`, which is built into Python 3.3+.
///
/// Supports tagged venvs for worktree-based workflows:
/// - Untagged: `.venv`
/// - Tagged: `.venv-{tag}` (e.g., `.venv-main-win-py311`)
pub struct VenvProvider;

impl VenvProvider {
    /// Create a new VenvProvider instance.
    pub fn new() -> Self {
        Self
    }

    /// Check if Python is available on the system.
    async fn check_python_available(&self) -> bool {
        Command::new("python")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Check if Python is available (synchronous version for testing).
    pub fn check_python_available_sync(&self) -> bool {
        std::process::Command::new("python")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Check if a virtual environment exists at the expected location.
    fn check_venv_exists(&self, context: &Context) -> bool {
        self.check_venv_at_path(&context.venv_path())
    }

    /// Check if a virtual environment exists at a specific path.
    pub fn check_venv_at_path(&self, venv_path: &NormalizedPath) -> bool {
        let python_path = if cfg!(windows) {
            venv_path.join("Scripts").join("python.exe")
        } else {
            venv_path.join("bin").join("python")
        };
        python_path.exists()
    }

    /// Create a tagged virtual environment synchronously.
    ///
    /// # Arguments
    /// - `root`: The root directory where the venv should be created
    /// - `tag`: The tag for the venv (e.g., "main-win-py311")
    ///
    /// # Returns
    /// The path to the created virtual environment.
    pub fn create_tagged_sync(&self, root: &Path, tag: &str) -> Result<NormalizedPath> {
        let venv_name = format!(".venv-{}", tag);
        let venv_path = NormalizedPath::new(root).join(&venv_name);

        let status = std::process::Command::new("python")
            .args(["-m", "venv"])
            .arg(venv_path.as_ref())
            .current_dir(root)
            .status()
            .map_err(|_| Error::PythonNotFound)?;

        if !status.success() {
            return Err(Error::VenvCreationFailed {
                path: venv_path.to_string(),
            });
        }

        Ok(venv_path)
    }

    /// Create a tagged virtual environment asynchronously.
    ///
    /// # Arguments
    /// - `root`: The root directory where the venv should be created
    /// - `tag`: The tag for the venv (e.g., "main-win-py311")
    ///
    /// # Returns
    /// The path to the created virtual environment.
    pub async fn create_tagged(&self, root: &Path, tag: &str) -> Result<NormalizedPath> {
        let venv_name = format!(".venv-{}", tag);
        let venv_path = NormalizedPath::new(root).join(&venv_name);

        let status = Command::new("python")
            .args(["-m", "venv"])
            .arg(venv_path.as_ref())
            .current_dir(root)
            .status()
            .await
            .map_err(|_| Error::PythonNotFound)?;

        if !status.success() {
            return Err(Error::VenvCreationFailed {
                path: venv_path.to_string(),
            });
        }

        Ok(venv_path)
    }

    /// Generate a tag for the current environment.
    ///
    /// Format: `{worktree}-{platform}-py{version}`
    /// Example: `main-win-py312`
    pub fn generate_tag(worktree: &str, python_version: Option<&str>) -> String {
        let platform = if cfg!(windows) {
            "win"
        } else if cfg!(target_os = "macos") {
            "mac"
        } else {
            "linux"
        };

        let version = python_version.unwrap_or("3");

        format!("{}-{}-py{}", worktree, platform, version)
    }
}

impl Default for VenvProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PresetProvider for VenvProvider {
    fn id(&self) -> &str {
        "env:python-venv"
    }

    async fn check(&self, context: &Context) -> Result<CheckReport> {
        // First check if python is available
        if !self.check_python_available().await {
            return Ok(CheckReport::broken(
                "Python not found. Install Python 3.3+ to use venv.",
            ));
        }

        // Check if venv exists
        if !self.check_venv_exists(context) {
            return Ok(CheckReport::missing("Virtual environment not found"));
        }

        Ok(CheckReport::healthy())
    }

    async fn apply(&self, context: &Context) -> Result<ApplyReport> {
        let venv_path = context.venv_path();

        let status = Command::new("python")
            .args(["-m", "venv"])
            .arg(venv_path.to_native())
            .current_dir(context.root.to_native())
            .status()
            .await
            .map_err(|_| Error::PythonNotFound)?;

        if !status.success() {
            return Ok(ApplyReport::failure(vec![
                "Failed to create virtual environment with python -m venv".to_string(),
            ]));
        }

        Ok(ApplyReport::success(vec![format!(
            "Created virtual environment at {}",
            venv_path
        )]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_venv_provider_id() {
        let provider = VenvProvider::new();
        assert_eq!(provider.id(), "env:python-venv");
    }

    #[test]
    fn test_venv_provider_default() {
        let provider = VenvProvider;
        assert_eq!(provider.id(), "env:python-venv");
    }

    #[test]
    fn test_venv_provider_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<VenvProvider>();
    }

    #[test]
    fn test_generate_tag() {
        let tag = VenvProvider::generate_tag("main", Some("312"));
        #[cfg(windows)]
        assert_eq!(tag, "main-win-py312");
        #[cfg(target_os = "macos")]
        assert_eq!(tag, "main-mac-py312");
        #[cfg(target_os = "linux")]
        assert_eq!(tag, "main-linux-py312");
    }

    #[test]
    fn test_generate_tag_default_version() {
        let tag = VenvProvider::generate_tag("feature", None);
        assert!(tag.starts_with("feature-"));
        assert!(tag.ends_with("-py3"));
    }

    #[test]
    fn test_create_tagged_sync_creates_venv() {
        let provider = VenvProvider::new();

        // Skip if Python is not available
        if !provider.check_python_available_sync() {
            eprintln!("Skipping test: Python not available");
            return;
        }

        let temp = TempDir::new().unwrap();
        let result = provider.create_tagged_sync(temp.path(), "test-tag");

        assert!(result.is_ok(), "Failed to create venv: {:?}", result.err());

        let venv_path = result.unwrap();
        assert!(venv_path.exists(), "Venv directory should exist");

        // Check that python binary exists
        let python_path = if cfg!(windows) {
            venv_path.join("Scripts").join("python.exe")
        } else {
            venv_path.join("bin").join("python")
        };
        assert!(python_path.exists(), "Python binary should exist in venv");
    }

    #[test]
    fn test_check_venv_at_path() {
        let provider = VenvProvider::new();

        // Skip if Python is not available
        if !provider.check_python_available_sync() {
            eprintln!("Skipping test: Python not available");
            return;
        }

        let temp = TempDir::new().unwrap();

        // Create a tagged venv
        let venv_path = provider
            .create_tagged_sync(temp.path(), "check-test")
            .unwrap();

        // Verify it exists
        assert!(provider.check_venv_at_path(&venv_path));

        // Verify non-existent path returns false
        let fake_path = NormalizedPath::new(temp.path()).join(".venv-nonexistent");
        assert!(!provider.check_venv_at_path(&fake_path));
    }

    #[tokio::test]
    async fn test_create_tagged_async_creates_venv() {
        let provider = VenvProvider::new();

        // Skip if Python is not available
        if !provider.check_python_available().await {
            eprintln!("Skipping test: Python not available");
            return;
        }

        let temp = TempDir::new().unwrap();
        let result = provider.create_tagged(temp.path(), "async-test").await;

        assert!(result.is_ok(), "Failed to create venv: {:?}", result.err());

        let venv_path = result.unwrap();
        assert!(venv_path.exists(), "Venv directory should exist");

        // Check that python binary exists
        let python_path = if cfg!(windows) {
            venv_path.join("Scripts").join("python.exe")
        } else {
            venv_path.join("bin").join("python")
        };
        assert!(python_path.exists(), "Python binary should exist in venv");
    }
}
