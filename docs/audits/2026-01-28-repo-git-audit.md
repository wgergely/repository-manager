# repo-git Crate Audit - 2026-01-28

## Executive Summary

This audit is a follow-up to the previous security and robustness audit conducted on 2026-01-23. Since the last audit, the crate has undergone significant development with 17 commits, including a major refactoring to extract shared worktree helpers and the addition of push/pull/merge operations.

**Overall Assessment:** APPROVED with minor recommendations

The crate maintains its strong security posture with zero `unsafe` code blocks in the repo-git source. The codebase demonstrates mature error handling patterns, proper use of thread-safe primitives (OnceLock), and comprehensive test coverage. Previous audit concerns around incomplete cleanup have been addressed through improved logging. The TOCTOU race condition in `create_feature` remains as an accepted design trade-off.

Key improvements since last audit:
- Shared helpers extracted to `helpers.rs` reducing code duplication
- Thread-safe repository caching via `OnceLock` in `ContainerLayout`
- Branch deletion failures now logged with `tracing::warn`
- Unicode/emoji branch name handling tested and working

## Changes Since Last Audit

### Commits Since 2026-01-23 (17 total)

| Commit | Description |
|--------|-------------|
| `e3358ba` | feat: close 9 implementation gaps via parallel background agents |
| `eed912e` | refactor(repo-git): extract shared worktree helpers |
| `2511f50` | style: fix clippy warnings |
| `ce32ac8` | Merge feature/rust-core: Layer 0 implementation complete |
| `735e70d` | feat: finalize Layer 0 implementation with robustness improvements |
| `da1b77b` | fix(repo-git): use OnceLock for thread-safe repo caching |
| `c3bfee1` | chore: apply formatting and clippy fixes |
| `94ec572` | feat(repo-git): robust audit, optimization, and cleanup |
| `cb8bfe5` | chore: final cleanup with comprehensive test suite |
| `9c1aec5` | feat(repo-git): finalize module exports |
| `2406743` | feat(repo-git): implement InRepoWorktreesLayout |
| `ed90cf5` | feat(repo-git): implement ContainerLayout with git2 worktrees |
| `558566c` | feat(repo-git): implement ClassicLayout with migration hints |
| `90128a5` | feat(repo-git): define LayoutProvider trait and WorktreeInfo |
| `2bf57a3` | feat(repo-git): implement branch naming strategies |
| `1039c2c` | feat(repo-git): add error types |
| `8be7fc5` | chore: initialize cargo workspace with repo-fs and repo-git crates |

### New Files Since Last Audit

- `src/helpers.rs` - Shared worktree helper functions (NEW)
- `tests/robustness_unicode.rs` - Unicode/emoji branch name tests (NEW)
- `tests/git_operations_tests.rs` - Push/pull/merge operation tests (NEW)

### Modified Files

- `src/container.rs` - Added `OnceLock` for thread-safe caching, delegated to helpers
- `src/in_repo_worktrees.rs` - Delegated worktree operations to helpers
- `src/lib.rs` - Added new public exports for helpers

## Previous Issues Status

### Issue 1: TOCTOU Race Condition in `create_feature`
**Status:** ACKNOWLEDGED - Design Trade-off

The check-then-act pattern remains in both `InRepoWorktreesLayout` and `ContainerLayout`:

```rust
// container.rs lines 112-118
if worktree_path.exists() {
    return Err(Error::WorktreeExists {
        name: name.to_string(),
        path: worktree_path.to_native(),
    });
}
```

**Assessment:** This is now considered an acceptable trade-off. The pre-check provides user-friendly error messages. The underlying `git2::worktree()` call will still fail safely if a race occurs, returning a generic `git2::Error`. The window for race conditions is minimal in typical single-user workflows.

### Issue 2: Incomplete Cleanup on `remove_feature` Failure
**Status:** IMPROVED

Branch deletion failures are now logged with `tracing::warn` instead of being silently ignored:

```rust
// helpers.rs lines 81-89
if let Ok(mut branch) = repo.find_branch(name, BranchType::Local)
    && let Err(e) = branch.delete()
{
    tracing::warn!(
        branch = %name,
        error = %e,
        "Failed to delete branch after worktree removal"
    );
}
```

**Assessment:** Users are now informed of partial cleanup. Consider exposing this as a return value in a future version for programmatic handling.

### Issue 3: Dependency Audit (`git2`/`libgit2`)
**Status:** PENDING

The recommendation to run `cargo audit` remains valid. The `git2` crate continues to be a transitive source of `unsafe` code through `libgit2`.

