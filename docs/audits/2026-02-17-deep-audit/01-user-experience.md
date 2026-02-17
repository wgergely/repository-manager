# User Experience Audit - Repository Manager

**Date**: 2026-02-17
**Auditor**: JohnDoe (simulated first-time user, senior developer)
**Scope**: README, documentation, CLI usability, user journey simulation

---

## Executive Summary

Repository Manager has an ambitious and compelling vision: a unified control plane for agentic development workspaces. The concept of a Single Source of Truth that "unrolls" intent into tool-specific configs is genuinely valuable. However, the current user experience has significant gaps between the documented vision and the implemented reality. A first-time user will understand *what* the tool wants to be, but will struggle to actually *use* it effectively due to sparse onboarding, unclear installation steps, and mismatches between design docs and CLI behavior.

**Overall UX Rating: 2.5 / 5**

---

## 1. First Impressions - README.md

**Rating: 2 / 5**

### What Works
- The one-line description ("A Rust-based CLI tool for orchestrating agentic development workspaces") is clear enough.
- The three layout modes are documented with visual directory trees, which is helpful.
- Development commands (cargo test, cargo check, cargo build) are present.

### What's Missing or Problematic

1. **No installation instructions.** This is the single biggest blocker. A new user has no idea how to obtain the `repo` binary. There is no `cargo install`, no download link, no release page reference, no brew formula. A first-time user is immediately stuck.

2. **No usage examples.** The README mentions "Crates" (repo-fs, repo-git) but never shows the actual CLI. I have to dig into `docs/design/spec-cli.md` to learn the tool is invoked as `repo`. A quick "Getting Started" section with 3-4 commands would transform the experience.

3. **No mention of the core value proposition.** The README says "orchestrating agentic development workspaces" but never explains what that means in practical terms. The project-overview.md does this brilliantly -- that content should be front and center.

4. **Missing crates in the listing.** The README only lists `repo-fs` and `repo-git`, but the project actually has `repo-cli`, `repo-core`, `repo-tools`, `repo-meta`, `repo-presets`, and `repo-blocks`. This is stale.

5. **No badge or status indicator.** No CI status, no version badge, no "alpha/beta/stable" label. A user cannot gauge maturity.

### Recommendations
- Add "Installation" section (even if just `cargo install --path crates/repo-cli`)
- Add "Quick Start" section with `repo init --interactive` example
- Port the first 2 paragraphs from project-overview.md into the README
- Update crate listing to match actual workspace
- Add a status badge (e.g., "Alpha - API may change")

---

## 2. Documentation Quality

### 2.1 Project Overview (`docs/project-overview.md`)

**Rating: 4 / 5**

This is the best document in the project. It clearly explains:
- The problem being solved (fragmented dev environments in 2026)
- The solution (unified control plane, Single Source of Truth)
- The three key capabilities (SSOT, Workspace Virtualization, Preset System)
- The crate architecture

Minor issues:
- Uses emojis (inconsistent with other docs)
- Links to `design/_index.md` and `research/_index.md` which a user may not find from the README
- No mention of current implementation status -- a user does not know what percentage of the vision is working

### 2.2 CLI Specification (`docs/design/spec-cli.md`)

**Rating: 3 / 5**

Good reference for the intended CLI surface. Well-structured with examples. However:

- **Spec vs. Implementation mismatch**: The spec describes `repo add-tool` and `repo remove-tool` as flat commands (which matches the implementation), but the actual config.toml schema described in `config-schema.md` uses `[active] tools = [...]` while the implementation uses a top-level `tools = [...]` array. This is confusing.
- The spec mentions `repo push`, `repo pull`, `repo merge` as "convenience wrappers" but does not explain *why* a user would use these instead of raw git commands. The value proposition of each wrapper is unclear.
- No mention of `repo status`, `repo diff`, `repo list-tools`, `repo list-presets`, `repo list-rules`, or `repo completions` -- all of which exist in the implementation. The spec is out of date.

### 2.3 Tools Specification (`docs/design/spec-tools.md`)

**Rating: 3 / 5**

