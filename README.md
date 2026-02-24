# Repository Manager

[![CI](https://img.shields.io/github/actions/workflow/status/your-org/repository-manager/ci.yml?branch=main&label=CI)](https://github.com/your-org/repository-manager/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/repo-cli.svg)](https://crates.io/crates/repo-cli)

**One config file. 14 AI tools. Zero drift.**

Repository Manager generates and synchronizes AI tool configurations (Cursor, Claude, VS Code, Copilot, and 10 more) from a single `.repository/config.toml` source of truth.

> **Status: Alpha** — API may change. Feedback welcome.

---

## The Problem

In 2026, a typical project touches a dozen AI tools. Each has its own config format, and they drift apart immediately.

```
.cursorrules                      (manually written)
CLAUDE.md                         (manually written, different rules)
.vscode/settings.json             (manually written, yet another format)
.github/copilot-instructions.md   (manually written, forgotten)
```

## The Solution

Declare your intent once. `repo sync` generates the rest.

```
.repository/config.toml    ← you edit this
        |
        v  repo sync
        |
        ├── .cursorrules                        (generated)
        ├── CLAUDE.md                           (generated)
        ├── .vscode/settings.json               (generated)
        ├── .github/copilot-instructions.md     (generated)
        └── ... 10 more tools                   (generated)
```

---

## Supported Tools (14 built-in)

| Tool | Config Path | Category |
|------|-------------|----------|
| Cursor | `.cursorrules` | IDE |
| VS Code | `.vscode/settings.json` | IDE |
| Windsurf | `.windsurfrules` | IDE |
| JetBrains | `.aiassistant/rules/` | IDE |
| Zed | `.rules` | IDE |
| Claude | `CLAUDE.md` | CLI Agent |
| Claude Desktop | `claude_desktop_config.json` | CLI Agent |
| Gemini | `GEMINI.md` | CLI Agent |
| Aider | `.aider.conf.yml` | CLI Agent |
| Copilot | `.github/copilot-instructions.md` | Copilot |
| Cline | `.clinerules` | Autonomous |
| Roo | `.roo/rules/` | Autonomous |
| Amazon Q | `.amazonq/rules/` | Autonomous |
| Antigravity | `.agent/rules.md` | Autonomous |

Custom tools can be defined in `.repository/tools/`.

---

## Installation

```bash
# From source
cargo install --path crates/repo-cli

# Or build locally
git clone <repo-url>
cd repository-manager
cargo build --release
./target/release/repo --help
```

---

## Quick Start

```bash
# 1. Initialize a project (creates .repository/config.toml)
repo init my-project --tools cursor,claude,vscode

# 2. Add more tools later
repo add-tool copilot
repo add-tool windsurf

# 3. Apply a language preset (sets sensible defaults)
repo add-preset rust

# 4. Add a custom rule
repo add-rule no-unwrap --instruction "Avoid .unwrap() in library code; use ? or handle errors explicitly"

# 5. Generate all tool configs from the source of truth
repo sync

# 6. Verify output
repo status
```

---

## Example Session

```
$ repo init . --tools cursor,claude,vscode,copilot
Initialized repository configuration at .repository/config.toml
Tools registered: cursor, claude, vscode, copilot

$ repo add-preset rust
Added preset: rust

$ repo sync
Syncing 4 tools...
  cursor      → .cursorrules               [written]
  claude      → CLAUDE.md                  [written]
  vscode      → .vscode/settings.json      [written]
  copilot     → .github/copilot-instructions.md [written]
Sync complete. 4 files written.

$ repo status
Repository: my-project (standard mode)
Tools:  cursor, claude, vscode, copilot
Rules:  no-unwrap (+ 3 from rust preset)
Status: in sync
```

---

## Key Commands

| Command | Description |
|---------|-------------|
| `repo init <name>` | Initialize repository configuration |
| `repo sync` | Generate all tool configs from source of truth |
| `repo status` | Show current sync status |
| `repo diff` | Preview what sync would change |
| `repo check` | Check for configuration drift |
| `repo fix` | Fix drift automatically |
| `repo add-tool <name>` | Add a tool integration |
| `repo remove-tool <name>` | Remove a tool integration |
| `repo add-preset <name>` | Apply a language/stack preset |
| `repo add-rule <id>` | Add a coding rule |
| `repo list-rules` | List active rules |
| `repo list-tools` | List available tools |
| `repo tool-info <name>` | Show tool details |
| `repo config show` | Display current configuration |
| `repo rules-lint` | Validate rule consistency |
| `repo rules-diff` | Show config drift |
| `repo rules-export` | Export rules to AGENTS.md |
| `repo rules-import` | Import rules from AGENTS.md |
| `repo hooks list/add/remove` | Manage lifecycle hooks |
| `repo extension install/list` | Manage extensions |
| `repo open <worktree>` | Open worktree in IDE |
| `repo branch add/remove/list` | Manage branches (worktree mode) |

---

## Configuration

`.repository/config.toml` is the single source of truth:

```toml
[core]
mode = "standard"   # standard | worktrees | container

[tools]
enabled = ["cursor", "claude", "vscode", "copilot"]

[presets]
enabled = ["rust"]
```

Rules live in `.repository/rules/` as Markdown files and are injected into each tool's config on sync.

---

## Crate Architecture

| Crate | Description |
|-------|-------------|
| **repo-cli** | Command-line interface (`repo` binary) |
| **repo-core** | Core orchestration and sync engine |
| **repo-fs** | Filesystem abstraction with atomic I/O |
| **repo-git** | Git operations and worktree management |
| **repo-tools** | 14 tool integrations and config writers |
| **repo-content** | Content parsing, editing, diffing |
| **repo-blocks** | Managed block system (UUID-tagged sections) |
| **repo-meta** | Ledger, registry, schema definitions |
| **repo-presets** | Environment presets (Python, Node, Rust) |
| **repo-extensions** | Extension lifecycle management |
| **repo-mcp** | MCP server (Model Context Protocol) |

---

## Development

```bash
cargo test --workspace
cargo check --workspace
cargo clippy --workspace
```

---

## Documentation

- [Project Overview](docs/project-overview.md) — Vision, architecture, and capabilities
- [Design Documentation](docs/design/_index.md) — Internal design specs

---

## License

MIT
