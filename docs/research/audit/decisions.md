# Architecture Decision Records

> **Purpose:** Capture key architectural decisions with context, options, and rationale.
> **Format:** ADR (Architecture Decision Record)
> **Last Updated:** 2026-01-29

---

## ADR-001: Docker Compose for Container Orchestration

**Date:** 2026-01-29
**Status:** Accepted
**Deciders:** Research phase

### Context

The test infrastructure needs to orchestrate multiple containers (one per tool) with shared volumes, environment variables, and network connectivity. We need a solution that:
- Works in CI environments (GitHub Actions, GitLab CI)
- Is easy for developers to use locally
- Supports selective tool testing (profiles/groups)
- Has good documentation and community support

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| **Docker Compose** | Industry standard, simple YAML, well-documented | Limited to single-host |
| **Kubernetes/Kind** | Production-like, scalable | Overkill for testing, complex setup |
| **Testcontainers** | Programmatic from Rust, tight integration | Learning curve, less visual |
| **Raw Docker** | No additional tools | Manual coordination, error-prone |

### Decision

**Docker Compose**

### Rationale

1. **Industry standard** - Most developers already know it
2. **CI-friendly** - First-class support in GitHub Actions, GitLab CI
3. **Sufficient features** - Profiles, depends_on, shared volumes meet our needs
4. **Low barrier** - No additional infrastructure required
5. **Testcontainers can be added later** - For programmatic Rust test integration

### Consequences

- Docker Compose v2 required on all development machines
- CI runners need Docker and Compose installed
- Single-host limitation acceptable for testing (not production deployment)
- May revisit if we need distributed testing

---

## ADR-002: Layered Base Images

**Date:** 2026-01-29
**Status:** Accepted
**Deciders:** Research phase

### Context

We need Docker images for 13 tools across 4 categories. Image strategy affects:
- Build time (CI performance)
- Storage costs (registry, local disk)
- Maintenance burden (updating dependencies)
- Rebuild frequency (cache invalidation)

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| **Monolithic** | Simple, one image | Huge size, slow builds, waste |
| **Per-tool images** | Modular, independent | Many images, no layer sharing |
| **Layered bases** | Shared deps, efficient storage | More complex hierarchy |
| **Matrix builds** | Flexible combinations | Complex CI config |

### Decision

**Layered base images with category inheritance**

```
base → cli-base → claude, aider, gemini
base → vscode-base → cline, roo, copilot, amazonq
base → gui-base → cursor, zed, windsurf
gui-base → jetbrains-base → intellij
```

### Rationale

1. **Layer sharing** - Common dependencies cached once
2. **Targeted rebuilds** - Update base only when system deps change
3. **Category consistency** - All VS Code extensions share same VS Code version
4. **Reasonable complexity** - 4 base images is manageable

### Consequences

- Must build base images before tool images
- Base image updates trigger cascading rebuilds
- Need clear documentation of layer hierarchy
- Estimated 5-8 GB total storage (vs 15-20 GB for independent images)

---

## ADR-003: Hybrid API Testing Strategy

**Date:** 2026-01-29
**Status:** Accepted
**Deciders:** Research phase

### Context

Tools require LLM API calls to function. Testing strategy must balance:
- Fast CI feedback (mocked, deterministic)
- Real-world validation (actual API calls)
- Cost control (API usage has real cost)
- Test reliability (network failures, rate limits)

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| **Mock only** | Fast, free, deterministic | Doesn't catch real issues |
| **Real only** | Catches real issues | Slow, costly, flaky |
| **Recorded/replay** | Realistic + deterministic | Maintenance burden |
| **Hybrid** | Best of both | More complex setup |

### Decision

**Hybrid approach with environment-based switching**

- `TEST_MODE=mock` - CI/PR checks use mock server
- `TEST_MODE=real` - Certification uses real APIs
- `TEST_MODE=hybrid` - Development with selective real calls

### Rationale

1. **Fast PR checks** - Mock mode keeps CI fast and free
2. **Real validation** - Periodic certification catches real issues
3. **Developer choice** - Hybrid mode for flexibility
4. **Cost control** - Real API calls only when needed

### Consequences

- Need to maintain mock server stubs
- Mock responses may drift from real API
- Certification runs have cost implications (~$0.40/month estimated)
- Must document which tests require real API

---

## ADR-004: .env File for Credential Management

**Date:** 2026-01-29
**Status:** Accepted
**Deciders:** Research phase

### Context

Real API testing requires credentials (API keys, tokens). These must be:
- Available to Docker containers
- Never committed to version control
- Easy for developers to configure
- Manageable in CI environments

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| **.env file** | Standard, well-known | Easy to accidentally commit |
| **Secret manager** | Secure, centralized | Requires infrastructure |
| **Environment vars** | No file to commit | Tedious to set up |
| **Config file** | Explicit | Same risk as .env |

### Decision

**.env file (gitignored) with .env.example template**

### Rationale

1. **Industry standard** - Developers expect .env pattern
2. **Docker Compose native** - Built-in env_file support
3. **Easy onboarding** - Copy .env.example, fill in values
4. **Git protection** - .gitignore prevents accidental commits
5. **CI compatible** - CI writes secrets to .env before tests

### Consequences

- Must maintain .env.example with all required variables
- .gitignore must include .env patterns
- Need documentation for obtaining API keys
- CI must inject secrets (GitHub Actions secrets → .env)

### Mitigations

- Pre-commit hook to check for .env commits
- .env.example updated when new credentials needed
- Documentation for each credential source

---

## ADR-005: Tiered GUI Testing Approach

**Date:** 2026-01-29
**Status:** Pending Research
**Deciders:** Research phase

### Context

