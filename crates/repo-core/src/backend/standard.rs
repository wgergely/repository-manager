//! Standard (single-directory) Git repository backend

use std::process::Command;

use crate::{Error, Result};
use repo_fs::NormalizedPath;

use super::{BranchInfo, ModeBackend};

/// Backend for traditional single-directory Git repositories.
///
/// In this mode:
/// - Only one branch can be active at a time
/// - Branches are switched via `git checkout`
/// - Configuration lives in `{repo}/.repository`
pub struct StandardBackend {
    /// Repository root directory (where .git lives)
    root: NormalizedPath,
}

impl StandardBackend {
    /// Create a new StandardBackend for the given repository root.
    ///
    /// Verifies that `.git` exists in the root directory.
    pub fn new(root: NormalizedPath) -> Result<Self> {
        let git_dir = root.join(".git");
        if !git_dir.exists() {
            return Err(Error::Fs(repo_fs::Error::LayoutValidation {
                message: format!("Not a git repository: .git not found at {}", root.as_str()),
            }));
        }

        Ok(Self { root })
    }

    /// Get the root directory of the repository.
    pub fn root(&self) -> &NormalizedPath {
        &self.root
    }

    /// Run a git command and return the output.
    fn git_command(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(self.root.to_native())
            .output()
            .map_err(Error::Io)?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(Error::SyncError {
                message: format!("Git command failed: {}", stderr.trim()),
            })
        }
    }

    /// Get the current branch name.
    fn current_branch(&self) -> Result<String> {
        self.git_command(&["rev-parse", "--abbrev-ref", "HEAD"])
    }

    /// Check if a branch exists.
    fn branch_exists(&self, name: &str) -> bool {
        self.git_command(&["rev-parse", "--verify", &format!("refs/heads/{}", name)])
            .is_ok()
    }

    /// Get the main branch name (main or master).
    fn main_branch_name(&self) -> String {
        // Try to determine main branch from remote HEAD or common names
        if self.branch_exists("main") {
            "main".to_string()
        } else if self.branch_exists("master") {
            "master".to_string()
        } else {
            // Fall back to trying to get it from remote
            self.git_command(&["symbolic-ref", "--short", "refs/remotes/origin/HEAD"])
                .map(|s| s.trim_start_matches("origin/").to_string())
                .unwrap_or_else(|_| "main".to_string())
        }
    }
}

impl ModeBackend for StandardBackend {
    fn config_root(&self) -> NormalizedPath {
        self.root.join(".repository")
    }

    fn working_dir(&self) -> &NormalizedPath {
        &self.root
    }

    fn create_branch(&self, name: &str, base: Option<&str>) -> Result<()> {
        // Use "--" to separate branch names from git flags (defense-in-depth)
        let args = match base {
            Some(base_branch) => vec!["branch", "--", name, base_branch],
            None => vec!["branch", "--", name],
        };

        self.git_command(&args)?;
        Ok(())
    }

    fn delete_branch(&self, name: &str) -> Result<()> {
        // Don't allow deleting the current branch
        let current = self.current_branch()?;
        if current == name {
            return Err(Error::SyncError {
                message: format!("Cannot delete current branch: {}", name),
            });
        }

        // Use "--" to separate branch names from git flags (defense-in-depth)
        self.git_command(&["branch", "-d", "--", name])?;
        Ok(())
    }

    fn list_branches(&self) -> Result<Vec<BranchInfo>> {
        let output =
            self.git_command(&["for-each-ref", "--format=%(refname:short)", "refs/heads/"])?;

        let current = self.current_branch()?;
        let main_branch = self.main_branch_name();

        let branches: Vec<BranchInfo> = output
            .lines()
            .filter(|line| !line.is_empty())
            .map(|name| {
                let is_current = name == current;
                let is_main = name == main_branch;
                BranchInfo::standard(name, is_current, is_main)
            })
            .collect();

        Ok(branches)
    }

    fn switch_branch(&self, name: &str) -> Result<NormalizedPath> {
        // Check if branch exists
        if !self.branch_exists(name) {
            return Err(Error::Git(repo_git::Error::BranchNotFound {
                name: name.to_string(),
            }));
        }

        self.git_command(&["checkout", name])?;
        Ok(self.root.clone())
    }

    fn rename_branch(&self, old_name: &str, new_name: &str) -> Result<()> {
        if !self.branch_exists(old_name) {
            return Err(Error::Git(repo_git::Error::BranchNotFound {
                name: old_name.to_string(),
            }));
        }

        self.git_command(&["branch", "-m", "--", old_name, new_name])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_test_utils::git::fake_git_dir;
    use tempfile::TempDir;

    #[test]
    fn test_root() {
        let temp = TempDir::new().unwrap();
        fake_git_dir(temp.path());
        let root = NormalizedPath::new(temp.path());
        let backend = StandardBackend::new(root.clone()).unwrap();

        assert_eq!(backend.root().as_str(), root.as_str());
    }
}
