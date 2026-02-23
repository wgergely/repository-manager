# Repository Manager - Implementation Plan

**Date:** 2026-02-18
**Informed by:** Marketing Audit (9 reports), 23 Architecture Decision Records

---

## Executive Summary

Repository Manager is a technically ambitious Rust CLI that solves a real, timely problem: unifying AI coding agent configuration across 13+ tools from a single source of truth. The core product concept is strong — the sync engine is functional, drift detection is real, worktree management works, and the MCP server is fully operational. However, as of the February 2026 assessment, the project is not ready for public release. All three independent audits converged on the same conclusion: internal engineering quality is ahead of external packaging, documentation, and distribution infrastructure.

The goal of this implementation plan is to transform Repository Manager from an internal alpha with zero distribution infrastructure into a publicly credible v1.0 release with professional presence across multiple installation channels. The plan is organized into three phases spanning approximately 8-12 weeks. Phase 1 resolves the 7 P0 blockers required for a credible public alpha. Phase 2 builds the adoption readiness layer — crates.io publishing, multi-channel distribution, documentation, and the two most impactful missing features (snap mode and tool expansion). Phase 3 delivers post-GA polish based on 6 accepted P2 ADRs that add competitive differentiation without blocking initial launch.

The overall marketing readiness score of 2.5/10 at time of assessment is bridgeable. The primary gap is not product quality but distribution and documentation — work that is bounded, well-scoped, and achievable within the timeline below. With focused execution, the project can reach a state suitable for a public announcement, Hacker News submission, and community building by approximately week 8.

---

## Current State Assessment

- **Marketing readiness:** 2.5/10
- **Documentation quality:** 2.4/5 (developer-facing, not user-facing)
- **Setup ease:** 5/10 overall (7/10 for Rust developers, 3/10 for non-Rust developers)
- **Packaging and distribution:** 1/5 (no pre-built binaries, no crates.io publish, no release pipeline)
- **Product-market positioning:** 3.5/5 (strong concept, inconsistent execution in docs)
- **P0 blockers identified:** 7 (must resolve before public alpha)
- **Architecture Decision Records accepted:** 23 (covering all phases)

The project has genuine differentiators: it is the only tool combining rules sync + worktree management + preset system + MCP server in a single Rust binary. The core sync engine, 13 tool integrations, drift detection, and MCP server are all substantially implemented and functional. The implementation is more complete than initial audits suggested — the gap is distribution, not engineering depth.

---

## Phase 1: Ship Alpha (Weeks 1-2)

**Goal:** Resolve all P0 blockers for a credible public alpha release.

See [Phase 1 Detail](2026-02-18-implementation-plan-phase1.md) for task-by-task breakdown, code changes, and acceptance criteria.

### Key Deliverables

- Replace placeholder repository URL (`github.com/user/repository-manager`) in workspace `Cargo.toml`
- Enable `git2` vendored feature to eliminate cmake/libssl-dev build prerequisites
- Add `[profile.release]` with LTO + strip (30-50% binary size reduction)
- Fix `--tools` comma-delimiter bug in `repo-cli/src/cli.rs` (silent failure on documented syntax)
- Update README Quick Start with correct syntax and prerequisites
- Create `CHANGELOG.md` using Keep a Changelog format
- Create `.github/workflows/ci.yml` with cross-platform matrix (Linux, macOS, Windows)
- Bootstrap cargo-dist release pipeline (generates `.github/workflows/release.yml`)
- Configure cargo-release for coordinated workspace version bumping
- Cut and publish first release tag: `v0.1.0`

### Effort Estimate

12-20 hours of focused work (1.5-2.5 days). Critical path: URL fix → vendored git2 → release profile → cargo-dist → cargo-release → v0.1.0 tag. Tasks 1.4, 1.5, 1.6, and 2.1 can be parallelized with the critical path.

### Milestone Criteria: Alpha Release

