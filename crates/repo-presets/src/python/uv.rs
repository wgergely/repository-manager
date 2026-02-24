//! uv-based Python environment provider

use crate::context::Context;
use crate::error::{Error, Result};
use crate::provider::{ApplyReport, PresetCheckReport, PresetProvider};
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

    /// Determine whether to pass `--python VERSION` to `uv venv`.
    ///
    /// Per ADR-010 §10.2:
    /// - A single-bound constraint (`>=X.Y`, `==X.Y`, bare `X.Y`) is passed directly.
    /// - A range constraint (`>=X,<Y`) is **not** passed; uv will fall back to the
    ///   `.python-version` file in the working directory (if present) or its system default.
    ///
    /// Returns `Some(version)` when the version should be forwarded to uv, or `None`
    /// when delegation to `.python-version` / uv defaults is preferred.
    fn resolve_python_arg(version: &str) -> Option<String> {
        let trimmed = version.trim();
        // A range constraint contains a comma (e.g., ">=3.10,<3.14").
        // Count actual specifier tokens: if more than one, skip --python.
        let specifier_count = trimmed
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .count();
        if specifier_count > 1 {
            None
        } else {
            Some(trimmed.to_string())
        }
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

    async fn check(&self, context: &Context) -> Result<PresetCheckReport> {
        // First check if uv is available
        if !self.check_uv_available().await {
            return Ok(PresetCheckReport::broken(
                "uv not found. Install uv: https://docs.astral.sh/uv/",
            ));
        }

        // Check if venv exists
        if !self.check_venv_exists(context) {
            return Ok(PresetCheckReport::missing("Virtual environment not found"));
        }

        Ok(PresetCheckReport::healthy())
    }

    async fn apply(&self, context: &Context) -> Result<ApplyReport> {
        let venv_path = context.venv_path();
        let python_version = context.python_version();

        // ADR-010 §10.2: only forward --python for single-bound constraints.
        // For range constraints (e.g. ">=3.10,<3.14"), omit --python so that
        // uv can pick up the version from a .python-version file or its own defaults.
        let python_arg = Self::resolve_python_arg(&python_version);

        let mut cmd = Command::new("uv");
        cmd.arg("venv");
        if let Some(ref version) = python_arg {
            cmd.args(["--python", version]);
        }
        cmd.arg(venv_path.to_native())
            .current_dir(context.root.to_native());

        let status = cmd.status().await.map_err(|_| Error::UvNotFound)?;

        if !status.success() {
            let detail = match python_arg {
                Some(ref v) => format!("Failed to create venv with Python {}", v),
                None => "Failed to create venv (delegated Python version to uv)".to_string(),
            };
            return Ok(ApplyReport::failure(vec![detail]));
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
        let provider = UvProvider;
        assert_eq!(provider.id(), "env:python");
    }

    // --- resolve_python_arg (ADR-010 §10.2) ---

    #[test]
    fn test_single_bound_gte_forwarded() {
        // ">=3.12" is a single specifier — pass it to uv
        let result = UvProvider::resolve_python_arg(">=3.12");
        assert_eq!(result, Some(">=3.12".to_string()));
    }

    #[test]
    fn test_pinned_version_forwarded() {
        // Bare pinned version "3.12.4" is a single specifier
        let result = UvProvider::resolve_python_arg("3.12.4");
        assert_eq!(result, Some("3.12.4".to_string()));
    }

    #[test]
    fn test_eq_constraint_forwarded() {
        // "==3.12.0" is a single specifier
        let result = UvProvider::resolve_python_arg("==3.12.0");
        assert_eq!(result, Some("==3.12.0".to_string()));
    }

    #[test]
    fn test_range_constraint_not_forwarded() {
        // ">=3.10,<3.14" has two specifiers — delegate to .python-version
        let result = UvProvider::resolve_python_arg(">=3.10,<3.14");
        assert_eq!(result, None);
    }

    #[test]
    fn test_range_constraint_three_parts_not_forwarded() {
        // ">=3.10,<3.14,!=3.11" has three specifiers
        let result = UvProvider::resolve_python_arg(">=3.10,<3.14,!=3.11");
        assert_eq!(result, None);
    }

    #[test]
    fn test_whitespace_trimmed() {
        let result = UvProvider::resolve_python_arg("  3.12  ");
        assert_eq!(result, Some("3.12".to_string()));
    }

    #[test]
    fn test_whitespace_around_range_not_forwarded() {
        let result = UvProvider::resolve_python_arg("  >=3.10,<3.14  ");
        assert_eq!(result, None);
    }
}
