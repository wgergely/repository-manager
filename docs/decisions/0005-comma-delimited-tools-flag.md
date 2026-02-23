# ADR-0005: Support comma-delimited values in --tools flag

**Status:** Accepted
**Date:** 2026-02-18
**Decision Makers:** Project Team
**Informed by:** 2026-02-18-documentation-audit.md (Section 1), 2026-02-18-setup-ease-audit.md (Section 2.2), 2026-02-18-marketing-audit-consolidated.md (blocker #2), 2026-02-18-feature-gap-analysis.md (P0-3)

## Context and Problem Statement

The README documents `repo init my-project --tools cursor,claude,vscode` as the canonical usage pattern, but the actual clap implementation uses `Vec<String>` without a value delimiter configured. Comma-separated input is treated as a single tool name "cursor,claude,vscode" and silently fails — no error is raised, no tools are configured. A documentation audit flagged this as "trust-destroying": users follow the documented syntax, get no feedback that it failed, and are left with a misconfigured project. This is a P0 blocker for user trust.

## Decision Drivers

- The README explicitly shows comma-separated syntax; users will follow it and silently get wrong behavior
- Silent failure with no error message makes the bug extremely hard to diagnose
- The fix is a single-line attribute change with no architectural impact
- Both `-t cursor -t claude` (repeated flag) and `--tools cursor,claude` (comma syntax) should work
- Consistency across all Vec<String> flags (e.g., `presets`) is desirable

## Considered Options

1. Add comma delimiter support - add `value_delimiter = ','` to clap attribute. Supports both `-t cursor -t claude` AND `--tools cursor,claude`. 1-line change.
2. Fix README only - change docs to show repeated flags (`--tools cursor --tools claude`). Zero code change but less ergonomic and deviates from conventional CLI UX.

## Decision Outcome

**Chosen option:** "Add comma delimiter support", because it's a 1-line attribute change that makes both syntaxes work. README becomes correct, users get the ergonomic comma syntax they expect, and repeated-flag syntax continues to work. Fixing the README alone would trade a code bug for a UX regression.

### Consequences

**Good:**
- README example works as documented
- Both comma-separated and repeated-flag syntaxes are supported
- Silent failure is eliminated
- Low risk: single attribute addition to existing clap field

**Bad:**
- Comma characters in tool names would be interpreted as delimiters (acceptable — tool names do not contain commas)
- Requires applying the same fix to other `Vec<String>` flags for consistency, adding minor scope

## More Information

- **Related ADRs:** None
- **Audit Reports:** docs/audits/2026-02-18-documentation-audit.md (Section 1), docs/audits/2026-02-18-setup-ease-audit.md (Section 2.2), docs/audits/2026-02-18-marketing-audit-consolidated.md (blocker #2), docs/audits/2026-02-18-feature-gap-analysis.md (P0-3)
- **Implementation:** Change clap attribute in `crates/repo-cli/src/cli.rs` lines 55-57 from:
  ```rust
  #[arg(short, long)]
  tools: Vec<String>,
  ```
  to:
  ```rust
  #[arg(short, long, value_delimiter = ',')]
  tools: Vec<String>,
  ```
  Apply the same fix to any other `Vec<String>` flags (e.g., `presets`) for consistency.
