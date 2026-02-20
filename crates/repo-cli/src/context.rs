//! Repository context detection
//!
//! Detects the type of repository and its root path from any directory.
//! This enables git-like behavior where commands work from anywhere in the repo.

use std::path::{Path, PathBuf};

/// The type of repository context detected
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepoContext {
    /// Container root with worktrees mode
    /// Path points to the container root (where .repository lives)
    ContainerRoot { path: PathBuf },

    /// Inside a worktree within a container
    /// container: path to container root
    /// worktree: name of the current worktree
    Worktree {
        container: PathBuf,
        worktree: String,
    },

    /// Standard repository (not using worktrees)
    StandardRepo { path: PathBuf },

    /// Not inside any recognized repository
    NotARepo,
}

/// Detect the repository context from the given directory
///
/// This walks up the directory tree looking for repository markers:
/// - `.repository/config.toml` with `mode = "worktrees"` indicates container root
/// - `.repository/config.toml` with `mode = "standard"` indicates standard repo
/// - If we find a worktree structure, we identify which worktree we're in
pub fn detect_context(cwd: &Path) -> RepoContext {
    // First, try to find .repository directory by walking up
    let mut current = cwd.to_path_buf();

    loop {
        let repo_dir = current.join(".repository");
        let config_path = repo_dir.join("config.toml");

        if config_path.exists() {
            // Found a repository - determine mode
            match std::fs::read_to_string(&config_path) {
                Ok(content) => {
                    let mode = parse_mode(&content);

                    return match mode.as_str() {
                        "worktrees" | "worktree" => RepoContext::ContainerRoot {
                            path: current.clone(),
                        },
                        _ => RepoContext::StandardRepo {
                            path: current.clone(),
                        },
                    };
                }
                Err(e) => {
                    // Config exists but couldn't be read (permissions, encoding, etc.)
                    // Warn the user rather than silently falling back.
                    eprintln!(
                        "warning: found {} but could not read it: {}",
                        config_path.display(),
                        e
                    );
                    return RepoContext::StandardRepo {
                        path: current.clone(),
                    };
                }
            }
        }

        // Check if we're inside a worktree
        // Worktrees are sibling directories to the container that has .repository
        if let Some(parent) = current.parent() {
            let parent_repo = parent.join(".repository");
            let parent_config = parent_repo.join("config.toml");

            if parent_config.exists() {
                match std::fs::read_to_string(&parent_config) {
                    Ok(content) => {
                        let mode = parse_mode(&content);

                        if mode == "worktrees" || mode == "worktree" {
                            // We're in a worktree
                            let worktree_name = current
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown")
                                .to_string();

                            return RepoContext::Worktree {
                                container: parent.to_path_buf(),
                                worktree: worktree_name,
                            };
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "warning: found {} but could not read it: {}",
                            parent_config.display(),
                            e
                        );
                    }
                }
            }
        }

        // Move up to parent directory
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            // Reached filesystem root
            break;
        }
    }

    RepoContext::NotARepo
}

/// Minimal struct for parsing mode from config.toml
#[derive(serde::Deserialize)]
struct ConfigMode {
    #[serde(default)]
    core: CoreMode,
}

#[derive(serde::Deserialize, Default)]
struct CoreMode {
    #[serde(default = "default_standard")]
    mode: String,
}

fn default_standard() -> String {
    "standard".to_string()
}

