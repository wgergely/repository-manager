# repo-cli Crate Audit - 2026-01-28

## Executive Summary

The `repo-cli` crate implements the command-line interface for the Repository Manager tool. The audit found the codebase to be well-structured with good defensive programming practices. Notable strengths include comprehensive input validation, proper error handling using the `thiserror` crate, and absence of unsafe code. The primary areas for improvement relate to a single production `expect()` call, potential edge cases in user input handling for external commands, and opportunities to improve robustness in interactive mode error handling.

**Overall Risk Assessment: LOW**

| Category | Status | Notes |
|----------|--------|-------|
| Security | Good | Path traversal protection, input validation present |
| Memory Safety | Excellent | No unsafe code |
| Error Handling | Good | One production expect() needs attention |
| Panics | Good | Test-only panics, one production expect |
| Performance | Good | No obvious issues |

## Crate Overview

**Purpose:** CLI interface providing commands for repository initialization, synchronization, tool/preset management, rule management, and branch operations.

**Key Files:**
- `src/main.rs` - Entry point and command dispatch
- `src/cli.rs` - Clap-based argument parsing
- `src/error.rs` - Error types using thiserror
- `src/context.rs` - Repository context detection
- `src/interactive.rs` - Interactive prompts using dialoguer
- `src/commands/*.rs` - Command implementations

**Dependencies:**
- `clap` 4.x with derive features for CLI parsing
- `colored` 2.x for terminal output
- `dialoguer` 0.11 for interactive prompts
- `thiserror` for error handling
- `tracing`/`tracing-subscriber` for logging
- Internal crates: `repo-core`, `repo-fs`, `repo-meta`

## Findings

### Security

#### S1: Path Traversal Protection in Rule Management [GOOD]
**Location:** `src/commands/rule.rs:13-28`

The rule management commands properly validate rule IDs to prevent path traversal attacks:
```rust
fn validate_rule_id(id: &str) -> Result<()> {
    if id.is_empty() { ... }
    if id.len() > 64 { ... }
    if id.contains('/') || id.contains('\\') || id.contains("..") { ... }
    if !id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') { ... }
    Ok(())
}
```
This effectively blocks attempts like `../../../etc/passwd` and similar injection patterns.

#### S2: Project Name Sanitization [GOOD]
**Location:** `src/commands/init.rs:77-105`

Project names are sanitized before being used as directory names:
```rust
pub fn sanitize_project_name(name: &str) -> String {
    // Converts to lowercase
    // Replaces spaces/underscores with hyphens
    // Removes special characters
    // Collapses multiple hyphens
    // Provides fallback for empty result
}
```
This prevents directory traversal and special character injection in project names.

#### S3: Git Remote URL Passed to External Command [LOW RISK]
**Location:** `src/commands/init.rs:206-221`

The `add_git_remote` function passes user-provided URLs to git:
```rust
let output = Command::new("git")
    .args(["remote", "add", "origin", remote_url])
    .current_dir(path)
    .output()?;
```
**Analysis:** This is acceptable because:
1. The URL is passed as a single argument (not shell-interpolated)
2. Git itself validates the remote URL format
3. Rust's `Command` API prevents shell injection

However, consider validating URL format before passing to git for better error messages.

#### S4: Tool/Preset Name Validation [ADVISORY]
**Location:** `src/commands/tool.rs:30-39, 114-123`

Tool and preset names are checked against known registries but unknown names are accepted with a warning:
```rust
if !tool_registry.is_known(name) {
    eprintln!("warning: Unknown tool '{}'...", name);
}
```
This is reasonable UX but could be strengthened to optionally reject unknown tools via a `--strict` flag.

### Performance

#### P1: No Performance Concerns [GOOD]
The CLI performs simple operations with minimal resource usage. No memory-intensive loops, unbounded allocations, or performance anti-patterns were identified.

#### P2: Mode Detection Efficiency [GOOD]
**Location:** `src/context.rs:61-127`

The `detect_context` function walks up the directory tree looking for repository markers. This is efficient as it:
- Exits early when markers are found
- Only reads small config files
- Uses path operations without globbing

### Memory Safety

#### M1: No Unsafe Code [EXCELLENT]
**Verification:** `grep -r "unsafe" src/` returned no matches.

The crate is entirely safe Rust with no unsafe blocks.

#### M2: No Manual Memory Management
All memory is managed through Rust's ownership system and standard library types.

### Error Handling

