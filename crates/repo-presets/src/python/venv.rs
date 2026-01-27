//! Python built-in venv module provider

use crate::context::Context;
use crate::error::{Error, Result};
use crate::provider::{ActionType, ApplyReport, CheckReport, PresetProvider, PresetStatus};
use async_trait::async_trait;
use std::process::Stdio;
use tokio::process::Command;

/// Provider for Python virtual environments using Python's built-in venv module.
///
/// This provider handles creation and management of Python virtual environments
/// using `python -m venv`, which is built into Python 3.3+.
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

    /// Check if a virtual environment exists at the expected location.
    fn check_venv_exists(&self, context: &Context) -> bool {
        let venv_path = context.venv_path();
        let python_path = if cfg!(windows) {
            venv_path.join("Scripts").join("python.exe")
        } else {
            venv_path.join("bin").join("python")
        };
        python_path.exists()
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
        "env:python"
    }

    async fn check(&self, context: &Context) -> Result<CheckReport> {
        // First check if python is available
        if !self.check_python_available().await {
            return Ok(CheckReport {
                status: PresetStatus::Broken,
                details: vec!["Python not found. Install Python 3.3+ to use venv.".to_string()],
                action: ActionType::Install,
            });
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

    #[test]
    fn test_venv_provider_id() {
        let provider = VenvProvider::new();
        assert_eq!(provider.id(), "env:python");
    }

    #[test]
    fn test_venv_provider_default() {
        let provider = VenvProvider::default();
        assert_eq!(provider.id(), "env:python");
    }

    #[test]
    fn test_venv_provider_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<VenvProvider>();
    }
}
