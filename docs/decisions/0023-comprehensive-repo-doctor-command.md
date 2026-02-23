# ADR-0023: Implement Comprehensive Repo Doctor Command

**Status:** Accepted
**Date:** 2026-02-18
**Decision Makers:** Project Team
**Informed by:** [2026-02-18-setup-ease-audit.md](../audits/2026-02-18-setup-ease-audit.md), [2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md)

## Context and Problem Statement

There is no self-diagnostic command. Users must debug issues manually by interpreting error messages and checking their environment. The setup audit rated onboarding at 5/10, partly due to unclear prerequisite errors. A `repo doctor` command that checks the full environment state would reduce support burden and improve the onboarding experience.

## Decision Drivers

- Users need actionable guidance when something goes wrong, not raw error messages
- Onboarding experience must improve to reach a higher ease rating
- Self-service debugging reduces maintainer support burden
- Environment issues (wrong git version, missing binaries) should be caught before they cause confusing downstream errors

## Considered Options

1. Full repo doctor — checks git, config, sync state, prerequisites, tool installations, and worktree health
2. Minimal doctor — git version, config parse, and sync state only
3. Enhance existing error messages in place rather than adding a new command

## Decision Outcome

**Chosen option:** "Full repo doctor", because a comprehensive diagnostic surface gives users (and maintainers helping users) a single command that exposes the complete health of the local setup. Minimal checks leave too many failure modes unaddressed, and improving inline error messages alone does not provide a proactive summary view.

Checks to implement:
- Git version compatibility
- Repository detection
- `config.toml` parse validity
- Sync status and managed file integrity
- Vaultspec availability (optional subsystem — warn only if absent)
- Tool binary detection for all configured tools

Output format: clear pass/fail per check with an actionable fix suggestion for each failure.

### Consequences

**Good:**
- Self-service debugging — users can diagnose and fix most issues without filing issues
- Reduces maintainer support burden
- Improves onboarding experience by surfacing environment problems with clear guidance
- Catches environment issues before they produce confusing cascading errors

**Bad:**
- Estimated 2-3 days of implementation effort
- Ongoing maintenance cost as new checks become necessary when new features are added

## More Information

- **Related ADRs:** [ADR-0017](0017-vaultspec-optional-subsystem.md) (doctor checks vaultspec availability as optional), [ADR-0005](0005-comma-delimited-tools-flag.md)
- **Audit Reports:** [../audits/2026-02-18-setup-ease-audit.md](../audits/2026-02-18-setup-ease-audit.md) (Section 2.5), [../audits/2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md) (P2-4)
- **Implementation:** Add `repo doctor` subcommand. Implement check modules: `GitCheck`, `ConfigCheck`, `SyncCheck`, `ToolCheck`, `WorktreeCheck`, `VaultspecCheck` (optional). Each module returns a `CheckResult(Pass/Warn/Fail, message, fix_suggestion)`. Format output with colors and a summary line. Estimated 2-3 days.
