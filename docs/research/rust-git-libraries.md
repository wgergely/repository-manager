# Rust Git Libraries

Evaluation of git operation libraries for repo-manager.

## Recommendation: gix (gitoxide)

Pure Rust implementation, rapidly maturing, used by cargo.

## Comparison Matrix

| Crate | Implementation | Performance | Safety | Features | Size |
|-------|----------------|-------------|--------|----------|------|
| **gix** | Pure Rust | Excellent | Memory-safe | Growing | Large |
| git2 | libgit2 bindings | Good | C FFI | Complete | Medium |

## gix Advantages

- Memory safe (no C dependencies)
- Excellent performance (often faster than libgit2)
- Cross-compilation friendly
- Active development by Byron (sponsored by GitButler)
- Used in production by cargo

## gix Example

```rust
use gix::{Repository, progress::Discard};
use std::path::Path;

struct GitOperations {
    repo: Repository,
}

impl GitOperations {
    fn open(path: &Path) -> Result<Self> {
        let repo = gix::open(path)?;
        Ok(Self { repo })
    }

    fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>> {
        let worktrees = self.repo.worktrees()?;
        let mut result = Vec::new();

        for wt in worktrees {
            let wt = wt?;
            result.push(WorktreeInfo {
                name: wt.id().to_string(),
                path: wt.base()?.to_path_buf(),
                branch: wt.head_ref()?.map(|r| r.name().to_string()),
                locked: wt.is_locked(),
            });
        }

        Ok(result)
    }

    fn create_worktree(&self, name: &str, branch: &str, path: &Path) -> Result<()> {
        let reference = self.repo.find_reference(branch)?;
        self.repo.worktree_add(name, path, reference)?;
        Ok(())
    }

    fn remove_worktree(&self, name: &str, force: bool) -> Result<()> {
        let worktree = self.repo.find_worktree(name)?;
        if force || !worktree.is_locked() {
            worktree.remove()?;
        }
        Ok(())
    }

    fn clone_bare(url: &str, path: &Path) -> Result<Repository> {
        let mut prepare = gix::prepare_clone_bare(url, path)?;
        let (repo, _) = prepare.fetch_then_checkout(
            Discard,
            &std::sync::atomic::AtomicBool::new(false)
        )?;
        Ok(repo)
    }

    fn is_worktree(&self) -> bool {
        self.repo.is_worktree()
    }
}

#[derive(Debug)]
struct WorktreeInfo {
    name: String,
    path: PathBuf,
    branch: Option<String>,
    locked: bool,
}
```

## git2 Alternative

Use when you need features not yet in gix:

```rust
use git2::{Repository, WorktreePruneOptions};

fn git2_worktree_operations(repo_path: &Path) -> Result<()> {
    let repo = Repository::open(repo_path)?;

    // List worktrees
    let worktrees = repo.worktrees()?;
    for name in worktrees.iter() {
        if let Some(name) = name {
            let wt = repo.find_worktree(name)?;
            println!("Worktree: {} at {:?}", name, wt.path());
        }
    }

    Ok(())
}
```

## Cargo Dependencies

```toml
[dependencies]
gix = { version = "0.60", default-features = false, features = [
    "worktree",
    "clone",
    "status",
    "revision",
    "index",
] }

# Optional fallback
[dependencies.git2]
version = "0.18"
optional = true

[features]
default = []
libgit2 = ["git2"]
```

---

*Last updated: 2026-01-23*
*Status: Complete*
