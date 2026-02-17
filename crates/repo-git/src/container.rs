//! Container layout implementation with .gt database

use std::sync::OnceLock;

use crate::{
    Error, Result, helpers,
    naming::{NamingStrategy, branch_to_directory},
    provider::{LayoutProvider, WorktreeInfo},
};
use git2::{BranchType, MergeOptions, Repository};
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

    fn open_repo(&self) -> Result<&Repository> {
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

            let is_main = name == "main" || wt_path.ends_with("main");

            result.push(WorktreeInfo {
                name: name.to_string(),
                path: NormalizedPath::new(wt_path),
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
        helpers::get_current_branch(repo)
    }

    fn push(&self, remote: Option<&str>, branch: Option<&str>) -> Result<()> {
        let repo = self.open_repo()?;
        let remote_name = remote.unwrap_or("origin");
        let branch_name = match branch {
            Some(b) => b.to_string(),
            None => self.current_branch()?,
        };

        let mut remote = repo
            .find_remote(remote_name)
            .map_err(|_| Error::RemoteNotFound {
                name: remote_name.to_string(),
            })?;

        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);

        // Push using default options (relies on credential helpers)
        remote
            .push(&[&refspec], None)
            .map_err(|e| Error::PushFailed {
                message: e.message().to_string(),
            })?;

        Ok(())
    }

    fn pull(&self, remote: Option<&str>, branch: Option<&str>) -> Result<()> {
        let repo = self.open_repo()?;
        let remote_name = remote.unwrap_or("origin");
        let branch_name = match branch {
            Some(b) => b.to_string(),
            None => self.current_branch()?,
        };

        // Fetch from remote
        let mut remote = repo
            .find_remote(remote_name)
            .map_err(|_| Error::RemoteNotFound {
                name: remote_name.to_string(),
            })?;

        remote
            .fetch(&[&branch_name], None, None)
            .map_err(|e| Error::PullFailed {
                message: format!("Fetch failed: {}", e.message()),
            })?;

        // Get FETCH_HEAD
        let fetch_head = repo
            .find_reference("FETCH_HEAD")
            .map_err(|e| Error::PullFailed {
                message: format!("Could not find FETCH_HEAD: {}", e.message()),
            })?;

        let fetch_commit = fetch_head.peel_to_commit().map_err(|e| Error::PullFailed {
            message: format!("Could not resolve FETCH_HEAD: {}", e.message()),
        })?;

        // Get current HEAD commit
        let head = repo.head()?;
        let head_commit = head.peel_to_commit()?;

        // Check if we can fast-forward
        let (merge_analysis, _) =
            repo.merge_analysis(&[&repo.find_annotated_commit(fetch_commit.id())?])?;

        if merge_analysis.is_up_to_date() {
            // Already up to date
            return Ok(());
        }

        if merge_analysis.is_fast_forward() {
            // Fast-forward merge
            let refname = format!("refs/heads/{}", branch_name);
            let mut reference = repo.find_reference(&refname)?;
            reference.set_target(
                fetch_commit.id(),
                &format!("pull: fast-forward to {}", fetch_commit.id()),
            )?;

            // Update working directory in main worktree
            let main_repo = Repository::open(self.main_dir.to_native())?;
            main_repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
            return Ok(());
        }

        // Cannot fast-forward
        Err(Error::CannotFastForward {
            message: format!(
                "Cannot fast-forward {} from {} to {}. Manual merge required.",
                branch_name,
                head_commit.id(),
                fetch_commit.id()
            ),
        })
    }

    fn merge(&self, source: &str) -> Result<()> {
        let repo = self.open_repo()?;

        // Find the source branch
        let source_branch =
            repo.find_branch(source, BranchType::Local)
                .map_err(|_| Error::BranchNotFound {
                    name: source.to_string(),
                })?;

        let source_commit = source_branch.get().peel_to_commit()?;
        let annotated_commit = repo.find_annotated_commit(source_commit.id())?;

        // Analyze what kind of merge we can do
        let (merge_analysis, _) = repo.merge_analysis(&[&annotated_commit])?;

        if merge_analysis.is_up_to_date() {
            // Nothing to do
            return Ok(());
        }

        if merge_analysis.is_fast_forward() {
            // Fast-forward merge
            let current_branch = self.current_branch()?;
            let refname = format!("refs/heads/{}", current_branch);
            let mut reference = repo.find_reference(&refname)?;
            reference.set_target(
                source_commit.id(),
                &format!("merge {}: fast-forward", source),
            )?;

            // Update working directory in main worktree
            let main_repo = Repository::open(self.main_dir.to_native())?;
            main_repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
            return Ok(());
        }

        // Normal merge required - need to work in the main worktree for merge operations
        let main_repo = Repository::open(self.main_dir.to_native())?;

        let mut merge_opts = MergeOptions::new();
        main_repo.merge(
            &[&main_repo.find_annotated_commit(source_commit.id())?],
            Some(&mut merge_opts),
            None,
        )?;

        // Check for conflicts
        let mut index = main_repo.index()?;
        if index.has_conflicts() {
            // Clean up merge state
            main_repo.cleanup_state()?;
            return Err(Error::MergeConflict {
                message: format!("Merge of '{}' resulted in conflicts", source),
            });
        }

        // Create merge commit
        let signature = main_repo.signature()?;
        let tree_id = index.write_tree()?;
        let tree = main_repo.find_tree(tree_id)?;
        let head_commit = main_repo.head()?.peel_to_commit()?;
        let source_commit_in_main = main_repo.find_commit(source_commit.id())?;

        let message = format!("Merge branch '{}'", source);
        main_repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &message,
            &tree,
            &[&head_commit, &source_commit_in_main],
        )?;

        // Clean up merge state
        main_repo.cleanup_state()?;

        Ok(())
    }
}