- [ ] Placeholder URL replaced; `cargo metadata` shows real GitHub URL for all crates
- [ ] `cargo build --release` succeeds on a machine without cmake, libssl-dev, or pkg-config
- [ ] Binary size reduced by at least 20% after LTO
- [ ] `repo init test --tools cursor,claude,vscode` creates config with all three tools
- [ ] README Quick Start commands run successfully when copy-pasted by a new user
- [ ] `CHANGELOG.md` exists with v0.1.0 section
- [ ] CI passes on ubuntu-latest, windows-latest, macos-latest
- [ ] `cargo dist plan` runs without errors
- [ ] GitHub Release at `v0.1.0` with multi-platform binaries attached
- [ ] Shell installer works on a clean machine: `curl ... | sh && repo --version`

---

## Phase 2: Adoption Readiness (Weeks 3-7)

**Goal:** Make Repository Manager ready for broad adoption with professional open-source presence, multi-channel distribution, and key competitive feature parity.

See [Phase 2 Detail](2026-02-18-implementation-plan-phase2.md) for task-by-task breakdown, code changes, and acceptance criteria.

### Sub-Phase 2A: Publishing and Governance (Weeks 3-4)

**ADRs:** 0007, 0008, 0009, 0015, 0016

- Declare MSRV at Rust 1.85 in workspace `Cargo.toml`; add MSRV CI job
- Complete cargo metadata (homepage, documentation, keywords, categories) for all 11 crates
- Set up `cargo-deny` with `deny.toml` for supply chain security and license compliance
- Publish all 11 crates to crates.io in leaf-first dependency order
- Set up Winget and Scoop distribution channels for Windows; confirm Homebrew tap from cargo-dist

### Sub-Phase 2B: Documentation and UX (Weeks 4-5)

**ADRs:** 0010, 0011, 0014, 0017

- Fix non-interactive `repo init` default to standard mode (not worktrees)
- Write `docs/guides/mcp-server.md` with client-specific configuration examples
- Fix preset documentation (detection-only, not "automatically installs binaries")
- Document vaultspec as an optional subsystem; fix hardcoded version in `plugins.rs`
- Create `CONTRIBUTING.md` (with tool addition guide) and `SECURITY.md`

### Sub-Phase 2C: Feature Development (Weeks 5-7)

**ADRs:** 0012, 0013

- Expand tool support from 13 to 20+ integrations (OpenCode, Kilo Code, Continue.dev, Amp, Codex CLI, Kiro)
- Implement snap mode: `repo branch add <name> --snap <agent>` for one-command agent lifecycle

### Milestone Criteria: Beta Release

- [ ] `cargo install repo-cli` works from crates.io
- [ ] MSRV declared; CI validates with Rust 1.85 job
- [ ] `cargo deny check` passes with zero errors in CI
- [ ] Homebrew tap confirmed working on macOS
- [ ] Winget manifest submitted; Windows users can `winget install`
- [ ] `repo init` (non-interactive, no flags) creates standard mode repository
- [ ] `docs/guides/mcp-server.md` exists with Claude Desktop, Cursor, VS Code examples
- [ ] Preset docs no longer claim "installs binaries"
- [ ] `docs/guides/agent-spawning.md` exists; vaultspec documented as optional
- [ ] `repo plugins status` shows dynamic vaultspec version (not hardcoded v4.1.1)
- [ ] `CONTRIBUTING.md` and `SECURITY.md` exist at workspace root
- [ ] Tool count reaches 20+; README reflects accurate count
- [ ] `repo branch add --snap claude` executes full snap workflow
- [ ] Snap mode degrades gracefully when vaultspec is absent

---

## Phase 3: Post-GA Polish (Weeks 8-12+)

**Goal:** Deliver the six P2 ADRs that add competitive differentiation, improve onboarding, and close remaining gaps after the initial GA launch.

### Overview

Phase 3 covers all six accepted P2 ADRs (0018-0023). These features are not required for GA launch but significantly strengthen the product's competitive position, user experience, and developer confidence in the project. They can be executed in parallel across two tracks: infrastructure/distribution (ADR-0018, ADR-0019, ADR-0020) and feature development (ADR-0021, ADR-0022, ADR-0023).

### Task Breakdown

---

#### Task 3.1: Publish Docker Image to ghcr.io

