# ADR-0003: Add release profile optimizations

**Status:** Accepted
**Date:** 2026-02-18
**Decision Makers:** Project Team
**Informed by:** 2026-02-18-packaging-distribution-audit.md, 2026-02-18-marketing-audit-consolidated.md, 2026-02-18-feature-gap-analysis.md

## Context and Problem Statement

The workspace `Cargo.toml` has no `[profile.release]` section. Rust's default release profile includes debug symbols and uses 16 codegen units, which produces binaries that are larger and run slower than necessary. For a CLI tool distributed as a binary, binary size and startup time directly affect user perception of quality.

With cargo-dist adopted in ADR-0002, release builds will be produced in CI for distribution. This makes it the right time to configure the release profile, as the compile-time cost falls on CI machines rather than developer laptops.

## Decision Drivers

- Binary size affects download time and storage, especially for users in bandwidth-constrained environments
- Startup performance is critical for a CLI tool used in scripting and automation
- Release builds happen in CI, so longer compile times are acceptable
- 30-50% binary size reduction is achievable with LTO and symbol stripping

## Considered Options

1. Full optimization - `lto = true`, `codegen-units = 1`, `strip = true` - smallest binary, best runtime performance, longest CI compile time
2. Thin LTO - `lto = "thin"`, `codegen-units = 1`, `strip = true` - good size reduction with faster compile than full LTO
3. Strip only - `strip = true` - minimal compile-time impact but misses the 20-30% size reduction from LTO

## Decision Outcome

**Chosen option:** "Option 1 - Full optimization", because release builds happen in CI rather than on developer machines, making the extra compile time invisible to users. Full LTO with a single codegen unit produces the smallest possible binary and best runtime performance. Expected outcome is a 30-50% binary size reduction compared to default release builds.

### Consequences

**Good:**
- Smallest possible distributed binary size (30-50% reduction expected)
- Best possible runtime performance for the distributed binary
- Compile-time cost falls entirely on CI, not developers
- `strip = true` removes debug symbols that serve no purpose in distributed binaries

**Bad:**
- Longer CI compile times for release builds compared to thin LTO or no LTO
- Full LTO may expose rare LTO-specific compilation bugs (mitigated by CI testing)
- Debug symbols are stripped, making crash reports from distributed binaries harder to symbolicate without a separate debug build

## More Information

- **Related ADRs:** ADR-0002 (release pipeline using cargo-dist)
- **Audit Reports:** docs/audits/2026-02-18-packaging-distribution-audit.md (P0-6), docs/audits/2026-02-18-marketing-audit-consolidated.md, docs/audits/2026-02-18-feature-gap-analysis.md (P0-4)
- **Implementation:** Add the following section to the workspace `Cargo.toml`:

```toml
[profile.release]
lto = true
codegen-units = 1
strip = true
```
