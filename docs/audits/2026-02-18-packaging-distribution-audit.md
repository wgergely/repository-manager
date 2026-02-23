# Packaging and Distribution Readiness Audit

**Date:** 2026-02-18
**Auditor:** MarketingAgent3
**Project:** Repository Manager v0.1.0 (Alpha)
**Scope:** CI/CD, crates.io readiness, Docker, build dependencies, versioning, security

---

## Executive Summary

Repository Manager is **not ready for public distribution** in its current state. The project has a solid internal testing infrastructure (Docker-based integration tests, comprehensive test scripts) but is missing nearly every prerequisite for a public release: no release pipeline, placeholder repository URL, no CHANGELOG, no release tags, no SECURITY.md, no deny.toml, and no `[profile.release]` optimizations. Each of these is a concrete blocker.

---

## 1. CI/CD

### What Exists

A single workflow: `.github/workflows/docker-integration.yml`

This workflow:
- Triggers on push to `main` or `registry-architecture`, and on PRs affecting `docker/`, `crates/`, or `test-fixtures/`
- Builds a layered Docker image hierarchy (base, CLI tools, VS Code extensions, repo-manager)
- Runs smoke tests and integration tests against a mock API (WireMock)
- Uses GHA artifact caching and matrix builds for tools (claude, aider, gemini, cursor) and VS Code extensions (cline, roo)

### What Is Missing

| Missing Item | Impact |
|---|---|
| Unit/integration test CI (cargo test) | No automated Rust test gating on PRs |
| Clippy/fmt CI check | Code quality not enforced in CI |
| Release pipeline | No automated binary builds on tag push |
| Cross-platform builds (macOS, Windows, Linux) | No prebuilt binaries for users |
| Docker image publishing step | Images are built but never pushed to a registry |
| `cargo publish` workflow | No automated crates.io publishing |
| Dependabot / Renovate config | No automated dependency update PRs |

**Critical gap:** There is no `cargo test` step in any workflow. The entire CI is Docker-focused. A developer submitting a PR that breaks unit tests would not be caught by CI.

---

## 2. crates.io Readiness

### Workspace Cargo.toml (`Y:\code\repository-manager-worktrees\main\Cargo.toml`)

```toml
[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"
repository = "https://github.com/user/repository-manager"
```

**Issues:**
- `repository` URL is a **placeholder** (`user/repository-manager`). This must be the real GitHub URL before publishing.
- No `homepage` field at workspace level.
- No `documentation` field.
- No `authors` field (optional but recommended).
- No `keywords` or `categories` fields (required for good crates.io discoverability).
- No `rust-version` (MSRV) specified.

### Per-Crate Metadata Audit

All 11 publishable crates inherit `version`, `edition`, `license` from the workspace. All have `description` fields. Summary:

| Crate | Description | `repository` | `homepage` | `documentation` | `keywords` | `categories` |
|---|---|---|---|---|---|---|
| repo-agent | Yes (inherited workspace URL - placeholder) | Placeholder | Missing | Missing | Missing | Missing |
| repo-blocks | Yes | Placeholder | Missing | Missing | Missing | Missing |
| repo-cli | Yes | Placeholder | Missing | Missing | Missing | Missing |
| repo-content | Yes | Placeholder | Missing | Missing | Missing | Missing |
| repo-core | Yes | Placeholder | Missing | Missing | Missing | Missing |
| repo-fs | Yes | Placeholder | Missing | Missing | Missing | Missing |
| repo-git | Yes | Placeholder | Missing | Missing | Missing | Missing |
| repo-meta | Yes | Placeholder | Missing | Missing | Missing | Missing |
| repo-mcp | Yes | Placeholder | Missing | Missing | Missing | Missing |
| repo-presets | Yes | Placeholder | Missing | Missing | Missing | Missing |
| repo-tools | Yes | Placeholder | Missing | Missing | Missing | Missing |

**Every crate is missing:** `homepage`, `documentation`, `keywords`, `categories`.

### Publishing Order (Dependency Graph)

Crates must be published in topological order because each references sibling crates via `path = "../..."` dependencies. Publishing order:

1. `repo-fs` (no internal dependencies)
2. `repo-content` (no internal dependencies)
3. `repo-agent` (no internal dependencies)
4. `repo-blocks` (depends on: repo-fs)
5. `repo-meta` (depends on: repo-fs)
6. `repo-git` (depends on: repo-fs)
7. `repo-tools` (depends on: repo-fs, repo-meta, repo-blocks)
8. `repo-presets` (depends on: repo-fs, repo-meta)
9. `repo-core` (depends on: repo-fs, repo-git, repo-meta, repo-tools, repo-presets, repo-content)
10. `repo-cli` (depends on: all above)
11. `repo-mcp` (depends on: repo-agent, repo-core, repo-fs, repo-meta, repo-presets)