- **ADR:** [ADR-0018](../decisions/0018-publish-user-facing-docker-image-to-ghcr.md)
- **Effort:** 1 day
- **Dependencies:** Phase 1 complete (real repository URL, release workflow in place)
- **Description:** Fix the critical PATH bug in `docker/repo-manager/Dockerfile` where the binary is not found at runtime (currently masked by `|| echo` fallback). Implement a multi-stage Dockerfile with a Rust builder stage and `debian:bookworm-slim` runtime stage. Add a ghcr.io publish step to the release workflow triggered on release tags. Document `docker run` usage in README.
- **Implementation Steps:**
  1. Fix `docker/repo-manager/Dockerfile` PATH bug — remove `|| echo` fallback and set correct `ENV PATH`
  2. Convert to multi-stage build:
     ```dockerfile
     FROM rust:1.85 AS builder
     WORKDIR /build
     COPY . .
     RUN cargo build --release

     FROM debian:bookworm-slim
     RUN apt-get update && apt-get install -y ca-certificates git && rm -rf /var/lib/apt/lists/*
     COPY --from=builder /build/target/release/repo /usr/local/bin/repo
     ENTRYPOINT ["repo"]
     ```
  3. Add ghcr.io publish step to `.github/workflows/release.yml`:
     ```yaml
     - name: Build and push Docker image
       uses: docker/build-push-action@v5
       with:
         push: true
         tags: ghcr.io/${{ github.repository_owner }}/repo-manager:${{ github.ref_name }}
     ```
  4. Add to README Installation section:
     ```markdown
     ### Docker (zero-install)
     ```bash
     docker run --rm -v "$(pwd):/workspace" -w /workspace \
       ghcr.io/YOUR_ORG/repo-manager:latest repo init --tools cursor,claude
     ```
- **Acceptance Criteria:**
  - `docker run ghcr.io/YOUR_ORG/repo-manager:latest repo --version` returns the version
  - PATH bug is eliminated; no `|| echo` fallback exists in any Dockerfile
  - Release workflow publishes a new image tag on every `v*.*.*` release
  - Image is documented in README under Installation
  - Base image is `debian:bookworm-slim` (not a Rust builder image)

---

#### Task 3.2: Auto-generate JSON Schema from Rust Config Types

- **ADR:** [ADR-0019](../decisions/0019-auto-generate-json-schema-from-rust-config-types.md)
- **Effort:** 1-2 days
- **Dependencies:** Phase 1 complete; `repo-meta` crate accessible
- **Description:** Add the `schemars` crate to `repo-meta` and derive `JsonSchema` on all config structs alongside existing `serde` derives. Add a `repo schema` subcommand that outputs the generated JSON Schema to stdout. Include the schema file in release artifacts produced by cargo-dist. This enables editor autocompletion for `.repository/config.toml` and provides a machine-readable description of the config format for MCP clients.
- **Implementation Steps:**
  1. Add dependency to `crates/repo-meta/Cargo.toml`:
     ```toml
     schemars = "0.8"
     ```
  2. Add `JsonSchema` derive to config structs in `crates/repo-meta/src/`:
     ```rust
     #[derive(Debug, Serialize, Deserialize, JsonSchema)]
     pub struct RepoConfig {
         // existing fields
     }
     ```
  3. Add `repo schema` subcommand to CLI:
     ```rust
     pub fn cmd_schema() -> Result<()> {
         let schema = schemars::schema_for!(RepoConfig);
         println!("{}", serde_json::to_string_pretty(&schema)?);
         Ok(())
     }
     ```
  4. Include schema in cargo-dist release artifacts by generating it in the release workflow:
     ```yaml
     - name: Generate JSON Schema
       run: cargo run --bin repo -- schema > repo-config-schema.json
     - name: Upload schema artifact
       uses: actions/upload-artifact@v4
       with:
         name: repo-config-schema.json
         path: repo-config-schema.json
     ```
  5. Document VS Code / Even Better TOML integration in `docs/guides/`:
     ```json
     // .vscode/settings.json
     {
       "evenBetterToml.schema.associations": {
         "**/.repository/config.toml": "https://github.com/YOUR_ORG/repository-manager/releases/latest/download/repo-config-schema.json"
       }
     }
     ```
- **Acceptance Criteria:**
  - `repo schema` outputs valid JSON Schema to stdout
  - Schema accurately reflects all fields in `RepoConfig` and nested types
  - Schema is included as a release artifact in each GitHub Release
  - `schemars` dependency added to `repo-meta` (not to `repo-cli` or other crates)
  - Editor autocompletion works for `.repository/config.toml` in VS Code with Even Better TOML

---

#### Task 3.3: Update Documentation for Drift Detection (Defer Read-Only Enforcement)

- **ADR:** [ADR-0020](../decisions/0020-defer-read-only-enforcement-update-documentation.md)
- **Effort:** 2-4 hours
- **Dependencies:** None (documentation-only change)
- **Description:** Update `docs/project-overview.md` to remove the claim that the tool "validates that agents have not hallucinated changes to read-only configuration." Replace with accurate language describing drift detection + repair. Create a GitHub issue tracking enterprise read-only enforcement for future prioritization. This builds user trust by ensuring documentation matches implementation.
- **Implementation Steps:**
  1. In `docs/project-overview.md`, locate and update:
     ```markdown
     # Before (inaccurate)
     "validate that agents have not hallucinated changes to read-only configuration"

     # After (accurate per ADR-0020)
     "detect configuration drift and automatically repair managed files"
     ```
  2. Search for all related overpromising claims across docs:
     ```bash
     grep -r "read-only\|readonly\|permission" docs/ --include="*.md"
     ```
  3. Create GitHub issue titled "Feature: Read-only config enforcement (enterprise)" with context from ADR-0020
  4. Add a "Drift Detection" section to `docs/project-overview.md` clearly explaining:
     - What drift detection does: checksum comparison for FileManaged, TextBlock, JsonKey projections
     - What `repo check` reports
     - What `repo fix` repairs
     - What is NOT currently implemented: a distinct permission model preventing writes to managed files
- **Acceptance Criteria:**
  - `docs/project-overview.md` no longer overpromises read-only enforcement
  - Drift detection + repair is accurately described as a first-class capability
  - GitHub issue exists for enterprise read-only enforcement tracking
  - All audit-identified overpromising language removed across all documentation files
  - No documentation references "permission model," "read-only validation," or similar concepts that do not exist in the implementation

---

#### Task 3.4: Implement Git-Based Remote Rule Includes

- **ADR:** [ADR-0021](../decisions/0021-git-based-remote-rule-includes.md)
- **Effort:** 3-5 days
- **Dependencies:** ADR-0001 (git2 vendored feature from Phase 1 provides the git implementation); Phase 2 complete
- **Description:** Add support for `[rules.remote]` config sections in `.repository/config.toml`. Rules can be pulled from remote git repositories, enabling team and org-level rule sharing without manual copy-paste. Rules are cached in `.repository/.cache/rules/` for offline use. A new `repo rules update` command refreshes cached remote rules.
- **Implementation Steps:**
  1. Add `[rules.remote]` section to config schema in `crates/repo-meta/src/`:
     ```toml
     [rules.remote."org-standards"]
     source = "https://github.com/org/rules.git"
     path = "rust/"
     ref = "main"  # optional, defaults to HEAD
     ```
  2. Implement remote rule fetching in `crates/repo-git/src/remote_rules.rs`:
     ```rust
     pub struct RemoteRuleSource {
         pub name: String,
         pub source: String,
         pub path: Option<String>,
         pub git_ref: Option<String>,
     }

     pub fn fetch_remote_rules(source: &RemoteRuleSource, cache_dir: &Path) -> Result<Vec<Rule>> {
         // Clone or fetch the remote repo into .repository/.cache/rules/<name>/
         // Read rules from the specified path
         // Return resolved Rule objects
     }
     ```
  3. Add `repo rules update` CLI subcommand
  4. Integrate remote rule resolution into the sync engine so remote rules are included in sync output
  5. Add offline fallback: use cached rules if fetch fails; warn with actionable message
  6. Update config schema documentation to describe `[rules.remote]` syntax
- **Acceptance Criteria:**
  - `[rules.remote]` section in `config.toml` is parsed without error
  - `repo rules update` fetches remote rules and updates the local cache
  - `repo sync` includes remote rules in generated tool configs
  - Rules are cached in `.repository/.cache/rules/`
  - If network is unavailable, cached rules are used with a warning (not a hard failure)
  - HTTP URLs are rejected with a clear error explaining that only git:// and https:// git remotes are supported
  - Authentication uses git's existing credential mechanisms (no new credential management)

---

#### Task 3.5: Implement Tool Config Import from Top 3 Formats

- **ADR:** [ADR-0022](../decisions/0022-import-top-3-tool-config-formats.md)
- **Effort:** 2-3 days per format (6-9 days total, parallelizable)
- **Dependencies:** Phase 2 complete; existing markdown parsing infrastructure in `repo-content`
- **Description:** Add `repo rules-import --from <format>` subcommands for the top 3 tool-specific config formats: `.cursorrules` (Cursor), `CLAUDE.md` (Claude), and `.windsurfrules` (Windsurf). These are all markdown-like formats making parsing straightforward. This directly addresses the "I already have rules, why switch?" migration barrier. Add a `--dry-run` flag to preview import without writing files.
- **Implementation Steps:**
  1. Extend the existing `repo rules-import` command with `--from` flag:
     ```bash
     repo rules-import --from cursor        # reads .cursorrules
     repo rules-import --from claude        # reads CLAUDE.md
     repo rules-import --from windsurf      # reads .windsurfrules
     repo rules-import --from cursor --dry-run  # preview without writing
     ```
  2. Implement format-specific parsers in `crates/repo-core/src/import/`:
     ```rust
     pub fn import_from_cursorrules(path: &Path) -> Result<Vec<ImportedRule>> { ... }
     pub fn import_from_claude_md(path: &Path) -> Result<Vec<ImportedRule>> { ... }
     pub fn import_from_windsurfrules(path: &Path) -> Result<Vec<ImportedRule>> { ... }
     ```
  3. Each parser reads the file, splits content into logical sections (by heading or delimiter), and produces `ImportedRule { id, title, content }` values
  4. Write imported rules to `.repository/rules/<tool>-<id>.md` with appropriate tags
  5. Report results: "Imported 12 rules from .cursorrules into .repository/rules/"
  6. Add `--dry-run` flag that prints what would be imported without writing files
- **Acceptance Criteria:**
  - `repo rules-import --from cursor` successfully imports rules from `.cursorrules`
  - `repo rules-import --from claude` successfully imports rules from `CLAUDE.md`
  - `repo rules-import --from windsurf` successfully imports rules from `.windsurfrules`
  - `--dry-run` flag prints import preview without writing files
  - Imported rules are placed in `.repository/rules/` with correct tags
  - If the source file does not exist, a clear error is shown with the expected path
  - Parsing handles the common markdown structures in each format without panicking on edge cases
  - After import, `repo sync` successfully generates tool configs using the imported rules

---

#### Task 3.6: Implement Comprehensive Repo Doctor Command

- **ADR:** [ADR-0023](../decisions/0023-comprehensive-repo-doctor-command.md)
- **Effort:** 2-3 days
- **Dependencies:** Phase 2 complete (vaultspec documented as optional per ADR-0017)
- **Description:** Add a `repo doctor` subcommand that checks the full environment state and reports pass/fail per check with an actionable fix suggestion for each failure. This reduces support burden and improves the onboarding experience by surfacing environment problems with clear guidance. The setup ease audit rated onboarding at 5/10 partly due to unclear prerequisite errors.
- **Implementation Steps:**
  1. Add `repo doctor` CLI subcommand in `crates/repo-cli/src/commands/doctor.rs`
  2. Implement check modules, each returning `CheckResult { status: Pass | Warn | Fail, message: String, fix_suggestion: Option<String> }`:
     ```rust
     pub struct GitCheck;        // git version compatibility (>= 2.28)
     pub struct RepoCheck;       // repository detection (are we in a managed repo?)
     pub struct ConfigCheck;     // config.toml parse validity
     pub struct SyncCheck;       // sync status and managed file integrity
     pub struct ToolCheck;       // binary detection for all configured tools
     pub struct WorktreeCheck;   // worktree health (if in worktree mode)
     pub struct VaultspecCheck;  // vaultspec availability (warn only if absent)
     ```
  3. Format output with clear pass/warn/fail indicators and a summary line:
     ```
     repo doctor

     [PASS] Git version: 2.43.0 (>= 2.28 required)
     [PASS] Repository: .repository/config.toml found and valid
     [PASS] Sync state: 3 managed files, 0 drift detected
     [PASS] Tools: cursor found at /usr/local/bin/cursor
     [WARN] Vaultspec: not installed (required only for agent spawn)
              Fix: pip install vaultspec
     [FAIL] Git worktree: orphaned worktree detected at ../feature-branch
              Fix: run 'repo branch remove feature-branch' to clean up

     Result: 4 passed, 1 warning, 1 failure
     ```
  4. Use colors for terminal output (reuse existing color infrastructure from `repo status`)
  5. Add `--json` flag for machine-readable output
  6. Wire `repo doctor` into error handling: when a command fails with an environment error, suggest running `repo doctor`
- **Acceptance Criteria:**
  - `repo doctor` runs without errors from any directory
  - Each check reports Pass, Warn, or Fail with an actionable fix suggestion for failures
  - Git version check warns if git < 2.28 (minimum for worktree support)
  - Config parse check reports the specific parse error if `config.toml` is invalid
  - Vaultspec check is Warn (not Fail) if vaultspec is absent
  - `repo doctor --json` outputs machine-readable results
  - Total effort matches the 2-3 day estimate from ADR-0023

---

### Milestone: v1.0 Readiness Checklist

#### Infrastructure
- [ ] Docker image published to ghcr.io; `docker run` usage documented in README (Task 3.1)
- [ ] PATH bug eliminated from all Dockerfiles (Task 3.1)
- [ ] `repo schema` outputs valid JSON Schema for `config.toml` (Task 3.2)
- [ ] JSON Schema included as release artifact in GitHub Releases (Task 3.2)
- [ ] Editor autocompletion documented for `.repository/config.toml` (Task 3.2)

#### Documentation Accuracy
- [ ] `docs/project-overview.md` accurately describes drift detection + repair (Task 3.3)
- [ ] No documentation overpromises read-only enforcement (Task 3.3)
- [ ] GitHub issue created for enterprise read-only enforcement tracking (Task 3.3)

#### Features
- [ ] `[rules.remote]` config section supported; `repo rules update` command works (Task 3.4)
- [ ] Remote rules cached in `.repository/.cache/rules/`; offline fallback works (Task 3.4)
- [ ] `repo rules-import --from cursor` imports `.cursorrules` rules (Task 3.5)
- [ ] `repo rules-import --from claude` imports `CLAUDE.md` rules (Task 3.5)
- [ ] `repo rules-import --from windsurf` imports `.windsurfrules` rules (Task 3.5)
- [ ] `--dry-run` flag works for all import subcommands (Task 3.5)
- [ ] `repo doctor` passes all checks in a correctly configured environment (Task 3.6)
- [ ] `repo doctor` provides actionable fix suggestions for all failure modes (Task 3.6)
- [ ] `repo doctor --json` outputs machine-readable results (Task 3.6)

---

## Timeline Overview

```
Week 1-2: Phase 1 - Ship Alpha
  [1.1] Fix placeholder URL                     Day 1
  [1.2] git2 vendored feature                   Day 1-2
  [1.3] Release profile optimizations           Day 2
  [1.4] Fix --tools comma delimiter             Day 1
  [1.5] Update README                           Day 2-3
  [1.6] Create CHANGELOG.md                     Day 2
  [2.1] CI workflow + Dependabot                Day 3-4
  [2.2] cargo-dist bootstrap                    Day 4-5
  [2.3] cargo-release config                    Day 5
  [2.4] First release tag v0.1.0                Day 6
  [2.5] Final README + badge                    Day 7
  ----------------------------------------
  MILESTONE: Alpha released (v0.1.0)

