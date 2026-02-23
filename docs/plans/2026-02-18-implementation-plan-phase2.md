# Implementation Plan - Phase 2: Adoption Readiness

**Target Duration:** 2-5 weeks (after Phase 1)
**Goal:** Make Repository Manager ready for broad adoption
**Prerequisite:** Phase 1 complete (alpha released)
**Date:** 2026-02-18

> **Reference:** See [Phase 1 Plan](./2026-02-18-implementation-plan-phase1.md) for P0 prerequisites.
> The Phase 1 plan covers: release pipeline (cargo-dist), CI setup, README fixes, placeholder URL
> correction, CHANGELOG, and build prerequisite documentation.

---

## Sub-Phase 2A: Publishing & Governance (Weeks 3-4)

**ADRs covered:** [ADR-0007](../decisions/0007-publish-all-crates.md),
[ADR-0008](../decisions/0008-package-manager-channel-rollout.md),
[ADR-0009](../decisions/0009-cargo-deny-supply-chain-security.md),
[ADR-0015](../decisions/0015-declare-msrv-rust-1-85.md),
[ADR-0016](../decisions/0016-complete-cargo-metadata.md)

**Goal:** Professional crates.io presence, supply chain security, and multi-channel distribution.

---

### Task 2A-1: Declare MSRV in Workspace Cargo.toml

- **ADR:** [ADR-0015](../decisions/0015-declare-msrv-rust-1-85.md)
- **Files to modify:** `Cargo.toml` (workspace root)
- **Effort:** 30 minutes
- **Dependencies:** Phase 1 complete (repository URL must be fixed first per ADR-0007)
- **Acceptance Criteria:**
  - `[workspace.package]` contains `rust-version = "1.85"`
  - `cargo check` succeeds
  - CI matrix includes a Rust 1.85-specific job alongside stable
  - MSRV documented in README under "Requirements" section

**Changes to `Cargo.toml`:**
```toml
[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"
rust-version = "1.85"
repository = "https://github.com/YOUR_ORG/repository-manager"
```

**CI job to add in `.github/workflows/ci.yml`:**
```yaml
msrv:
  name: MSRV (Rust 1.85)
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@1.85
    - run: cargo check --workspace
```

---

### Task 2A-2: Complete Cargo Metadata for All Crates

- **ADR:** [ADR-0016](../decisions/0016-complete-cargo-metadata.md)
- **Files to modify:** `Cargo.toml` (workspace root)
- **Effort:** 1-2 hours
- **Dependencies:** Task 2A-1 (confirming actual GitHub URL), Phase 1 placeholder URL fix
- **Acceptance Criteria:**
  - `[workspace.package]` contains `homepage`, `documentation`, `keywords`, `categories`
  - `repository` URL is the real GitHub URL (not the placeholder)
  - All 11 publishable crates inherit metadata automatically
  - `cargo package --list` for any crate shows correct metadata fields
  - No crate-level overrides are needed (verify via `cargo metadata`)

**Full `[workspace.package]` block:**
```toml
[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"
rust-version = "1.85"
repository = "https://github.com/YOUR_ORG/repository-manager"
homepage = "https://github.com/YOUR_ORG/repository-manager"
documentation = "https://docs.rs/repo-cli"
keywords = ["cli", "workspace", "ai-agent", "configuration", "devtools"]
categories = ["command-line-utilities", "development-tools", "config"]
```

> Note: Replace `YOUR_ORG` with the actual GitHub org/user before publishing.
> The `documentation` field uses the docs.rs pattern; individual crates will resolve to
> their own docs.rs pages automatically once published.

---

### Task 2A-3: Set Up cargo-deny for Supply Chain Security

- **ADR:** [ADR-0009](../decisions/0009-cargo-deny-supply-chain-security.md)
- **Files to modify:**
  - `deny.toml` (create at workspace root)
  - `.github/workflows/ci.yml` (add deny job)
- **Effort:** 2-4 hours (including initial triage of findings)
- **Dependencies:** None (can run in parallel with other 2A tasks)
- **Acceptance Criteria:**
  - `deny.toml` exists at workspace root with advisories, licenses, bans, and sources sections
  - License allowlist permits MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Unicode-DFS-2016
  - Explicit `skip` entries for known C FFI crates: `unsafe-libyaml` (via serde_yaml) and `libgit2-sys` (via git2)
  - `cargo deny check` passes with zero errors locally
  - CI job runs `cargo deny check` on every PR and push to main
  - All advisories either resolved or have documented `ignore` entries with justification

