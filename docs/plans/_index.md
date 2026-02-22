# Implementation Plans & Knowledge Log

Index of implementation plans, roadmaps, and the daily knowledge journal.

## Active Journals

- **[2026-01-28-knowledge-journal.md](2026-01-28-knowledge-journal.md)**: Daily knowledge compaction and stale instruction summary.

## Active Task Plans (2026-02-22)

- **[../tasks/_index.md](../tasks/_index.md)**: Master task index — domain-separated implementation plans from deep audit
  - **P0**: [Sync Engine & Tool Paths](../tasks/P0-sync-engine-and-tool-paths.md) — C-1, C-2, M-3
  - **P1**: [Hooks System](../tasks/P1-hooks-system.md) — C-3
  - **P1**: [Extension System](../tasks/P1-extension-system.md) — C-4
  - **P2**: [MCP Server](../tasks/P2-mcp-server.md) — H-1, H-2
  - **P2**: [Preset Providers](../tasks/P2-preset-providers.md) — H-3, H-4
  - **P3**: [Config Hierarchy](../tasks/P3-config-hierarchy.md) — M-1
  - **P3**: [Test Hygiene](../tasks/P3-test-hygiene.md) — M-2, coverage gaps

## Test Health Remediation Chain (2026-02-22)

Five-phase implementation plan from the [Test Health Audit](../audits/2026-02-22-test-health-audit.md).
Each phase is self-contained with explicit dependencies. Phases 1-2 parallelize; 3-5 are sequential.

| Phase | Plan | Finding | Scope | Depends On |
|-------|------|---------|-------|------------|
| 1 | [Taxonomy & Renaming](2026-02-22-test-health-phase1-taxonomy.md) | TH-1 | 3 files renamed | None |
| 2 | [Shared Fixture Library](2026-02-22-test-health-phase2-fixtures.md) | TH-2 | New `repo-test-utils` crate | None |
| 3 | [Format Validation Tests](2026-02-22-test-health-phase3-format-validation.md) | TH-3 | 5 new test files, ~16 tests | Phase 2 |
| 4 | [Golden File Rehabilitation](2026-02-22-test-health-phase4-golden-files.md) | TH-4 | 3 golden files + fixture_tests.rs | Phase 3 |
| 5 | [Tautological Test Elimination](2026-02-22-test-health-phase5-tautological-tests.md) | TH-5 | ~120 tests reviewed, ~40 upgraded | Phase 3 |

```
Phase 1 ──┐
           ├──→ Phase 3 ──→ Phase 4
Phase 2 ──┘          └────→ Phase 5
```

## Prior Plans (superseded by task plans above)

- **[2026-01-28-final-gap-closure.md](2026-01-28-final-gap-closure.md)**: *(Superseded)* Closing remaining critical gaps (GAP-004, GAP-022).
- **[2026-01-28-audit-remediation.md](2026-01-28-audit-remediation.md)**: *(Superseded)* Addressing findings from the Jan 28th audit.

## Design Specifications

- **[../design/2026-01-29-tool-registry-overhaul.md](../design/2026-01-29-tool-registry-overhaul.md)**: Unified tool registration system design specification.

## Implementation Plans

- **[2026-01-29-capability-based-registry.md](2026-01-29-capability-based-registry.md)**: Capability-based registry architecture - phased implementation (Phase 1: 8 tasks)
- **[2026-01-29-tool-registry-implementation.md](2026-01-29-tool-registry-implementation.md)**: *(Superseded)* Original tool registry overhaul plan

## Historical / Completed (Daily Summaries)

- **[2026-01-23.md](2026-01-23.md)**: Vertical Slice & Core Building Blocks.
- **[2026-01-26.md](2026-01-26.md)**: Content System & Git Layers.
- **[2026-01-27.md](2026-01-27.md)**: Integration & Gap Closure.

## Archive

- **[Archive](./archive/)**: Full text of superseded plans.