GUI-based tools (Cursor, Zed, Windsurf, JetBrains) present Docker challenges:
- May require display/GPU
- Installation may be interactive
- Testing config loading requires running the app
- Some may not work headless at all

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| **CLI-only** | Simple, works everywhere | Misses GUI tools |
| **Xvfb** | Virtual display, CI-friendly | May not work for all |
| **VNC** | Visual debugging | Overhead, still needs X |
| **Tiered** | Pragmatic coverage | Inconsistent coverage |

### Decision

**Tiered approach (pending research validation)**

- **Tier 1 (Full):** CLI tools, VS Code - Full integration testing
- **Tier 2 (Headless):** Xvfb-compatible tools - Basic integration
- **Tier 3 (Config-only):** Tools that can't run headless - File validation only

### Rationale

1. **Pragmatic** - Test what's testable, don't block on impossible
2. **Progressive** - Can upgrade tiers as we learn more
3. **Transparent** - Document what's actually tested per tool
4. **Research-driven** - Final tiers determined by hands-on testing

### Status: Pending

Requires hands-on research to validate:
- [ ] Which tools work with Xvfb?
- [ ] Which tools have usable headless modes?
- [ ] Which tools absolutely cannot run in Docker?

### Consequences

- Some tools may have limited test coverage
- Must clearly document test coverage per tool
- May need to revisit as tools evolve

---

## ADR-006: Per-Tool Versioning Strategies

**Date:** 2026-01-29
**Status:** Accepted
**Deciders:** Research phase

### Context

Tools have different versioning models:
- Traditional semver (JetBrains, VS Code)
- Package manager semver (Claude CLI, Aider)
- Auto-updating (Cursor, Zed, Windsurf)
- Extension-bound (Cline, Roo, Copilot)

One-size-fits-all approach won't work.

### Decision

**Category-specific strategies:**

| Category | Strategy |
|----------|----------|
| Traditional | Pin specific version, update quarterly |
| Package manager | Pin version, use Dependabot/Renovate |
| Auto-updating | Snapshot by date, rebuild weekly |
| Extension-bound | Pin IDE + extension version pair |

### Rationale

1. **Respects tool nature** - Don't fight auto-updating tools
2. **Practical** - Can pin what's pinnable
3. **Documented** - Clear expectations per tool
4. **Maintainable** - Automated updates where possible

### Consequences

- Inconsistent version control across tools
- Auto-updating tools may introduce surprises
- Need version checking automation
- Compatibility matrix maintenance burden

---

## ADR-007: WireMock for API Mocking

**Date:** 2026-01-29
**Status:** Accepted
**Deciders:** Research phase

### Context

Mock API server needed for CI testing. Requirements:
- Support Anthropic and OpenAI API formats
- Handle streaming responses
- Configurable via files (version controllable)
- Reliable in CI environments

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| **WireMock** | Feature-rich, industry standard | Java-based, larger image |
| **Prism** | OpenAPI-native | Requires specs, less flexible |
| **Custom Rust** | Tailored, lightweight | Development effort |
| **mockserver** | Node.js, simple | Less feature-rich |

### Decision

**WireMock**

### Rationale

1. **Battle-tested** - Used in enterprise environments
2. **Recording** - Can record real API for replay
3. **Streaming** - Supports SSE for streaming responses
4. **JSON config** - Stubs are version-controllable
5. **Community** - Good documentation and support

### Consequences

- Java/JVM in mock container (larger footprint)
- Need to learn WireMock stub format
- May consider Rust replacement later if size matters

---

## ADR-008: Ubuntu 22.04 as Base Image

**Date:** 2026-01-29
**Status:** Accepted
**Deciders:** Research phase

### Context

Base OS for Docker images affects:
- Package availability
- Library compatibility
- Support lifecycle
- Image size

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| **Ubuntu 22.04 LTS** | Long support, good packages | Larger than Alpine |
| **Ubuntu 24.04 LTS** | Newer packages | Just released, less tested |
| **Debian** | Stable, smaller than Ubuntu | Older packages |
| **Alpine** | Tiny images | musl libc compatibility issues |

### Decision

**Ubuntu 22.04 LTS**

### Rationale

1. **LTS support** - Until April 2027 (standard), 2032 (extended)
2. **Package availability** - All tools installable
3. **glibc** - No musl compatibility issues
4. **CI runners** - Matches GitHub Actions ubuntu-latest
5. **Documentation** - Most tool docs assume Ubuntu

### Consequences

- Larger images than Alpine (~200MB base vs ~5MB)
- Acceptable tradeoff for compatibility
- Consider Ubuntu 24.04 LTS in ~2025

---

## Decision Log

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| 001 | Docker Compose for orchestration | Accepted | 2026-01-29 |
| 002 | Layered base images | Accepted | 2026-01-29 |
| 003 | Hybrid API testing strategy | Accepted | 2026-01-29 |
| 004 | .env for credentials | Accepted | 2026-01-29 |
| 005 | Tiered GUI testing | Pending Research | 2026-01-29 |
| 006 | Per-tool versioning | Accepted | 2026-01-29 |
| 007 | WireMock for mocking | Accepted | 2026-01-29 |
| 008 | Ubuntu 22.04 base | Accepted | 2026-01-29 |

---

## Template for New ADRs

```markdown
## ADR-XXX: [Title]

**Date:** YYYY-MM-DD
**Status:** Proposed | Accepted | Deprecated | Superseded
**Deciders:** [who made the decision]

### Context

[What is the issue? Why do we need to make a decision?]

### Options Considered

| Option | Pros | Cons |
|--------|------|------|
| Option 1 | ... | ... |
| Option 2 | ... | ... |

### Decision

[What was decided]

### Rationale

[Why this option was chosen]

### Consequences

[What are the implications? What changes?]
```
