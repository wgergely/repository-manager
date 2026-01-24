# `repo-git` Performance Audit

**Date:** 2026-01-23
**Author:** Gemini Code Assist

## 1. Overview

This document provides a performance audit of the `repo-git` crate. The goal is to identify potential performance bottlenecks, assess the efficiency of its git operations, and provide recommendations for improvement.

## 2. Methodology

The audit was conducted through a static analysis of the source code in the `crates/repo-git/src` directory and a review of the existing benchmark suite in `crates/repo-git/benches`. No new performance tests were executed; the findings are based on code inspection and known performance characteristics of the `git2` library and underlying Git operations.

## 3. Findings

### Finding 1: Inefficient Worktree Listing (Critical)

The `list_worktrees` function in both `ContainerLayout` and `InRepoWorktreesLayout` exhibits a significant performance anti-pattern.

- **Observation:** The implementation iterates through the list of worktree names provided by `repo.worktrees()`. Within the loop, it calls `Repository::open(wt_path)` for *each individual worktree*.
- **Impact:** Opening a `git2::Repository` object is a relatively expensive operation. It involves locating the git directory, reading its configuration, and parsing object files. This creates an "N+1" problem, where listing N worktrees results in N+1 repository open operations (one for the main repo, and one for each linked worktree). The performance of this operation will degrade linearly as the number of feature worktrees grows.
- **Severity:** Critical. This is the most significant performance issue identified.

### Finding 2: I/O Intensive Feature Management (High)

The core operations for managing features, `create_feature` and `remove_feature`, are fundamentally I/O-bound.

- **Observation:**
    - `create_feature` uses `git2::Repository::worktree` to create a new worktree. This involves checking out files to the working directory, which is equivalent to a `git worktree add` command. The cost is proportional to the size of the index and the number of files to be checked out.
    - `remove_feature` uses `git2::Worktree::prune`, which deletes the entire worktree directory from the filesystem.
- **Impact:** The performance of these operations is directly tied to disk I/O speed. For large repositories with many files, or for worktrees containing large, untracked build artifacts, these operations can be slow. While the `git2` implementation is correct, this inherent cost is a major factor in the user-perceived performance of the tool.
- **Severity:** High. While the implementation is not "wrong," it's a primary source of potential slowness.

### Finding 3: Insufficient Benchmark Coverage (Medium)

The existing benchmark suite is minimal and does not provide a comprehensive view of the crate's performance.

- **Observation:** The suite contains a single benchmark for `ContainerLayout::create_feature` on a newly initialized, empty repository.
- **Impact:** There are no benchmarks for:
    - The `list_worktrees` operation, which is the most critical bottleneck found.
    - The `remove_feature` operation.
    - The `InRepoWorktreesLayout`.
    - Scenarios with a large number of files or a large number of existing worktrees.
- **Severity:** Medium. The lack of benchmarks prevents proactive performance monitoring and makes it difficult to validate the impact of optimizations.

## 4. Recommendations

### Recommendation 1: Optimize `list_worktrees`

The `list_worktrees` implementation should be refactored to avoid opening a repository for each worktree.

- **Proposed Solution:** The branch information for a worktree can be retrieved without fully opening a `Repository` object. The `HEAD` of a linked worktree is typically stored in a file within the main git database (e.g., `.git/worktrees/<name>/HEAD`). A more efficient approach would be:
    1.  Use `repo.worktrees()` to get the list of worktree names.
    2.  For each worktree `wt` returned by `repo.find_worktree(name)`, use `wt.path()` to get its location.
    3.  Instead of opening the path, inspect the worktree's metadata directly from the main `Repository` object. `git2` should provide a way to get the ref (`HEAD`) associated with a worktree. A direct file read of `wt.path().join("HEAD")` could also be a much cheaper, albeit more manual, alternative if `git2` does not expose this easily.

- **Example (Conceptual):**
  ```rust,ignore
  // Inside list_worktrees...
  let repo = self.open_repo()?;
  let worktree_names = repo.worktrees()?;

  for name_ref in worktree_names.iter() {
      if let Some(name) = name_ref {
          let wt = repo.find_worktree(name)?;
          let wt_repo = Repository::open_from_worktree(&wt)?; // Hypothetical efficient method
          
          // Or, even better, get branch without opening a new repo object
          let head_ref = wt.head()?;
          let branch = head_ref.shorthand().unwrap_or("HEAD").to_string();
          // ...
      }
  }
  ```

### Recommendation 2: Expand the Benchmark Suite

A comprehensive set of benchmarks should be created to accurately measure and track performance.

- **Action Items:**
    1.  **Benchmark `list_worktrees`:** Create a benchmark that measures the time to list worktrees, and test it with 1, 10, 50, and 100 worktrees.
    2.  **Benchmark `remove_feature`:** Add a benchmark for the feature removal operation.
    3.  **Cover All Layouts:** Implement benchmarks for `InRepoWorktreesLayout` in addition to `ContainerLayout`.
    4.  **Simulate Realistic Repositories:** Create test setups with repositories of varying sizes (e.g., 100 files, 10,000 files) to measure performance under more realistic conditions.

### Recommendation 3: Investigate `git2` Object Caching

- **Suggestion:** For operations that happen in quick succession, investigate if the main `Repository` object can be cached or kept alive rather than being reopened in every function call (`self.open_repo()?`). While `git2` performs some internal caching, avoiding the `Repository::open` call entirely can save time. This may be an application-level optimization but is worth considering in the design of the consumer of the `repo-git` crate.