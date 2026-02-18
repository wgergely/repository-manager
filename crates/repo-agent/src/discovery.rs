//! Agent discovery: find Python 3.13+, locate vaultspec framework directory
//!
//! This module handles detecting whether the agent subsystem is available
//! by searching for a compatible Python interpreter and the vaultspec
//! framework directory.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::Result;
use crate::types::HealthReport;

/// Minimum required Python major version
const MIN_PYTHON_MAJOR: u32 = 3;
/// Minimum required Python minor version
const MIN_PYTHON_MINOR: u32 = 13;

/// Default vaultspec framework directory name
const VAULTSPEC_DIR: &str = ".vaultspec";

/// Agent manager: discovers and manages the agent subsystem
#[derive(Debug)]
pub struct AgentManager {
    /// Repository root path
    root: PathBuf,
    /// Path to the vaultspec framework directory, if found
    vaultspec_path: Option<PathBuf>,
    /// Path to the Python interpreter, if found
    python_path: Option<PathBuf>,
    /// Detected Python version string
    python_version: Option<String>,
}

impl AgentManager {
    /// Discover the agent subsystem by searching for Python and vaultspec
    ///
    /// This does not fail if components are missing -- use `is_available()`
    /// to check whether the subsystem is ready.
    pub fn discover(root: &Path) -> Result<Self> {
        let vaultspec_path = find_vaultspec_dir(root);
        let (python_path, python_version) = find_python();

        Ok(Self {
            root: root.to_path_buf(),
            vaultspec_path,
            python_path,
            python_version,
        })
    }

    /// Check whether the agent subsystem is fully available
    ///
    /// Returns true only if both Python 3.13+ and vaultspec are found.
    pub fn is_available(&self) -> bool {
        self.python_path.is_some() && self.vaultspec_path.is_some()
    }

    /// Perform a health check and return a detailed report
    pub fn health_check(&self) -> Result<HealthReport> {
        let mut messages = Vec::new();

        // Check Python
        let python_ok = if let Some(ref path) = self.python_path {
            let version = self.python_version.as_deref().unwrap_or("unknown");
            messages.push(format!("Python found: {} ({})", path.display(), version));
            true
        } else {
            messages.push(format!(
                "Python {}.{}+ not found. Install Python {}.{} or later.",
                MIN_PYTHON_MAJOR, MIN_PYTHON_MINOR, MIN_PYTHON_MAJOR, MIN_PYTHON_MINOR
            ));
            false
        };

        // Check vaultspec
        let vaultspec_ok = if let Some(ref path) = self.vaultspec_path {
            messages.push(format!("Vaultspec found: {}", path.display()));
            true
        } else {
            let expected = self.root.join(VAULTSPEC_DIR);
            messages.push(format!("Vaultspec not found at {}", expected.display()));
            false
        };

        // Count agents if vaultspec is available
        let agent_count = if vaultspec_ok {
            count_agents(self.vaultspec_path.as_deref().unwrap())
        } else {
            0
        };

        if agent_count > 0 {
            messages.push(format!("{} agent(s) discovered", agent_count));
        }

        Ok(HealthReport {
            available: python_ok && vaultspec_ok,
            python_path: self.python_path.as_ref().map(|p| p.display().to_string()),
            python_version: self.python_version.clone(),
            vaultspec_path: self
                .vaultspec_path
                .as_ref()
                .map(|p| p.display().to_string()),
            agent_count,
            messages,
        })
    }

    /// Get the repository root path
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Get the vaultspec framework directory path, if found
    pub fn vaultspec_path(&self) -> Option<&Path> {
        self.vaultspec_path.as_deref()
    }

    /// Get the Python interpreter path, if found
    pub fn python_path(&self) -> Option<&Path> {
        self.python_path.as_deref()
    }
}

/// Search for the vaultspec framework directory
///
/// Looks for `.vaultspec/` in the repository root and checks that
/// the expected subdirectory structure exists.
fn find_vaultspec_dir(root: &Path) -> Option<PathBuf> {
    let vaultspec_dir = root.join(VAULTSPEC_DIR);

    if !vaultspec_dir.is_dir() {
        return None;
    }

    // Validate expected structure: lib/scripts/ should exist
    let scripts_dir = vaultspec_dir.join("lib").join("scripts");
    if !scripts_dir.is_dir() {
        return None;
    }

    Some(vaultspec_dir)
}

/// Search for a compatible Python interpreter (3.13+)
///
/// Tries `python3`, `python`, and `python3.13` in PATH.
/// Returns the path and version string if found.
fn find_python() -> (Option<PathBuf>, Option<String>) {
    let candidates = ["python3", "python", "python3.13"];

    for candidate in &candidates {
        if let Some((path, version)) = try_python(candidate) {
            return (Some(path), Some(version));
        }
    }

    (None, None)
}