#### E1: Production expect() Call [MEDIUM]
**Location:** `src/main.rs:36`

```rust
tracing::subscriber::set_global_default(subscriber)
    .expect("Failed to set tracing subscriber");
```

**Issue:** This will panic if called twice (e.g., in tests or if user code sets a global subscriber first).

**Recommendation:** Convert to a proper error:
```rust
if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
    eprintln!("Warning: Could not set tracing subscriber: {}", e);
}
```

#### E2: Error Type Design [GOOD]
**Location:** `src/error.rs`

The error enum properly wraps underlying errors with transparent forwarding:
```rust
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error(transparent)]
    Core(#[from] repo_core::Error),
    #[error(transparent)]
    Fs(#[from] repo_fs::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Interactive prompt error: {0}")]
    Dialoguer(#[from] dialoguer::Error),
    #[error("{message}")]
    User { message: String },
}
```

#### E3: Error Messages Do Not Leak Sensitive Information [GOOD]
Error messages are user-friendly and do not expose internal paths, stack traces, or implementation details inappropriately.

#### E4: Graceful Degradation on Sync Failure [GOOD]
**Location:** `src/commands/tool.rs:245-254`

When tool sync fails, the CLI continues and warns rather than failing the operation:
```rust
Err(e) => {
    eprintln!("warning: Sync failed: {}", e);
    Ok(()) // Config change succeeded
}
```

### UX/Robustness

#### U1: Interactive Mode Terminal Requirements [ADVISORY]
**Location:** `src/interactive.rs`

The interactive mode uses `dialoguer` which requires a TTY. If stdin is not a terminal, the prompts will fail. Consider adding detection:
```rust
if !std::io::stdin().is_terminal() {
    return Err(CliError::user("Interactive mode requires a terminal"));
}
```

#### U2: Rule Overwrite Without Warning [ADVISORY]
**Location:** `src/commands/rule.rs:45-54`

Adding a rule with an existing ID silently overwrites it. Consider prompting for confirmation or requiring a `--force` flag.

#### U3: Branch Name Validation [ADVISORY]
**Location:** `src/commands/branch.rs:35`

Branch names from user input are passed directly to the backend without validation. While git itself validates branch names, providing client-side validation would give better error messages. Consider checking for:
- Invalid characters (`~`, `^`, `:`, `\`, `?`, `*`, `[`)
- Names starting with `-`
- Names containing `..`

#### U4: Context Detection Fallback [GOOD]
**Location:** `src/context.rs:102-106`

When file names cannot be converted to strings, a sensible fallback is used:
```rust
let worktree_name = current
    .file_name()
    .and_then(|n| n.to_str())
    .unwrap_or("unknown")
    .to_string();
```

### Test-Only Panics (Acceptable)

All `panic!`, `expect()`, and `unwrap()` calls in production code outside of `#[cfg(test)]` modules:

| Location | Call | Assessment |
|----------|------|------------|
| `src/main.rs:36` | `expect("Failed to set tracing subscriber")` | Should be converted (see E1) |
| `src/commands/rule.rs:99` | `unwrap()` on `file_stem()` | Safe - only called after extension check |

All other panicking calls are in test code, which is appropriate.

## Recommendations

### High Priority

1. **Replace expect() in main.rs** - Convert the tracing subscriber setup to use proper error handling instead of panicking.

### Medium Priority

2. **Add terminal detection for interactive mode** - Check if stdin is a terminal before entering interactive mode.

3. **Add branch name validation** - Validate branch names client-side before passing to git for better error messages.

### Low Priority

4. **Consider --force flag for rule overwrite** - Add explicit overwrite confirmation for existing rules.

5. **Consider --strict mode for tool validation** - Allow users to opt into strict validation that rejects unknown tool names.

6. **Add URL format validation for git remotes** - Basic URL validation would provide better error messages than git's errors.

## Test Coverage Observations

The crate has comprehensive tests including:
- Unit tests for CLI parsing (`cli.rs`)
- Unit tests for sanitization and validation
- Unit tests for context detection
- Integration tests using `assert_cmd`
- E2E workflow tests

Test code appropriately uses `unwrap()` and `expect()` which is standard practice.

## Conclusion

The `repo-cli` crate demonstrates good security practices and defensive programming. The codebase is clean, well-organized, and follows Rust idioms. The main actionable item is converting the single production `expect()` call to proper error handling. The advisory items are UX improvements that would enhance robustness but are not critical issues.