**Generate initial config:**
```bash
cargo deny init
```

**Key `deny.toml` sections to configure:**
```toml
[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"

[licenses]
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-DFS-2016",
]
deny = ["GPL-2.0", "GPL-3.0", "AGPL-3.0"]
copyleft = "warn"

[[licenses.clarify]]
name = "ring"
# ring has a complex license; add if used transitively

[bans]
multiple-versions = "warn"
wildcards = "deny"

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
```

**CI job addition:**
```yaml
deny:
  name: Dependency Security (cargo-deny)
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: EmbarkStudios/cargo-deny-action@v1
      with:
        log-level: warn
        command: check
        arguments: --all-features
```

---

### Task 2A-4: Publish All Crates to crates.io

- **ADR:** [ADR-0007](../decisions/0007-publish-all-crates.md)
- **Files to modify:**
  - `Cargo.toml` (workspace root - already done in 2A-2)
  - `tests/integration/Cargo.toml` (verify `publish = false` from Phase 1)
  - `.cargo/credentials.toml` (local only - configure crates.io API token)
  - `release.toml` (create cargo-release configuration)
- **Effort:** 1 day
- **Dependencies:** Tasks 2A-1, 2A-2, 2A-3 all complete; Phase 1 complete; crates.io account
- **Acceptance Criteria:**
  - All 11 crates published to crates.io in correct dependency order
  - `cargo install repo-cli` succeeds from crates.io without cloning the repo
  - `cargo install repo-mcp` succeeds from crates.io
  - Each crate page on crates.io shows correct metadata (description, keywords, license, links)
  - docs.rs builds succeed for all crates

**Publish order (leaf-first, per ADR-0007):**
```
1. repo-fs
2. repo-content
3. repo-git
4. repo-blocks
5. repo-meta
6. repo-tools
7. repo-presets
8. repo-core
9. repo-agent
10. repo-mcp
11. repo-cli
```

**cargo-release configuration (`release.toml`):**
```toml
[workspace]
publish-order = [
    "repo-fs",
    "repo-content",
    "repo-git",
    "repo-blocks",
    "repo-meta",
    "repo-tools",
    "repo-presets",
    "repo-core",
    "repo-agent",
    "repo-mcp",
    "repo-cli",
]
```

**First publish command (dry-run first):**
```bash
# Dry run to verify all packages
cargo release publish --dry-run

# Actual publish (each crate in order due to path deps)
cargo release publish
```

**Verify installation works:**
```bash
cargo install repo-cli --version 0.1.0
repo --version
```

---

### Task 2A-5: Add Windows Package Manager Channels (Winget + Scoop)

- **ADR:** [ADR-0008](../decisions/0008-package-manager-channel-rollout.md)
- **Files to modify:**
  - `.github/workflows/release.yml` (add winget-releaser step)
  - Create Scoop bucket repository (separate GitHub repo: `scoop-repository-manager`)
- **Effort:** 4-8 hours
- **Dependencies:** Phase 1 cargo-dist setup complete; at least one GitHub Release tag exists
- **Acceptance Criteria:**
  - `winget install repository-manager` works on Windows (after manifest propagation, ~24-48h)
  - Scoop bucket JSON manifest is valid and parseable
  - GitHub Actions winget-releaser job runs automatically on new releases
  - Homebrew tap (from Phase 1 cargo-dist) is confirmed working on macOS
  - Docker PATH bug identified, documented as known issue (full fix is Phase 3)

**Winget releaser GitHub Actions step (add to release workflow):**
```yaml
- name: Submit to Winget
  uses: vedantmgoyal9/winget-releaser@v2
  with:
    identifier: YourOrg.RepositoryManager
    installers-regex: '\.msi$'
  env:
    WINGET_TOKEN: ${{ secrets.WINGET_TOKEN }}
```

