//! Classic single-checkout layout implementation

use crate::{
    Error, Result,
    provider::{LayoutProvider, WorktreeInfo},
};
use git2::{BranchType, MergeOptions, Repository};
use repo_fs::NormalizedPath;

/// Classic single-checkout git repository layout.
///
/// Does not support parallel worktrees. Feature operations
/// return errors with migration guidance.
pub struct ClassicLayout {
    root: NormalizedPath,
    git_dir: NormalizedPath,
}

impl ClassicLayout {
    /// Create a new ClassicLayout for the given root directory.
    pub fn new(root: NormalizedPath) -> Result<Self> {
        let git_dir = root.join(".git");
        if !git_dir.exists() {
            return Err(Error::Fs(repo_fs::Error::LayoutValidation {
                message: "Not a git repository: .git not found".into(),
            }));
        }
        Ok(Self { root, git_dir })
    }
}

impl LayoutProvider for ClassicLayout {
    fn git_database(&self) -> &NormalizedPath {
        &self.git_dir
    }

    fn main_worktree(&self) -> &NormalizedPath {
        &self.root
    }

    fn feature_worktree(&self, name: &str) -> NormalizedPath {
        // Classic layout doesn't have feature worktrees,
        // but we return a hypothetical path for error messages
        self.root.join(name)
    }

    fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>> {
        // Classic layout only has one "worktree" - the repo itself
        let branch = self.current_branch().unwrap_or_else(|_| "unknown".into());
        Ok(vec![WorktreeInfo {
            name: "main".into(),
            path: self.root.clone(),
            branch,
            is_main: true,
        }])
    }

    fn create_feature(&self, _name: &str, _base: Option<&str>) -> Result<NormalizedPath> {
        Err(Error::LayoutUnsupported {
            operation: "create_feature".into(),
            layout: "Classic".into(),
            hint: "Run `repo migrate --layout in-repo-worktrees` to enable parallel worktrees."
                .into(),
        })
    }

    fn remove_feature(&self, _name: &str) -> Result<()> {
        Err(Error::LayoutUnsupported {
            operation: "remove_feature".into(),
            layout: "Classic".into(),
            hint: "Run `repo migrate --layout in-repo-worktrees` to enable parallel worktrees."
                .into(),
        })
    }

    fn current_branch(&self) -> Result<String> {
        let repo = Repository::open(self.git_dir.to_native())?;
        let head = repo.head()?;

        if head.is_branch() {
            Ok(head.shorthand().unwrap_or("HEAD").to_string())
        } else {
            // Detached HEAD
            Ok("HEAD".to_string())
        }
    }

    fn push(&self, remote: Option<&str>, branch: Option<&str>) -> Result<()> {
        let repo = Repository::open(self.root.to_native())?;
        let remote_name = remote.unwrap_or("origin");
        let branch_name = match branch {
            Some(b) => b.to_string(),
            None => self.current_branch()?,
        };

        let mut remote = repo.find_remote(remote_name).map_err(|_| Error::RemoteNotFound {
            name: remote_name.to_string(),
        })?;

        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);

        // Push using default options (relies on credential helpers)
        remote.push(&[&refspec], None).map_err(|e| Error::PushFailed {
            message: e.message().to_string(),
        })?;

        Ok(())
    }

    fn pull(&self, remote: Option<&str>, branch: Option<&str>) -> Result<()> {
        let repo = Repository::open(self.root.to_native())?;
        let remote_name = remote.unwrap_or("origin");
        let branch_name = match branch {
            Some(b) => b.to_string(),
            None => self.current_branch()?,
        };

        // Fetch from remote
        let mut remote = repo.find_remote(remote_name).map_err(|_| Error::RemoteNotFound {
            name: remote_name.to_string(),
        })?;

        remote
            .fetch(&[&branch_name], None, None)
            .map_err(|e| Error::PullFailed {
                message: format!("Fetch failed: {}", e.message()),
            })?;

        // Get FETCH_HEAD
        let fetch_head = repo.find_reference("FETCH_HEAD").map_err(|e| Error::PullFailed {
            message: format!("Could not find FETCH_HEAD: {}", e.message()),
        })?;

        let fetch_commit = fetch_head.peel_to_commit().map_err(|e| Error::PullFailed {
            message: format!("Could not resolve FETCH_HEAD: {}", e.message()),
        })?;

        // Get current HEAD commit
        let head = repo.head()?;
        let head_commit = head.peel_to_commit()?;

        // Check if we can fast-forward
        let (merge_analysis, _) = repo.merge_analysis(&[&repo.find_annotated_commit(fetch_commit.id())?])?;

        if merge_analysis.is_up_to_date() {
            // Already up to date
            return Ok(());
        }

        if merge_analysis.is_fast_forward() {
            // Fast-forward merge
            let refname = format!("refs/heads/{}", branch_name);
            let mut reference = repo.find_reference(&refname)?;
            reference.set_target(fetch_commit.id(), &format!("pull: fast-forward to {}", fetch_commit.id()))?;

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
        let repo = Repository::open(self.root.to_native())?;

        // Find the source branch
        let source_branch = repo
            .find_branch(source, BranchType::Local)
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
            reference.set_target(source_commit.id(), &format!("merge {}: fast-forward", source))?;

            // Update working directory
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
            return Ok(());
        }

        // Normal merge required
        let mut merge_opts = MergeOptions::new();
        repo.merge(&[&annotated_commit], Some(&mut merge_opts), None)?;

        // Check for conflicts
        let mut index = repo.index()?;
        if index.has_conflicts() {
            // Clean up merge state
            repo.cleanup_state()?;
            return Err(Error::MergeConflict {
                message: format!("Merge of '{}' resulted in conflicts", source),
            });
        }

        // Create merge commit
        let signature = repo.signature()?;
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
