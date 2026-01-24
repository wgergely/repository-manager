# Robustness Audit: `repo-git`

**Date:** 2026-01-23
**Auditor:** Gemini

## 1. Introduction

This document presents the findings of a robustness audit of the `repo-git` crate. The audit focused on identifying potential issues related to error handling, edge cases, race conditions, and interactions with the filesystem and the `git2` library. The goal is to ensure the crate behaves predictably and safely under non-ideal conditions.

## 2. Methodology

The audit was conducted via a manual source code review of the `repo-git` crate's `src` directory. The review focused on:

- **Error Handling:** How `git2` and `repo_fs` errors are propagated and handled.
- **Resource Management:** Cleanup of worktrees, branches, and filesystem directories.
- **Edge Cases:** Behavior with unusual but valid inputs (e.g., branch names with special characters).
- **Concurrency:** Potential for race conditions during filesystem operations.
- **External Dependencies:** Resilience to corruption or unexpected states in the underlying Git repository.

## 3. Findings

The `repo-git` crate is generally well-structured, with a clear separation of concerns and a good error-handling foundation. However, several areas could be improved to increase its robustness.

### 3.1. Important Findings

#### 3.1.1. TOCTOU Race Condition in `create_feature`

- **Severity:** Important
- **Locations:** `in_repo_worktrees.rs`, `container.rs`

**Description:**
Both `InRepoWorktreesLayout::create_feature` and `ContainerLayout::create_feature` implement a "check-then-act" pattern that is vulnerable to a Time-of-check to time-of-use (TOCTOU) race condition.

```rust
// Simplified example from both layouts
if worktree_path.exists() {
    return Err(Error::WorktreeExists { ... });
}

// ... some time later ...
repo.worktree(&dir_name, worktree_path.to_native().as_path(), ...)?;
```

If another process or thread creates a file or directory at `worktree_path` after the `exists()` check but before `repo.worktree()` is called, the behavior of `git2`'s `worktree()` function is not well-defined in the context of this crate. It may fail, but the error returned might be a generic `git2::Error` instead of the more specific `Error::WorktreeExists`.

**Recommendation:**
While completely eliminating this race condition is difficult without filesystem-level transactions, the operation can be made more robust. The `git2::worktree` function should be treated as the atomic operation. The code should attempt the creation and then handle the specific error that `git2` returns if the path already exists. This turns the "check-then-act" into an "act-then-handle" pattern, which is more robust. The initial check can be kept as a fast-path to provide a better error message, but the code should not assume it is sufficient.

#### 3.1.2. Incomplete Cleanup on `remove_feature` Failure

- **Severity:** Important
- **Locations:** `in_repo_worktrees.rs`, `container.rs`

**Description:**
The `remove_feature` function correctly prunes the git worktree but makes a best-effort attempt to delete the associated git branch, explicitly ignoring any errors.

```rust
// From InRepoWorktreesLayout::remove_feature
if let Ok(mut branch) = repo.find_branch(&dir_name, BranchType::Local) {
    let _ = branch.delete(); // Error is ignored
}
```

If `branch.delete()` fails (e.g., the branch contains unmerged commits), the branch will be left behind while the worktree directory is gone. This can lead to a cluttered repository state and confuse users who expect a clean removal.

**Recommendation:**
Instead of ignoring the error, it should be captured. If deleting the branch fails, the function should return an error or at least log a warning. A more advanced solution would be to return a compound result, indicating that the worktree was removed but the branch could not be. For example, `Ok(Some(Warning::BranchNotDeleted))` or a similar pattern.

### 3.2. Minor Findings & Defensive Programming

#### 3.2.1. Handling of Corrupted Git Repositories

- **Severity:** Minor
- **Location:** All layout implementations.

**Description:**
The crate relies on `git2::Repository::open` and other `git2` functions. If the underlying git repository is corrupted, these functions may panic or return opaque errors. The current error handling wraps these in `Error::Git`, which is reasonable, but it provides no specific guidance to the user on how to resolve the issue.

**Recommendation:**
While the crate cannot be expected to fix a corrupted repository, the documentation for functions that interact with `git2` could mention that repository corruption can lead to `Error::Git` and that running `git fsck` might be a necessary diagnostic step for the user. This adds a layer of user-friendliness for difficult-to-debug situations.

#### 3.2.2. Robustness of Branch Name Sanitization

- **Severity:** Minor
- **Location:** `naming.rs`

**Description:**
The `slugify` and `sanitize_hierarchical` functions are responsible for cleaning branch names for use as directory names. While they handle common cases, they have not been exhaustively tested against malicious or unusual inputs:
- **Path Traversal:** `sanitize_hierarchical` does not explicitly handle `.` or `..` path components. While the `NormalizedPath` type used elsewhere should prevent traversal, the sanitization function itself could be made more robust by handling these cases explicitly.
- **Reserved Names:** The functions do not appear to check for reserved filesystem names (e.g., `CON`, `NUL`, `PRN` on Windows). Creating a worktree directory with such a name could fail or lead to unexpected behavior.

**Recommendation:**
- Add explicit handling for `.` and `..` components in `sanitize_hierarchical`.
- Consider adding a check for common reserved filenames, at least on Windows, if that is a target platform.
- Expand test cases to include more complex and potentially malicious inputs.

#### 3.2.3. Inefficient and Silent Failures in `list_worktrees`

- **Severity:** Minor
- **Locations:** `in_repo_worktrees.rs`, `container.rs`

**Description:**
The `list_worktrees` function iterates through worktree names and then opens each worktree's path as a new `Repository` instance (`Repository::open(wt_path)`). This is inefficient as it involves repeated disk I/O. Furthermore, if opening a specific worktree fails, the loop continues to the next, meaning the final list might be silently incomplete.

**Recommendation:**
- Investigate if `git2` provides a more direct way to get worktree metadata (like the checked-out branch) without fully opening a new repository instance for each one.
- If an error occurs when inspecting a single worktree, it should be logged or collected. The function could return a `Result<Vec<Result<WorktreeInfo>>>` or a struct containing both the successful results and any errors that occurred.

## 4. Conclusion

The `repo-git` crate provides a solid abstraction for managing git worktrees. Its use of specific error types and separation of layouts are significant strengths.

The most critical improvements relate to handling race conditions in `create_feature` and providing more robust cleanup in `remove_feature`. Addressing these issues will significantly improve the predictability and reliability of the crate. The minor findings represent opportunities for defensive programming that will make the crate more resilient to unexpected states and inputs.

Overall, the crate is in a good state, and the recommended changes would move it from a solid implementation to a truly robust and production-ready component.