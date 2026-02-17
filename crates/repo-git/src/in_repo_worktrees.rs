//! In-repo worktrees layout implementation

use crate::{
    Error, Result, helpers,
    naming::{NamingStrategy, branch_to_directory},
    provider::{LayoutProvider, WorktreeInfo},
};
use git2::{BranchType, MergeOptions, Repository};
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
            &repo,
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

        helpers::remove_worktree_and_branch(&repo, &dir_name)
    }

    fn current_branch(&self) -> Result<String> {
        let repo = self.open_repo()?;
        helpers::get_current_branch(&repo)
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

            // Update working directory
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
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

            // Update working directory
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
            return Ok(());
        }

        // Normal merge required
        let mut merge_opts = MergeOptions::new();
        repo.merge(&[&annotated_commit], Some(&mut merge_opts), None)?;

        // Check for conflicts
        let index = repo.index()?;
        if index.has_conflicts() {
            // Clean up merge state
            repo.cleanup_state()?;
            return Err(Error::MergeConflict {
                message: format!("Merge of '{}' resulted in conflicts", source),
            });
        }

        // Create merge commit
        let signature = repo.signature()?;
        let mut index = repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let head_commit = repo.head()?.peel_to_commit()?;

        let message = format!("Merge branch '{}'", source);
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &message,
            &tree,
            &[&head_commit, &source_commit],
        )?;

        // Clean up merge state
        repo.cleanup_state()?;

        Ok(())
    }
}
