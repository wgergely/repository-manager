# Implementation Plan - Phase 1: Ship Alpha

**Target Duration:** 1-2 weeks
**Goal:** Resolve all P0 blockers for a credible public alpha release
**Date:** 2026-02-18
**ADR Sources:** ADR-0001 through ADR-0006

---

## Prerequisites

Before starting any task in this plan:

1. **Git access confirmed** - You must be able to push to the real GitHub repository
2. **Real repository URL known** - The GitHub URL that will replace the placeholder
3. **Rust toolchain installed** - `rustup show` confirms stable toolchain is active
4. **cargo-dist installed** - `cargo install cargo-dist` (or `cargo binstall cargo-dist`)
5. **cargo-release installed** - `cargo install cargo-release` (or `cargo binstall cargo-release`)
6. **CI/CD access** - GitHub Actions enabled on the repository
7. **Branch protection** - You are working in a feature branch per worktree workflow rules

> Note: Per worktree workflow, all changes go through feature branches. Create a feature worktree
> at the container level before starting:
> ```bash
> cd Y:\code\repository-manager-worktrees
> git worktree add phase1-alpha -b phase1-alpha
> ```

---

## Task Breakdown

### Week 1: Foundation

All Week 1 tasks can be batched into a single feature branch. They are primarily configuration
and metadata changes - none touch application logic.

---

#### Task 1.1: Fix the placeholder repository URL

- **Task**: Replace `https://github.com/user/repository-manager` with the real GitHub repository URL in workspace `Cargo.toml`. This propagates to all 11 published crates via `[workspace.package]`.
- **ADR**: [ADR-0002](../decisions/0002-cargo-dist-and-cargo-release.md) (prerequisite for cargo-release and crates.io publishing)
- **Files to modify**:
  - `Y:\code\repository-manager-worktrees\main\Cargo.toml` — line 9
- **Effort**: 30 minutes
- **Dependencies**: None (do this first)
- **Acceptance Criteria**:
  - `grep -r "github.com/user" .` returns no matches
  - `cargo metadata --no-deps | jq '.packages[].repository'` shows the real URL for all crates
- **Commands to run**:
  ```bash
  # After editing Cargo.toml:
  cargo metadata --no-deps --format-version 1 | \
    python3 -c "import sys,json; pkgs=json.load(sys.stdin)['packages']; [print(p['name'],p.get('repository')) for p in pkgs]"
  ```
- **Code change**:
  ```toml
  # In Cargo.toml, line 9 - change from:
  repository = "https://github.com/user/repository-manager"
  # to:
  repository = "https://github.com/YOUR_ORG/repository-manager"
  ```

---

#### Task 1.2: Add git2 vendored feature

- **Task**: Change `git2 = "0.20"` to `git2 = { version = "0.20", features = ["vendored"] }` in workspace `Cargo.toml`. This statically links libgit2, eliminating the undocumented cmake/C compiler/libssl-dev build prerequisites for end users. Also update `docs/design/key-decisions.md` to document the gix deferral decision.
- **ADR**: [ADR-0001](../decisions/0001-git2-vendored-feature.md)
- **Files to modify**:
  - `Y:\code\repository-manager-worktrees\main\Cargo.toml` — line 30 (git2 dependency)
  - `Y:\code\repository-manager-worktrees\main\docs\design\key-decisions.md` — resolve gix vs git2 contradiction
- **Effort**: 1-2 hours (including the longer compile time for vendored build)
- **Dependencies**: None (can be done in parallel with Task 1.1)
- **Acceptance Criteria**:
  - `cargo build --release` succeeds on a machine **without** cmake, libssl-dev, or pkg-config installed
  - Binary size increases are acceptable (vendored libgit2 adds ~2-3 MB before LTO)
  - `key-decisions.md` no longer contradicts the codebase by mentioning gix as the current library
- **Commands to run**:
  ```bash
  # Verify vendored build works:
  cargo build --release 2>&1 | grep -E "(error|warning|Compiling libgit2)"
  # Check binary size before and after LTO is added:
  ls -lh target/release/repo
  ```
