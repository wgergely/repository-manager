//! Container layout implementation with .gt database

use std::sync::OnceLock;

use crate::{
    Error, Result, helpers,
    naming::{NamingStrategy, branch_to_directory},
    provider::{LayoutProvider, WorktreeInfo},
};
use git2::Repository;
use repo_fs::NormalizedPath;

/// Container layout with `.gt/` database and sibling worktrees.
///
/// ```text
/// {container}/
/// ├── .gt/          # Git database
/// ├── main/         # Main branch worktree
/// └── feature-x/    # Feature worktree
/// ```
pub struct ContainerLayout {
    root: NormalizedPath,
    git_dir: NormalizedPath,
    main_dir: NormalizedPath,
    naming: NamingStrategy,
    repo_cache: OnceLock<Repository>,
}

impl ContainerLayout {
    /// Create a new ContainerLayout for the given root directory.
    pub fn new(root: NormalizedPath, naming: NamingStrategy) -> Result<Self> {
        let git_dir = root.join(".gt");
        let main_dir = root.join("main");

        Ok(Self {
            root,
            git_dir,
            main_dir,
            naming,
            repo_cache: OnceLock::new(),
        })
    }

    /// Open the cached repository handle.
    pub fn open_repo(&self) -> Result<&Repository> {
        if let Some(repo) = self.repo_cache.get() {
            return Ok(repo);
        }
        let repo = Repository::open(self.git_dir.to_native())?;
        // OnceLock::set is thread-safe; if another thread won the race, use their value
        let _ = self.repo_cache.set(repo);
        Ok(self.repo_cache.get().expect("just initialized"))
    }
}

impl LayoutProvider for ContainerLayout {
    fn git_database(&self) -> &NormalizedPath {
        &self.git_dir
    }

    fn main_worktree(&self) -> &NormalizedPath {
        &self.main_dir
    }

    fn feature_worktree(&self, name: &str) -> NormalizedPath {
        let dir_name = branch_to_directory(name, self.naming);
        self.root.join(&dir_name)
    }

    fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>> {
        let repo = self.open_repo()?;
        let worktree_names = repo.worktrees()?;

        let mut result = Vec::new();

        for name in worktree_names.iter() {
            let name = match name {
                Some(n) => n,
                None => continue,
            };

            let wt = repo.find_worktree(name)?;
            let wt_path = wt.path();

            // Get branch for this worktree
            let wt_repo = Repository::open(wt_path)?;
            let branch = wt_repo
                .head()
                .ok()
                .and_then(|h| h.shorthand().map(String::from))
                .unwrap_or_else(|| "HEAD".into());

            // Compare against self.main_dir instead of checking name == "main"
            let wt_normalized = NormalizedPath::new(wt_path);
            let is_main = wt_normalized.as_str() == self.main_dir.as_str();

            result.push(WorktreeInfo {
                name: name.to_string(),
                path: wt_normalized,
                branch,
                is_main,
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
