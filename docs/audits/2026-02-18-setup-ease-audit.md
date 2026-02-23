# Setup Ease and Onboarding Audit

**Date:** 2026-02-18
**Auditor:** MarketingAgent2
**Scope:** New user installation and onboarding experience for Repository Manager v0.1.0 (Alpha)

---

## Executive Summary

Repository Manager has a **moderate onboarding friction level** for its target audience of developers. The CLI is well-designed with helpful color output, guided next steps after `init`, and an interactive mode. However, the installation path requires Rust and Cargo, there is no pre-built binary distribution, and the gap between "installed" and "producing value" requires understanding concepts (modes, tools, presets, sync) that are not self-evident. A new user without Rust installed faces a 5-15 minute setup process before running their first command.

**Overall Onboarding Rating: 5/10** (for Rust developers: 7/10; for non-Rust developers: 3/10)

---

## 1. Installation Process

### 1.1 Current Installation Methods

From `README.md`:

```bash
# From source
cargo install --path crates/repo-cli

# Or build locally
git clone <repo-url>
cd repository-manager
cargo build --release
```

**Findings:**

- **No pre-built binaries.** Users must have a working Rust toolchain. There is no `brew install`, `apt install`, `winget install`, `scoop install`, `npm install -g`, or downloadable `.exe`/`.tar.gz` release.
- **No crates.io publish.** `cargo install repo-cli` does not work; users must clone the full repository. The `repository` field in `Cargo.toml` (workspace) contains a placeholder URL: `"https://github.com/user/repository-manager"`.
- **Build time is substantial.** The workspace has 11 crates plus a full integration test suite, including tokio, clap, serde, git2, and tracing. A cold build on a standard machine will take 2-5 minutes.

### 1.2 libgit2 Dependency

The `repo-git` crate depends on `git2 = "0.20"` (via workspace `Cargo.toml`). The `git2` crate links to `libgit2`. By default, the `git2` crate builds and vendors libgit2 from source using cmake. This means:

- **cmake must be installed** on the build system. On Windows this typically requires Visual Studio Build Tools. This is an undocumented prerequisite.
- **No `vendored` feature flag is explicitly set** in `Cargo.toml`, meaning it relies on git2's default behavior. If a system libgit2 is present, it may be used instead, which can cause version mismatches.
- The Docker base image (`docker/base/Dockerfile.base`) installs `build-essential` and `pkg-config` but does **not** explicitly install cmake or libgit2-dev. The Docker build relies on Rust's git2 vendored mode, which may add several minutes to the build.

### 1.3 Prerequisites Assessment

| Prerequisite | Required? | Documented? |
|---|---|---|
| Rust stable toolchain | Yes (mandatory) | Yes (implied by `cargo install`) |
| cargo | Yes (mandatory) | Yes |
| git | Yes (for worktrees functionality) | No |
| cmake | Yes (for git2/libgit2 vendoring on most platforms) | No |
| C compiler (gcc/clang/MSVC) | Yes (for git2 native compilation) | No |
| libssl-dev / OpenSSL | Yes on Linux (git2 TLS) | No |
| pkg-config | Yes on Linux | No |

The `README.md` documents zero prerequisites beyond implying Rust is needed.

### 1.4 Platform Support

- **Linux:** Should work after installing `build-essential`, `pkg-config`, `libssl-dev`. Not documented.
- **macOS:** Should work with Xcode Command Line Tools. Not documented.
- **Windows:** Requires either MSVC or MinGW. cmake must be on PATH. This is the highest-friction platform for git2. Not documented.
- The Docker images target Ubuntu 22.04 (`Dockerfile.base`), providing a known-good Linux environment for testing.
- No Windows-specific CI or documentation. The shell scripts in `docker/scripts/` use bash; Windows users would need WSL or Git Bash.

---

## 2. First-Run Experience

### 2.1 `repo` (no command)

Running `repo` with no arguments displays:

```
repo Repository Manager CLI

Run repo --help for available commands.
```

This is minimal but functional. It does not suggest `repo init` as the obvious first step.

### 2.2 `repo init`

**Source:** `crates/repo-cli/src/commands/init.rs`, `crates/repo-cli/src/cli.rs`

The `init` command is the best-designed part of the onboarding flow.

**Positives:**
- Default mode is `worktrees` (set in `cli.rs:52`), which is the recommended mode for the tool's primary use case.
- Creates the `.repository/` directory and `config.toml` automatically.
- Initializes git if `.git` does not exist.
- Post-init output provides explicit next steps:
  - If tools were specified: `"Next step: run repo sync to generate tool configurations"`
  - If no tools specified: `"Next step: run repo add-tool <name>, then repo sync"`
  - Plus: `"Run repo list-tools to see available tools"`
