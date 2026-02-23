# ADR-0011: Default to standard mode with worktrees recommended

**Status:** Accepted
**Date:** 2026-02-18
**Decision Makers:** Project Team
**Informed by:** [audits/2026-02-18-setup-ease-audit.md](../audits/2026-02-18-setup-ease-audit.md) (Sections 2.2, 6), [audits/2026-02-18-documentation-audit.md](../audits/2026-02-18-documentation-audit.md) (Section 2), [audits/2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md) (Section 1.1)

## Context and Problem Statement

`repo init` currently defaults to "worktrees" mode (interactive prompt default index 0). This creates a `main/` subdirectory that surprises users unfamiliar with git worktrees. Standard mode is more familiar to the broader developer audience. The "Agentic Workspace Manager" positioning (ADR-0004) needs worktrees to be visible and promoted, but not forced on users who run non-interactive init (scripts, CI, quick setup).

## Decision Drivers

- Non-interactive init should be safe and unsurprising for new users
- Worktrees are the key differentiating feature and should be prominently recommended
- Scripts and CI pipelines benefit from predictable, standard-mode defaults
- Guided interactive setup is the right place to promote the worktrees workflow

## Considered Options

1. Keep worktrees as default for both interactive and non-interactive
2. Default to standard mode for non-interactive; recommend worktrees in interactive prompt
3. Default to standard mode for both interactive and non-interactive

## Decision Outcome

**Chosen option:** "Default to standard mode for non-interactive; recommend worktrees in interactive prompt", because it gives safe defaults for scripts and quick use while actively promoting the key feature during guided setup.

Non-interactive `repo init` defaults to "standard" mode. Interactive mode lists worktrees first with a "(recommended for multi-agent workflows)" label, defaulting to index 0 (worktrees). Users who go through the prompt are guided toward the recommended workflow; users running scripts get a predictable, familiar result.

### Consequences

**Good:**
- Non-interactive scripts and CI pipelines get standard mode without surprises
- Interactive users are guided toward the recommended worktrees workflow
- Reduces friction for users evaluating the tool for the first time
- Preserves worktrees as the promoted default in guided setup

**Bad:**
- Users who relied on non-interactive init creating worktrees mode must update scripts
- Slightly inconsistent default behavior between interactive and non-interactive modes

## More Information

- **Related ADRs:** [ADR-0004](0004-agentic-workspace-manager-positioning.md), [ADR-0012](0012-snap-mode-agent-lifecycle.md)
- **Audit Reports:** [audits/2026-02-18-setup-ease-audit.md](../audits/2026-02-18-setup-ease-audit.md), [audits/2026-02-18-documentation-audit.md](../audits/2026-02-18-documentation-audit.md), [audits/2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md)
- **Implementation:** Change default in non-interactive init from "worktrees" to "standard". Update interactive prompt to show: `["worktrees (recommended for multi-agent workflows)", "standard"]` with worktrees still at index 0. Add `--mode` flag documentation.