/// Parse the mode from config.toml content
fn parse_mode(content: &str) -> String {
    match toml::from_str::<ConfigMode>(content) {
        Ok(config) => config.core.mode,
        Err(_) => "standard".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_repo_config(path: &Path, mode: &str) {
        let repo_dir = path.join(".repository");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(
            repo_dir.join("config.toml"),
            format!("[core]\nmode = \"{}\"\n", mode),
        )
        .unwrap();
    }

    #[test]
    fn test_detect_standard_repo_at_root() {
        let temp = TempDir::new().unwrap();
        create_repo_config(temp.path(), "standard");

        let context = detect_context(temp.path());

        match &context {
            RepoContext::StandardRepo { path } => {
                assert_eq!(path, temp.path());
            }
            _ => panic!("Expected StandardRepo, got {:?}", context),
        }
    }

    #[test]
    fn test_detect_worktrees_container_at_root() {
        let temp = TempDir::new().unwrap();
        create_repo_config(temp.path(), "worktrees");

        let context = detect_context(temp.path());

        match &context {
            RepoContext::ContainerRoot { path } => {
                assert_eq!(path, temp.path());
            }
            _ => panic!("Expected ContainerRoot, got {:?}", context),
        }
    }

    #[test]
    fn test_detect_worktree_inside_container() {
        let temp = TempDir::new().unwrap();
        create_repo_config(temp.path(), "worktrees");

        // Create a worktree directory
        let worktree_dir = temp.path().join("feature-branch");
        fs::create_dir_all(&worktree_dir).unwrap();

        let context = detect_context(&worktree_dir);

        match &context {
            RepoContext::Worktree {
                container,
                worktree,
            } => {
                assert_eq!(container, temp.path());
                assert_eq!(worktree, "feature-branch");
            }
            _ => panic!("Expected Worktree context, got {:?}", context),
        }
    }

    #[test]
    fn test_detect_from_subdirectory_in_standard_repo() {
        let temp = TempDir::new().unwrap();
        create_repo_config(temp.path(), "standard");

        // Create a nested directory
        let nested = temp.path().join("src").join("lib");
        fs::create_dir_all(&nested).unwrap();

        let context = detect_context(&nested);

        match &context {
            RepoContext::StandardRepo { path } => {
                assert_eq!(path, temp.path());
            }
            _ => panic!("Expected StandardRepo, got {:?}", context),
        }
    }

    #[test]
    fn test_detect_from_subdirectory_in_worktree() {
        let temp = TempDir::new().unwrap();
        create_repo_config(temp.path(), "worktrees");

        // Create a worktree with nested directory
        let worktree_dir = temp.path().join("main");
        let nested = worktree_dir.join("src").join("components");
        fs::create_dir_all(&nested).unwrap();

        let context = detect_context(&nested);

        match &context {
            RepoContext::Worktree {
                container,
                worktree,
            } => {
                assert_eq!(container, temp.path());
                assert_eq!(worktree, "main");
            }
            _ => panic!("Expected Worktree context, got {:?}", context),
        }
    }

    #[test]
    fn test_detect_not_a_repo() {
        let temp = TempDir::new().unwrap();
        // No .repository directory

        let context = detect_context(temp.path());

        assert!(matches!(context, RepoContext::NotARepo));
    }

    #[test]
    fn test_detect_worktree_mode_alias() {
        let temp = TempDir::new().unwrap();
        // Use "worktree" (singular) as alias
        create_repo_config(temp.path(), "worktree");

        let context = detect_context(temp.path());

        assert!(matches!(context, RepoContext::ContainerRoot { .. }));
    }

    #[test]
    fn test_parse_mode_basic() {
        let content = "[core]\nmode = \"worktrees\"\n";
        assert_eq!(parse_mode(content), "worktrees");
    }

    #[test]
    fn test_parse_mode_with_other_config() {
        let content = "[core]\nmode = \"standard\"\nname = \"test\"\n\n[tools]\ncursor = {}\n";
        assert_eq!(parse_mode(content), "standard");
    }

    #[test]
    fn test_parse_mode_missing() {
        let content = "[core]\nname = \"test\"\n";
        assert_eq!(parse_mode(content), "standard");
    }

    #[test]
    fn test_detect_distinguishes_repo_from_non_repo() {
        let temp = TempDir::new().unwrap();

        // Not a repo
        let context = detect_context(temp.path());
        assert!(matches!(context, RepoContext::NotARepo));

        // Standard repo
        create_repo_config(temp.path(), "standard");
        let context = detect_context(temp.path());
        assert!(matches!(context, RepoContext::StandardRepo { .. }));
    }
}