**Scoop manifest template (`bucket/repo-cli.json`):**
```json
{
    "version": "0.1.0",
    "description": "A unified control plane for agentic development workspaces",
    "homepage": "https://github.com/YOUR_ORG/repository-manager",
    "license": "MIT",
    "architecture": {
        "64bit": {
            "url": "https://github.com/YOUR_ORG/repository-manager/releases/download/v0.1.0/repo-cli-x86_64-pc-windows-msvc.zip",
            "hash": "REPLACE_WITH_SHA256"
        }
    },
    "bin": "repo-cli.exe",
    "checkver": {
        "github": "https://github.com/YOUR_ORG/repository-manager"
    },
    "autoupdate": {
        "architecture": {
            "64bit": {
                "url": "https://github.com/YOUR_ORG/repository-manager/releases/download/v$version/repo-cli-x86_64-pc-windows-msvc.zip"
            }
        }
    }
}
```

---

## Sub-Phase 2B: Documentation & UX (Weeks 4-5)

**ADRs covered:** [ADR-0010](../decisions/0010-mcp-server-documentation-first.md),
[ADR-0011](../decisions/0011-default-standard-mode.md),
[ADR-0014](../decisions/0014-detection-only-presets-mise-integration.md),
[ADR-0017](../decisions/0017-vaultspec-optional-subsystem.md)

**Goal:** Fix UX defaults, accurate documentation, and user-facing guides for all key features.

---

### Task 2B-1: Fix Non-Interactive Init Default (Standard Mode)

- **ADR:** [ADR-0011](../decisions/0011-default-standard-mode.md)
- **Files to modify:**
  - `crates/repo-cli/src/commands/init.rs`
  - `crates/repo-cli/src/interactive.rs`
- **Effort:** 2-4 hours
- **Dependencies:** Phase 1 complete
- **Acceptance Criteria:**
  - `repo init` (non-interactive, no flags) creates standard mode repository
  - `repo init --interactive` lists `"worktrees (recommended for multi-agent workflows)"` first with index 0
  - `repo init --mode worktrees` still works for explicit worktrees selection
  - `repo init --mode standard` explicitly selects standard mode
  - Existing test suite passes (update any tests that assume worktrees default)

**Code change in `crates/repo-cli/src/commands/init.rs`:**
```rust
// Before (current behavior)
fn default_mode() -> RepoMode {
    RepoMode::Worktrees  // or equivalent
}

// After (ADR-0011)
fn default_mode() -> RepoMode {
    RepoMode::Standard
}
```

**Interactive prompt update in `crates/repo-cli/src/interactive.rs`:**
```rust
// Interactive mode: worktrees listed first with recommendation label
let mode_options = vec![
    "worktrees (recommended for multi-agent workflows)",
    "standard",
];
let selection = Select::new()
    .with_prompt("Repository mode")
    .items(&mode_options)
    .default(0)  // worktrees is default in interactive
    .interact()?;
```

---

### Task 2B-2: Write MCP Server Documentation Guide

- **ADR:** [ADR-0010](../decisions/0010-mcp-server-documentation-first.md)
- **Files to modify:**
  - `docs/guides/mcp-server.md` (create)
  - `README.md` (add link to MCP guide)
- **Effort:** 4-6 hours
- **Dependencies:** Phase 1 complete (users can install the binary)
- **Acceptance Criteria:**
  - `docs/guides/mcp-server.md` exists with installation, configuration, and client-specific sections
  - Guide covers Claude Desktop, Cursor, VS Code, and generic MCP client setup
  - Each client section includes a working JSON config snippet
  - README links to the guide under a "MCP Server" section
  - A developer unfamiliar with MCP can follow the guide and connect their client in under 5 minutes

**Document structure for `docs/guides/mcp-server.md`:**
```markdown
# MCP Server Guide

## Overview
Repository Manager ships a built-in MCP server (`repo-mcp`) with 25 tools and 3 resources.
Transport: JSON-RPC over stdio.

## Installation
...

## Client Configuration

### Claude Desktop
Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "repository-manager": {
      "command": "repo-mcp",
      "args": [],
      "env": {}
    }
  }
}
```

### Cursor
Add to `.cursor/mcp.json` in your project:
```json
{
  "mcpServers": {
    "repository-manager": {
      "command": "repo-mcp",
      "args": []
    }
  }
}
```

### VS Code (with MCP extension)
...

## Available Tools
[List all 25 tool names and descriptions]