- Handles name sanitization transparently (spaces/underscores become hyphens).
- `--interactive` flag launches a full guided setup using `dialoguer` with:
  - Project name prompt
  - Mode selection (worktrees vs standard) via arrow keys
  - Multi-select tool picker from the live registry
  - Multi-select preset picker
  - Optional git remote prompt
  - Summary + confirmation before executing

**Issues:**
- `repo init` without `--interactive` requires knowing mode/tool names upfront. Discovery requires `repo list-tools` first.
- The command docs example in `cli.rs` shows `repo init my-project` but the actual default mode is `worktrees`, not `standard`. This mismatch between README example (`--mode standard`) and actual default could confuse users.
- The README Quick Start shows: `repo init my-project --mode standard --tools cursor,claude,vscode` — but `--tools` takes repeated flags (`-t cursor -t claude`), not a comma-separated list. Comma-separated syntax would fail silently by treating `"cursor,claude,vscode"` as a single tool name.

### 2.3 `repo sync`

**Source:** `crates/repo-cli/src/commands/sync.rs`

- Runs the sync engine to generate tool configuration files.
- Clear colored output: green `OK` for success, yellow for missing files, red for drift/errors.
- Supports `--dry-run` to preview changes — good for new users to understand what will happen.
- Supports `--json` for CI/CD integration — good for advanced users.
- Error message when not in a repo is clear: `"Not in a repository. Run 'repo init' to create one."`

**Issues:**
- `repo sync` silently succeeds with no tools configured (reports "Already synchronized"). A new user who forgets to add tools may not realize nothing was generated.
- No indication of which files were generated or where they are located during first sync.

### 2.4 `repo status`

**Source:** `crates/repo-cli/src/commands/status.rs`

Shows a clean, readable status board:

```
Repository Status

  Mode: worktrees
  Root: /path/to/project
  Tools: cursor, claude
  Rules: 2 active
  Sync: healthy
```

- Color-coded: cyan for mode, yellow for path, green for tools/sync, dimmed for empty values.
- Supports `--json` for scripting.
- Shows local overrides if present.

**Issues:**
- Does not show list of generated files — user cannot quickly verify what was created.
- Does not show which presets are active.

### 2.5 Help and Discoverability

- `repo --help` lists all commands with their one-line descriptions (via clap).
- `repo <command> --help` gives per-command help.
- The `init` command has inline examples in `cli.rs` (visible via `--help`):
  ```
  repo init                    # Initialize in current directory
  repo init my-project         # Create and initialize my-project/
  repo init --interactive      # Guided setup
  repo init -t claude -t cursor # With specific tools
  ```
- Shell completions are available via `repo completions <shell>` for bash, zsh, fish, and others. This is a significant quality-of-life feature that most Alpha CLIs lack.
- `repo list-tools` and `repo list-presets` provide self-service discovery.

**Missing:**
- No `repo tutorial` or `repo quickstart` command.
- No `repo doctor` to diagnose environment issues (missing git, wrong directory, etc.).

---

## 3. Configuration Complexity

### 3.1 What Users Need to Understand

**Concepts required before first productive use:**
1. **Modes:** `standard` vs `worktrees` — the difference is non-trivial and affects the directory structure significantly. The default (worktrees) creates a `main/` subdirectory, which can surprise users expecting a typical git repository layout.
2. **Tools:** The list of 13 supported tools and which apply to their workflow.
3. **Presets:** The concept of environment presets (env:python, etc.) — this is an advanced concept not needed for basic use.
4. **Sync cycle:** The edit-config → sync → files-appear workflow is the core mental model but is not explained in one place.

### 3.2 Configuration File Format

From `test-fixtures/repos/config-test/.repository/config.toml`:

```toml
# Repository Manager Configuration
tools = ["cursor", "claude"]

[core]
mode = "standard"
```

The TOML format is straightforward for the basic case. The generated config from `init` is minimal and readable.

**Issues:**
- The full design in `docs/design/config-strategy.md` shows a significantly more complex schema with `[presets."env:python"]`, `[presets."config:git"]`, etc. This advanced format is not documented for new users and would be encountered only when using presets.
- No `.repository/config.toml` template or example with comments explaining available options is shipped to the user on `init`. The generated file has no inline documentation.

### 3.3 Local Overrides

The config system supports a local overrides file (`.repository/config.local.toml`, git-ignored). This feature exists in the design but:
- Is not mentioned in the README.
- Is not created by `repo init`.
- Users must discover it through `repo status` output or source code.

---

## 4. Docker Experience

### 4.1 What Exists

The `docker/` directory contains a full integration testing infrastructure:
- Multiple Dockerfiles for each supported AI tool
- Build scripts, test scripts, a mock API server
- An Ubuntu 22.04 base with Rust, Node.js 20, and Python 3.12 pre-installed

### 4.2 Assessment

The Docker setup is **for developers of Repository Manager**, not for end users. There is:
- No `docker run` one-liner to try the tool without installing Rust.
- No Docker Hub image or GitHub Container Registry image.
- The `docker/repo-manager/Dockerfile` builds from source and would require the user to have the source code anyway.

