//! Extension installation helpers.
//!
//! This module provides functions to execute an extension's install command,
//! check for required binaries on PATH, and query the active Python version.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Error, Result};

/// Build a shell [`Command`] that executes `cmd_str` via the system shell.
///
/// - Unix: `sh -c "{cmd_str}"`
/// - Windows: `cmd /C "{cmd_str}"`
fn shell_command(cmd_str: &str) -> Command {
    #[cfg(windows)]
    {
        let mut c = Command::new("cmd");
        c.args(["/C", cmd_str]);
        c
    }
    #[cfg(not(windows))]
    {
        let mut c = Command::new("sh");
        c.arg("-c").arg(cmd_str);
        c
    }
}

/// Execute the extension's install command.
///
/// On Unix: `sh -c "{install_cmd}"`
/// On Windows: `cmd /C "{install_cmd}"`
///
/// The working directory is set to `working_dir` (the extension source directory).
/// The environment inherits the parent process environment with the additional
/// variables `REPO_EXTENSION_NAME`, `REPO_EXTENSION_VERSION`, and `REPO_ROOT`.
///
/// Stdout and stderr are both inherited (streamed live to the terminal) so that
/// warnings and progress output are visible during installation. On failure the
/// error message instructs the user to check the output above. A non-zero exit
/// code returns [`Error::InstallFailed`].
pub fn run_install(
    name: &str,
    version: &str,
    install_cmd: &str,
    working_dir: &Path,
    repo_root: &Path,
) -> Result<()> {
    let mut cmd = shell_command(install_cmd);
    cmd.current_dir(working_dir)
        .env("REPO_EXTENSION_NAME", name)
        .env("REPO_EXTENSION_VERSION", version)
        .env("REPO_ROOT", repo_root)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit());

    let status = cmd
        .status()
        .map_err(|_e| Error::InstallFailed {
            name: name.to_string(),
            command: install_cmd.to_string(),
            exit_code: None,
        })?;

    if !status.success() {
        return Err(Error::InstallFailed {
            name: name.to_string(),
            command: install_cmd.to_string(),
            exit_code: status.code(),
        });
    }

    Ok(())
}

/// Verify a binary is on PATH. Returns the resolved path or [`Error::PackageManagerNotFound`].
pub fn check_binary_on_path(tool: &str) -> Result<PathBuf> {
    // Use `which`-style search: iterate PATH entries
    let path_var = std::env::var_os("PATH").unwrap_or_default();
    let extensions: Vec<String> = if cfg!(windows) {
        std::env::var("PATHEXT")
            .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string())
            .split(';')
            .map(|s| s.to_ascii_lowercase())
            .collect()
    } else {
        vec![String::new()]
    };

    for dir in std::env::split_paths(&path_var) {
        for ext in &extensions {
            let candidate = if ext.is_empty() {
                dir.join(tool)
            } else {
                dir.join(format!("{}{}", tool, ext))
            };
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }

    Err(Error::PackageManagerNotFound {
        tool: tool.to_string(),
        hint: install_hint(tool).map(str::to_string),
    })
}

fn install_hint(tool: &str) -> Option<&'static str> {
    match tool {
        "uv" => Some("\n  Install: curl -LsSf https://astral.sh/uv/install.sh | sh"),
        "npm" => Some("\n  Install: https://nodejs.org"),
        "cargo" => Some("\n  Install: https://rustup.rs"),
        _ => None,
    }
}

/// Returns the active Python version string (e.g., `"3.13.1"`).
///
/// Tries in order:
/// 1. `uv python find --quiet` (returns a path; runs `{path} --version`)
/// 2. `python3 --version`
/// 3. `python --version`
///
/// Parses `"Python 3.13.1"` → `"3.13.1"`.
pub fn query_python_version(python_cmd: Option<&str>) -> Result<String> {
    if let Some(cmd) = python_cmd {
        return run_python_version_cmd(cmd);
    }

    // Try uv python find first
    if let Ok(uv_path) = check_binary_on_path("uv") {
        if let Ok(output) = Command::new(uv_path)
            .args(["python", "find", "--quiet"])
            .output()
        {
            if output.status.success() {
                let python_path = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string();
                if !python_path.is_empty() {
                    if let Ok(version) = run_python_version_cmd(&python_path) {
                        return Ok(version);
                    }
                }
            }
        }
    }

    // Try python3
    if let Ok(version) = run_python_version_cmd("python3") {
        return Ok(version);
    }

    // Try python
    run_python_version_cmd("python")
}

fn run_python_version_cmd(cmd: &str) -> Result<String> {
    let output = Command::new(cmd)
        .arg("--version")
        .output()
        .map_err(|e| Error::PackageManagerNotFound {
            tool: cmd.to_string(),
            hint: Some(e.to_string()),
        })?;

    // Python 2 writes to stderr; Python 3 to stdout. Check both.
    let raw = if !output.stdout.is_empty() {
        String::from_utf8_lossy(&output.stdout).into_owned()
    } else {
        String::from_utf8_lossy(&output.stderr).into_owned()
    };

    parse_python_version(&raw).ok_or_else(|| Error::PackageManagerNotFound {
        tool: cmd.to_string(),
        hint: Some(format!("unexpected output: {}", raw.trim())),
    })
}