## Available Resources
[List all 3 resource URIs]
```

---

### Task 2B-3: Fix Preset Documentation (Detection-Only Behavior)

- **ADR:** [ADR-0014](../decisions/0014-detection-only-presets-mise-integration.md)
- **Files to modify:**
  - `docs/project-overview.md`
  - `README.md` (preset description section)
- **Effort:** 1-2 hours
- **Dependencies:** None
- **Acceptance Criteria:**
  - `docs/project-overview.md` no longer says presets "automatically installs binaries" or "creates virtual environments"
  - Replaced with: "detects and verifies environment configuration"
  - README preset description accurately describes current detection-only behavior
  - A GitHub issue is created tracking future mise integration
  - All mentions of installation via presets are updated across all documentation files

**Specific text changes:**

In `docs/project-overview.md`, locate and update:
```
# Before (inaccurate)
"automatically install binaries, create virtual environments"

# After (accurate per ADR-0014)
"detect and verify development environment configuration (Python/UV/venv, Node, Rust)"
```

**Search for all inaccurate preset claims:**
```bash
grep -r "install" docs/ --include="*.md" | grep -i "preset\|provider"
```

---

### Task 2B-4: Document Vaultspec as Optional Subsystem

- **ADR:** [ADR-0017](../decisions/0017-vaultspec-optional-subsystem.md)
- **Files to modify:**
  - `docs/guides/agent-spawning.md` (create)
  - `README.md` (add feature matrix distinguishing core vs optional)
  - `crates/repo-cli/src/commands/plugins.rs` (make version dynamic)
- **Effort:** 4-6 hours
- **Dependencies:** None
- **Acceptance Criteria:**
  - `docs/guides/agent-spawning.md` explains vaultspec requirements (Python 3.13+, `.vaultspec/` dir)
  - README contains an "Implementation Status" matrix with done/partial/planned markers
  - README clearly distinguishes core features (no external deps) from optional features (require vaultspec)
  - `repo plugins status` shows dynamic vaultspec version from discovery, not hardcoded "v4.1.1"
  - `repo agent spawn` gives a clear, actionable error when vaultspec is not installed

**Implementation Status Matrix for README:**
```markdown
## Implementation Status

### Core Features (no external dependencies)
| Feature | Status |
|---------|--------|
| `repo init` - workspace initialization | Done |
| `repo sync` - config generation for 14 tools | Done |
| `repo check` / `repo fix` - drift detection | Done |
| `repo diff` - preview sync changes | Done |
| Worktree management (branch add/remove/list) | Done |
| Rules system (add/remove/list/lint/diff) | Done |
| AGENTS.md export/import | Done |
| MCP server (25 tools, 3 resources) | Done |
| Shell completions (bash/zsh/fish/PowerShell) | Done |

### Optional Features (require vaultspec + Python 3.13+)
| Feature | Status |
|---------|--------|
| `repo agent spawn/stop/status` | Partial (discovery works; spawning requires vaultspec) |
| `repo plugins install/status` | Partial (hardcoded to vaultspec; no general plugin system) |

### Planned Features
| Feature | Status |
|---------|--------|
| Snap mode (one-command agent lifecycle) | Planned (ADR-0012) |
| MCP server config propagation | Planned (ADR-0010) |
| Expand to 20+ tool integrations | Planned (ADR-0013) |
| mise integration for runtime installation | Planned (ADR-0014) |
```

**Code change in `crates/repo-cli/src/commands/plugins.rs`:**
```rust
// Before (hardcoded)
let version = "vaultspec v4.1.1";

// After (dynamic discovery)
let version = discover_vaultspec_version()
    .unwrap_or_else(|| "vaultspec (not installed)".to_string());

fn discover_vaultspec_version() -> Option<String> {
    // Run: vaultspec --version or python -m vaultspec --version
    // Parse output to extract version string
    let output = std::process::Command::new("python")
        .args(["-m", "vaultspec", "--version"])
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}
```

---

### Task 2B-5: Create CONTRIBUTING.md and SECURITY.md

- **ADR:** Referenced in feature gap analysis (P1-5)
- **Files to modify:**
  - `CONTRIBUTING.md` (create at workspace root)
  - `SECURITY.md` (create at workspace root)
- **Effort:** 2-4 hours
- **Dependencies:** None
- **Acceptance Criteria:**
  - `CONTRIBUTING.md` covers: development setup, running tests, adding tool integrations, PR process
  - `CONTRIBUTING.md` includes a section on adding new tool integrations (how to use GenericToolIntegration)
  - `SECURITY.md` provides a responsible disclosure policy and contact method
  - Both files are linked from README

**Key sections for `CONTRIBUTING.md`:**
```markdown
## Adding a New Tool Integration