Week 3-4: Phase 2A - Publishing & Governance
  [2A-1] Declare MSRV (Rust 1.85)              Week 3
  [2A-2] Complete Cargo metadata               Week 3
  [2A-3] Set up cargo-deny                     Week 3
  [2A-4] Publish all crates to crates.io       Week 4
  [2A-5] Winget + Scoop channels               Week 4

Week 4-5: Phase 2B - Documentation & UX
  [2B-1] Fix init default (standard mode)      Week 4
  [2B-2] MCP server documentation              Week 4-5
  [2B-3] Fix preset documentation              Week 4
  [2B-4] Vaultspec optional docs               Week 5
  [2B-5] CONTRIBUTING.md + SECURITY.md         Week 5

Week 5-7: Phase 2C - Feature Development
  [2C-1] Expand to 20+ tools                   Weeks 5-7 (parallelizable)
  [2C-2] Snap mode implementation              Weeks 5-7
  ----------------------------------------
  MILESTONE: Beta released; public announcement ready

Week 8-9: Phase 3A - Infrastructure Polish
  [3.1] Docker image to ghcr.io               Week 8
  [3.2] JSON Schema generation                Week 8-9
  [3.3] Documentation accuracy update         Week 8

Week 9-12: Phase 3B - Feature Polish
  [3.4] Remote rule includes                  Weeks 9-11
  [3.5] Tool config import (3 formats)        Weeks 10-12 (parallelizable)
  [3.6] repo doctor command                   Weeks 9-10
  ----------------------------------------
  MILESTONE: v1.0 released
