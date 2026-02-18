//! In-repo worktrees layout implementation

use std::sync::OnceLock;

use crate::{
    Error, Result, helpers,
    naming::{NamingStrategy, branch_to_directory},
    provider::{LayoutProvider, WorktreeInfo},
};
use git2::Repository;
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
    repo_cache: OnceLock<Repository>,
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
            repo_cache: OnceLock::new(),
        })
    }

    /// Open the cached repository handle.
    pub fn open_repo(&self) -> Result<&Repository> {
        if let Some(repo) = self.repo_cache.get() {
            return Ok(repo);
        }
        let repo = Repository::open(self.root.to_native())?;
        let _ = self.repo_cache.set(repo);
        Ok(self.repo_cache.get().expect("just initialized"))
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
        tracing::debug!(name, base, "Creating feature worktree");
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

        helpers::create_worktree_with_branch(
            repo,
            worktree_path.to_native().as_path(),
            &dir_name,
            base,
        )?;

        Ok(worktree_path)
    }

    fn remove_feature(&self, name: &str) -> Result<()> {
        tracing::debug!(name, "Removing feature worktree");
        let repo = self.open_repo()?;
        let dir_name = branch_to_directory(name, self.naming);

        helpers::remove_worktree_and_branch(repo, &dir_name)
    }

    fn current_branch(&self) -> Result<String> {
        let repo = self.open_repo()?;
        helpers::get_current_branch(repo).map(|opt| opt.unwrap_or_else(|| "HEAD".to_string()))
    }
}
