# Audit Report: `repo-fs` Crate

**Date:** 2026-01-23
**Scope:** Security, Robustness, Correctness, Optimization, Benchmarking, Interface Quality.

## 1. Executive Summary

A logical and automated audit of the `repo-fs` crate was conducted. The crate is found to be **mostly robust** with a clean and consistent interface, but contains **one critical robustness bug** in layout validation and **several missing standard dependencies** expected for a repository manager filesystem layer.

## 2. Interface & Architecture Assessment

The `repo-fs` crate provides a solid foundation for cross-platform filesystem operations.

* **Strengths:**
  * **`NormalizedPath`**: Enforces forward-slash consistency, critical for cross-platform repository logic.
  * **`ConfigStore`**: Simple, atomic, format-agnostic configuration handling.
  * **`WorkspaceLayout`**: Clear detection logic with well-defined precedence (Container > Worktrees > Classic).
* **Weaknesses:**
  * **Manual Path Cleaning**: `NormalizedPath::clean` is a manual implementation. While it seems correct, it poses a maintenance risk compared to standard crates like `path-clean`.
  * **Rigid Detection**: Layout detection order is hardcoded.

### Missing Dependencies

To fulfill the mission of a "Repository Manager", the following crates are recommended:

* **`walkdir`** (High Priority): For recursive file listing.
* **`ignore`** (Critical Priority): To respect `.gitignore` rules during traversal.
* **`glob`**: For pattern matching.

## 3. Robustness & Correctness Findings

*Agent: Robustness Reviewer*

The robustness audit verified error handling and edge cases.

* **Critical Finding**: `WorkspaceLayout::validate()` uses `exists()` instead of `is_dir()` to verify layout components.
  * **Impact**: A file named `.gt` (instead of a directory) allows validation to pass, leading to confusing errors later.
  * **Recommendation**: Change `exists()` to `is_dir()` in `src/layout.rs`.
* **Positive Findings**:
  * `detect_at` correctly handles file-vs-directory ambiguity by failing safe.
  * Error types in `src/error.rs` are semantic and useful (`thiserror`).

## 4. Performance & Optimization Findings

*Agent: Performance Reviewer*

A benchmark suite was created in `benches/fs_benchmarks.rs` using `criterion`.

* **Coverage**:
  * `io::write_atomic`: Benchmarks the write-sync-rename cycle.
  * `WorkspaceLayout::detect`: Benchmarks detection in deep directory hierarchies.
* **Observations**:
  * `write_atomic` robustness comes at the cost of multiple fsyncs. This is acceptable for configuration but should be monitored for high-frequency writes.
  * `detect` mechanism is generally fast (`lstat` based) but depth-dependent.

## 5. Security Findings

*Agent: Security Reviewer*

A security audit focused on `src/path.rs` and `src/io.rs` was performed.

* **Path Traversal**: Low Risk. `NormalizedPath` correctly neutralizes traversal attempts (e.g., `../a/b` -> `a/b`).
* **Symlink Attacks**: **High Risk**.
  * **Vulnerability**: `io::write_atomic` uses `fs::create_dir_all` which follows symlinks. If an attacker controls a parent directory component and replaces it with a symlink, files can be written outside the intended directory.
  * **Race Conditions**: The check-then-act sequence in `write_atomic` is vulnerable to TOCTOU race conditions.
* **Recommendation**:
  * Enforce a "Jail" or "Root" directory concept.
  * Use `lstat`-based validation for path components to detect symlinks before writing.

## 6. Recommendations

1. **Fix Validation Bug**: Update `WorkspaceLayout::validate` to ensure `.gt`, `.git`, `main`, and `.worktrees` are directories.
2. **Add Dependencies**: Add `walkdir`, `ignore`, and `glob` to `Cargo.toml`.
3. **Enhance Path Cleaning**: Evaluate replacing manual `clean` with `path-clean` crate.
4. **Adopt Benchmarks**: Integrate the new `benches/fs_benchmarks.rs` into CI.