Repository Manager uses a generic integration framework that makes adding new tools
a configuration task, not new Rust code.

### Option A: Generic Integration (preferred for most tools)
Create a tool definition in `.repository/tools/<toolname>.toml`:
```toml
[tool]
name = "my-tool"
config_path = ".my-tool/config.json"
format = "json"
```

### Option B: Built-in Integration (for tools needing custom logic)
1. Create `crates/repo-tools/src/my_tool.rs`
2. Implement `ToolIntegration` trait
3. Register in `crates/repo-tools/src/registry/builtin.rs`
4. Add test in `crates/repo-tools/src/my_tool/tests.rs`
```

---

## Sub-Phase 2C: Feature Development (Weeks 5-7)

**ADRs covered:** [ADR-0012](../decisions/0012-snap-mode-agent-lifecycle.md),
[ADR-0013](../decisions/0013-expand-tool-support-generic-integration.md)

**Goal:** Close the two most important competitive feature gaps: tool count and agent lifecycle UX.

---

### Task 2C-1: Expand Tool Support to 20+ Integrations

- **ADR:** [ADR-0013](../decisions/0013-expand-tool-support-generic-integration.md)
- **Files to modify:**
  - `crates/repo-tools/src/` (add new tool modules or generic definitions)
  - `crates/repo-tools/src/registry/builtin.rs` (register new tools)
  - `README.md` (update tool count from "13" to actual count)
- **Effort:** 1-2 days per tool; 6 tools = 6-12 days total; can be parallelized
- **Dependencies:** Task 2B-5 (CONTRIBUTING.md with tool addition guide)
- **Acceptance Criteria:**
  - Tool count reaches 20+ (currently 14 actual, 13 documented)
  - README tool count is corrected from "13" to accurate number immediately (regardless of new tools)
  - Each new tool has: config file path researched and verified, tool definition or module created, integration test passing
  - `repo sync` successfully generates config for each new tool
  - Contributor documentation explains how to add new integrations

**Priority tool additions (from ADR-0013):**

| Tool | Config File | Format | Effort |
|------|-------------|--------|--------|
| OpenCode | `.opencode/config.json` | JSON | 1 day |
| Kilo Code | `.kilocode/config.json` or similar | JSON | 1 day |
| Continue.dev | `.continue/config.json` | JSON | 1 day |
| Amp | `.amp/config.toml` or similar | TOML | 1 day |
| Codex CLI | `~/.codex/config.json` | JSON | 1 day |
| Kiro | `.kiro/config.json` or similar | JSON | 1 day |

**Research checklist per tool:**
```bash
# 1. Find official documentation for config file location
# 2. Create sample config to understand format
# 3. Check if GenericToolIntegration covers the format
# 4. If yes, create tool definition file in .repository/tools/
# 5. If no, create dedicated module in crates/repo-tools/src/
# 6. Write integration test
# 7. Register in builtin registry
```

**Example generic tool definition (`.repository/tools/opencode.toml`):**
```toml
[tool]
name = "opencode"
display_name = "OpenCode"
description = "AI coding assistant by OpenCode"
config_path = ".opencode/rules.md"
format = "markdown"
supports_rules = true
```

**README fix (immediate - no code change needed):**
- Change `"Supports 13 AI coding tools"` to `"Supports 14 AI coding tools"` (accurate count today)
- Update the tool list to include all 14 currently supported tools

---

### Task 2C-2: Implement Snap Mode for Agent Lifecycle

- **ADR:** [ADR-0012](../decisions/0012-snap-mode-agent-lifecycle.md)
- **Files to modify:**
  - `crates/repo-cli/src/commands/branch.rs` (add `--snap` flag)
  - `crates/repo-agent/src/snap.rs` (create - snap mode orchestration logic)
  - `crates/repo-git/src/helpers.rs` (add file-copy utilities for .env and gitignored files)
  - `crates/repo-core/src/hooks.rs` (verify post-branch-create hook fires)
- **Effort:** 3-5 days
- **Dependencies:** Task 2B-4 (vaultspec documented as optional); Task 2B-1 (init defaults fixed)
- **Acceptance Criteria:**
  - `repo branch add my-feature --snap claude` executes the full snap workflow
  - Snap workflow steps: (1) create worktree, (2) copy .env/.envrc/gitignored files, (3) launch agent if vaultspec available, (4) clean up on completion
  - `repo branch add my-feature --snap claude --no-cleanup` leaves worktree intact after agent completes
  - When vaultspec is unavailable, snap mode completes steps 1-2 and prints a clear message about step 3 being skipped
  - `repo branch add my-feature` (without --snap) behavior is unchanged
  - Integration test covers: snap with mock agent, snap without vaultspec (graceful degradation), snap with --no-cleanup

**CLI interface (`crates/repo-cli/src/commands/branch.rs`):**
```rust
#[derive(Args)]
pub struct BranchAddArgs {
    /// Branch name
    pub name: String,

