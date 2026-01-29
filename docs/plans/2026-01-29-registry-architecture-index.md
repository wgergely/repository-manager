# Unified Registry Architecture - Plan Index

**Status:** Active
**Created:** 2026-01-29

## Overview

A capability-driven tool management system replacing ad-hoc registration and block injection with a proper registry → translator → writer pipeline.

## Architecture

```
┌─────────────────────────────────────────┐
│  LAYER 1: Tool Registry                 │  ← Phase 1
│  (Single source of truth for tools)     │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│  LAYER 2: Capability Translator         │  ← Phase 2
│  (Respects tool.capabilities)           │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│  LAYER 3: Config Writers                │  ← Phase 3
│  (Semantic merge, format-aware)         │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│  LAYER 4: Integration                   │  ← Phase 4
│  (ToolSyncer, migration)                │
└─────────────────────────────────────────┘
```

## Phase Documents

| Phase | Document | Tasks | Status |
|-------|----------|-------|--------|
| 1 | [2026-01-29-phase1-tool-registry.md](2026-01-29-phase1-tool-registry.md) | 5 | Pending |
| 2 | [2026-01-29-phase2-capability-translator.md](2026-01-29-phase2-capability-translator.md) | 3 | Pending |
| 3 | [2026-01-29-phase3-config-writers.md](2026-01-29-phase3-config-writers.md) | 4 | Pending |
| 4 | [2026-01-29-phase4-integration.md](2026-01-29-phase4-integration.md) | 3 | Pending |

## Related Documents

- **Research:** [../research/2026-01-29-rust-registry-patterns.md](../research/2026-01-29-rust-registry-patterns.md)
- **Design:** [../design/2026-01-29-tool-registry-overhaul.md](../design/2026-01-29-tool-registry-overhaul.md)

## Supersedes

- `2026-01-29-tool-registry-implementation.md` (archived)
- `2026-01-29-capability-based-registry.md` (archived)

---

*Last updated: 2026-01-29*
