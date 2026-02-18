//! Worktree (container-based) Git repository backend

use std::process::Command;

use crate::{Error, Result};
use repo_fs::NormalizedPath;

use super::{BranchInfo, ModeBackend};

/// Backend for container-based Git repositories with multiple worktrees.
///
/// In this mode:
/// - Multiple branches can be active simultaneously as separate directories
/// - The container holds `.gt/` (git database) and worktree directories
/// - Configuration is shared at `{container}/.repository`
pub struct WorktreeBackend {
    /// Container directory (where .gt and worktrees live)
    container: NormalizedPath,

    /// Current active worktree directory
    current_worktree: NormalizedPath,

    /// Path to the git database (.gt directory)
    git_dir: NormalizedPath,
}

impl WorktreeBackend {
    /// Create a new WorktreeBackend for the given container directory.
    ///
    /// Verifies that `.gt` exists and defaults to the `main` worktree.
    pub fn new(container: NormalizedPath) -> Result<Self> {
        let git_dir = container.join(".gt");
        if !git_dir.exists() {
            return Err(Error::Fs(repo_fs::Error::LayoutValidation {
                message: format!(
                    "Not a worktree container: .gt not found at {}",
                    container.as_str()
                ),
            }));
        }

        let main_worktree = container.join("main");

        Ok(Self {
            container,
            current_worktree: main_worktree,
            git_dir,
        })
    }

    /// Create a new WorktreeBackend with a specific worktree as the current context.
    ///
    /// # Arguments
    /// - `container`: Container directory path
    /// - `worktree`: Specific worktree to use as current context
    pub fn with_worktree(container: NormalizedPath, worktree: NormalizedPath) -> Result<Self> {
        let git_dir = container.join(".gt");
        if !git_dir.exists() {
            return Err(Error::Fs(repo_fs::Error::LayoutValidation {
                message: format!(
                    "Not a worktree container: .gt not found at {}",
                    container.as_str()
                ),
            }));
        }

        Ok(Self {
            container,
            current_worktree: worktree,
            git_dir,
        })
    }

    /// Get the container directory.
    pub fn container(&self) -> &NormalizedPath {
        &self.container
    }

    /// Get the git database directory.
    pub fn git_dir(&self) -> &NormalizedPath {
        &self.git_dir
    }

    /// Run a git command from a specific worktree.
    fn git_command_in_worktree(&self, worktree: &NormalizedPath, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(worktree.to_native())
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

    /// Get the main branch name.
    fn main_branch_name(&self) -> String {
        // In container mode, main is typically the default
        "main".to_string()
    }

    /// Get the worktree path for a branch name.
    fn worktree_path(&self, name: &str) -> NormalizedPath {
        self.container.join(name)
    }

    /// Check if a worktree exists for the given branch.
    fn worktree_exists(&self, name: &str) -> bool {
        self.worktree_path(name).exists()
    }

    /// Parse git worktree list output.
    fn parse_worktree_list(&self) -> Result<Vec<(NormalizedPath, String, bool)>> {
        // Use porcelain format for reliable parsing
        let output = self.git_command_in_worktree(
            &self.current_worktree,
            &["worktree", "list", "--porcelain"],
        )?;

        let mut worktrees = Vec::new();
        let mut current_path: Option<NormalizedPath> = None;
        let mut current_branch: Option<String> = None;
        let mut is_bare = false;

        for line in output.lines() {
            if let Some(path_str) = line.strip_prefix("worktree ") {
                // Save previous worktree if any
                if let (Some(path), Some(branch)) = (current_path.take(), current_branch.take())
                    && !is_bare
                {
                    worktrees.push((path, branch, false));
                }
                current_path = Some(NormalizedPath::new(path_str));
                current_branch = None;
                is_bare = false;
            } else if let Some(branch_str) = line.strip_prefix("branch refs/heads/") {
                current_branch = Some(branch_str.to_string());
            } else if line.starts_with("HEAD ") {
                // Detached HEAD, use abbreviated commit
                if current_branch.is_none() {
                    current_branch = Some("HEAD".to_string());
                }
            } else if line == "bare" {
                is_bare = true;
            }
        }

        // Don't forget the last worktree
        if let (Some(path), Some(branch)) = (current_path, current_branch)
            && !is_bare
        {
            worktrees.push((path, branch, false));
        }

        Ok(worktrees)
    }
}

impl ModeBackend for WorktreeBackend {
    fn config_root(&self) -> NormalizedPath {
        // Config is shared at container level
        self.container.join(".repository")
    }

