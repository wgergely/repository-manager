# Rust Configuration Libraries

Evaluation of configuration management crates for repo-manager.

## Recommendation: figment

Modern, type-safe configuration with layering support.

## Comparison Matrix

| Crate | Format Support | Layering | Env Vars | Type Safety | Profiles |
|-------|---------------|----------|----------|-------------|----------|
| **figment** | TOML, JSON, YAML, ENV | Yes | Yes | Strong | Yes |
| config | TOML, JSON, YAML, INI | Yes | Yes | Runtime | Yes |
| toml | TOML only | No | No | Strong | No |
| serde_yaml | YAML only | No | No | Strong | No |

## figment Example

```rust
use figment::{Figment, providers::{Format, Toml, Env, Serialized}};

// Pattern: Hierarchical config loading with layered overrides
fn load_config() -> Result<Config, figment::Error> {
    Figment::new()
        .merge(Serialized::defaults(Config::default()))   // Defaults
        .merge(Toml::file("~/.config/repo-manager/config.toml"))  // User
        .merge(Toml::file(".repository/config.toml"))     // Project
        .merge(Env::prefixed("REPO_MANAGER_").split("_")) // Environment
        .extract()
}
```

## Example Config File

```toml
# .repo-manager.toml
[global]
container_dir = "~/dev/containers"
default_branch = "main"
auto_sync = true

[worktrees]
pattern = "Centralized"
worktrees_dir = "worktrees"
symlink_configs = true

[tools.claude]
enabled = true
rules_template = "~/.config/repo-manager/templates/CLAUDE.md"
permissions = ["Bash", "Read", "Write", "Edit"]

[[tools.claude.mcp_servers]]
name = "filesystem"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem"]

[tools.cursor]
enabled = true
```

## Cargo Dependencies

```toml
[dependencies]
figment = { version = "0.10", features = ["toml", "json", "yaml", "env"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
```

## Trade-offs

**figment strengths**:
- Strong compile-time type checking
- Layering support (system -> user -> project -> env)
- Multiple format support
- Profile-based configuration

**config-rs strengths**:
- Mature, widely used
- Runtime flexibility
- Well-documented

---

*Last updated: 2026-01-23*
*Status: Complete*
