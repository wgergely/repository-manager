# Implementation Tasks — Domain-Separated Work Plans

**Created**: 2026-02-22
**Source**: [Deep Implementation Audit](../audits/2026-02-22-deep-implementation-audit.md)
**Supersedes**: All prior gap-closure plans (Jan 28 remediation, Jan 29 capability registry)

---

## Testing Mandate (APPLIES TO EVERY TASK BELOW)

**Every task document in this directory inherits these non-negotiable rules.**

### Rule 1: Code quality first, tests second

Write correct, complete implementation code before writing any test. A test
written before the implementation exists tends to be shaped around the stub,
not around the contract. Implementation defines the contract; tests verify it.

### Rule 2: No false-positive tests

A test that passes when the feature is broken is **worse than no test**. It
creates a false sense of security that actively hides bugs. Every test must be
written to **fail** when the feature it guards is removed or broken. If you
cannot demonstrate that your test fails when the feature is absent, the test
is not valid.

Specific anti-patterns that are **banned**:

| Anti-pattern | Why it's dangerous | What to do instead |
|---|---|---|
| `assert!(path.exists())` on a primary path when the task is about secondary paths | Passes even if secondary path work is missing | Assert the specific secondary path |
| Testing that a function returns `Ok(())` without checking side effects | Stubs return `Ok(())` too | Assert the observable side effect (file created, field changed, hook ran) |
| Testing struct construction without calling the method under test | Proves the compiler works, not your code | Call the method and assert its output |
| `#[ignore]` with a TODO comment | Ignored tests are invisible; they rot | Either implement the test or delete it with a tracking issue |
| Snapshot tests that were approved without reading the snapshot | The snapshot may encode broken behavior | Verify snapshot content matches specification |
| Testing only the happy path | Errors are where bugs hide | Test at least one error/edge case per public function |

### Rule 3: Test the contract, not the implementation

Tests should assert **observable behavior** (files written, output produced,
errors returned) — not internal state. This makes tests resilient to
refactoring while still catching regressions.

### Rule 4: One test, one failure mode

Each test should verify one specific behavior. If a test has 8 assertions
across unrelated features, a failure in assertion 3 hides the status of
assertions 4-8. Split into focused tests.

---

## Priority Ordering

Tasks are ordered by **blast radius** (how many users are silently affected)
and **trust damage** (does the tool lie about what it did).

| Priority | Task | Audit IDs | Domain | Why this priority |
|----------|------|-----------|--------|-------------------|
| **P0** | [Sync Engine & Tool Paths](P0-sync-engine-and-tool-paths.md) | C-1, C-2, M-3 | `repo-tools` | **Core value proposition is broken.** 5 tools claim to write secondary configs and don't. Users trust `repo sync` output. |
| **P1** | [Hooks System](P1-hooks-system.md) | C-3 | `repo-core`, `repo-cli` | Users configure hooks that silently never execute. Trust violation. |
| **P1** | [Extension System](P1-extension-system.md) | C-4 | `repo-cli`, `repo-mcp`, `repo-extensions` | Returns `success:true` for operations that do nothing. Trust violation. |
| **P2** | [MCP Server](P2-mcp-server.md) | H-1, H-2 | `repo-mcp` | Announces tools that always error; skips initialization validation. |
| **P2** | [Preset Providers](P2-preset-providers.md) | H-3, H-4 | `repo-presets` | `apply()` returns fake success on Node and Rust providers. |
| **P3** | [Config Hierarchy](P3-config-hierarchy.md) | M-1 | `repo-core` | Missing global/org config layers. Functional without them. |
| **P3** | [Test Hygiene & Stale Tests](P3-test-hygiene.md) | M-2, coverage gaps | `tests/` | Stale GAP-019 test; missing coverage for secondary paths, hooks, MCP runtime. |

---

## Dependency Graph

```
P0: Sync Engine & Tool Paths
│   (no dependencies — start here)
│
├──→ P1: Hooks System
│    (hook call sites go in sync code touched by P0)
│
├──→ P1: Extension System
│    (independent — can parallelize with P1 Hooks)
│
├──→ P2: MCP Server
│    (depends on P0 for sync; uses same tool definitions)
│
├──→ P2: Preset Providers
│    (independent of P0 — can parallelize)
│
├──→ P3: Config Hierarchy
│    (independent — deferred complexity)
│
└──→ P3: Test Hygiene
     (runs LAST — cleans up stale tests after all domains are fixed)
```

**Parallelization opportunities:**
- P0 must go first (everything else touches sync output)
- P1 Hooks + P1 Extensions can run in parallel
- P2 MCP + P2 Presets can run in parallel
- P3 Config + P3 Test Hygiene can run in parallel (run last)

---

## Cross-References

- [2026-02-22 Deep Implementation Audit](../audits/2026-02-22-deep-implementation-audit.md) — source findings
- [2026-02-17 Roadmap](../plans/2026-02-17-roadmap.md) — strategic context
- [ADR documents](../adr/) — architectural decisions

---

*Index created: 2026-02-22*
