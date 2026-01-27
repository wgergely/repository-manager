# Repository Manager Testing Framework

## Overview

This directory contains the **conceptual testing framework** for validating Repository Manager's implementation against its specifications.

## Philosophy

The framework is built on **spec-driven testing**:

1. Read the specification documents
2. Write tests that validate spec claims
3. Run tests to discover implementation gaps
4. Document gaps systematically

## Files

| File | Purpose |
|------|---------|
| `CONCEPTUAL_TESTING_FRAMEWORK.md` | Full framework documentation with mission-based test scenarios |
| `GAP_TRACKING.md` | Registry of discovered implementation gaps with priorities |
| `README.md` | This file |

## Test Results Summary (2026-01-27)

```
Mission Tests: 42 total
├── Passed:  32
├── Failed:   0
└── Ignored: 10 (documented gaps)

Gap Categories:
├── Critical: 5 (sync/fix incomplete, MCP missing, config parsing bug)
├── High:     5 (git ops, tool sync triggers)
├── Medium:   8 (additional tools, preset providers)
└── Low:      4 (config providers)
```

## Running Tests

```bash
# Run all mission tests
cargo test --test mission_tests -- --test-threads=1

# Run with output (see test_summary)
cargo test --test mission_tests -- --nocapture

# See ignored tests (documented gaps)
cargo test --test mission_tests -- --ignored

# Run specific mission category
cargo test --test mission_tests m1_init
cargo test --test mission_tests m2_branch
cargo test --test mission_tests m3_sync
```

## Key Discoveries

Through this framework, we discovered:

1. **GAP-021**: `SyncEngine` reads `tools` instead of `active.tools` from config
2. **GAP-022**: `ToolSyncer` and `repo-tools` integrations have different file paths for Claude
3. **GAP-004/005**: `sync()` and `fix()` are more complete than the audit suggested, but still have gaps
4. **API Evolution**: Tool integrations now use factory functions (`cursor_integration()`) instead of struct constructors

## Contributing

When adding new tests:

1. Follow the mission-based organization (M1, M2, etc.)
2. Document which spec the test validates
3. If testing an unimplemented feature, add to `gaps` module with `#[ignore]`
4. Update `GAP_TRACKING.md` with new discoveries
