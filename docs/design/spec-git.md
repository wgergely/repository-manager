# Git Management Subsystem Specification

**Crate**: `repo-git`

## 1. Overview

The Git Management subsystem creates an abstraction layer over raw Git operations. It decouples the *intent* (e.g., "Start working on a new feature") from the *mechanism* (e.g., `git checkout -b` vs. `git worktree add`).

## 2. Architecture

### 2.1 The `GitProvider` Trait

We define a core trait that both `StandardProvider` and `WorktreeProvider` must implement.

```rust
pub trait GitProvider {
    /// Get the current branch name
    fn current_branch(&self) -> Result<String>;

    /// Create a new workspace for a feature
    fn create_feature(&self, name: &str, base: Option<&str>) -> Result<WorkspacePath>;

    /// Switch context to an existing branch
    fn switch_to(&self, branch: &str) -> Result<WorkspacePath>;

    /// Publish the current branch to origin
    fn publish(&self) -> Result<()>;

    /// Clean up a feature branch and its workspace
    fn delete_feature(&self, name: &str) -> Result<()>;
}
```

## 3. Implementations

### 3.1 `StandardProvider`

* **Maps to**: Classic Git flow.
* **`create_feature`**: Runs `git checkout -b {name} {base}`. Returns the current directory.
* **`switch_to`**: Runs `git checkout {name}`.
* **`delete_feature`**: Runs `git branch -D {name}`.

### 3.2 `WorktreeProvider`

* **Maps to**: Parallel directory development.
* **`create_feature`**:
    1. Determines target logical name (e.g., `feat/login`).
    2. Determines physical path (e.g., `{root}/feat-login` or `{root}/worktrees/feat-login`).
    3. Runs `git worktree add {path} {name}`.
    4. **Critical Step**: Bootstraps the new directory (copying `.vscode`, `.env` if needed - delegated to `repo-presets`).
* **`delete_feature`**:
    1. Runs `git worktree remove {path}`.
    2. Runs `git branch -D {name}`.
    3. Cleans up any residual directory artifacts.

## 4. Worktree Naming Strategy

To avoid directory sprawl, the Worktree Provider enforces a naming schema:

* **Main Branch**: Lives in `{root}/main` or `{root}/trunk`.
* **Feature Branches**: Lives in `{root}/{branch_slug}`.
* **Heuristics**: The provider must handle branch names with slashes `feat/user-auth` -> `feat-user-auth` for directory safety.

## 5. Remote Syncing

The subsystem handles the complexity of "Pushing from a worktree".

* In most modern Git, `git push` works fine from a worktree.
* **Challenge**: Upstream tracking. The provider must ensure `git push -u origin {branch}` is automatically configured.

## 6. Dependencies

* **`git2` Crate**: Used for high-performance status checks and read operations.
* **`process::Command`**: Used for complex write operations (checkout, worktree) where the C implementation might be too low-level or ensuring CLI compatibility is safer.