**Blocker:** Path dependencies (`path = "../repo-fs"`) must be replaced with version dependencies before `cargo publish`. Each `path` dependency must be replaced with `{ version = "0.1.0" }` at publish time. This requires either manual editing or a release automation tool (e.g., `cargo-release`).

**Also:** `integration-tests` workspace member has no `publish = false` set, which will cause `cargo publish --workspace` to fail or attempt to publish a test-only crate.

---

## 3. Docker

### Images Built

- `repo-test/base` - Ubuntu 22.04 + Node.js 20 + Python 3.12 + Rust stable
- `repo-test/cli-base` - CLI tool base
- `repo-test/vscode-base` - VS Code headless base
- `repo-test/claude`, `repo-test/aider`, `repo-test/gemini`, `repo-test/cursor` - Tool images
- `repo-test/cline`, `repo-test/roo` - VS Code extension images
- `repo-test/repo-manager` - The main repo-manager image

### Repo-Manager Dockerfile Issues

```dockerfile
FROM repo-test/base:latest         # Depends on a test image, not a lean production base
COPY crates /workspace/crates      # Copies entire crates directory including source
RUN cd crates/repo-cli && cargo build --release  # Builds inside the container - no multi-stage build
ENV PATH="/workspace/crates/target/release:${PATH}"  # Incorrect PATH: target is at /workspace/target, not /workspace/crates/target
```

**Critical bugs:**
- `PATH` is wrong: `cargo build --release` in `crates/repo-cli` outputs to `/workspace/target/release`, not `/workspace/crates/target/release`. The `RUN repo --help` step uses `|| echo "..."` to silently swallow this error.
- No multi-stage build: the image ships the entire Rust toolchain (~1.5GB), all source code, and all build artifacts. A minimal production image should use a multi-stage build: build in a Rust image, copy only the binary to a slim runtime image.
- Based on `repo-test/base` which is intended for testing (includes Node.js, Python, build tools). A production image should use `debian:bookworm-slim` or `alpine` with just the binary.

### Publishing

**No Docker images are published anywhere.** The CI workflow builds images but never pushes them to Docker Hub, GHCR, or any other registry. There are no `push: true` steps, no registry login steps, and no image tags with version numbers.

### Documentation

`docker/README.md` documents the testing infrastructure well but does not cover:
- How to pull and run a published image (because none are published)
- Production deployment considerations
- Image size or system requirements

---

## 4. Build Dependencies

### System Dependencies Required

| Dependency | Required By | Build Method |
|---|---|---|
| `libgit2` + `libssl` + `pkg-config` + `cmake` | `git2` crate (via `libgit2-sys`) | Compiled from source by default (bundled) |
| C compiler (`cc`/`gcc`) | `libgit2-sys` build script | Must be present on build host |
| `pkg-config` | `openssl-sys` | Must be present on build host |

### git2 / libgit2-sys Notes

The `git2` crate (v0.20) uses `libgit2-sys` which by default **bundles** libgit2 and compiles it from source. This means:
- Users building from source need: a C compiler, cmake, pkg-config, libssl-dev
- Binary distributors get a statically linked binary (no libgit2 runtime dependency)
- Build times are significantly longer (~2-3 min extra) due to the C compilation
- The `LIBGIT2_SYS_USE_PKG_CONFIG=1` env var can use a system libgit2 instead

The base Docker image correctly installs `build-essential`, `pkg-config`, and `libssl-dev`.

**Recommendation:** For pre-built binary distribution, static linking (current default) is correct. Document the build requirements for source installs.

### No `[profile.release]` Optimizations

The workspace `Cargo.toml` has **no** `[profile.release]` section. The default release profile uses:
- `opt-level = 3` (fine)
- `debug = false` (fine)
- `lto = false` (missed optimization opportunity)
- `codegen-units = 16` (missed optimization opportunity)
- `strip = false` (binaries include debug symbols, bloating size)

For a CLI tool targeting distribution, the following optimizations are standard:

```toml
[profile.release]
lto = true
codegen-units = 1
strip = true
```

This typically reduces binary size by 30-50% and improves runtime performance.

---

## 5. Versioning Strategy

### Current State

- Version: `0.1.0` across all crates (workspace-synchronized)
- No git tags exist (confirmed: `git tag --list` returns empty)
- No `CHANGELOG.md` or `CHANGELOG` file
- No release notes
- No versioning documentation

### What Is Missing

| Missing Item | Impact |
|---|---|
| Git release tags (e.g., `v0.1.0`) | No way for users/tools to track releases |
| CHANGELOG.md | Users cannot see what changed between versions |
| Versioning policy (semver?) | Contributors don't know version bump rules |
| Release automation (cargo-release, release-plz) | Manual multi-crate version bumps are error-prone |

