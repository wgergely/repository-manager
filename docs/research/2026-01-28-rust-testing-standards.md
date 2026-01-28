# Rust Testing Best Practices (January 2026)

This document summarizes current best practices for testing Rust applications, with a focus on CLI tools that interact with the filesystem and git.

## Table of Contents

1. [Recommended Test Dependencies](#recommended-test-dependencies)
2. [Test Organization Structure](#test-organization-structure)
3. [Testing Filesystem Operations](#testing-filesystem-operations)
4. [Testing Git Operations](#testing-git-operations)
5. [CLI Testing Frameworks](#cli-testing-frameworks)
6. [Property-Based Testing](#property-based-testing)
7. [Parameterized Testing](#parameterized-testing)
8. [Async Testing](#async-testing)
9. [Code Coverage](#code-coverage)
10. [Anti-Patterns to Avoid](#anti-patterns-to-avoid)

---

## Recommended Test Dependencies

Add these to your `Cargo.toml`:

```toml
[dev-dependencies]
# Core testing utilities
rstest = "0.23"              # Fixtures and parameterized tests
pretty_assertions = "1.4"    # Better diff output for assert_eq!
assert_matches = "1.5"       # Pattern matching assertions

# Filesystem testing
tempfile = "3.14"            # Temporary files and directories
assert_fs = "1.1"            # Filesystem assertions

# CLI testing
assert_cmd = "2.0"           # CLI integration testing
predicates = "3.1"           # Predicate assertions for CLI output
trycmd = "0.15"              # Snapshot testing for CLI (optional, for many tests)

# Property-based testing (choose one)
proptest = "1.5"             # More flexible, better shrinking
# quickcheck = "1.0"         # Simpler, faster generation

# Async testing (if needed)
tokio-test = "0.4"           # Tokio testing utilities

# Mocking (if needed)
mockall = "0.13"             # Powerful mock object library
```

**Optional but recommended:**

```toml
# Test runner (install globally)
# cargo install cargo-nextest --locked

# Coverage (install globally)
# cargo install cargo-llvm-cov
# or: cargo install cargo-tarpaulin (Linux only)
```

---

## Test Organization Structure

### Directory Layout

```
src/
  lib.rs              # Library crate root
  main.rs             # Binary crate (thin wrapper)
  module/
    mod.rs
    submodule.rs      # Unit tests inline with #[cfg(test)]
tests/
  integration_test.rs # Integration tests (each file = separate crate)
  cli/
    mod.rs            # CLI integration tests
  common/
    mod.rs            # Shared test utilities
  fixtures/           # Test data files
    sample_config.toml
```

### Unit Tests

Place unit tests in the same file as the code they test:

```rust
// src/config.rs
pub fn parse_config(input: &str) -> Result<Config, Error> {
    // implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_config() {
        let result = parse_config("key = value");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_config() {
        let result = parse_config("invalid");
        assert!(result.is_err());
    }
}
```

### Integration Tests

Create separate files in `tests/` directory:

```rust
// tests/integration_test.rs
use my_crate::Config;

#[test]
fn test_full_workflow() {
    // Test public API as external user would
}
```

### Shared Test Utilities

```rust
// tests/common/mod.rs
pub fn setup_test_environment() -> TestEnv {
    // Shared setup code
}

// tests/integration_test.rs
mod common;

#[test]
fn test_something() {
    let env = common::setup_test_environment();
    // ...
}
```

---

## Testing Filesystem Operations

### Using `tempfile` for Isolation

```rust
use tempfile::{tempdir, TempDir};
use std::fs;

#[test]
fn test_file_creation() {
    // Creates a temporary directory that is automatically
    // deleted when `dir` goes out of scope
    let dir = tempdir().unwrap();

    let file_path = dir.path().join("test.txt");
    fs::write(&file_path, "content").unwrap();

    assert!(file_path.exists());
    // dir is automatically cleaned up here
}
```

### Using `assert_fs` for Richer Assertions

```rust
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn test_creates_config_file() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Run code that should create files
    my_crate::init(temp.path());

    // Assert file exists and has expected content
    temp.child("config.toml")
        .assert(predicate::path::exists())
        .assert(predicate::str::contains("version"));
}
```

### Key Patterns

1. **Always use temporary directories** - Never write to real filesystem locations
2. **Let RAII handle cleanup** - Don't manually delete; let `TempDir` drop
3. **Use `dir.path()` for paths** - Construct all paths relative to temp dir
4. **Keep temp dir in scope** - Store it in a variable to prevent premature deletion

```rust
// WRONG: temp dir deleted immediately
fn bad_setup() -> PathBuf {
    let dir = tempdir().unwrap();
    dir.path().to_path_buf()  // dir dropped here!
}

// CORRECT: return the TempDir to keep it alive
fn good_setup() -> TempDir {
    tempdir().unwrap()
}
```

---

## Testing Git Operations

### Creating Test Repositories

```rust
use tempfile::tempdir;
use std::process::Command;

fn create_test_repo() -> tempfile::TempDir {
    let dir = tempdir().unwrap();

    Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to init git repo");

    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to configure git");

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to configure git");

    dir
}
```

### Using `git2` for Repository Setup

```rust
use git2::{Repository, Signature};
use tempfile::tempdir;

fn create_test_repo_with_git2() -> (tempfile::TempDir, Repository) {
    let dir = tempdir().unwrap();
    let repo = Repository::init(dir.path()).unwrap();

    // Configure the repo
    let mut config = repo.config().unwrap();
    config.set_str("user.name", "Test User").unwrap();
    config.set_str("user.email", "test@test.com").unwrap();

    (dir, repo)
}

fn create_initial_commit(repo: &Repository) {
    let sig = Signature::now("Test User", "test@test.com").unwrap();
    let tree_id = repo.index().unwrap().write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();

    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "Initial commit",
        &tree,
        &[],
    ).unwrap();
}
```

### Abstracting Git Operations for Testing

For better testability, abstract git operations behind traits:

```rust
// src/git.rs
pub trait GitOperations {
    fn init(&self, path: &Path) -> Result<(), Error>;
    fn commit(&self, message: &str) -> Result<(), Error>;
    fn current_branch(&self) -> Result<String, Error>;
}

pub struct RealGit;
impl GitOperations for RealGit {
    // Real implementations using git2 or Command
}

// In tests
#[cfg(test)]
mod tests {
    use mockall::mock;

    mock! {
        pub Git {}
        impl GitOperations for Git {
            fn init(&self, path: &Path) -> Result<(), Error>;
            fn commit(&self, message: &str) -> Result<(), Error>;
            fn current_branch(&self) -> Result<String, Error>;
        }
    }

    #[test]
    fn test_workflow_with_mock_git() {
        let mut mock = MockGit::new();
        mock.expect_current_branch()
            .returning(|| Ok("main".to_string()));

        // Test code that uses GitOperations trait
    }
}
```

---

## CLI Testing Frameworks

### assert_cmd for CLI Integration Tests

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("my-cli").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn test_cli_with_args() {
    let mut cmd = Command::cargo_bin("my-cli").unwrap();
    cmd.args(["init", "--name", "test-project"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized"));
}

#[test]
fn test_cli_error_handling() {
    let mut cmd = Command::cargo_bin("my-cli").unwrap();
    cmd.arg("--invalid-flag")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}
```

### Using trycmd for Snapshot Testing

Create test cases in `.toml` or `.md` files:

```toml
# tests/cmd/help.toml
bin.name = "my-cli"
args = ["--help"]
status.code = 0
stdout = """
my-cli 0.1.0
Usage: my-cli [OPTIONS] <COMMAND>
...
"""
```

```rust
// tests/cli_tests.rs
#[test]
fn cli_tests() {
    trycmd::TestCases::new()
        .case("tests/cmd/*.toml")
        .case("README.md"); // Can also test examples in docs
}
```

### When to Use Each

| Tool | Best For |
|------|----------|
| `assert_cmd` | Tests requiring custom verification, dynamic assertions |
| `trycmd` | Many simple CLI tests, documentation testing |
| `snapbox` | Custom test harnesses, one-off snapshot tests |

---

## Property-Based Testing

### Using proptest

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_parse_roundtrip(s in "[a-z]{1,10}") {
        let parsed = parse(&s);
        let serialized = serialize(&parsed);
        assert_eq!(s, serialized);
    }

    #[test]
    fn test_config_always_valid(
        name in "[a-zA-Z][a-zA-Z0-9_]*",
        value in any::<i32>()
    ) {
        let config = Config::new(&name, value);
        assert!(config.validate().is_ok());
    }
}
```

### proptest vs quickcheck

| Feature | proptest | quickcheck |
|---------|----------|------------|
| Shrinking | Better, stateful | Basic, stateless |
| Strategies | Flexible, composable | Type-based only |
| Performance | Slower generation | Faster generation |
| MSRV | 1.84 | 1.71 |

**Recommendation:** Use `proptest` for complex types; consider `quickcheck` for simple cases where speed matters.

---

## Parameterized Testing

### Using rstest

```rust
use rstest::*;

// Simple parameterized test
#[rstest]
#[case("hello", 5)]
#[case("world", 5)]
#[case("", 0)]
fn test_string_length(#[case] input: &str, #[case] expected: usize) {
    assert_eq!(input.len(), expected);
}

// Using fixtures
#[fixture]
fn test_repo() -> TempDir {
    let dir = tempdir().unwrap();
    // Setup code
    dir
}

#[rstest]
fn test_with_fixture(test_repo: TempDir) {
    // test_repo is automatically created
    assert!(test_repo.path().exists());
}

// Combining fixtures, cases, and values
#[rstest]
#[case::valid("config.toml")]
#[case::alternate("settings.toml")]
fn test_config_files(
    test_repo: TempDir,
    #[case] filename: &str,
    #[values("json", "toml", "yaml")] format: &str,
) {
    // Generates 6 tests (2 cases x 3 values)
}
```

### Once Fixtures (Shared Setup)

```rust
use rstest::*;
use std::sync::LazyLock;

#[fixture]
#[once]
fn expensive_setup() -> ExpensiveResource {
    // Called only once, shared across all tests
    ExpensiveResource::new()
}
```

### Async Tests with rstest

```rust
use rstest::*;
use std::time::Duration;

#[rstest]
#[tokio::test]
#[timeout(Duration::from_secs(5))]
async fn test_async_operation() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

---

## Async Testing

### Using tokio::test

```rust
#[tokio::test]
async fn test_async_function() {
    let result = my_async_fn().await;
    assert_eq!(result, expected);
}

// Multi-threaded runtime
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_concurrent() {
    // ...
}

// Paused time for deterministic tests
#[tokio::test(start_paused = true)]
async fn test_with_time_control() {
    tokio::time::advance(Duration::from_secs(60)).await;
    // Time has advanced without real delay
}
```

### Using tokio-test for Block-On

```rust
use tokio_test::block_on;

#[test]
fn test_sync_wrapper() {
    let result = block_on(async {
        my_async_fn().await
    });
    assert!(result.is_ok());
}
```

---

## Code Coverage

### cargo-llvm-cov (Recommended)

```bash
# Install
cargo install cargo-llvm-cov

# Run coverage
cargo llvm-cov

# Generate HTML report
cargo llvm-cov --html

# With nextest
cargo llvm-cov nextest

# Output formats: html, json, lcov, cobertura
cargo llvm-cov --lcov --output-path coverage.lcov
```

### cargo-tarpaulin (Linux)

```bash
# Install
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin

# With LLVM backend (recommended)
cargo tarpaulin --engine llvm

# Generate HTML
cargo tarpaulin --out Html
```

### Platform Comparison

| Tool | Linux | macOS | Windows |
|------|-------|-------|---------|
| cargo-llvm-cov | Yes | Yes | Yes |
| cargo-tarpaulin | Best | Limited | No |

**Recommendation:** Use `cargo-llvm-cov` for cross-platform support.

---

## Anti-Patterns to Avoid

### 1. Tests That Depend on Each Other

```rust
// BAD: Test order dependency
static mut COUNTER: i32 = 0;

#[test]
fn test_a() { unsafe { COUNTER += 1; } }

#[test]
fn test_b() { unsafe { assert_eq!(COUNTER, 1); } } // May fail!

// GOOD: Independent tests
#[test]
fn test_a() {
    let mut counter = 0;
    counter += 1;
    assert_eq!(counter, 1);
}
```

### 2. Testing Multiple Things in One Test

```rust
// BAD: Multiple concerns
#[test]
fn test_everything() {
    let config = parse_config("...");
    assert!(config.is_ok());
    assert_eq!(config.name, "test");
    assert!(validate(&config).is_ok());
    assert!(save(&config).is_ok());
}

// GOOD: Focused tests
#[test]
fn test_parse_config() { /* ... */ }

#[test]
fn test_validate_config() { /* ... */ }

#[test]
fn test_save_config() { /* ... */ }
```

### 3. Using unwrap() Without Context

```rust
// BAD: Unhelpful panic message
let result = parse(input).unwrap();

// GOOD: Clear failure context
let result = parse(input).expect("Failed to parse test input");

// BETTER: Use proper assertions
assert!(parse(input).is_ok(), "parse should succeed for valid input");
```

### 4. Hardcoding Paths

```rust
// BAD: Brittle path
let config = read_config("/home/user/project/test_config.toml");

// GOOD: Use tempdir or relative paths
let dir = tempdir().unwrap();
let config_path = dir.path().join("config.toml");
```

### 5. Not Cleaning Up Resources

```rust
// BAD: Manual cleanup (might not run on failure)
#[test]
fn test_with_file() {
    fs::write("/tmp/test.txt", "data").unwrap();
    // test code...
    fs::remove_file("/tmp/test.txt").unwrap(); // Skipped if test panics!
}

// GOOD: RAII cleanup
#[test]
fn test_with_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.txt");
    fs::write(&path, "data").unwrap();
    // test code...
    // Automatic cleanup when dir drops
}
```

### 6. Ignoring Test Results in CI

```rust
// BAD: Silently ignoring failures
#[test]
#[ignore]
fn test_flaky() { /* ... */ }

// GOOD: Fix the flakiness or document why it's ignored
#[test]
#[ignore = "Requires network access, run with --ignored"]
fn test_network() { /* ... */ }
```

### 7. Testing Private Implementation Details

```rust
// BAD: Tight coupling to internals
#[test]
fn test_internal_cache_size() {
    let obj = MyStruct::new();
    assert_eq!(obj.internal_cache.len(), 0); // Testing internals
}

// GOOD: Test behavior through public API
#[test]
fn test_caching_improves_performance() {
    let obj = MyStruct::new();
    let first_call = measure(|| obj.compute(input));
    let second_call = measure(|| obj.compute(input));
    assert!(second_call < first_call); // Test observable behavior
}
```

---

## Test Runner: cargo-nextest

Consider using `cargo-nextest` for better test execution:

```bash
# Install
cargo install cargo-nextest --locked

# Run tests
cargo nextest run

# Benefits:
# - Up to 3x faster execution
# - Process-per-test isolation
# - Better output formatting
# - Built-in flaky test retry
# - JUnit XML output for CI
```

**Note:** Doctests must still be run separately with `cargo test --doc`.

---

## References

- [Rust Book: Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- [Rust CLI Book: Testing](https://rust-cli.github.io/book/tutorial/testing.html)
- [rstest Documentation](https://docs.rs/rstest)
- [assert_cmd Documentation](https://docs.rs/assert_cmd)
- [tempfile Documentation](https://docs.rs/tempfile)
- [proptest Documentation](https://docs.rs/proptest)
- [cargo-nextest](https://nexte.st/)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
- [Rust Design Patterns: Anti-Patterns](https://rust-unofficial.github.io/patterns/anti_patterns/index.html)