Clear explanation of the tool integration architecture. The `ToolIntegration` trait and per-tool strategies (VSCode JSON, Cursor hybrid, Claude JSON) are well-documented. However:
- No list of *actually implemented* tools vs. planned tools
- A user reading this expects to be able to `repo add-tool vscode` and have it "just work," but the boundary between "designed" and "built" is invisible

### 2.4 Presets Specification (`docs/design/spec-presets.md`)

**Rating: 3 / 5**

The `PresetProvider` trait and typology (`env:*`, `config:*`, `tool:*`) are well-designed. The dependency graph resolution (e.g., `tool:ruff` depends on `env:python`) is smart. But:
- No list of which presets actually exist in the current build
- The interactive init mode (`interactive.rs`) only offers a hardcoded list: `vscode, cursor, claude, windsurf, gemini, antigravity`. Presets are not offered in interactive mode at all (`presets: Vec::new()` with a comment "Could add preset selection later").

### 2.5 Configuration Schema (`docs/design/config-schema.md`)

**Rating: 2.5 / 5**

Thorough design document, but has a significant problem: the schema described here does not match the actual implementation.

- **Doc says**: `[active] tools = [...]` and `presets = [...]` under an `[active]` section
- **Implementation says**: `tools = [...]` at the top level, presets as `[presets."name"] = {}`
- **Doc says**: `[core] version = "1.0"` exists
- **Implementation**: no version field
- **Doc says**: `[project] name = "..."` section exists
- **Implementation**: no project section
- **Doc says**: `[sync] strategy = "smart-append"` exists
- **Implementation**: no sync section

This divergence will confuse anyone trying to hand-edit config.toml based on the docs.

### 2.6 Git Specification (`docs/design/spec-git.md`)

**Rating: 3.5 / 5**

Clean design. The `GitProvider` trait abstracting Standard vs. Worktree operations is sound. The worktree naming strategy (slash-to-hyphen conversion) is practical. The note about `git push -u origin {branch}` auto-configuration shows attention to real workflow needs.

### 2.7 FS Specification (`docs/design/spec-fs.md`)

**Rating: 3.5 / 5**

Good coverage of path normalization, atomic I/O, and Windows symlink challenges. The Container Root / Context Root distinction is clearly explained. The Windows considerations section is a nice touch for cross-platform users.

---

## 3. CLI Usability Assessment

### 3.1 Command Structure

**Rating: 3.5 / 5**

The implemented command structure is mostly intuitive:

| Command | Purpose | Intuitive? |
|---------|---------|------------|
| `repo init` | Initialize | Yes |
| `repo status` | Show status | Yes |
| `repo diff` | Preview changes | Yes |
| `repo check` | Check for drift | Yes |
| `repo sync` | Synchronize configs | Yes |
| `repo fix` | Auto-repair | Yes |
| `repo add-tool <name>` | Add a tool | Yes |
| `repo remove-tool <name>` | Remove a tool | Yes |
| `repo add-preset <name>` | Add a preset | Yes |
| `repo remove-preset <name>` | Remove a preset | Yes |
| `repo add-rule` | Add a rule | Mostly (see below) |
| `repo remove-rule` | Remove a rule | Yes |
| `repo list-tools` | List available tools | Yes |
| `repo list-presets` | List available presets | Yes |
| `repo list-rules` | List active rules | Yes |
| `repo branch add/remove/list/checkout` | Branch management | Yes |
| `repo push/pull/merge` | Git wrappers | Acceptable |
| `repo completions` | Shell completions | Yes |
| `repo superpowers install/status/uninstall` | Plugin management | Confusing name |

**Good decisions:**
- `repo init --interactive` for guided setup is excellent
- `repo sync --dry-run` and `repo fix --dry-run` for safe previewing
- JSON output flags (`--json`) for scripting/CI integration
- Shell completion generation

**Problematic areas:**

1. **`add-tool` vs `tool add` pattern**: The spec uses `repo add-tool` (flat) while branch uses `repo branch add` (nested). This inconsistency is jarring. A user might try `repo tool add claude` or `repo branch add-tool cursor`. Pick one pattern.

2. **`repo add-rule` requires `--instruction` flag**: The syntax `repo add-rule python-style --instruction "Use snake_case"` is verbose. Consider allowing positional: `repo add-rule python-style "Use snake_case"`.