    /// Enable snap mode: create worktree, copy files, launch agent, cleanup
    #[arg(long, value_name = "AGENT")]
    pub snap: Option<String>,

    /// With --snap: skip cleanup after agent completes (for debugging)
    #[arg(long, requires = "snap")]
    pub no_cleanup: bool,
}
```

**Snap mode orchestration (`crates/repo-agent/src/snap.rs`):**
```rust
pub struct SnapConfig {
    pub branch_name: String,
    pub agent: String,
    pub no_cleanup: bool,
}

pub async fn run_snap_mode(config: SnapConfig) -> Result<()> {
    // Step 1: Create worktree
    create_worktree(&config.branch_name)?;

    // Step 2: Copy gitignored files (.env, .envrc, etc.)
    copy_gitignored_files(&config.branch_name)?;

    // Step 3: Launch agent (graceful degradation if not available)
    match discover_agent(&config.agent) {
        Ok(agent_path) => {
            launch_agent(agent_path, &config.branch_name).await?;
        }
        Err(e) => {
            eprintln!("Note: {} not found, skipping agent launch: {}", config.agent, e);
        }
    }

    // Step 4: Cleanup (unless --no-cleanup)
    if !config.no_cleanup {
        remove_worktree(&config.branch_name)?;
    }

    Ok(())
}
```

**File copy utility (`crates/repo-git/src/helpers.rs` addition):**
```rust
/// Copy .env, .envrc, and all gitignored files from the main worktree
/// into the newly created worktree directory.
pub fn copy_gitignored_files(worktree_path: &Path) -> Result<Vec<PathBuf>> {
    // Use git check-ignore --stdin to identify gitignored files
    // Copy each to the new worktree path
    // Return list of copied files for logging
    todo!("implement gitignored file copying")
}
```

---

## Dependency Graph

```
Phase 1 (prerequisite)
    |
    +-- 2A-1: Declare MSRV (Rust 1.85)
    |       |
    |       +-- 2A-2: Complete Cargo Metadata
    |               |
    |               +-- 2A-4: Publish All Crates to crates.io
    |
    +-- 2A-3: Set up cargo-deny (independent)
    |
    +-- 2A-5: Winget + Scoop channels (requires Phase 1 release)
    |
    +-- 2B-1: Fix init default to standard mode (independent)
    |
    +-- 2B-2: MCP Server Documentation (requires installable binary)
    |
    +-- 2B-3: Fix preset documentation (independent)
    |
    +-- 2B-4: Vaultspec optional docs + dynamic version
    |
    +-- 2B-5: CONTRIBUTING.md + SECURITY.md
    |       |
    |       +-- 2C-1: Expand tool support (needs contributor guide)
    |
    +-- 2C-2: Snap mode (requires 2B-4 vaultspec docs, 2B-1 init fix)