- **Code change**:
  ```toml
  # In Cargo.toml, change line ~30 from:
  git2 = "0.20"
  # to:
  git2 = { version = "0.20", features = ["vendored"] }
  ```
- **Note**: The vendored build compiles libgit2 from source in CI. This is expected and acceptable
  per ADR-0001 (compile-time cost falls on CI, not users). First local build after this change
  will take 2-5 minutes longer.

---

#### Task 1.3: Add release profile optimizations

- **Task**: Add `[profile.release]` section to workspace `Cargo.toml` with `lto = true`, `codegen-units = 1`, `strip = true`. This reduces distributed binary size by 30-50%.
- **ADR**: [ADR-0003](../decisions/0003-release-profile-optimizations.md)
- **Files to modify**:
  - `Y:\code\repository-manager-worktrees\main\Cargo.toml` — add section after `[workspace.dependencies]`
- **Effort**: 30 minutes
- **Dependencies**: Task 1.2 (do after vendored build is confirmed working, so you can measure size delta)
- **Acceptance Criteria**:
  - `cargo build --release` succeeds
  - `target/release/repo` binary is at least 20% smaller than without LTO (measure before and after)
  - No debug symbols in the released binary: `file target/release/repo` shows "stripped" on Linux/macOS
- **Commands to run**:
  ```bash
  # Measure size before adding profile:
  ls -lh target/release/repo
  # Add the profile section, then rebuild:
  cargo build --release
  ls -lh target/release/repo
  # On Linux/macOS, confirm stripped:
  file target/release/repo
  ```
- **Code change**:
  ```toml
  # Add to Cargo.toml after [workspace.dependencies]:
  [profile.release]
  lto = true
  codegen-units = 1
  strip = true
  ```
- **Warning**: LTO builds take significantly longer. First build after this change may take
  5-15 minutes. This is expected and only affects release builds (dev builds are unaffected).

---

#### Task 1.4: Fix --tools comma-delimiter bug

- **Task**: Add `value_delimiter = ','` to the `tools` and `presets` clap attributes in `cli.rs`. This makes `repo init --tools cursor,claude,vscode` work as documented. Currently the comma-separated string is treated as a single tool name, silently failing.
- **ADR**: [ADR-0005](../decisions/0005-comma-delimited-tools-flag.md)
- **Files to modify**:
  - `Y:\code\repository-manager-worktrees\main\crates\repo-cli\src\cli.rs` — lines 55-57 (tools field) and the presets field (nearby)
- **Effort**: 30 minutes
- **Dependencies**: None
- **Acceptance Criteria**:
  - `repo init test-proj --tools cursor,claude,vscode` creates a config with all three tools enabled
  - `repo init test-proj -t cursor -t claude -t vscode` also works (repeated flag syntax unchanged)
  - `repo init test-proj --tools cursor` (single tool, no comma) still works
  - The README Quick Start example runs without silent failure
- **Commands to run**:
  ```bash
  # Test comma syntax:
  cargo run --bin repo -- init /tmp/test-comma --tools cursor,claude,vscode
  cat /tmp/test-comma/.repository/config.toml | grep -A5 "tools"
  # Test repeated flag syntax still works:
  cargo run --bin repo -- init /tmp/test-repeat -t cursor -t claude
  cat /tmp/test-repeat/.repository/config.toml | grep -A5 "tools"
  # Test single tool:
  cargo run --bin repo -- init /tmp/test-single --tools cursor
  ```
- **Code change**:
  ```rust
  // In crates/repo-cli/src/cli.rs, change the tools field from:
  #[arg(short, long)]
  tools: Vec<String>,
  // to:
  #[arg(short, long, value_delimiter = ',')]
  tools: Vec<String>,

  // Apply the same fix to the presets field:
  #[arg(short, long, value_delimiter = ',')]
  presets: Vec<String>,
  ```

---

#### Task 1.5: Update README with correct syntax and prerequisites

