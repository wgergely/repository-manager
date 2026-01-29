# Remediation Plans Index - 2026-01-29

This document indexes all implementation plans created from the architecture and DX audits.

---

## Audit Sources

| Document | Focus | Date |
|----------|-------|------|
| [2026-01-29-developer-experience-audit.md](../audits/2026-01-29-developer-experience-audit.md) | DX landscape comparison | 2026-01-29 |
| [2026-01-28-audit-index.md](../audits/2026-01-28-audit-index.md) | Security/robustness audit | 2026-01-28 |
| [GAP_TRACKING.md](../testing/GAP_TRACKING.md) | Implementation gaps | 2026-01-28 |

---

## Implementation Plans

### Critical Priority

| Plan | Addresses | Effort | Status |
|------|-----------|--------|--------|
| [2026-01-29-mcp-server-completion.md](./2026-01-29-mcp-server-completion.md) | DX-001: MCP non-functional | 2-3 days | NOT STARTED |
| [2026-01-29-audit-remediation.md](./2026-01-29-audit-remediation.md) | DATA-1, DATA-2, SYNC-1, ERR-02 | 1-2 days | NOT STARTED |

### High Priority

| Plan | Addresses | Effort | Status |
|------|-----------|--------|--------|
| [2026-01-29-tool-integration-testing.md](./2026-01-29-tool-integration-testing.md) | DX-002, DX-003: Untested sync | 1-2 days | NOT STARTED |
| [2026-01-29-cli-improvements.md](./2026-01-29-cli-improvements.md) | DX-004 through DX-007 | 1 day | NOT STARTED |

### Previously Created

| Plan | Addresses | Status |
|------|-----------|--------|
| [2026-01-29-dx-audit-remediation.md](./2026-01-29-dx-audit-remediation.md) | Initial DX remediation (superseded) | SUPERSEDED |

---

## Gap-to-Plan Mapping

| Gap ID | Gap Description | Plan |
|--------|-----------------|------|
| **DX-001** | MCP server non-functional | mcp-server-completion |
| **DX-002** | No integration tests for tool sync | tool-integration-testing |
| **DX-003** | No real-world workflow testing | tool-integration-testing |
| **DX-004** | No `repo status` command | cli-improvements |
| **DX-005** | No `repo diff` command | cli-improvements |
| **DX-006** | branch checkout missing | cli-improvements |
| **DX-007** | Dry-run output too sparse | cli-improvements |
| **DATA-1** | TOCTOU race conditions | audit-remediation |
| **DATA-2** | Non-atomic ledger writes | audit-remediation |
| **SYNC-1** | Concurrent sync operations | audit-remediation |
| **ERR-02** | JSON root panic | audit-remediation |
| **S2** | No input size validation | audit-remediation |
| **E1** | Production expect() | audit-remediation |

---

## Recommended Execution Order

1. **Week 1: Foundation Fixes**
   - [ ] `audit-remediation.md` - Fix data integrity issues first
   - [ ] Run full test suite after each task

2. **Week 1-2: MCP Server**
   - [ ] `mcp-server-completion.md` - Enable agentic integration
   - [ ] Test with Claude Desktop or Cursor

3. **Week 2: Verification**
   - [ ] `tool-integration-testing.md` - Verify sync actually works
   - [ ] Review and accept snapshots

4. **Week 2: Polish**
   - [ ] `cli-improvements.md` - Professional CLI experience
   - [ ] Generate completions for shells

---

## Execution Instructions

Each plan is designed for the `superpowers:executing-plans` skill:

```
For Claude: Use superpowers:executing-plans to implement <plan-name>.md task-by-task.
```

Or use subagent-driven development:

```
For Claude: Use superpowers:subagent-driven-development to execute tasks from <plan-name>.md
```

---

## Success Criteria

After all plans are executed:

1. **MCP Server**: AI agents can call `repo_check` and `repo_sync` via MCP
2. **Data Integrity**: Ledger writes are atomic, concurrent access is safe
3. **Tool Sync**: Integration tests verify correct output for top 5 tools
4. **CLI**: All commands work with `--json` flag for scripting
5. **Tests**: All tests pass, no regressions

---

## Post-Remediation Verification

```bash
# Run all tests
cargo test --workspace

# Test MCP server
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | cargo run -p repo-mcp

# Test CLI
cargo run -p repo-cli -- status --json
cargo run -p repo-cli -- diff
cargo run -p repo-cli -- sync --dry-run --json

# Run integration tests
cargo test -p repo-tools --test '*_test'
```

---

*Index created: 2026-01-29*
