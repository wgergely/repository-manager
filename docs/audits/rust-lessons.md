# Rust Architecture & Performance: Knowledge Base

This document consolidates audit findings, performance optimizations, and architectural standards for the Rust crates within the `repository-manager` project.

## Core Architectural Standards

- **Separation of Concerns**: Maintain discrete crates (`repo-fs`, `repo-git`, `repo-meta`, etc.) with clear responsibilities.
- **Strict Abstractions**: Use types like `NormalizedPath` and `LayoutProvider` to enforce correctness across platforms and avoid ad-hoc path manipulation.
- **Standardized Logging**:
  - Use `tracing` for all observability.
  - `debug!`: Entry points for key operations (writes, git commands).
  - `warn!`: Graceful failure paths and non-fatal errors.
  - **No Swallowed Errors**: Avoid `let _ = ...` on `Result` types. Always log warnings for ignored results.

## Performance Optimization

### [repo-fs] Allocation Management

- **`NormalizedPath`**: Avoid unconditional `String` allocations. Check for required normalization (e.g., presence of `\`) before performing replacements.
- **Tree Traversal**: Be cautious of owned `String` allocations in methods like `parent()`. Prefer returning `&str` slices of existing components where possible.

### [repo-fs] I/O Efficiency

- **Redundant Check**: `fs::create_dir_all` should only be called by high-level orchestrators, not on every individual atomic write.
- **Durability Trade-offs**: `sync_all()` (fsync) is expensive. Document where it is used to guarantee durability vs. where it can be omitted for performance.

## Safety & Robustness

### [CRITICAL] Inter-process Locking

- **Incorrect Pattern**: Locking a `.tmp` file (unique by PID) fails to prevent race conditions between processes.
- **Correct Pattern**: Use a dedicated, shared lock file (e.g., `.filename.lock`) via cross-platform advisory locks (`fs2`).

### [CRITICAL] Atomic Writes

- Use the "write-to-temp-then-rename" pattern to prevent partial files or data corruption.

### [CRITICAL] Error Propagation

- Use `thiserror` for structured, context-rich error types. Use `map_err` to add context as errors bubble up through the layers.
- Avoid `.expect()` or `.unwrap()` on dynamic data. Propagate errors (e.g., regex compilation failures) instead of panicking.
