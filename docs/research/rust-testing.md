# Rust Testing Strategy

Testing approaches and crates for repo-manager.

## Testing Crates

| Crate | Purpose |
|-------|---------|
| **assert_fs** | Temporary filesystem fixtures |
| **predicates** | Assertion predicates |
| **assert_cmd** | CLI integration testing |
| **insta** | Snapshot testing |
| **tempfile** | Temporary files/dirs |

## Unit Tests

```rust
#[test]
fn test_config_loading() {
    let temp = TempDir::new().unwrap();
    std::fs::write(temp.path().join("config.toml"), "[global]\ndefault_branch = \"main\"").unwrap();
    let loaded = load_config_from(temp.path()).unwrap();
    assert_eq!(loaded.global.default_branch, "main");
}
```

## Integration Tests

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_init_command() {
    let temp = assert_fs::TempDir::new().unwrap();
    Command::cargo_bin("repo-manager").unwrap()
        .args(["init", "--claude"]).current_dir(temp.path())
        .assert().success().stdout(predicate::str::contains("Initialized"));
    temp.child(".repository").assert(predicate::path::is_dir());
}
```

## Snapshot Tests

```rust
use insta::assert_yaml_snapshot;

#[test]
fn test_template_rendering() {
    let output = render_template("CLAUDE.md", &context).unwrap();
    assert_yaml_snapshot!(output);  // Auto-generates/compares snapshots
}
```

## Additional Crates

| Crate | Purpose |
|-------|---------|
| **mockall** | Mock trait implementations |
| **proptest** | Property-based testing |

## Cargo Dependencies

```toml
[dev-dependencies]
assert_fs = "1.1"
predicates = "3.1"
assert_cmd = "2.0"
insta = { version = "1.38", features = ["yaml"] }
tempfile = "3.10"
mockall = "0.12"      # Optional: mock traits
proptest = "1.4"      # Optional: property testing
```

## Test Organization

```
tests/
├── integration/
│   ├── cli.rs           # CLI command tests
│   ├── git.rs           # Git operation tests
│   └── sync.rs          # Sync functionality tests
├── fixtures/
│   ├── configs/         # Test configuration files
│   └── templates/       # Test templates
└── common/
    └── mod.rs           # Shared test utilities
```

---

*Last updated: 2026-01-23*
*Status: Complete*