/// Try a specific Python interpreter candidate
///
/// Runs `<candidate> --version` and parses the output to check
/// if it meets the minimum version requirement.
fn try_python(candidate: &str) -> Option<(PathBuf, String)> {
    let output = Command::new(candidate).arg("--version").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let version_str = String::from_utf8_lossy(&output.stdout);
    let version_str = version_str.trim();

    // Parse "Python X.Y.Z"
    let version = version_str.strip_prefix("Python ")?;
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 2 {
        return None;
    }

    let major: u32 = parts[0].parse().ok()?;
    let minor: u32 = parts[1].parse().ok()?;

    if major > MIN_PYTHON_MAJOR || (major == MIN_PYTHON_MAJOR && minor >= MIN_PYTHON_MINOR) {
        // Resolve the full path using `which` / `where`
        let path = resolve_path(candidate).unwrap_or_else(|| PathBuf::from(candidate));
        Some((path, version.to_string()))
    } else {
        None
    }
}

/// Resolve a command name to its full path
fn resolve_path(command: &str) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    let which_cmd = "where";
    #[cfg(not(target_os = "windows"))]
    let which_cmd = "which";

    let output = Command::new(which_cmd).arg(command).output().ok()?;

    if output.status.success() {
        let path_str = String::from_utf8_lossy(&output.stdout);
        let first_line = path_str.lines().next()?.trim();
        if !first_line.is_empty() {
            return Some(PathBuf::from(first_line));
        }
    }
    None
}

/// Count agents in the vaultspec framework directory
///
/// Looks for YAML/TOML agent definition files in `.vaultspec/agents/`.
fn count_agents(vaultspec_path: &Path) -> usize {
    let agents_dir = vaultspec_path.join("agents");
    if !agents_dir.is_dir() {
        return 0;
    }

    std::fs::read_dir(&agents_dir)
        .ok()
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| {
                    let path = e.path();
                    path.is_file()
                        && matches!(
                            path.extension().and_then(|ext| ext.to_str()),
                            Some("yaml" | "yml" | "toml" | "json")
                        )
                })
                .count()
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_discover_no_vaultspec() {
        let temp = TempDir::new().unwrap();
        let manager = AgentManager::discover(temp.path()).unwrap();
        assert!(!manager.is_available());
        assert!(manager.vaultspec_path().is_none());
    }

    #[test]
    fn test_discover_with_vaultspec_dir() {
        let temp = TempDir::new().unwrap();
        let scripts_dir = temp.path().join(".vaultspec/lib/scripts");
        std::fs::create_dir_all(&scripts_dir).unwrap();

        let manager = AgentManager::discover(temp.path()).unwrap();
        assert!(manager.vaultspec_path().is_some());
    }

    #[test]
    fn test_discover_incomplete_vaultspec() {
        let temp = TempDir::new().unwrap();
        // Create .vaultspec but without lib/scripts
        std::fs::create_dir_all(temp.path().join(".vaultspec")).unwrap();

        let manager = AgentManager::discover(temp.path()).unwrap();
        assert!(manager.vaultspec_path().is_none());
    }

    #[test]
    fn test_health_check_unavailable() {
        let temp = TempDir::new().unwrap();
        let manager = AgentManager {
            root: temp.path().to_path_buf(),
            vaultspec_path: None,
            python_path: None,
            python_version: None,
        };

        let report = manager.health_check().unwrap();
        assert!(!report.available);
        assert!(!report.messages.is_empty());
    }

    #[test]
    fn test_health_check_with_components() {
        let temp = TempDir::new().unwrap();
        let vaultspec_dir = temp.path().join(".vaultspec");
        std::fs::create_dir_all(vaultspec_dir.join("lib/scripts")).unwrap();

        let manager = AgentManager {
            root: temp.path().to_path_buf(),
            vaultspec_path: Some(vaultspec_dir),
            python_path: Some(PathBuf::from("/usr/bin/python3")),
            python_version: Some("3.13.1".to_string()),
        };

        let report = manager.health_check().unwrap();
        assert!(report.available);
        assert!(report.python_version.is_some());
        assert!(report.vaultspec_path.is_some());
    }

    #[test]
    fn test_count_agents_empty() {
        let temp = TempDir::new().unwrap();
        let vaultspec_dir = temp.path().join(".vaultspec");
        std::fs::create_dir_all(&vaultspec_dir).unwrap();

        assert_eq!(count_agents(&vaultspec_dir), 0);
    }

    #[test]
    fn test_count_agents_with_files() {
        let temp = TempDir::new().unwrap();
        let agents_dir = temp.path().join(".vaultspec/agents");
        std::fs::create_dir_all(&agents_dir).unwrap();

        std::fs::write(agents_dir.join("researcher.yaml"), "name: researcher").unwrap();
        std::fs::write(agents_dir.join("coder.toml"), "name = \"coder\"").unwrap();
        std::fs::write(agents_dir.join("readme.md"), "not an agent").unwrap();

        let count = count_agents(&temp.path().join(".vaultspec"));
        assert_eq!(count, 2); // .yaml and .toml, not .md
    }

    #[test]
    fn test_find_vaultspec_dir_valid() {
        let temp = TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join(".vaultspec/lib/scripts")).unwrap();

        let result = find_vaultspec_dir(temp.path());
        assert!(result.is_some());
    }

    #[test]
    fn test_find_vaultspec_dir_missing() {
        let temp = TempDir::new().unwrap();
        let result = find_vaultspec_dir(temp.path());
        assert!(result.is_none());
    }
}
