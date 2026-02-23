# ADR-0020: Defer read-only enforcement, update documentation

**Status:** Accepted
**Date:** 2026-02-18
**Decision Makers:** Project Team
**Informed by:** [2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md), [2026-02-18-competitor-analysis.md](../audits/2026-02-18-competitor-analysis.md), [2026-02-18-research-consolidated.md](../audits/2026-02-18-research-consolidated.md)

## Context and Problem Statement

The project overview claims the tool will "validate that agents have not hallucinated changes to read-only configuration." The actual implementation provides drift detection (checksum comparison) and `repo fix` for repair. No distinct permission model or enforcement mechanism exists. This gap between documented capability and implementation is a form of overpromising that erodes user trust and creates false expectations for enterprise evaluators.

## Decision Drivers

- Documentation must accurately reflect implemented behavior
- Drift detection + repair is genuinely valuable and deserves clear documentation on its own merits
- Implementing a full permission ledger is non-trivial engineering work with no current enterprise demand signal
- Speculative engineering for unvalidated features carries opportunity cost
- The distinction between detection and prevention is meaningful to security-conscious users

## Considered Options

1. Implement distinct read-only enforcement with a ledger-based permission model
2. Accept that current drift detection is sufficient and make no documentation changes
3. Implement git hook enforcement to prevent commits touching managed files
4. Defer enforcement implementation, update documentation to accurately describe drift detection + repair, create GitHub issue for future tracking

## Decision Outcome

**Chosen option:** "Defer and update docs to accurately describe drift detection + repair", because the current implementation is valuable on its own and honest documentation serves users better than marketing copy that overstates capabilities. A GitHub issue provides a clear signal path for when enterprise demand materializes, without committing engineering resources prematurely. Drift detection already catches modifications — the remaining gap is prevention vs. detection, which is a distinct feature with distinct user needs.

### Consequences

**Good:**
- Honest documentation builds trust with users and evaluators
- No speculative engineering for unvalidated requirements
- Drift detection + repair is accurately surfaced as a first-class capability
- Clear roadmap item for enterprise read-only enforcement when demand materializes

**Bad:**
- Loses a marketing talking point until true enforcement is implemented
- Enterprise users evaluating the tool may find the gap between detection and prevention significant

## More Information

- **Related ADRs:** [ADR-0004](0004-agentic-workspace-manager-positioning.md), [ADR-0014](0014-detection-only-presets-mise-integration.md)
- **Audit Reports:** [2026-02-18-feature-gap-analysis.md (Sections 1.2, P2-6)](../audits/2026-02-18-feature-gap-analysis.md), [2026-02-18-competitor-analysis.md (Section 6)](../audits/2026-02-18-competitor-analysis.md), [2026-02-18-research-consolidated.md (Section 4)](../audits/2026-02-18-research-consolidated.md)
- **Implementation:** Update `project-overview.md`: change "validate that agents have not hallucinated changes to read-only configuration" to "detect configuration drift and automatically repair managed files." Create GitHub issue tracking enterprise read-only enforcement feature for future prioritization.