3. **`repo superpowers`**: This name is opaque. A first-time user has no idea what "superpowers" means. It appears to be a Claude Code plugin installer but the name gives no hint. Consider `repo plugin` or `repo extensions`.

4. **No `repo info` or `repo show` command**: There is no way to inspect the details of a specific tool, preset, or rule. `repo list-tools` shows names but not what a specific tool *does* or what configs it manages.

### 3.2 Missing Commands

A user would reasonably expect these:

| Expected Command | Status | Notes |
|-----------------|--------|-------|
| `repo init` | Implemented | Works |
| `repo add-tool / remove-tool` | Implemented | Works |
| `repo add-preset / remove-preset` | Implemented | Works |
| `repo branch add/remove/list` | Implemented | Works |
| `repo sync / check / fix` | Implemented | Works |
| `repo status` | Implemented | Works |
| `repo upgrade` / `repo self-update` | Missing | No way to update the tool itself |
| `repo config show` | Missing | No way to dump current config.toml contents |
| `repo config edit` | Missing | No way to open config in editor |
| `repo tool info <name>` | Missing | Cannot inspect a specific tool's details |
| `repo preset info <name>` | Missing | Cannot inspect a specific preset's details |
| `repo branch rename <old> <new>` | Missing | No branch rename support |
| `repo branch status <name>` | Missing | No per-branch status |
| `repo reset` | Missing | No way to reset to clean state |
| `repo export` / `repo import` | Missing | No config portability |
| `repo doctor` | Missing | Comprehensive health check (beyond `check`) |

### 3.3 Help Text Quality

**Rating: 3 / 5**

The clap derive annotations include doc comments that become help text. Examples:
- `repo init` has good examples in the `///` comments
- `repo add-tool` references `repo list-tools` to discover options
- `repo completions` shows shell-specific installation paths

Missing:
- No `repo --help` overview that explains the *workflow* (init -> add-tool -> sync)
- No `repo help` subcommand (only `--help` flag)
- No man pages or long-form help

---

## 4. User Journey Simulations

### Journey 1: "I want to set up a new project with Python and Claude Code support"

**Path**: `repo init my-project --presets python --tools claude`

**Assessment: Partially Clear (Rating: 2.5 / 5)**

- A user would not know the preset is called "python" vs "env:python" vs "python-web". The spec uses `env:python` notation, but the interactive mode and CLI examples just use "python". Need to clarify naming.
- After init, what happens? The `init_repository()` function creates `.repository/config.toml` and a `.git` directory. But it does NOT run `sync`. So no `.claude/config.json` is generated. A new user would expect tool configs to be created immediately.
- The init command in worktrees mode creates a `main/` directory but does NOT set it up as an actual git worktree. It is just an empty folder. This is misleading.
- No post-init guidance: "Run `repo sync` to generate tool configurations" or "Run `repo status` to verify".

**Blockers**: After `repo init`, the user has a config.toml but no actual tool configurations. They must know to run `repo sync` separately. This is not documented anywhere in the CLI output.

### Journey 2: "I want to add a new feature branch as a worktree"

**Path**: `repo branch add feature-x`

**Assessment: Good (Rating: 3.5 / 5)**

- The command is intuitive and the default base branch is "main"
- In worktrees mode, the output shows the path to the new worktree and a helpful `cd` hint
- The branch checkout command in worktrees mode also shows the path with a `cd` hint
- `repo branch list` shows current branch markers and paths

**Gaps**:
- No indication of whether tool configs are automatically copied/linked to the new worktree
- In worktrees mode, the spec says bootstrapping (copying `.vscode`, `.env`) should happen. The implementation delegates to `ModeBackend::create_branch()` but it is unclear if this actually copies shared configs.
- No `--track` option for setting upstream

### Journey 3: "I want to add VSCode support to my existing project"

**Path**: `repo add-tool vscode`

**Assessment: Good (Rating: 3.5 / 5)**

- Clean command, validates against known tools with a warning for unknown ones
- Automatically triggers `sync` after adding, which generates config files
- Reports what files were created/updated

**Gaps**:
- No preview of what will be generated before committing
- No `--dry-run` flag on add-tool (sync has it, but the combined operation does not)
- If the user wants to see what tools are available first, they need to know about `repo list-tools`, which is not suggested in the error message for invalid tool names

