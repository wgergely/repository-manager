# Rust Git Libraries

Evaluation of git operation libraries for repo-manager.

## Recommendation: git2

Battle-tested libgit2 bindings with complete feature coverage.

**Alternative**: `gix` (gitoxide) for pure-Rust preference or when avoiding C dependencies.

## Comparison Matrix

| Crate | Implementation | Performance | Safety | Features | Size |
|-------|----------------|-------------|--------|----------|------|
| **git2** | libgit2 bindings | Good | C FFI | Complete | Medium |
| gix | Pure Rust | Excellent | Memory-safe | Growing | Large |

## git2 Advantages

- Complete feature coverage (worktrees, remotes, all git operations)
- Battle-tested in production (used by cargo historically)
- Stable API with excellent documentation
- Wide ecosystem support and examples

## gix Advantages (Alternative)

- Memory safe (no C dependencies)
- Cross-compilation friendly
- Active development by Byron (sponsored by GitButler)
- Used in production by cargo

## Code Examples

Either library can be used. Examples show typical patterns:

```rust
// git2 (recommended) - worktree operations
use git2::Repository;

let repo = Repository::open(path)?;
let worktrees = repo.worktrees()?;
for name in worktrees.iter().flatten() {
    let wt = repo.find_worktree(name)?;
    println!("Worktree: {} at {:?}", name, wt.path());
}

// gix alternative - similar pattern
let repo = gix::open(path)?;
let worktrees = repo.worktrees()?;
```

## Cargo Dependencies

```toml
[dependencies]
git2 = "0.19"

# Optional pure-Rust alternative
[dependencies.gix]
version = "0.60"
optional = true
default-features = false
features = ["worktree", "clone", "status"]

[features]
default = []
pure-rust = ["gix"]
```

---

*Last updated: 2026-01-23*
*Status: Complete*
