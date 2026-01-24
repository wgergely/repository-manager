//! uv-based Python environment provider

use crate::context::Context;
use crate::error::{Error, Result};
use crate::provider::{ActionType, ApplyReport, CheckReport, PresetProvider, PresetStatus};
use async_trait::async_trait;
use std::process::Stdio;
use tokio::process::Command;

/// Provider for Python virtual environments using uv.
///
/// This provider handles creation and management of Python virtual environments
/// using the uv package manager (https://docs.astral.sh/uv/).
pub struct UvProvider;

impl UvProvider {
    /// Create a new UvProvider instance.
    pub fn new() -> Self {
        Self
    }

    /// Check if uv is available on the system.
    async fn check_uv_available(&self) -> bool {
        Command::new("uv")
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

impl Default for UvProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PresetProvider for UvProvider {
    fn id(&self) -> &str {
        "env:python"
    }

    async fn check(&self, context: &Context) -> Result<CheckReport> {
        // First check if uv is available
        if !self.check_uv_available().await {
            return Ok(CheckReport {
                status: PresetStatus::Broken,
                details: vec!["uv not found. Install uv: https://docs.astral.sh/uv/".to_string()],
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
        let python_version = context.python_version();

        let status = Command::new("uv")
            .args(["venv", "--python", &python_version])
            .arg(venv_path.to_native())
            .current_dir(context.root.to_native())
            .status()
            .await
            .map_err(|_| Error::UvNotFound)?;

        if !status.success() {
            return Ok(ApplyReport::failure(vec![format!(
                "Failed to create venv with Python {}",
                python_version
            )]));
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
    fn test_uv_provider_default() {
        let provider = UvProvider::default();
        assert_eq!(provider.id(), "env:python");
    }
}