**Dependencies (from Cargo.toml):**
- `git2` (workspace) - FFI wrapper around libgit2
- `thiserror` (workspace) - Safe derive macros
- `tracing` (workspace) - Safe logging
- `repo-fs` (local) - Filesystem abstractions

## New Findings

### Security

#### SEC-1: No Command Injection Vectors
**Severity:** Informational (Positive)

The production code (`src/*.rs`) does not use `std::process::Command`. All git operations are performed through the `git2` crate's type-safe API, eliminating shell injection vulnerabilities.

Test files (`tests/*.rs`) do use `Command::new("git")` for test setup, but:
- Arguments are string literals or controlled variables
- Tests run in isolated temp directories
- No user input is passed to commands

**Assessment:** No security concerns.

#### SEC-2: Branch Name Sanitization
**Severity:** Low

The `naming.rs` module sanitizes branch names for filesystem safety:

```rust
// naming.rs - slugify function
for c in branch.chars() {
    if c.is_alphanumeric() || c == '-' || c == '_' {
        // safe characters preserved
    } else {
        // replaced with dash
    }
}
```

**Positive findings:**
- Path traversal characters (`/`, `..`) are handled
- Unicode alphanumeric characters are preserved safely
- Leading/trailing dashes are stripped
- Multiple consecutive dashes are collapsed

**Gaps:**
- Windows reserved names (CON, NUL, PRN, etc.) are not explicitly blocked
- Empty strings after sanitization are not handled

**Recommendation:** Add validation to reject or transform Windows reserved names when targeting Windows platforms.

#### SEC-3: No Credential Handling
**Severity:** Informational

Push/pull operations rely on git credential helpers configured in the system:

```rust
// classic.rs line 102-103
// Push using default options (relies on credential helpers)
remote.push(&[&refspec], None).map_err(|e| Error::PushFailed { ... })?;
```

**Assessment:** This is the correct approach. The crate does not handle or store credentials directly, delegating to the user's configured credential helper.

### Performance

#### PERF-1: Thread-Safe Repository Caching
**Severity:** Informational (Positive)

`ContainerLayout` now uses `OnceLock` for thread-safe, lazy repository initialization:

```rust
// container.rs lines 27-28, 45-53
repo_cache: OnceLock<Repository>,

fn open_repo(&self) -> Result<&Repository> {
    if let Some(repo) = self.repo_cache.get() {
        return Ok(repo);
    }
    let repo = Repository::open(self.git_dir.to_native())?;
    let _ = self.repo_cache.set(repo);
    Ok(self.repo_cache.get().expect("just initialized"))
}
```

**Assessment:** This is a significant improvement for multi-threaded usage. The `expect()` on line 52 is safe because `set()` was just called.

#### PERF-2: InRepoWorktreesLayout Opens Repository Per Call
**Severity:** Low

Unlike `ContainerLayout`, `InRepoWorktreesLayout` opens a new `Repository` handle on each operation:

```rust
// in_repo_worktrees.rs line 42-44
fn open_repo(&self) -> Result<Repository> {
    Ok(Repository::open(self.root.to_native())?)
}
```

**Recommendation:** Consider adding `OnceLock` caching to `InRepoWorktreesLayout` for consistency and performance parity with `ContainerLayout`.

#### PERF-3: list_worktrees Opens Repository Per Worktree
**Severity:** Low

The `list_worktrees` implementation opens a new `Repository` for each worktree to read the branch name:

```rust
// container.rs lines 86-91
let wt_repo = Repository::open(wt_path)?;
let branch = wt_repo
    .head()
    .ok()
    .and_then(|h| h.shorthand().map(String::from))
    .unwrap_or_else(|| "HEAD".into());
```

**Assessment:** This is necessary because `git2` does not provide a direct API to read worktree HEAD from the main repository. The performance impact is negligible for typical worktree counts (< 20).

### Memory Safety

#### MEM-1: No Unsafe Code in repo-git
**Severity:** Informational (Positive)

A grep for `unsafe` in the source directory confirms zero `unsafe` blocks:

```
$ grep -r "unsafe" src/
src/naming.rs:6:    /// Convert slashes to dashes, remove unsafe characters.
src/naming.rs:57:/// Sanitize for hierarchical naming, keeping slashes but removing unsafe chars.
```

Both matches are in documentation comments, not code.

#### MEM-2: Transitive Unsafe in git2
**Severity:** Informational

The `git2` crate uses `unsafe` internally to interface with `libgit2`. This is expected and necessary for FFI. The repo-git crate properly handles all `git2::Error` returns, preventing undefined behavior from propagating.

### Error Handling

