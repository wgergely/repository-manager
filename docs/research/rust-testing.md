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
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_loading() {
        let config = r#"
            [global]
            default_branch = "main"

            [tools.claude]
            enabled = true
        "#;

        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("config.toml"), config).unwrap();

        let loaded = load_config_from(temp.path()).unwrap();
        assert_eq!(loaded.global.default_branch, "main");
        assert!(loaded.tools.claude.unwrap().enabled);
    }

    #[test]
    fn test_provider_registration() {
        let registry = ProviderRegistry::new();
        assert!(registry.get("claude").is_some());
        assert!(registry.get("cursor").is_some());
        assert!(registry.get("unknown").is_none());
    }
}
```

## Integration Tests

```rust
// tests/integration/cli.rs
use assert_cmd::Command;
use predicates::prelude::*;
use assert_fs::prelude::*;

#[test]
fn test_init_command() {
    let temp = assert_fs::TempDir::new().unwrap();

    Command::cargo_bin("repo-manager")
        .unwrap()
        .args(["init", "--claude", "--worktrees"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized"));

    temp.child(".agentic").assert(predicate::path::is_dir());
    temp.child(".agentic/claude").assert(predicate::path::is_dir());
}

#[test]
fn test_create_worktree() {
    let temp = setup_test_container();

    Command::cargo_bin("repo-manager")
        .unwrap()
        .args(["create", "feature-test", "--from", "main"])
        .current_dir(temp.path())
        .assert()
        .success();

    temp.child("worktrees/feature-test").assert(predicate::path::is_dir());
    temp.child("worktrees/feature-test/.claude").assert(predicate::path::is_symlink());
}
```

## Snapshot Tests

```rust
use insta::assert_yaml_snapshot;

#[test]
fn test_template_rendering() {
    let ctx = ProjectContext {
        name: "test-project".into(),
        tech_stack: vec!["Rust".into(), "TypeScript".into()],
        // ...
    };

    let engine = TemplateEngine::new("templates/").unwrap();
    let output = engine.render_claude_md(&ctx).unwrap();

    assert_yaml_snapshot!(output);
}
```

## Cargo Dependencies

```toml
[dev-dependencies]
assert_fs = "1.1"
predicates = "3.1"
assert_cmd = "2.0"
insta = { version = "1.38", features = ["yaml"] }
tempfile = "3.10"
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