The project uses Cargo 2024 edition and workspace version inheritance, which is a good foundation. But without tags and a CHANGELOG, there is no release history.

---

## 6. Security

### SECURITY.md

No `SECURITY.md` file exists. There is no documented process for:
- Reporting vulnerabilities
- Security response timelines
- Supported versions

### deny.toml / cargo-deny

No `deny.toml` file exists. `cargo-deny` has not been configured. Without it:
- No automated scanning for known vulnerabilities in dependencies (RustSec advisory database)
- No license compliance checking
- No duplicate dependency detection

### Supply Chain Assessment

Dependencies are generally well-regarded crates from established authors. Key observations:
- `git2` (0.20) - actively maintained, widely used
- `serde` (1.0), `tokio` (1.42), `clap` (4) - foundational, low risk
- `serde_yaml` (0.9) - uses `unsafe-libyaml` internally; monitor for advisories
- `backoff` (0.4.0) - older version, check for advisories
- `dirs` (5.0) - standard platform dirs, low risk
- No `cargo-deny` advisories check has been run

**Recommendation:** Run `cargo deny check` and add a `deny.toml` before any public release.

---

## 7. Complete List of Blockers to Public Release

### P0 - Must Fix Before Any Public Release

1. **Placeholder repository URL** - `https://github.com/user/repository-manager` must be replaced with the real URL in `Cargo.toml` before publishing to crates.io or linking anywhere publicly.

2. **No release pipeline** - No CI workflow builds and publishes binaries on git tag push. Users have no way to install without building from source.

3. **No Docker image publishing** - Images are built in CI but never pushed to any registry. Users cannot `docker pull repo-manager`.

4. **PATH bug in repo-manager Dockerfile** - `ENV PATH="/workspace/crates/target/release:${PATH}"` is incorrect; the binary will not be found. The `|| echo` masks this silently.

5. **Path dependencies prevent `cargo publish`** - All inter-crate dependencies use `path = "../..."`. These must be converted to version references before any crate can be published to crates.io.

6. **No `[profile.release]` section** - Missing LTO, strip settings mean distributed binaries are large and suboptimally compiled.

7. **`integration-tests` crate not marked `publish = false`** - Will fail or pollute crates.io if workspace publish is attempted.

### P1 - Should Fix Before Beta/GA

8. **No CHANGELOG.md** - Users and downstream tools cannot track what changed.

9. **No git release tags** - No version history exists in git.

10. **No `cargo-deny` / `deny.toml`** - No supply chain security scanning.

11. **No SECURITY.md** - No vulnerability disclosure process.

12. **Missing crates.io metadata** - `homepage`, `documentation`, `keywords`, `categories` are absent from all 11 crates, hurting discoverability.

13. **No `rust-version` (MSRV)** - Users don't know the minimum Rust version required.

14. **No unit test CI** - `cargo test` is not run in any CI pipeline.

15. **No Clippy/fmt CI enforcement** - Code quality not gated on PRs.

16. **No multi-stage Dockerfile** - Production image ships ~1.5GB of toolchain and source instead of just the binary.

17. **No cross-platform binary builds** - No prebuilt binaries for macOS (arm64/x86_64), Linux (gnu/musl), or Windows.

18. **No `cargo-release` or `release-plz` configuration** - Manual version bumping of 11 crates is error-prone.

### P2 - Nice to Have

19. **No Homebrew formula / Scoop manifest / apt repository** - Common install paths for CLI tools are absent.

20. **No shell completion packaging** - `clap_complete` is a dependency but no install-time completion setup exists.

21. **No Dependabot/Renovate configuration** - Dependency updates require manual work.

---

## Summary Table

| Area | Status | Severity |
|---|---|---|
| CI - Unit/Integration tests | Missing entirely | P0 |
| CI - Release pipeline | Missing entirely | P0 |
| Repository URL | Placeholder | P0 |
| Docker publishing | Not configured | P0 |
| Dockerfile correctness | PATH bug, no multi-stage | P0 |
| crates.io path deps | Blocks publishing | P0 |
| Release profile | No optimizations | P0 |
| CHANGELOG | Missing | P1 |
| Release tags | None exist | P1 |
| cargo-deny | Not configured | P1 |
| SECURITY.md | Missing | P1 |
| crates.io metadata | Incomplete | P1 |
| MSRV declaration | Missing | P1 |
| Unit test CI | Missing | P1 |
| Multi-stage Dockerfile | Missing | P1 |
| Cross-platform builds | Missing | P1 |

**Overall verdict: Alpha-internal only. Estimated effort to reach publishable state: 2-4 weeks of focused distribution engineering work.**