#### ERR-1: Comprehensive Error Type Coverage
**Severity:** Informational (Positive)

The `error.rs` module provides specific error variants for all failure modes:

| Error Variant | Usage |
|--------------|-------|
| `Git(git2::Error)` | Wraps all libgit2 errors |
| `Fs(repo_fs::Error)` | Wraps filesystem errors |
| `WorktreeExists` | Clear duplicate worktree message |
| `WorktreeNotFound` | Named worktree not found |
| `BranchNotFound` | Named branch not found |
| `LayoutUnsupported` | Classic layout doesn't support worktrees |
| `InvalidBranchName` | Reserved for validation errors |
| `RemoteNotFound` | Push/pull to missing remote |
| `NoUpstreamBranch` | Reserved for tracking errors |
| `MergeConflict` | Merge resulted in conflicts |
| `CannotFastForward` | Non-fast-forward pull |
| `PushFailed` | Push operation failed |
| `PullFailed` | Pull operation failed |

**Assessment:** Excellent error taxonomy enabling precise error handling by consumers.

#### ERR-2: Single expect() Usage
**Severity:** Low

There is one `expect()` call in production code:

```rust
// container.rs line 52
Ok(self.repo_cache.get().expect("just initialized"))
```

**Assessment:** This is safe. The `expect()` is called immediately after `set()`, and the `OnceLock` guarantees the value exists. The comment explains the invariant.

#### ERR-3: Unwrap Usage in Tests Only
**Severity:** Informational

All other `unwrap()` calls are in test code (`helpers.rs` unit tests, benchmark, and test files), which is appropriate for test assertions.

#### ERR-4: Merge Conflict Handling
**Severity:** Informational (Positive)

Merge operations properly clean up state on conflicts:

```rust
// classic.rs lines 209-217
if index.has_conflicts() {
    repo.cleanup_state()?;  // Clean up merge state
    return Err(Error::MergeConflict {
        message: format!("Merge of '{}' resulted in conflicts", source),
    });
}
```

**Assessment:** The repository is left in a clean state even when merge fails.

## Recommendations

### Priority 1 (Should Address)

1. **Add OnceLock caching to InRepoWorktreesLayout** - For performance parity with ContainerLayout and to avoid repeated Repository opens in multi-operation workflows.

2. **Validate Windows reserved names in naming.rs** - Add checks for CON, NUL, PRN, AUX, etc. when running on Windows to prevent filesystem errors.

### Priority 2 (Consider)

3. **Return branch deletion status from remove_feature** - Consider returning a structured result like `RemovalResult { worktree_removed: bool, branch_removed: bool }` instead of just logging the warning.

4. **Run cargo-audit in CI** - The previous audit's recommendation to integrate `cargo audit` into CI remains valid. This should be addressed at the workspace level.

### Priority 3 (Nice to Have)

5. **Add fuzz testing for naming functions** - The `slugify` and `sanitize_hierarchical` functions would benefit from fuzz testing to catch edge cases.

6. **Document git2 version requirements** - The crate should document which minimum version of `git2` (and corresponding `libgit2`) it has been tested with.

## Test Coverage Assessment

The crate has comprehensive test coverage across multiple test files:

| Test File | Coverage Area |
|-----------|--------------|
| `classic_tests.rs` | Classic layout basic operations |
| `container_tests.rs` | Container layout worktree operations |
| `in_repo_worktrees_tests.rs` | InRepo layout worktree operations |
| `naming_tests.rs` | Branch name sanitization |
| `git_operations_tests.rs` | Push/pull/merge operations |
| `robustness_unicode.rs` | Unicode and emoji branch names |
| `benches/git_benchmarks.rs` | Performance benchmarking |

**Positive observations:**
- Tests use real git repositories via `Command::new("git")` for realistic scenarios
- Error conditions are tested (missing remotes, non-existent branches)
- Unicode/emoji handling is explicitly tested
- Tests clean up after themselves using `TempDir`

## Conclusion

The `repo-git` crate has matured significantly since the previous audit. Key improvements include:

- Shared helper functions reduce code duplication and improve maintainability
- Thread-safe caching improves performance in concurrent scenarios
- Comprehensive error handling with specific error types
- Good test coverage including edge cases (unicode, errors)

The crate follows Rust best practices with no `unsafe` code, proper error propagation, and defensive programming. The remaining recommendations are minor improvements rather than critical issues.

**Final Assessment:** APPROVED for production use with minor recommendations tracked above.

---

*Audit conducted: 2026-01-28*
*Files reviewed: 8 source files, 7 test files, 1 benchmark file*
*Lines of code audited: ~1,100 (src/), ~700 (tests/)*
