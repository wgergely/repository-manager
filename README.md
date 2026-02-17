# Repository Manager

A unified control plane for agentic development workspaces. Generate configuration for 13 AI/IDE tools from a single source of truth.

> **Status: Alpha** -- API may change. Feedback welcome.

## What It Does

In 2026, development environments are fragmented. A human uses VSCode; one agent uses a CLI interface; another agent lives inside the IDE. Each has its own way of defining "good code" (linters, formatters) and "how to work" (tasks, scripts).

Repository Manager solves this by providing a **unified control plane**. It establishes a **Single Source of Truth** (`.repository/config.toml`) that abstracts away the specific configuration formats of individual tools. You declare your intent once -- which tools to use, which rules to enforce, which presets to apply -- and the manager generates and synchronizes the correct configuration files for each tool automatically.

**Before:** You maintain each tool's config separately, and they drift apart.

```
.cursorrules          (manually written)
CLAUDE.md             (manually written, different rules)
.vscode/settings.json (manually written, yet another format)
```

**After:** One config drives them all.

```
.repository/config.toml    <-- you edit this
    |
    v  repo sync
    |
    ├── .cursorrules              (generated)
    ├── CLAUDE.md                 (generated)
    └── .vscode/settings.json     (generated)
```

## Supported Tools

| Tool | Config Path | Category |
|------|-------------|----------|
| Cursor | `.cursorrules` | IDE |
| Claude | `CLAUDE.md` | CLI Agent |
| VS Code | `.vscode/settings.json` | IDE |
| Windsurf | `.windsurfrules` | IDE |
| Gemini | `GEMINI.md` | CLI Agent |
| Copilot | `.github/copilot-instructions.md` | Copilot |
| Cline | `.clinerules` | Autonomous |
| Roo | `.roo/rules/` | Autonomous |
| JetBrains | `.aiassistant/rules/` | IDE |
| Zed | `.rules` | IDE |
| Aider | `.aider.conf.yml` | CLI Agent |
| Amazon Q | `.amazonq/rules/` | Autonomous |
| Antigravity | `.agent/rules.md` | Autonomous |

Custom tool definitions can be added via `.repository/tools/`.

## Installation

```bash
# From source
cargo install --path crates/repo-cli

# Or build locally
git clone <repo-url>
cd repository-manager
cargo build --release
```

## Quick Start

```bash
# Initialize a new project
repo init my-project --mode standard --tools cursor,claude,vscode

# Or initialize in current directory
repo init . --tools cursor,claude

# Generate tool configurations
repo sync

# Check sync status
repo status

# Add another tool
repo add-tool windsurf

# Add a coding rule
repo add-rule python-style --instruction "Use snake_case for all Python functions"
```

## Layout Modes

<details>
<summary><strong>Container Layout</strong> (worktrees mode)</summary>

```
{container}/
├── .gt/          # Git database
├── main/         # Main branch worktree
└── feature-x/    # Feature worktree
```
</details>

<details>
<summary><strong>In-Repo Worktrees Layout</strong></summary>

```
{repo}/
├── .git/
├── .worktrees/
│   └── feature-x/
└── src/
```
</details>

<details>
<summary><strong>Classic Layout</strong> (standard mode)</summary>

```
{repo}/
├── .git/
└── src/
```
</details>

## Crate Architecture

| Crate | Description |
|-------|-------------|
| **repo-cli** | Command-line interface |
| **repo-core** | Core orchestration and sync engine |
| **repo-fs** | Filesystem abstraction with atomic I/O |
| **repo-git** | Git operations and worktree management |
| **repo-tools** | 13 tool integrations and config writers |
| **repo-content** | Content parsing, editing, diffing |
| **repo-blocks** | Managed block system (UUID-tagged sections) |
| **repo-meta** | Ledger, registry, schema definitions |
| **repo-presets** | Environment presets (Python, Node, Rust) |
| **repo-mcp** | MCP server (Model Context Protocol) |
| **integration-tests** | End-to-end test suite |

## Development

```bash
cargo test --workspace
cargo check --workspace
cargo clippy --workspace
```

## Documentation

- [Project Overview](docs/project-overview.md) -- Vision, architecture, and capabilities
- [Design Documentation](docs/design/_index.md) -- Internal design specs and architecture decisions

## License

MIT
