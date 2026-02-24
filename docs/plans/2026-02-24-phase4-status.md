# Phase 4 Status — 2026-02-24

**Date**: 2026-02-24
**Context**: Phase 3 core items confirmed closed. This document assesses Phase 4 readiness.

---

## Phase 4 Item Audit

### 4.1 Issue-to-Worktree Automation

**Status**: Fresh start — no partial implementation found.

Grep for "issue" in `crates/repo-cli/src/` returns only governance lint comments. No
GitHub/GitLab API client, no issue-URL parsing, no auto-worktree-from-issue logic exists.

**What is needed**:
- GitHub/GitLab REST API client (likely `octocrab` or `reqwest`-based)
- `repo issue assign <issue-url>` CLI command
- Worktree naming convention derived from issue number/title
- PR creation linking back to the originating issue

---

### 4.2 Optional Cloud Bridge

**Status**: Fresh start — no partial implementation found.

Grep for "cloud" or "daytona" across all `.rs` files returns nothing. No remote execution
scaffolding, no provider abstraction, no cloud config keys exist.

**What is needed**:
- Provider trait abstraction (local vs. remote executor)
- Daytona API client or generic SSH/container bridge
- `repo cloud spawn` CLI command
- Config opt-in flag (local-first default must be preserved)

---

### 4.3 Interactive TUI

**Status**: Fresh start — no partial implementation found.

Grep for "tui", "ratatui", or "crossterm" in all `Cargo.toml` files returns nothing. No TUI
crate dependency exists anywhere in the workspace.

**What is needed**:
- New crate (e.g., `repo-tui`) — this cannot live cleanly inside `repo-cli`
- `ratatui` + `crossterm` dependencies
- Worktree list/selection widget
- Config health dashboard panel
- Agent log streaming panel (if agents are re-introduced; otherwise a sync-status panel)

**Cross-cutting concern**: This requires a new workspace crate. Adding TUI to the existing
`repo-cli` binary would significantly inflate binary size and compile time for users who
never use the TUI. Recommend `repo-tui` as an optional `[[bin]]` or a separate crate
behind a feature flag.

---

### 4.4 Skills Marketplace Bridge

**Status**: Near-fresh start — "skills" appears only as an extension content-type enum value
in `crates/repo-extensions/src/manifest.rs` (line 30, 402, 444). This is the extension
manifest's `content_types` field, not a marketplace integration.

No registry client, no `repo skills` CLI commands, and no Tessl API integration exist.

**What is needed**:
- Registry protocol decision (Tessl, custom, or OCI-based)
- `repo skills search / install / list` CLI commands
- Extension manifest already defines `skills` as a content type — this is a natural
  on-ramp; the manifest format can carry marketplace metadata with minimal changes

---

## Recommended Sequencing

| Priority | Item | Rationale |
|----------|------|-----------|
| 1 | **4.4 Skills Marketplace** | Lowest effort. Extension manifest already models `skills` as a content type. Adding a registry lookup and `repo skills` CLI commands builds directly on existing infrastructure. High user-visible value with modest scope. |
| 2 | **4.1 Issue-to-Worktree** | High daily-driver value for teams. Builds on existing `branch`/worktree commands. GitHub API integration is well-understood. Effort is moderate (API client + one new command). |
| 3 | **4.3 Interactive TUI** | High delight factor but requires a new crate and significant UI work. Tackle after 4.1 and 4.4 so the TUI has more surface area to display (issues, skills, worktrees). |
| 4 | **4.2 Cloud Bridge** | Largest scope, most speculative value. Keep local-first as the default and treat this as an opt-in integration. Defer until Phase 4 items 1-3 ship and user demand is validated. |

---

## Cross-Cutting Concerns

1. **New `repo-tui` crate** — 4.3 cannot ship cleanly inside `repo-cli`. Plan for a new
   workspace member before starting TUI work.

2. **Authentication surface** — 4.1 (GitHub API) and 4.4 (registry) both require storing
   tokens. A shared credential store (e.g., system keyring via `keyring` crate, or a
   `~/.config/repo/credentials.toml` with appropriate file permissions) should be designed
   once and used by both features.

3. **Async runtime** — `repo-mcp` already uses `tokio`. 4.1 and 4.4 API clients will also
   need async. Confirm `repo-cli` is already linked against `tokio` (it uses `repo-mcp`
   transitively), or add the dependency explicitly.

4. **Phase 3 loose ends** — Three sub-items remain open from Phase 3:
   - 3.2 Ledger-based audit trail
   - 3.4 MCP server health checks and discovery
   - 3.5 Org-level config layer

   These are lower priority than Phase 4 items 4.4 and 4.1 but should be tracked. They
   are not blockers for Phase 4.

---

*Created: 2026-02-24*
