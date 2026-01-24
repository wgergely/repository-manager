//! Classic single-checkout layout implementation

use crate::{
    Error, Result,
    provider::{LayoutProvider, WorktreeInfo},
};
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
        let repo = git2::Repository::open(self.git_dir.to_native())?;
        let head = repo.head()?;

        if head.is_branch() {
            Ok(head.shorthand().unwrap_or("HEAD").to_string())
        } else {
            // Detached HEAD
            Ok("HEAD".to_string())
        }
    }
}
