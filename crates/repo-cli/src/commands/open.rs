//! Open command implementation
//!
//! Launches an editor/IDE in a specified worktree directory after syncing configs.

use std::path::Path;
use std::process::Command;

use colored::Colorize;

use crate::error::{CliError, Result};

/// Known editor definitions: (slug, binary name, display name)
const EDITORS: &[(&str, &str, &str)] = &[
    ("cursor", "cursor", "Cursor"),
    ("vscode", "code", "VS Code"),
    ("zed", "zed", "Zed"),
];

/// Check if a binary is available on PATH
fn is_on_path(binary: &str) -> bool {
    which(binary).is_some()
}

/// Find the full path of a binary on PATH (cross-platform)
fn which(binary: &str) -> Option<std::path::PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    let exts = if cfg!(windows) {
        vec![".exe", ".cmd", ".bat", ""]
    } else {
        vec![""]
    };

    for dir in std::env::split_paths(&path_var) {
        for ext in &exts {
            let candidate = dir.join(format!("{}{}", binary, ext));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Detect which editors are installed
pub fn detect_editors() -> Vec<(&'static str, &'static str, &'static str)> {
    EDITORS
        .iter()
        .filter(|(_, binary, _)| is_on_path(binary))
        .copied()
        .collect()
}

/// Find editor binary name from a tool slug
fn resolve_editor(slug: &str) -> Result<&'static str> {
    for (s, binary, _) in EDITORS {
        if *s == slug {
            if is_on_path(binary) {
                return Ok(binary);
            } else {
                return Err(CliError::user(format!(
                    "Editor '{}' is not installed or not on PATH.",
                    slug
                )));
            }
        }
    }
    Err(CliError::user(format!(
        "Unknown editor '{}'. Supported: cursor, vscode, zed",
        slug
    )))
}

/// Auto-detect the best editor to use based on config tools and what's installed
fn auto_detect_editor(config_path: &Path) -> Result<&'static str> {
    // Try to read config to prefer tools listed there
    let config_file = config_path.join(".repository").join("config.toml");
    if config_file.exists()
        && let Ok(content) = std::fs::read_to_string(&config_file)
        && let Ok(manifest) = repo_core::Manifest::parse(&content)
    {
        // Check configured tools in order
        for tool_name in &manifest.tools {
            for (slug, binary, _) in EDITORS {
                if tool_name == *slug && is_on_path(binary) {
                    return Ok(binary);
                }
            }
        }
    }

    // Fall back to first installed editor
    let installed = detect_editors();
    if let Some((_, binary, _)) = installed.first() {
        return Ok(binary);
    }

    Err(CliError::user(
        "No supported editor found on PATH. Install cursor, code (VS Code), or zed.",
    ))
}

/// Run the open command
///
/// Resolves the worktree path, syncs configs, then launches the editor.
pub fn run_open(root: &Path, worktree: &str, tool: Option<&str>) -> Result<()> {
    // Resolve worktree path - it should be a sibling directory in worktree mode
    // or a subdirectory. Try both the worktree name directly and as a sibling.
    let worktree_path = if Path::new(worktree).is_absolute() && Path::new(worktree).is_dir() {
        std::path::PathBuf::from(worktree)
    } else {
        // Try as sibling of current root (worktree container pattern)
        let sibling = root.parent().map(|p| p.join(worktree));
        if let Some(ref p) = sibling {
            if p.is_dir() {
                p.clone()
            } else {
                // Try as child of root
                let child = root.join(worktree);
                if child.is_dir() {
                    child
                } else {
                    return Err(CliError::user(format!(
                        "Worktree '{}' not found. Tried:\n  - {}\n  - {}",
                        worktree,
                        sibling.unwrap().display(),
                        root.join(worktree).display()
                    )));
                }
            }
        } else {
            let child = root.join(worktree);
            if child.is_dir() {
                child
            } else {
                return Err(CliError::user(format!(
                    "Worktree '{}' not found at {}",
                    worktree,
                    child.display()
                )));
            }
        }
    };

    println!(
        "{} Opening worktree: {}",
        "=>".blue().bold(),
        worktree_path.display().to_string().cyan()
    );

    // Determine the editor to use
    let editor_binary = match tool {
        Some(slug) => resolve_editor(slug)?,
        None => auto_detect_editor(&worktree_path)?,
    };

    // Find display name for the editor
    let editor_name = EDITORS
        .iter()
        .find(|(_, b, _)| *b == editor_binary)
        .map(|(_, _, name)| *name)
        .unwrap_or(editor_binary);

    println!(
        "{} Using editor: {}",
        "=>".blue().bold(),
        editor_name.cyan()
    );

    // Try to sync configs in the worktree before opening
    let repo_config = worktree_path.join(".repository").join("config.toml");
    if repo_config.exists() {
        println!("{} Syncing configs...", "=>".blue().bold());
        match crate::commands::run_sync(&worktree_path, false, false) {
            Ok(()) => {}
            Err(e) => {
                // Don't fail the open if sync fails - just warn
                println!("{} Sync warning: {}", "WARN".yellow().bold(), e);
            }
        }
    }

    // Launch the editor
    println!("{} Launching {} ...", "=>".blue().bold(), editor_name);

    Command::new(editor_binary)
        .arg(&worktree_path)
        .spawn()
        .map_err(|e| CliError::user(format!("Failed to launch '{}': {}", editor_binary, e)))?;

    println!("{} Opened in {}.", "OK".green().bold(), editor_name);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editors_list_not_empty() {
        assert!(!EDITORS.is_empty());
    }

    #[test]
    fn test_which_finds_known_binary() {
        // cargo should always be on PATH in a rust dev environment
        let result = which("cargo");
        assert!(result.is_some(), "cargo should be on PATH");
    }

    #[test]
    fn test_which_returns_none_for_nonexistent() {
        let result = which("nonexistent_binary_that_does_not_exist_12345");
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_editors_returns_vec() {
        // Just verify it doesn't panic - actual results depend on environment
        let editors = detect_editors();
        // editors may be empty or non-empty depending on installed software
        assert!(editors.len() <= EDITORS.len());
    }

    #[test]
    fn test_resolve_editor_unknown() {
        let result = resolve_editor("emacs");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Unknown editor"));
    }

    #[test]
    fn test_open_nonexistent_worktree() {
        let temp = tempfile::TempDir::new().unwrap();
        let result = run_open(temp.path(), "nonexistent-worktree", None);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not found"));
    }

    #[test]
    fn test_open_with_unknown_tool() {
        let temp = tempfile::TempDir::new().unwrap();
        // Create a fake worktree directory
        let wt = temp.path().join("my-worktree");
        std::fs::create_dir_all(&wt).unwrap();

        let result = run_open(temp.path(), "my-worktree", Some("emacs"));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Unknown editor"));
    }

    #[test]
    fn test_auto_detect_no_editors_no_config() {
        // With a temp dir that has no config and possibly no editors on PATH,
        // this tests the fallback path
        let temp = tempfile::TempDir::new().unwrap();
        let result = auto_detect_editor(temp.path());
        // May succeed or fail depending on what's installed - just verify no panic
        let _ = result;
    }
}