- **Task**: Fix the README Quick Start to use correct CLI syntax and add a Prerequisites section listing Rust as the only build prerequisite (build prerequisites are eliminated by the vendored git2 from Task 1.2). Also update the Installation section to reference the future binary release channel and document the `cargo install` path.
- **ADR**: [ADR-0004](../decisions/0004-agentic-workspace-manager-positioning.md) (repositioning), [ADR-0005](../decisions/0005-comma-delimited-tools-flag.md) (correct syntax)
- **Files to modify**:
  - `Y:\code\repository-manager-worktrees\main\README.md`
- **Effort**: 2-3 hours
- **Dependencies**: Task 1.4 (confirm comma syntax works before documenting it)
- **Acceptance Criteria**:
  - Every command in the Quick Start section runs successfully when copy-pasted by a new user
  - Prerequisites section lists only: Rust (stable), git
  - Installation section includes: pre-built binaries (coming with v0.1.0), `cargo install repo-cli` (coming), `cargo build --release` (from source)
  - README tagline updated to "Agentic Workspace Manager" positioning per ADR-0004
  - The README no longer states `--tools cursor,claude,vscode` without the `value_delimiter` fix being in place
- **Key changes**:
  ```markdown
  ## Prerequisites
  - Rust (stable) — install via [rustup.rs](https://rustup.rs)
  - git

  ## Installation
  ### Pre-built binaries (recommended)
  Download from [GitHub Releases](https://github.com/YOUR_ORG/repository-manager/releases)

  ### Via cargo
  cargo install repo-cli  # coming soon on crates.io

  ### From source
  git clone https://github.com/YOUR_ORG/repository-manager
  cd repository-manager
  cargo build --release
  ./target/release/repo --version

  ## Quick Start
  # Initialize with comma-separated tools (works after ADR-0005 fix):
  repo init my-project --tools cursor,claude,vscode

  # Or use repeated flags:
  repo init my-project -t cursor -t claude -t vscode
  ```

---

#### Task 1.6: Create CHANGELOG.md

