//! In-repo worktrees layout implementation

use crate::{
    Error, Result,
    naming::{NamingStrategy, branch_to_directory},
    provider::{LayoutProvider, WorktreeInfo},
};
use git2::{BranchType, Repository, WorktreeAddOptions, WorktreePruneOptions};
use repo_fs::NormalizedPath;

/// In-repo worktrees layout with `.worktrees/` directory.
///
/// ```text
/// {repo}/
/// ├── .git/          # Git database
/// ├── .worktrees/    # Worktrees folder
/// │   └── feature-x/
/// └── src/           # Main branch files
/// ```
pub struct InRepoWorktreesLayout {
    root: NormalizedPath,
    git_dir: NormalizedPath,
    worktrees_dir: NormalizedPath,
    naming: NamingStrategy,
}

impl InRepoWorktreesLayout {
    /// Create a new InRepoWorktreesLayout for the given root directory.
    pub fn new(root: NormalizedPath, naming: NamingStrategy) -> Result<Self> {
        let git_dir = root.join(".git");
        let worktrees_dir = root.join(".worktrees");

        Ok(Self {
            root,
            git_dir,
            worktrees_dir,
            naming,
        })
    }

    fn open_repo(&self) -> Result<Repository> {
        Ok(Repository::open(self.root.to_native())?)
    }
}

impl LayoutProvider for InRepoWorktreesLayout {
    fn git_database(&self) -> &NormalizedPath {
        &self.git_dir
    }

    fn main_worktree(&self) -> &NormalizedPath {
        &self.root
    }

    fn feature_worktree(&self, name: &str) -> NormalizedPath {
        let dir_name = branch_to_directory(name, self.naming);
        self.worktrees_dir.join(&dir_name)
    }

    fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>> {
        let repo = self.open_repo()?;

        // Start with main worktree (the repo root)
        let main_branch = self.current_branch().unwrap_or_else(|_| "main".into());
        let mut result = vec![WorktreeInfo {
            name: "main".into(),
            path: self.root.clone(),
            branch: main_branch,
            is_main: true,
        }];

        // Add linked worktrees
        let worktree_names = repo.worktrees()?;
        for name in worktree_names.iter() {
            let name = match name {
                Some(n) => n,
                None => continue,
            };

            let wt = repo.find_worktree(name)?;
            let wt_path = wt.path();

            let wt_repo = Repository::open(wt_path)?;
            let branch = wt_repo
                .head()
                .ok()
                .and_then(|h| h.shorthand().map(String::from))
                .unwrap_or_else(|| "HEAD".into());

            result.push(WorktreeInfo {
                name: name.to_string(),
                path: NormalizedPath::new(wt_path),
                branch,
                is_main: false,
            });
        }

        Ok(result)
    }

    fn create_feature(&self, name: &str, base: Option<&str>) -> Result<NormalizedPath> {
        let repo = self.open_repo()?;
        let worktree_path = self.feature_worktree(name);
        let dir_name = branch_to_directory(name, self.naming);

        // Check if worktree already exists
        if worktree_path.exists() {
            return Err(Error::WorktreeExists {
                name: name.to_string(),
                path: worktree_path.to_native(),
            });
        }

        // Ensure .worktrees directory exists
        std::fs::create_dir_all(self.worktrees_dir.to_native())
            .map_err(|e| Error::Fs(repo_fs::Error::io(self.worktrees_dir.to_native(), e)))?;

        // Get the commit to base the new branch on
        let base_commit = match base {
            Some(base_name) => {
                let branch = repo
                    .find_branch(base_name, BranchType::Local)
                    .map_err(|_| Error::BranchNotFound {
                        name: base_name.to_string(),
                    })?;
                branch.get().peel_to_commit()?
            }
            None => {
                let head = repo.head()?;
                head.peel_to_commit()?
            }
        };

        // Create a new branch for the feature worktree
        let new_branch = repo.branch(&dir_name, &base_commit, false)?;
        let new_branch_ref = new_branch.into_reference();

        // Create worktree with the new branch
        let mut opts = WorktreeAddOptions::new();
        opts.reference(Some(&new_branch_ref));

        repo.worktree(&dir_name, worktree_path.to_native().as_path(), Some(&opts))?;

        Ok(worktree_path)
    }

    fn remove_feature(&self, name: &str) -> Result<()> {
        let repo = self.open_repo()?;
        let dir_name = branch_to_directory(name, self.naming);

        let wt = repo
            .find_worktree(&dir_name)
            .map_err(|_| Error::WorktreeNotFound {
                name: name.to_string(),
            })?;

        // Configure prune options to remove valid worktrees and their directories
        let mut prune_opts = WorktreePruneOptions::new();
        prune_opts.valid(true); // Allow pruning valid (existing) worktrees
        prune_opts.working_tree(true); // Also remove the working tree directory

        // Prune the worktree (removes directory and git references)
        wt.prune(Some(&mut prune_opts))?;

        // Also try to delete the branch
        if let Ok(mut branch) = repo.find_branch(&dir_name, BranchType::Local) {
            let _ = branch.delete();
        }

        Ok(())
    }

    fn current_branch(&self) -> Result<String> {
        let repo = self.open_repo()?;
        let head = repo.head()?;

        if head.is_branch() {
            Ok(head.shorthand().unwrap_or("HEAD").to_string())
        } else {
            Ok("HEAD".to_string())
        }
    }
}