A prospective user interested in "trying it in Docker" would need to build the image themselves from source, which is not faster than `cargo install`.

**Opportunity:** Publishing a pre-built Docker image (e.g., `ghcr.io/user/repo-manager:latest`) would allow zero-install experimentation.

---

## 5. Example Projects

### 5.1 Available Test Fixtures

`test-fixtures/repos/config-test/.repository/config.toml`:
```toml
tools = ["cursor", "claude"]

[core]
mode = "standard"
```

`test-fixtures/repos/simple-project/CLAUDE.md` — shows generated output for Claude tool.

These fixtures exist for test purposes but are not surfaced as user-facing examples or templates.

### 5.2 Missing Examples

No "example projects" are provided that a new user could clone to see a working setup. The README has code blocks showing the before/after concept but no runnable example repository.

---

## 6. Summary Findings

### Strengths

| Strength | Details |
|---|---|
| Interactive init mode | Full guided setup with multi-select tool/preset picker |
| Post-init guidance | Explicit next-step instructions printed after `repo init` |
| Color-coded output | Readable at a glance; consistent use of green/yellow/red |
| Shell completions | Available for bash, zsh, fish via `repo completions` |
| Dry-run support | `repo sync --dry-run` lets users preview safely |
| JSON output | `--json` flag on key commands for CI/CD use |
| Clear error messages | "Not in a repository. Run 'repo init' to create one." |
| Self-discovery | `repo list-tools`, `repo list-presets`, `repo --help` |

### Gaps and Issues

| Issue | Severity | Details |
|---|---|---|
| No pre-built binaries | High | Must have Rust toolchain; 2-5 min build |
| Undocumented prerequisites | High | cmake, libssl-dev, C compiler not mentioned |
| README example uses wrong `--tools` syntax | Medium | Comma-separated won't work; must use repeated flags |
| Default mode (worktrees) creates unexpected `main/` dir | Medium | Surprising behavior for users expecting a standard git repo |
| No `repo doctor` / environment diagnostics | Medium | Hard for new users to self-diagnose failures |
| No generated file listing after sync | Low | Users don't know what was created |
| No config.toml template with comments | Low | Generated config has no inline documentation |
| Docker not usable as a try-before-install path | Low | Docker infra is test-only, not user-facing |
| No published crates.io package | Low | `cargo install repo-cli` does not work |
| Preset system not accessible to new users | Low | No beginner-level docs on presets |

---

## 7. Recommendations

### Priority 1: Reduce Install Friction

1. **Publish pre-built binaries** via GitHub Releases for Linux x86_64, macOS (arm64 + x86_64), and Windows x86_64. Tools like `cargo-dist` or `goreleaser`-equivalent automate this.
2. **Document all prerequisites** in the README under an "Installation" section: Rust, git, cmake (if needed), OpenSSL/pkg-config on Linux.
3. **Publish to crates.io** with a proper package name so `cargo install repo-cli` works without cloning.

### Priority 2: Fix Quick Start Accuracy

4. **Fix the `--tools` syntax** in the README Quick Start. Change `--tools cursor,claude,vscode` to `-t cursor -t claude -t vscode` or add comma-split support in the CLI parser.
5. **Explain the modes** in the README with a one-paragraph plain-English description of when to use `worktrees` vs `standard`.
6. **Default to `standard` mode** for `repo init` unless the worktrees mode is explicitly chosen, or add a visible warning that worktrees mode creates a `main/` subdirectory.

### Priority 3: Improve Discoverability

7. **Add `repo doctor`** command that checks: git installed, inside a repo, config.toml parseable, sync state.
8. **Add generated file listing** to `repo sync` output: "Generated: CLAUDE.md, .cursorrules, .vscode/settings.json".
9. **Add inline comments** to the generated `config.toml` explaining the available fields.
10. **Promote `repo init --interactive`** more prominently in the README as the recommended first step.

### Priority 4: Distribution

11. **Publish a Docker image** to GitHub Container Registry for try-before-install.
12. **Add Homebrew formula** for macOS users.
13. **Add Scoop/Winget manifest** for Windows users.

---

## Appendix: Onboarding Flow Walkthrough

**Ideal happy path for a new user (current state):**

```
1. Install Rust (rustup.rs) — 5-10 min if not present
2. git clone <repo-url>
3. cargo install --path crates/repo-cli — 2-5 min build
4. cd my-project
5. repo init --interactive
   - Answer prompts: mode, tools, presets
6. repo sync
7. Verify generated files
```

**Total time to first value: 10-20 minutes** (vs. ~30 seconds for a tool with pre-built binaries).

**Ideal happy path for a new user (with recommended improvements):**

```
1. curl -L <release-url> | sh (or brew install / scoop install)
2. cd my-project
3. repo init --interactive
4. repo sync
```

**Total time to first value: 1-2 minutes.**