```

**Critical path:** 2A-1 → 2A-2 → 2A-4 (publishing chain)

**Parallel tracks:**
- Track A (Publishing): 2A-1 → 2A-2 → 2A-4
- Track B (Security): 2A-3 (independent)
- Track C (Windows): 2A-5 (after Phase 1 release)
- Track D (Docs): 2B-1, 2B-2, 2B-3, 2B-4, 2B-5 (mostly independent of each other)
- Track E (Features): 2C-1 (after 2B-5), 2C-2 (after 2B-1 and 2B-4)

---

## Milestone: Beta Release Checklist

- [ ] **All P1 ADRs implemented** (ADRs 0007-0017)
- [ ] **Published on crates.io** - `cargo install repo-cli` works from crates.io (Task 2A-4)
- [ ] **MSRV declared** - `rust-version = "1.85"` in workspace Cargo.toml (Task 2A-1)
- [ ] **Complete crates.io metadata** - keywords, categories, homepage, documentation (Task 2A-2)
- [ ] **Supply chain security** - `cargo deny check` passes in CI (Task 2A-3)
- [ ] **Homebrew tap available** - automated by Phase 1 cargo-dist setup
- [ ] **Winget manifest submitted** - Windows users can `winget install` (Task 2A-5)
- [ ] **20+ tool integrations** - closes competitive gap with Ruler/rulesync (Task 2C-1)
- [ ] **README tool count accurate** - not "13", reflects actual count (Task 2C-1)
- [ ] **MCP server documented** - `docs/guides/mcp-server.md` with client examples (Task 2B-2)
- [ ] **Snap mode working** - `repo branch add --snap <agent>` executes full lifecycle (Task 2C-2)
- [ ] **Snap graceful degradation** - works without vaultspec for steps 1-2 (Task 2C-2)
- [ ] **Init default fixed** - non-interactive defaults to standard mode (Task 2B-1)
- [ ] **Preset docs accurate** - no "installs binaries" claim (Task 2B-3)
- [ ] **Vaultspec documented** - `docs/guides/agent-spawning.md` exists (Task 2B-4)
- [ ] **Implementation status matrix** - README shows done/partial/planned (Task 2B-4)
- [ ] **CONTRIBUTING.md** - includes tool addition guide (Task 2B-5)
- [ ] **SECURITY.md** - responsible disclosure policy (Task 2B-5)
- [ ] **cargo-deny passing** - zero advisory/license violations in CI (Task 2A-3)
- [ ] **MSRV CI job** - Rust 1.85 build verified in CI (Task 2A-1)

---

## Effort Summary

| Task | Sub-Phase | Effort | Priority |
|------|-----------|--------|----------|
| 2A-1: Declare MSRV | 2A | 30 min | High |
| 2A-2: Complete Cargo metadata | 2A | 1-2 hours | High |
| 2A-3: Set up cargo-deny | 2A | 2-4 hours | High |
| 2A-4: Publish to crates.io | 2A | 1 day | High |
| 2A-5: Winget + Scoop channels | 2A | 4-8 hours | Medium |
| 2B-1: Fix init default | 2B | 2-4 hours | High |
| 2B-2: MCP server docs | 2B | 4-6 hours | High |
| 2B-3: Fix preset docs | 2B | 1-2 hours | High |
| 2B-4: Vaultspec optional docs | 2B | 4-6 hours | High |
| 2B-5: CONTRIBUTING + SECURITY | 2B | 2-4 hours | Medium |
| 2C-1: Expand to 20+ tools | 2C | 6-12 days | High |
| 2C-2: Snap mode | 2C | 3-5 days | High |

**Total estimated effort: 2-5 weeks**
- Sub-Phase 2A alone: ~2-3 days (mostly configuration/metadata)
- Sub-Phase 2B alone: ~2-3 days (documentation and small code changes)
- Sub-Phase 2C alone: ~2-3 weeks (the two major feature implementations)
- With parallelization across tracks: 2-5 weeks total

---

## Phase 3 Preview

After Phase 2 completes, Phase 3 focuses on competitive parity and post-GA features:
- MCP server config propagation (write MCP config into Claude Desktop/Cursor/VS Code files)
- Remote rule includes (pull rules from git/HTTP URLs)
- Profile/team variants for different contexts
- Tool config import (migrate from `.cursorrules`, `CLAUDE.md` etc.)
- `repo doctor` diagnostic command
- Docker image (fix PATH bug, publish to ghcr.io)
- Marketing launch (blog post, HN/r/rust announcement)

See [Phase 3 detail in the Master Implementation Plan](./2026-02-18-implementation-plan.md) for details.
