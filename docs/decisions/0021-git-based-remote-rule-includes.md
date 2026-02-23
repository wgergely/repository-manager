# ADR-0021: Git-based Remote Rule Includes

**Status:** Accepted
**Date:** 2026-02-18
**Decision Makers:** Project Team
**Informed by:** [2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md), [2026-02-18-competitor-analysis.md](../audits/2026-02-18-competitor-analysis.md)

## Context and Problem Statement

All rules are currently local files in `.repository/rules/`. Competitor ai-rulez supports remote includes from git/HTTP URLs. Team and org rule sharing requires remote sources, and there is no current mechanism to share or version rules across repositories or teams.

## Decision Drivers

- Enable team/org rule sharing without manual copy-paste
- Versioned rules for auditability and reproducibility
- Security must not be compromised for convenience
- Offline-capable workflows should be preserved after initial setup

## Considered Options

1. Git-based remote includes only
2. HTTP URL includes only
3. Both git + HTTP
4. Defer remote includes entirely

## Decision Outcome

**Chosen option:** "Git-based remote includes only", because it enables team rule sharing with versioning while avoiding the security risks of unauthenticated HTTP includes for a tool managing developer configurations.

Config syntax example:

```toml
[rules.remote."org-standards"]
source = "https://github.com/org/rules.git"
path = "rust/"
```

### Consequences

**Good:**
- Team/org rule sharing without manual duplication
- Versioned rules with full git history and rollback capability
- Works offline after initial clone via local cache
- Secure by default through git authentication mechanisms

**Bad:**
- Adds git fetch complexity to the implementation
- Requires a caching and update strategy for remote rule repos
- Estimated 3-5 days of implementation effort

## More Information

- **Related ADRs:** [ADR-0001](0001-git2-vendored-feature.md) (git library choice affects remote fetch implementation), [ADR-0013](0013-expand-tool-support-generic-integration.md)
- **Audit Reports:** [../audits/2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md) (Section 2.1, P2-1), [../audits/2026-02-18-competitor-analysis.md](../audits/2026-02-18-competitor-analysis.md) (Section 1.3)
- **Implementation:** Add `[rules.remote]` config section to schema. Implement git clone/fetch for remote rule repos. Cache fetched rules in `.repository/.cache/rules/`. Add `repo rules update` command to refresh remote rules. Estimated 3-5 days.