```

---

## Success Metrics

| Metric | Current State | Target (GA) |
|--------|--------------|-------------|
| Time-to-first-value | 10-20 minutes (build from source) | Under 60 seconds (curl installer) |
| Tool integrations | 13 (documented), 14 (actual) | 20+ |
| Documentation score | 2.4/5 | 4+/5 |
| Distribution channels | 0 | 5+ (GitHub Releases, crates.io, Homebrew, Winget, Docker/ghcr.io) |
| CI coverage | Docker integration tests only | Cross-platform matrix + cargo-deny |
| External contributors | 0 | First external PRs merged |
| Marketing readiness | 2.5/10 | 7+/10 |

---

## Risk Register

| # | Risk | Probability | Impact | Mitigation |
|---|------|-------------|--------|------------|
| 1 | **mise adds AI agent config features.** If mise (14K stars, active Rust development) adds AGENTS.md generation or MCP config propagation, its community dwarfs Repository Manager's audience. | Medium | High | Ship fast. Establish positioning before this happens. Consider mise plugin integration as a complementary angle rather than a competing one. |
| 2 | **Ruler adds worktree support.** Ruler has 2,500 stars and broadest tool support. Worktree addition would weaken Repository Manager's unique positioning. | Medium | High | The worktree + agent story (snap mode, MCP server, hooks lifecycle) must be excellent before public launch. Snap mode (Phase 2C) is the key differentiator to build. |
| 3 | **cargo-dist init fails due to workspace structure.** Complex multi-crate workspaces occasionally have issues with cargo-dist initialization. | Medium | High | Run `cargo dist plan` before committing. Review generated workflow carefully. Have a fallback plan using a hand-written GitHub Actions release workflow. |
| 4 | **Alpha quality perception damages launch.** Several features are scaffolded but not fully functional (agent spawn, plugins, preset installation). Users who discover these gaps may form negative impressions. | Medium | Medium | Be explicit about alpha status. Add implementation status matrix to README (Phase 2B). Do not market unimplemented features. |
| 5 | **Remote rule includes creates security concerns.** Git-based remote includes pull code from external sources. A malicious rule repository could inject harmful content into developer environments. | Low | High | Clear documentation on verification expectations. Only support git remotes (not unauthenticated HTTP). Use git authentication mechanisms. Add `repo rules update --dry-run` to preview before applying. |

---

## References

- [ADR Index](../decisions/README.md)
- [Phase 1 Detail](2026-02-18-implementation-plan-phase1.md)
- [Phase 2 Detail](2026-02-18-implementation-plan-phase2.md)
- [Marketing Audit Consolidated](../audits/2026-02-18-marketing-audit-consolidated.md)
- [Feature Gap Analysis](../audits/2026-02-18-feature-gap-analysis.md)
- [Research Consolidated](../audits/2026-02-18-research-consolidated.md)
