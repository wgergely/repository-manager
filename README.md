# Repository Manager

A Rust-based CLI tool for orchestrating agentic development workspaces.

## Crates

- **repo-fs** - Filesystem abstraction with path normalization and atomic I/O
- **repo-git** - Git abstraction supporting multiple worktree layouts

## Layout Modes

The repository manager supports three layout modes:

### Container Layout
```
{container}/
├── .gt/          # Git database
├── main/         # Main branch worktree
└── feature-x/    # Feature worktree
```

### In-Repo Worktrees Layout
```
{repo}/
├── .git/
├── .worktrees/
│   └── feature-x/
└── src/
```

### Classic Layout
```
{repo}/
├── .git/
└── src/
```

## Development

```bash
# Run tests
cargo test -- --test-threads=1

# Check compilation
cargo check

# Build
cargo build
```

## License

MIT
