# Knowledge Journal - 2026-01-28

## Log Entry: Documentation Refactor

**User Request**: Clean up documentation, address contradictions, compact knowledge.

### 1. Naming Standards

- **Issue**: Documentation referenced `repo-manager` crate.
- **Correction**: The core crate is `repo-core`. The CLI is `repo-cli`. The workspace is `repository-manager`.
- **Action**: Updated `architecture-core.md` and `spec-mcp-server.md` to reflect reality.

### 2. Gap Verification

- **Issue**: `GAP_TRACKING.md` listed GAP-012/013 as open, while `final-gap-closure.md` listed them as done.
- **Resolution**: Verified `repo-presets` contains implementation. Updated `GAP_TRACKING.md` to CLOSED.

### 3. Audit Archival

- **Action**: Superseded audit files (from Jan 23rd) moved to `docs/audits/archive/`.
- **Reference**: Use `2026-01-28-audit-index.md` for current findings.

## Compaction: Stale Instructions

*Summarizing key directives from older plans:*

### From 2026-01-26-full-roadmap.md

- **Vision**: Worktree-mode first. `repo-core` abstracts the complexity.
- **Pattern**: "Unrolling" config means deleting/reaping files we don't own anymore.

### From 2026-01-23-phase1-vertical-slice.md

- **Critical Path**: `repo init` -> `repo branch` -> `repo sync`.
- **Constraint**: No `unsafe` code in core crates.

---
*Journal updated: 2026-01-28*