### Journey 4: "I want to see what tools are currently configured"

**Path**: `repo status` or `repo list-tools`

**Assessment: Good (Rating: 3.5 / 5)**

- `repo status` shows mode, root, active tools, rules count, and sync status
- `repo list-tools` shows available tools grouped by category with config paths
- JSON output available for both

**Gaps**:
- `repo status` shows *active* tools but not *available* tools. Need both.
- No distinction between "configured tool" and "synced tool" -- a tool could be in config.toml but sync might not have run yet
- `repo list-tools` shows ALL available tools, not which ones are active in the current project. Would be helpful to mark active ones.

### Journey 5: "I want to remove a tool I no longer need"

**Path**: `repo remove-tool cursor`

**Assessment: Good (Rating: 3.5 / 5)**

- Clean command with confirmation output
- Triggers sync to clean up generated files
- Handles the "tool not found" case gracefully

**Gaps**:
- Does it remove the generated config files (e.g., `.cursorrules`)? The sync should handle this, but it is not explicitly confirmed in output.
- No `--keep-files` option if user wants to remove from management but keep existing configs

### Journey 6: "I want to rename a branch/worktree"

**Path**: ???

**Assessment: Not Possible (Rating: 0 / 5)**

- No `repo branch rename` command exists
- The user would need to: `repo branch add new-name`, manually move work, `repo branch remove old-name`
- This is a common operation that should be supported

---

## 5. Gaps and Pain Points Summary

### Critical (Blocks adoption)
1. **No installation instructions** - Users cannot install the tool
2. **`repo init` does not run sync** - First-time setup leaves the project in an incomplete state
3. **README is stale** - Missing crates, no usage examples, no value proposition

### Major (Significantly degrades experience)
4. **Spec/implementation config schema mismatch** - Docs describe a different config format than what the code produces
5. **No post-init guidance** - CLI does not tell users what to do next
6. **Interactive init skips presets** - The guided setup mode has `presets: Vec::new()` hardcoded
7. **Command pattern inconsistency** - `add-tool` (flat) vs `branch add` (nested)
8. **"Superpowers" naming** - Opaque, undiscoverable name for plugin management
9. **No `repo branch rename`** - Common operation unsupported

### Minor (Annoyances)
10. **No `repo config show`** - Cannot inspect current configuration without reading files
11. **`repo add-rule` verbose syntax** - Requires `--instruction` flag for what could be positional
12. **`list-tools` does not mark active tools** - Must cross-reference with `status`
13. **No version/status badges** in README
14. **Worktrees init creates empty `main/` folder** - Not an actual git worktree, misleading
15. **No `--dry-run` on add-tool/remove-tool** - Cannot preview before committing

---

## 6. Cross-References

- **Technical Audit** (`02-technical-audit.md`): The spec/implementation divergences noted in Section 2.5 should be verified at the code level. The init command's failure to run sync is an architectural question.
- **Test Verification** (`03-test-verification.md`): Tests cover the happy paths well but should verify the user journeys described in Section 4, especially the init-then-sync gap.
- **Competitive Analysis** (`04-competitive-landscape.md`): Compare the command structure and onboarding experience against competing tools (direnv, asdf, mise, devcontainers).
- **Documentation UX** (`06-documentation-ux.md`): The README rewrite and "Getting Started" guide are the highest-impact improvements for adoption.

---

## 7. Priority Recommendations

### Immediate (Before any public release)
1. Write installation instructions in README
2. Add "Quick Start" section to README with 4-5 commands
3. Make `repo init` either auto-run sync or print "Next: run `repo sync`"
4. Fix config.toml schema docs to match implementation

### Short-term (Next iteration)
5. Add preset selection to interactive init
6. Unify command pattern: either all flat (`add-tool`) or all nested (`tool add`)
7. Add `repo branch rename`
8. Rename "superpowers" to something discoverable
9. Mark active tools in `list-tools` output

### Medium-term (Before v1.0)
10. Add `repo config show` and `repo tool info <name>`
11. Add `--dry-run` to add-tool/remove-tool
12. Add `repo doctor` for comprehensive health checks
13. Generate man pages from clap
14. Add `repo upgrade` / self-update mechanism
