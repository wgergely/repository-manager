# ADR-0012: Implement snap mode for agent lifecycle

**Status:** Accepted
**Date:** 2026-02-18
**Decision Makers:** Project Team
**Informed by:** [audits/2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md) (Sections 2.2, P1-8, P1-9), [audits/2026-02-18-competitor-analysis.md](../audits/2026-02-18-competitor-analysis.md) (Section 2.2)

## Context and Problem Statement

Competitor agent-worktree offers `wt new -s claude` — a single command to create a worktree, launch an agent, merge, and clean up. Repository Manager currently requires separate commands for each step. A unified "snap mode" workflow is the "killer feature" that would differentiate Repository Manager for the agentic workflow positioning. Without it, users must manually coordinate worktree creation, file copying, agent launch, and cleanup.

## Decision Drivers

- Agentic workflow positioning requires a compelling one-command agent workflow
- Developers want minimal friction when starting agent-assisted tasks
- File copying (.env, secrets) is a recurring pain point when creating worktrees
- Graceful degradation is required — the feature should work without vaultspec

## Considered Options

1. Implement full snap mode with agent launch, file copy, and cleanup as a single command
2. Implement snap mode as worktree + file-copy only (no agent launch)
3. Document a shell alias/script approach rather than building it into the CLI

## Decision Outcome

**Chosen option:** "Implement full snap mode with agent launch, file copy, and cleanup as a single command", because it delivers the key differentiating workflow and provides graceful degradation when vaultspec is unavailable.

Snap mode is implemented via `repo branch add --snap <agent>`. The workflow is: (1) create worktree, (2) copy .env/gitignored files into the new worktree, (3) launch the specified agent if vaultspec is available (graceful degradation without it), (4) clean up worktree on completion. The worktree and file-copy steps work independently of vaultspec.

### Consequences

**Good:**
- Delivers a compelling one-command agentic workflow that matches competitors
- File copy for .env and gitignored files solves a universal pain point
- Graceful degradation means value is delivered even without full vaultspec setup
- `--no-cleanup` flag supports debugging and inspection after agent runs

**Bad:**
- Estimated 3-5 days implementation effort
- Introduces subprocess management complexity for agent invocation
- Behavior depends on external agent availability and vaultspec configuration

## More Information

- **Related ADRs:** [ADR-0004](0004-agentic-workspace-manager-positioning.md), [ADR-0011](0011-default-standard-mode.md), [ADR-0017](0017-vaultspec-optional-subsystem.md)
- **Audit Reports:** [audits/2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md), [audits/2026-02-18-competitor-analysis.md](../audits/2026-02-18-competitor-analysis.md)
- **Implementation:** Add `--snap` flag to `repo branch add` command. Implement file-copy for .env, .envrc, and other gitignored files. Integrate with repo-agent subprocess invocation. Add `--no-cleanup` flag for debugging. Estimated effort: 3-5 days.
