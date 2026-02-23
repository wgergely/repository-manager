# ADR-0022: Import from Top 3 Tool Config Formats

**Status:** Accepted
**Date:** 2026-02-18
**Decision Makers:** Project Team
**Informed by:** [2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md), [2026-02-18-competitor-analysis.md](../audits/2026-02-18-competitor-analysis.md)

## Context and Problem Statement

AGENTS.md import exists via `repo rules-import`, but importing from existing `.cursorrules`, `CLAUDE.md`, and `.windsurfrules` into `.repository/rules/` is not implemented. Competitor rulesync supports bidirectional import/export. Migration friction — the "I already have rules, why switch?" objection — is a key barrier to adoption.

## Decision Drivers

- Reduce migration friction for users with existing tool-specific rule files
- Directly address the adoption barrier of sunk cost in existing configurations
- Leverage existing markdown parsing infrastructure where possible
- Prioritize formats with the largest existing user bases

## Considered Options

1. Import from all 14 supported tool formats
2. Import from top 3 tools only (Cursor, Claude, Windsurf)
3. Universal plain-text import without format-specific parsing
4. Defer import functionality entirely

## Decision Outcome

**Chosen option:** "Import from top 3 tools only", because `.cursorrules`, `CLAUDE.md`, and `.windsurfrules` are all markdown-like formats making parsing straightforward, and these tools represent the majority of users likely to migrate. Expanding to more formats based on measured user demand avoids over-engineering for low-usage formats.

### Consequences

**Good:**
- Reduces migration friction for the majority of prospective users
- Leverages existing markdown parsing — no new parser infrastructure needed
- Creates a compelling onboarding path: run one command to migrate existing rules
- Scope is bounded and deliverable incrementally

**Bad:**
- Parsing is imperfect due to format-specific quirks in each tool's rule syntax
- Users of the remaining 11 supported tools have no import path initially

## More Information

- **Related ADRs:** [ADR-0013](0013-expand-tool-support-generic-integration.md), [ADR-0011](0011-default-standard-mode.md)
- **Audit Reports:** [../audits/2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md) (Section 2.1, P2-3), [../audits/2026-02-18-competitor-analysis.md](../audits/2026-02-18-competitor-analysis.md) (Section 1.2)
- **Implementation:** Add `repo rules-import --from cursor` (and `--from claude`, `--from windsurf`) subcommands. Parse markdown content from each tool's config file. Map parsed content to `.repository/rules/` structure. Add `--dry-run` flag to preview import without writing files. Estimated 2-3 days per format.