- **Task**: Create a `CHANGELOG.md` at the workspace root using Keep a Changelog format. Document v0.1.0 features. This is required for any versioned software release.
- **ADR**: Informed by marketing audit consolidated report (blocker #6)
- **Files to create**:
  - `Y:\code\repository-manager-worktrees\main\CHANGELOG.md`
- **Effort**: 2 hours
- **Dependencies**: None (can be done in parallel with other tasks)
- **Acceptance Criteria**:
  - File follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format
  - v0.1.0 section documents all implemented features (init, sync, check, fix, diff, status, 13 tool integrations, MCP server, shell completions, worktree support)
  - An `[Unreleased]` section exists at the top for future changes
- **Template**:
  ```markdown
  # Changelog

  All notable changes to this project will be documented in this file.

  The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
  and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

  ## [Unreleased]

  ## [0.1.0] - 2026-02-18

  ### Added
  - `repo init` command with interactive mode, tool selection, preset selection
  - `repo sync` command with dry-run and JSON output modes
  - `repo check` command for drift detection
  - `repo fix` command for automatic drift repair
  - `repo diff` command to preview sync changes
  - `repo status` command with color-coded and JSON output
  - 13 tool integrations: Cursor, Claude, VS Code, Windsurf, Gemini, Copilot, Cline, Roo, JetBrains, Zed, Aider, Amazon Q, Antigravity
  - Shell completions for bash, zsh, fish, PowerShell, elvish
  - Git worktree support with three layout strategies (Classic, Container, InRepoWorktrees)
  - MCP server for AI agent integration (JSON-RPC over stdio)
  - Rules system with lint, diff, and AGENTS.md export/import
  - Hooks system for lifecycle automation
  - Branch management commands (add, remove, list, checkout, rename)
  ```

---

### Week 2: Release Pipeline

Week 2 tasks set up the release infrastructure. These are higher-effort and require external tooling.

---

#### Task 2.1: Add comprehensive Rust CI workflow

- **Task**: Create `.github/workflows/ci.yml` with a full Rust CI pipeline: `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --check`, cross-platform matrix. Add `.github/dependabot.yml` for automated dependency updates.
- **ADR**: [ADR-0006](../decisions/0006-rust-ci-workflow.md)
- **Files to create**:
  - `Y:\code\repository-manager-worktrees\main\.github\workflows\ci.yml`
  - `Y:\code\repository-manager-worktrees\main\.github\dependabot.yml`
- **Effort**: 2-4 hours
- **Dependencies**: Tasks 1.1-1.4 merged (CI will run on the fixed codebase)
- **Acceptance Criteria**:
  - CI runs on every PR targeting main
  - Matrix includes ubuntu-latest, windows-latest, macos-latest
  - All three jobs pass: test, clippy, fmt
  - `cargo deny check` job exists but is gated (requires `deny.toml` - see ADR-0009, P1)
  - Dependabot configured for weekly Cargo dependency updates
- **Commands to verify locally before pushing**:
  ```bash
  cargo test --workspace
  cargo clippy --workspace -- -D warnings
  cargo fmt --check
  ```
- **File content for `.github/workflows/ci.yml`**:
  ```yaml
  name: CI

  on:
    push:
      branches: [main]
    pull_request:
      branches: [main]

  env:
    CARGO_TERM_COLOR: always
    RUST_BACKTRACE: 1

  jobs:
    test:
      name: Test (${{ matrix.os }})
      runs-on: ${{ matrix.os }}
      strategy:
        fail-fast: false
        matrix:
          os: [ubuntu-latest, windows-latest, macos-latest]
      steps:
        - uses: actions/checkout@v4
        - name: Install Rust stable
          uses: dtolnay/rust-toolchain@stable
        - name: Cache cargo registry
          uses: Swatinem/rust-cache@v2
        - name: Run tests
          run: cargo test --workspace

    clippy:
      name: Clippy
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
        - name: Install Rust stable with clippy
          uses: dtolnay/rust-toolchain@stable
          with:
            components: clippy
        - name: Cache cargo registry
          uses: Swatinem/rust-cache@v2
        - name: Run clippy
          run: cargo clippy --workspace -- -D warnings

    fmt:
      name: Format
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
        - name: Install Rust stable with rustfmt
          uses: dtolnay/rust-toolchain@stable
          with:
            components: rustfmt
        - name: Check formatting
          run: cargo fmt --check
  ```
- **File content for `.github/dependabot.yml`**:
  ```yaml
  version: 2
  updates:
    - package-ecosystem: cargo
      directory: "/"
      schedule:
        interval: weekly
        day: monday
      open-pull-requests-limit: 5
      reviewers:
        - YOUR_GITHUB_USERNAME
    - package-ecosystem: github-actions
      directory: "/"
      schedule:
        interval: weekly
        day: monday
  ```

---

#### Task 2.2: Bootstrap cargo-dist release pipeline

- **Task**: Run `cargo dist init` to generate the GitHub Actions release workflow. This produces `.github/workflows/release.yml` and updates `Cargo.toml` with cargo-dist configuration. Configure for Linux (x86_64-musl), macOS (arm64, x86_64), and Windows (x86_64) targets.
- **ADR**: [ADR-0002](../decisions/0002-cargo-dist-and-cargo-release.md)
- **Files created/modified** (by cargo-dist init):
  - `Y:\code\repository-manager-worktrees\main\.github\workflows\release.yml` (generated)
  - `Y:\code\repository-manager-worktrees\main\Cargo.toml` (dist metadata added)
  - `Y:\code\repository-manager-worktrees\main\dist-workspace.toml` (may be created)
- **Effort**: 2-4 hours (setup + reviewing generated workflow)
- **Dependencies**: Task 1.1 (real repository URL required), Task 1.3 (release profile in place)
- **Acceptance Criteria**:
  - `cargo dist plan` runs without errors and shows all target platforms
  - `.github/workflows/release.yml` exists and triggers on version tag push (`v*.*.*`)
  - Homebrew tap configuration is present in generated workflow
  - Shell installer URL will be `https://github.com/YOUR_ORG/repository-manager/releases/download/v0.1.0/repo-installer.sh`
- **Commands to run**:
  ```bash
  # Install cargo-dist if not present:
  cargo install cargo-dist

  # From the workspace root (main/ directory):
  cargo dist init

  # Review and accept the prompts:
  # - platforms: linux-x86_64-musl, macos-aarch64, macos-x86_64, windows-x86_64
  # - installers: shell, homebrew (if you have a tap repo)
  # - CI: github

  # Verify plan works:
  cargo dist plan

  # Check what would be built:
  cargo dist plan --verbose
  ```
- **Post-init review checklist**:
  - Confirm `[workspace.metadata.dist]` section in Cargo.toml has correct targets
  - Confirm release workflow triggers on `v[0-9]+.[0-9]+.[0-9]+` tags
  - Confirm `repo-cli` binary is listed as the distributable artifact

---

#### Task 2.3: Configure cargo-release for workspace

- **Task**: Create `.config/release.toml` to configure cargo-release for coordinated version bumping across all 11 workspace crates. This enables `cargo release minor` to bump all crates together and push a signed tag.
- **ADR**: [ADR-0002](../decisions/0002-cargo-dist-and-cargo-release.md)
- **Files to create**:
  - `Y:\code\repository-manager-worktrees\main\.config\release.toml`
- **Effort**: 1-2 hours
- **Dependencies**: Task 1.1 (real repository URL), Task 2.2 (cargo-dist in place)
- **Acceptance Criteria**:
  - `cargo release --dry-run patch` shows all 11 crates being bumped together
  - No path dependency errors during dry run
  - Tag format is `v{{version}}` (required by cargo-dist release workflow trigger)
- **Commands to run**:
  ```bash
  # Install cargo-release if not present:
  cargo install cargo-release

  # Dry run to verify configuration:
  cargo release --dry-run patch

  # Verify it would tag correctly:
  cargo release --dry-run patch 2>&1 | grep -E "(tag|publish|version)"
  ```
- **File content for `.config/release.toml`**:
  ```toml
  # cargo-release configuration for repository-manager workspace
  # See: https://github.com/crate-ci/cargo-release/blob/master/docs/reference.md

  # Bump all workspace crates together
  [workspace]
  shared-version = true

  # Tag format must match cargo-dist release trigger: v{{version}}
  tag-name = "v{{version}}"

  # Commit message for version bump
  pre-release-commit-message = "chore: release v{{version}}"

  # Push tag to trigger cargo-dist release workflow
  push = true
  tag = true

  # Do not publish to crates.io yet (path dependencies not resolved until P1)
  publish = false
  ```
- **Note**: `publish = false` is intentional for Phase 1. crates.io publishing with
  path dependency resolution is a P1 task (ADR-0007).

---

#### Task 2.4: Create first release tag (v0.1.0)

- **Task**: Create and push the v0.1.0 release tag. This triggers the cargo-dist release workflow, which builds multi-platform binaries and creates a GitHub Release.
- **ADR**: [ADR-0002](../decisions/0002-cargo-dist-and-cargo-release.md)
- **Files to modify**: None (tag only)
- **Effort**: 30 minutes (plus CI build time: ~20-30 minutes for multi-platform builds)
- **Dependencies**: Tasks 2.2, 2.3 complete and CI passing, CHANGELOG.md exists (Task 1.6)
- **Acceptance Criteria**:
  - GitHub Release at `https://github.com/YOUR_ORG/repository-manager/releases/tag/v0.1.0` exists
  - All platform binaries are attached: `repo-x86_64-unknown-linux-musl.tar.gz`, `repo-aarch64-apple-darwin.tar.gz`, `repo-x86_64-apple-darwin.tar.gz`, `repo-x86_64-pc-windows-msvc.zip`
  - Shell installer works: `curl --proto '=https' --tlsv1.2 -LsSf .../repo-installer.sh | sh`
  - `repo --version` returns `0.1.0`
- **Commands to run**:
  ```bash
  # From within your feature worktree, after all changes are committed and merged:
  # Option A: Use cargo-release (recommended):
  cargo release patch --execute

  # Option B: Manual tag (if cargo-release not ready):
  git tag -a v0.1.0 -m "Release v0.1.0 - Public Alpha"
  git push origin v0.1.0

  # Monitor the release workflow:
  # Watch at: https://github.com/YOUR_ORG/repository-manager/actions
  ```

---

#### Task 2.5: Update README with agentic positioning and install instructions

- **Task**: Final README pass after the release exists. Add the shell installer one-liner, update the tagline to "Agentic Workspace Manager", and add the GitHub Actions CI badge. This is the README that users see when they discover the project post-launch.
- **ADR**: [ADR-0004](../decisions/0004-agentic-workspace-manager-positioning.md)
- **Files to modify**:
  - `Y:\code\repository-manager-worktrees\main\README.md`
- **Effort**: 1-2 hours
- **Dependencies**: Task 2.4 complete (release URL must be known)
- **Acceptance Criteria**:
  - README opens with "Agentic Workspace Manager" positioning
  - Shell installer one-liner is the first installation option shown
  - CI badge shows green status
  - No dead links (repository URL is real, release URL exists)
- **Key sections to add/update**:
  ```markdown
  # Repository Manager

  **The Agentic Workspace Manager.** Git worktrees for parallel AI agents,
  unified config for 13+ tools, drift detection, and an MCP server — in a
  single Rust binary.

  [![CI](https://github.com/YOUR_ORG/repository-manager/actions/workflows/ci.yml/badge.svg)](...)

  ## Installation

  ### Recommended: Shell installer
  ```bash
  curl --proto '=https' --tlsv1.2 -LsSf \
    https://github.com/YOUR_ORG/repository-manager/releases/latest/download/repo-installer.sh | sh
  ```

  ### Manual download
  Download the binary for your platform from [GitHub Releases](https://github.com/YOUR_ORG/repository-manager/releases).
  ```

---

## Dependency Graph

```
Task 1.1 (fix URL) ──────────────────────────────┐
Task 1.2 (git2 vendored) ────────┐               │
Task 1.3 (release profile) ──────┤ (size delta)  │
Task 1.4 (comma delimiter) ──────┤               │
Task 1.5 (README syntax) ────────┘ (after 1.4)   │
Task 1.6 (CHANGELOG) ──────────────────────────┐ │
                                               │ │
Task 2.1 (CI workflow) ──────────────────────┐ │ │
Task 2.2 (cargo-dist) ───────────────────────┤ │ │  depends on: 1.1, 1.3
Task 2.3 (cargo-release) ────────────────────┤ │ │  depends on: 1.1, 2.2
Task 2.4 (first release) ────────────────────┘ ┘ ┘  depends on: ALL above
Task 2.5 (final README) ─────────────────────────── depends on: 2.4
```

**Critical path**: 1.1 → 1.2 → 1.3 → 2.2 → 2.3 → 2.4

Tasks 1.4, 1.5, 1.6, and 2.1 can be parallelized with the critical path.

---

## Milestone: Alpha Release Checklist

### Technical P0 Fixes
- [ ] Placeholder repository URL replaced in `Cargo.toml` (Task 1.1)
- [ ] `git2` vendored feature enabled — no more cmake/libssl-dev dependency (Task 1.2)
- [ ] `docs/design/key-decisions.md` updated to resolve gix vs git2 contradiction (Task 1.2)
- [ ] `[profile.release]` with LTO + strip added to `Cargo.toml` (Task 1.3)
- [ ] `--tools` comma-delimiter bug fixed in `crates/repo-cli/src/cli.rs` (Task 1.4)
- [ ] README Quick Start commands verified to work as written (Task 1.5)

### Documentation
- [ ] `CHANGELOG.md` created at workspace root with v0.1.0 section (Task 1.6)
- [ ] README updated with agentic positioning and correct install instructions (Tasks 1.5, 2.5)
- [ ] Prerequisites section in README (Rust, git only — no cmake required) (Task 1.5)

### Release Infrastructure
- [ ] `.github/workflows/ci.yml` created with cross-platform matrix (Task 2.1)
- [ ] `.github/dependabot.yml` created (Task 2.1)
- [ ] CI passing on ubuntu-latest, windows-latest, macos-latest (Task 2.1)
- [ ] `cargo dist init` completed, `.github/workflows/release.yml` generated (Task 2.2)
- [ ] `.config/release.toml` created for workspace-wide version management (Task 2.3)
- [ ] `cargo dist plan` runs without errors (Task 2.2)

### Release Verification
- [ ] First GitHub Release published at `v0.1.0` (Task 2.4)
- [ ] Multi-platform binaries attached to release (linux musl, macOS arm64+x86_64, Windows x86_64) (Task 2.4)
- [ ] Shell installer works on a clean machine: `curl ... | sh && repo --version` (Task 2.4)
- [ ] `cargo install repo-cli` works (P1 — requires crates.io publish, not in Phase 1 scope)
- [ ] README updated with real release URL and CI badge (Task 2.5)
- [ ] CI passing on main branch (Task 2.1)

---

## Risk Register

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| cargo-dist init fails due to workspace structure | Medium | High | Run `cargo dist init` carefully; review generated workflow before committing |
| LTO build exposes latent compilation bugs | Low | Medium | Run `cargo test --workspace` after adding release profile; fix before tagging |
| Vendored libgit2 adds >10MB to binary | Low | Low | Verify size after Task 1.3 (LTO will reduce it); ADR-0001 documents this tradeoff |
| CI times out on Windows with vendored build | Medium | Medium | Add `sccache` or `Swatinem/rust-cache@v2` to cache vendored build artifacts |
| Path dependencies block cargo-dist packaging | Low | High | cargo-dist handles workspace path deps natively; verify with `cargo dist plan` |

---

## Effort Summary

| Task | Description | Effort |
|------|-------------|--------|
| 1.1 | Fix placeholder URL | 30 min |
| 1.2 | git2 vendored feature + docs | 1-2 hr |
| 1.3 | Release profile optimizations | 30 min |
| 1.4 | Fix --tools comma delimiter | 30 min |
| 1.5 | Update README | 2-3 hr |
| 1.6 | Create CHANGELOG.md | 2 hr |
| 2.1 | CI workflow + Dependabot | 2-4 hr |
| 2.2 | cargo-dist bootstrap | 2-4 hr |
| 2.3 | cargo-release config | 1-2 hr |
| 2.4 | First release tag | 30 min |
| 2.5 | Final README + badge | 1-2 hr |
| **Total** | | **12-20 hr (1.5-2.5 days of focused work)** |

---

## Out of Scope for Phase 1

The following are important but deferred to Phase 2 (see `2026-02-18-implementation-plan-phase2.md`):

- crates.io publishing (requires path dependency resolution — ADR-0007, P1)
- Homebrew tap standalone setup (cargo-dist handles the tap; the tap repo may need separate setup)
- Winget manifest submission (ADR-0008, P1)
- CONTRIBUTING.md and SECURITY.md (P1 governance files)
- `cargo deny` / `deny.toml` (ADR-0009, P1 — CI job exists but gates until deny.toml created)
- Getting Started guide (P1-1)
- MSRV declaration (ADR-0015, P1)
- Docker image (P2)

---

## References

- [ADR-0001: git2 vendored feature](../decisions/0001-git2-vendored-feature.md)
- [ADR-0002: cargo-dist and cargo-release](../decisions/0002-cargo-dist-and-cargo-release.md)
- [ADR-0003: release profile optimizations](../decisions/0003-release-profile-optimizations.md)
- [ADR-0004: agentic workspace manager positioning](../decisions/0004-agentic-workspace-manager-positioning.md)
- [ADR-0005: comma-delimited tools flag](../decisions/0005-comma-delimited-tools-flag.md)
- [ADR-0006: Rust CI workflow](../decisions/0006-rust-ci-workflow.md)
- [Marketing Audit Consolidated](../audits/2026-02-18-marketing-audit-consolidated.md)
- [Feature Gap Analysis](../audits/2026-02-18-feature-gap-analysis.md)
