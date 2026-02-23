# ADR-0019: Auto-generate JSON Schema from Rust config types

**Status:** Accepted
**Date:** 2026-02-18
**Decision Makers:** Project Team
**Informed by:** [2026-02-18-competitor-analysis.md](../audits/2026-02-18-competitor-analysis.md), [2026-02-18-research-consolidated.md](../audits/2026-02-18-research-consolidated.md), [2026-02-18-feature-gap-analysis.md](../audits/2026-02-18-feature-gap-analysis.md), [2026-02-18-setup-ease-audit.md](../audits/2026-02-18-setup-ease-audit.md)

## Context and Problem Statement

`.repository/config.toml` uses a structured TOML schema backed by serde-deserialized Rust structs. No validation exists beyond deserialization errors at runtime. Users writing config files receive no editor guidance, no autocompletion, and no early error detection. Adding JSON Schema would enable IDE integration and make the config format machine-readable for tooling such as MCP clients.

## Decision Drivers

- Config errors are currently only caught at runtime during deserialization
- Editor autocompletion significantly reduces friction for users writing `.repository/config.toml`
- A hand-written schema would drift from the Rust types over time
- MCP clients and other tooling benefit from a machine-readable description of the config format
- The `schemars` crate integrates directly with `serde` derive macros

## Considered Options

1. Auto-generate JSON Schema via `schemars` crate derived from serde structs
2. Hand-written JSON Schema maintained separately from Rust types
3. No schema validation — rely on runtime deserialization errors only

## Decision Outcome

**Chosen option:** "Auto-generate via schemars crate from serde structs", because it guarantees the schema is always in sync with the actual config types at zero maintenance cost. Even with limited native TOML editor support, VS Code extensions (e.g., Even Better TOML) can validate against JSON Schema, and MCP clients benefit from understanding the format programmatically.

### Consequences

**Good:**
- Schema always in sync with Rust config types — no drift possible
- Editor autocompletion and inline validation for `.repository/config.toml`
- Better error messages before runtime deserialization
- Machine-readable config documentation for tooling and MCP clients
- Schema published alongside releases as a versioned artifact

**Bad:**
- Adds `schemars` dependency to `repo-meta` crate
- TOML-specific schema tooling ecosystem is more limited than JSON/YAML equivalents
- Requires a `repo schema` subcommand to expose the generated schema

## More Information

- **Related ADRs:** [ADR-0010](0010-mcp-server-documentation-first.md)
- **Audit Reports:** [2026-02-18-competitor-analysis.md (Section 6)](../audits/2026-02-18-competitor-analysis.md), [2026-02-18-research-consolidated.md (Section 2)](../audits/2026-02-18-research-consolidated.md), [2026-02-18-feature-gap-analysis.md (P2-8)](../audits/2026-02-18-feature-gap-analysis.md), [2026-02-18-setup-ease-audit.md (Section 3.2)](../audits/2026-02-18-setup-ease-audit.md)
- **Implementation:** Add `schemars` dependency to `repo-meta`. Derive `JsonSchema` on config structs alongside existing `serde` derives. Add `repo schema` subcommand to output the generated JSON Schema to stdout. Include schema file in release artifacts produced by cargo-dist.