/// Synthesize a pip install command from a packages list.
///
/// Returns `None` if `packages` is empty.
/// Uses `uv pip install` when `package_manager` is `Some("uv")`,
/// otherwise falls back to `pip install`.
pub fn synthesize_install_command(
    packages: &[String],
    package_manager: Option<&str>,
) -> Option<String> {
    if packages.is_empty() {
        return None;
    }
    let pkg_list = packages.join(" ");
    match package_manager {
        Some("uv") => Some(format!("uv pip install {}", pkg_list)),
        _ => Some(format!("pip install {}", pkg_list)),
    }
}

/// Parse `"Python 3.13.1\n"` → `"3.13.1"`.
fn parse_python_version(output: &str) -> Option<String> {
    let trimmed = output.trim();
    let version = trimmed.strip_prefix("Python ")?;
    // Take only the version token (no trailing metadata)
    let version = version.split_whitespace().next()?;
    Some(version.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_version_parse() {
        assert_eq!(
            parse_python_version("Python 3.13.1\n"),
            Some("3.13.1".to_string())
        );
        assert_eq!(
            parse_python_version("Python 3.10.0"),
            Some("3.10.0".to_string())
        );
        assert_eq!(parse_python_version("not python"), None);
        assert_eq!(parse_python_version(""), None);
    }

    #[test]
    fn test_run_install_exits_on_nonzero() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        // Use a shell command that always exits with non-zero
        #[cfg(windows)]
        let cmd = "exit /b 1";
        #[cfg(not(windows))]
        let cmd = "exit 1";

        let err = run_install("test-ext", "0.1.0", cmd, tmp.path(), tmp.path()).unwrap_err();
        assert!(
            matches!(err, Error::InstallFailed { ref name, .. } if name == "test-ext"),
            "expected InstallFailed, got: {err:?}"
        );
    }

    #[test]
    fn test_run_install_success() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        #[cfg(windows)]
        let cmd = "echo ok";
        #[cfg(not(windows))]
        let cmd = "true";

        run_install("test-ext", "0.1.0", cmd, tmp.path(), tmp.path()).unwrap();
    }

    #[test]
    fn test_check_binary_on_path_not_found() {
        let err = check_binary_on_path("nonexistent_tool_xyz_12345").unwrap_err();
        assert!(
            matches!(err, Error::PackageManagerNotFound { ref tool, .. } if tool == "nonexistent_tool_xyz_12345"),
            "expected PackageManagerNotFound, got: {err:?}"
        );
    }

    #[test]
    fn test_lock_file_written_after_install() {
        use crate::lock::{LockFile, LockedExtension};
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let lock_path = tmp.path().join("extensions.lock");

        let mut lock = LockFile::new();
        lock.upsert(LockedExtension {
            name: "vaultspec".to_string(),
            version: "0.1.0".to_string(),
            source: tmp.path().to_string_lossy().into_owned(),
            resolved_ref: None,
            runtime_type: Some("python".to_string()),
            python_version: Some("3.13.1".to_string()),
            package_manager: None,
            packages: vec![],
            venv_path: None,
        });
        lock.save(&lock_path).unwrap();

        let loaded = LockFile::load(&lock_path).unwrap();
        assert_eq!(loaded.len(), 1);
        let entry = loaded.get("vaultspec").unwrap();
        assert_eq!(entry.version, "0.1.0");
        assert_eq!(entry.python_version.as_deref(), Some("3.13.1"));
    }

    #[test]
    fn test_synthesize_uv_command() {
        let packages = vec!["httpx>=0.27".to_string()];
        let cmd = synthesize_install_command(&packages, Some("uv")).unwrap();
        assert_eq!(cmd, "uv pip install httpx>=0.27");
    }

    #[test]
    fn test_synthesize_pip_fallback() {
        let packages = vec!["httpx>=0.27".to_string(), "pydantic>=2.0".to_string()];
        let cmd = synthesize_install_command(&packages, None).unwrap();
        assert_eq!(cmd, "pip install httpx>=0.27 pydantic>=2.0");
    }

    #[test]
    fn test_synthesize_pip_explicit() {
        let packages = vec!["requests".to_string()];
        let cmd = synthesize_install_command(&packages, Some("pip")).unwrap();
        assert_eq!(cmd, "pip install requests");
    }

    #[test]
    fn test_packages_empty_no_command() {
        let result = synthesize_install_command(&[], Some("uv"));
        assert!(result.is_none());

        let result = synthesize_install_command(&[], None);
        assert!(result.is_none());
    }
}
