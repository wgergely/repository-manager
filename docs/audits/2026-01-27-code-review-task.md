# Code Review Task: Repository Manager Core & CLI

> **For Claude:** This document is a task specification for the `code-reviewer` agent. Execute this review against the current codebase.

**Date:** 2026-01-27
**Scope:** `repo-core`, `repo-cli`, and their integration with Layer 0 crates
**Priority:** High

---

## 1. Review Objectives

Conduct a comprehensive code review of the recently implemented orchestration layer (`repo-core`) and command-line interface (`repo-cli`) to ensure:

1. **Spec Compliance** - Implementation matches design documents
2. **Code Quality** - Rust idioms, error handling, testability
3. **Security** - No path traversal, injection, or TOCTOU vulnerabilities
4. **Robustness** - Edge cases handled, graceful degradation
5. **Integration** - Correct usage of Layer 0 crate APIs

---

## 2. Files to Review

### 2.1 repo-core (Priority: Critical)

| File | Focus Areas |
|------|-------------|
| `crates/repo-core/src/lib.rs` | Public API surface, re-exports |
| `crates/repo-core/src/error.rs` | Error variants completeness |
| `crates/repo-core/src/mode.rs` | Mode enum correctness |
| `crates/repo-core/src/ledger/mod.rs` | Ledger structure per spec |
| `crates/repo-core/src/ledger/intent.rs` | Intent tracking per spec |
| `crates/repo-core/src/ledger/projection.rs` | Projection types per spec |
| `crates/repo-core/src/backend/mod.rs` | ModeBackend trait design |
| `crates/repo-core/src/backend/standard.rs` | Standard mode implementation |
| `crates/repo-core/src/backend/worktree.rs` | Worktree mode implementation |
| `crates/repo-core/src/config/mod.rs` | Config module structure |
| `crates/repo-core/src/config/manifest.rs` | Manifest parsing |
| `crates/repo-core/src/config/resolver.rs` | Hierarchical config merge |
| `crates/repo-core/src/config/runtime.rs` | Runtime context |
| `crates/repo-core/src/sync/mod.rs` | Sync module structure |
| `crates/repo-core/src/sync/check.rs` | Check operation |
| `crates/repo-core/src/sync/engine.rs` | SyncEngine implementation |

### 2.2 repo-cli (Priority: High)

| File | Focus Areas |
|------|-------------|
| `crates/repo-cli/src/main.rs` | Entry point, error handling |
| `crates/repo-cli/src/cli.rs` | Clap argument structure |
| `crates/repo-cli/src/error.rs` | CLI-specific errors |
| `crates/repo-cli/src/commands/mod.rs` | Command dispatch |
| `crates/repo-cli/src/commands/init.rs` | Init command implementation |
| `crates/repo-cli/src/commands/sync.rs` | Sync command implementation |
| `crates/repo-cli/src/commands/branch.rs` | Branch command implementation |
| `crates/repo-cli/src/commands/tool.rs` | Tool add/remove commands |

---

## 3. Review Checklist

### 3.1 Spec Compliance

Reference documents:
- `docs/design/architecture-core.md` - Core architecture
- `docs/design/config-ledger.md` - Ledger system specification
- `docs/design/spec-cli.md` - CLI specification

**Check:**
- [ ] Ledger schema matches spec (Intent, Projection, ProjectionKind)
- [ ] CLI commands match spec (init, check, fix, sync, branch add/remove/list)
- [ ] ModeBackend trait matches spec operations
- [ ] Config resolution follows hierarchical merge strategy
- [ ] Default mode is `worktrees` (not `standard`)

### 3.2 Code Quality

**Check:**
- [ ] Proper use of `Result<T, Error>` - no unwraps in library code
- [ ] Meaningful error messages with context
- [ ] Documentation on public APIs
- [ ] No dead code or unused imports
- [ ] Consistent naming conventions
- [ ] Tests exist for critical paths

### 3.3 Security

**Check:**
- [ ] No path traversal vulnerabilities in file operations
- [ ] Symlink attacks prevented (use `symlink_metadata`)
- [ ] No command injection in git operations
- [ ] TOCTOU race conditions addressed
- [ ] User input validated before use

### 3.4 Robustness

**Check:**
- [ ] Handles missing `.repository` directory gracefully
- [ ] Handles corrupt/invalid ledger.toml
- [ ] Handles missing git repository
- [ ] Handles permission denied errors
- [ ] Non-UTF8 paths handled correctly

### 3.5 Integration

**Check:**
- [ ] Correct use of `repo_fs::NormalizedPath`
- [ ] Correct use of `repo_git` worktree operations
- [ ] Correct use of `repo_content` for file manipulation
- [ ] Correct use of `repo_meta` for config schema
- [ ] Correct use of `repo_tools` for tool integrations

---

## 4. Known Issues to Verify

From previous audits, verify these items are addressed:

| Issue | Location | Status to Verify |
|-------|----------|------------------|
| Symlink vulnerability | `repo-fs/src/io.rs` | Check if `contains_symlink` added |
| Dry-run support | `repo-core/src/sync/engine.rs` | Check `SyncOptions` struct exists |
| Default mode = worktrees | `repo-cli/src/cli.rs` | Check default value |
| Tool validation warnings | `repo-cli/src/commands/tool.rs` | Check registry validation |

---

## 5. Output Format

Produce a review report with:

1. **Summary** - Overall assessment (Pass/Needs Work/Fail)
2. **Findings by Category** - Organized by checklist sections
3. **Critical Issues** - Must fix before merge
4. **Recommendations** - Nice to have improvements
5. **Test Coverage Gaps** - Missing test scenarios

---

## 6. Execution Instructions

```bash
# Run tests first to ensure baseline
cd main
cargo test --workspace

# Check for warnings
cargo clippy --workspace -- -D warnings

# Verify documentation builds
cargo doc --workspace --no-deps
```

After running these, proceed with manual code review using the checklist above.