    fn working_dir(&self) -> &NormalizedPath {
        &self.current_worktree
    }

    fn create_branch(&self, name: &str, base: Option<&str>) -> Result<()> {
        // In worktree mode, creating a branch means creating a worktree
        if self.worktree_exists(name) {
            return Err(Error::Git(repo_git::Error::WorktreeExists {
                name: name.to_string(),
                path: self.worktree_path(name).to_native(),
            }));
        }

        let worktree_path = self.worktree_path(name);

        // Create worktree with new branch
        let args = match base {
            Some(base_branch) => vec![
                "worktree",
                "add",
                "-b",
                name,
                worktree_path.as_str(),
                base_branch,
            ],
            None => vec!["worktree", "add", "-b", name, worktree_path.as_str()],
        };

        self.git_command_in_worktree(&self.current_worktree, &args)?;
        Ok(())
    }

    fn delete_branch(&self, name: &str) -> Result<()> {
        let main_branch = self.main_branch_name();
        if name == main_branch {
            return Err(Error::SyncError {
                message: format!("Cannot delete main branch: {}", name),
            });
        }

        let worktree_path = self.worktree_path(name);

        // Check if worktree exists
        if !worktree_path.exists() {
            return Err(Error::Git(repo_git::Error::WorktreeNotFound {
                name: name.to_string(),
            }));
        }

        // Remove the worktree
        self.git_command_in_worktree(
            &self.current_worktree,
            &["worktree", "remove", worktree_path.as_str()],
        )?;

        // Also try to delete the branch
        let _ = self.git_command_in_worktree(&self.current_worktree, &["branch", "-d", name]);

        Ok(())
    }

    fn list_branches(&self) -> Result<Vec<BranchInfo>> {
        let worktrees = self.parse_worktree_list()?;
        let main_branch = self.main_branch_name();

        let branches: Vec<BranchInfo> = worktrees
            .into_iter()
            .map(|(path, branch, _)| {
                let is_main = branch == main_branch;
                let is_current = path.as_str() == self.current_worktree.as_str();
                BranchInfo::worktree(&branch, path, is_current, is_main)
            })
            .collect();

        Ok(branches)
    }

    fn switch_branch(&self, name: &str) -> Result<NormalizedPath> {
        let worktree_path = self.worktree_path(name);

        if worktree_path.exists() {
            // Worktree exists, just return its path
            Ok(worktree_path)
        } else {
            // Need to create the worktree first
            self.create_branch(name, None)?;
            Ok(worktree_path)
        }
    }

    fn rename_branch(&self, old_name: &str, new_name: &str) -> Result<()> {
        let main_branch = self.main_branch_name();
        if old_name == main_branch {
            return Err(Error::SyncError {
                message: format!("Cannot rename main branch: {}", old_name),
            });
        }

        let old_worktree_path = self.worktree_path(old_name);
        if !old_worktree_path.exists() {
            return Err(Error::Git(repo_git::Error::WorktreeNotFound {
                name: old_name.to_string(),
            }));
        }

        let new_worktree_path = self.worktree_path(new_name);

        // Rename the git branch
        self.git_command_in_worktree(
            &self.current_worktree,
            &["branch", "-m", old_name, new_name],
        )?;

        // Move the worktree directory
        self.git_command_in_worktree(
            &self.current_worktree,
            &[
                "worktree",
                "move",
                old_worktree_path.as_str(),
                new_worktree_path.as_str(),
            ],
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_container() -> TempDir {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join(".gt")).unwrap();
        fs::create_dir(dir.path().join("main")).unwrap();
        fs::write(dir.path().join(".gt/HEAD"), "ref: refs/heads/main\n").unwrap();
        fs::create_dir_all(dir.path().join(".gt/refs/heads")).unwrap();
        dir
    }

    #[test]
    fn test_container() {
        let temp = setup_container();
        let container = NormalizedPath::new(temp.path());
        let backend = WorktreeBackend::new(container.clone()).unwrap();

        assert_eq!(backend.container().as_str(), container.as_str());
    }

    #[test]
    fn test_git_dir() {
        let temp = setup_container();
        let container = NormalizedPath::new(temp.path());
        let backend = WorktreeBackend::new(container.clone()).unwrap();

        let expected = container.join(".gt");
        assert_eq!(backend.git_dir().as_str(), expected.as_str());
    }
}
