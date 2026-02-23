# Rust CLI Tool Distribution Best Practices (2026)

**Date:** 2026-02-18
**Author:** ResearchAgent3
**Purpose:** Comprehensive research on distribution options for Rust CLI tools, with applicability to Repository Manager (v0.1.0 Alpha)

---

## Executive Summary

Repository Manager currently only supports installation via `cargo install --path crates/repo-cli` or building from source. This severely limits adoption, particularly among non-Rust developers who are the likely primary audience for a developer tool. This report surveys the full spectrum of distribution options available to Rust CLI tools in 2026, with concrete recommendations for each channel.

The modern gold standard for Rust CLI distribution is a multi-channel approach:
1. **cargo-dist** for automated cross-platform binary release pipelines
2. **GitHub Releases** with pre-built binaries as the authoritative download source
3. **Homebrew tap** for macOS/Linux users
4. **Winget** for Windows users
5. **cargo install / cargo-binstall** for Rust developers
6. **crates.io** for library crates; optional for CLI tools

---

## 1. Package Managers

### 1.1 Homebrew (macOS / Linux)

Homebrew is the dominant package manager for macOS and is also widely used on Linux. It is the first installation channel most macOS developers try.

**Two Homebrew distribution paths exist:**

**A. Homebrew Core (homebrew/core)**
- The official Homebrew formula repository maintained by the Homebrew project
- Requires the tool to meet usage and quality thresholds (generally >100 downloads/day)
- Maintained by community volunteers after initial submission
- Popular Rust tools here: `ripgrep`, `bat`, `fd`, `starship`, `zoxide`, `delta`
- Submission process: Open a PR to `homebrew/homebrew-core` with a formula file
- Formula points to a versioned tarball of source code (builds from source by default) or can use pre-built bottles

**B. Homebrew Tap (Third-Party)**
- Any GitHub repo named `username/homebrew-tap` or `org/homebrew-*`
- Users install with: `brew tap your-org/tap && brew install your-org/tap/your-tool`
- Full control over formula and release timing
- cargo-dist automates tap management: it generates and updates Homebrew formulae pointing to GitHub Release binaries
- **Recommended first step** for new tools before reaching homebrew-core thresholds

**Key insight from cargo-dist v0.22+:** cargo-dist automatically generates a Homebrew formula that points to pre-built binaries, translates SPDX license expressions to Homebrew's license DSL, and updates your tap on each release.

### 1.2 Chocolatey (Windows)

- Windows package manager with a community repository at `chocolatey.org`
- Packages are Chocolatey-specific NuSpec manifests + PowerShell install scripts
- Widely used in enterprise Windows environments
- Manual submission and maintenance required
- Can reference GitHub Release binaries in the install script

### 1.3 Scoop (Windows)

- User-space Windows package manager (no admin required)
- JSON manifest files in a "bucket" (GitHub repository)
- Popular among developers; simpler than Chocolatey for maintainers
- Create a custom "bucket" at `github.com/your-org/scoop-bucket`
- `scoop bucket add your-bucket https://github.com/your-org/scoop-bucket`
- Manifests point to GitHub Release zip/exe files with SHA256 checksums

### 1.4 Winget (Windows)

- Microsoft's official Windows Package Manager, pre-installed on Windows 11
- Package manifests in the `winget-pkgs` GitHub repository maintained by Microsoft
- YAML manifests that point to installer URLs (can be GitHub Release binaries)
- SHA256 checksums are auto-generated or can be specified
- **winget-releaser**: A popular GitHub Action that automates opening PRs to `winget-pkgs` on each release
- Growing rapidly in adoption as Windows 11 market share increases
- Suitable for production-ready tools; maintainers are responsible for keeping manifests current

### 1.5 APT / RPM (Debian/Ubuntu and Red Hat/Fedora)

**Options:**
- **packagecloud.io**: Hosted APT/RPM repository service (paid tier for private; free tier limited)
- **Personal Package Archive (PPA)**: Ubuntu-specific via Launchpad (requires `.deb` packaging)
- **Copr**: Fedora's equivalent of PPA for RPM packages
- **GitHub Releases with .deb/.rpm**: Many tools publish `.deb` and `.rpm` as GitHub Release assets
- Tools like `cargo-deb` and `cargo-generate-rpm` can create these from Cargo.toml metadata

**Complexity level:** High. Most smaller tools skip native package formats and rely on other channels.

### 1.6 Nix / NixOS

- Nix is a purely functional package manager available on Linux and macOS
- NixOS is a Linux distribution built entirely on Nix
- Packages live in `nixpkgs` (the largest package collection in the world)
- Nix flakes provide reproducible, pinned builds for development environments
- **Distribution path:** Submit a package derivation to `NixOS/nixpkgs` via PR
- The `buildRustPackage` nixpkgs function handles Rust projects natively
- Alternative tools: `naersk`, `crane`, `crate2nix` for workspace builds
- Growing adoption among power users and DevOps engineers

### 1.7 MacPorts

- Older macOS package manager, smaller community than Homebrew
- Less relevant for new tools but still used in some organizations
- Not recommended as a primary channel

---

## 2. Binary Distribution

### 2.1 GitHub Releases

GitHub Releases is the **canonical binary distribution channel** for Rust CLI tools. All other package managers (Homebrew, Scoop, Winget, cargo-binstall) ultimately point to GitHub Releases.

**Standard practice:**
- Create a git tag (`v0.1.0`, following semver)
- GitHub Actions CI builds binaries for all target platforms
- Binaries are uploaded as release assets (`.tar.gz` for Unix, `.zip` for Windows)
- SHA256 checksums are published alongside binaries
- Release notes summarize changes (generated from CHANGELOG or git log)

**Typical release asset naming convention:**
```
repo-cli-v0.1.0-x86_64-unknown-linux-musl.tar.gz
repo-cli-v0.1.0-x86_64-apple-darwin.tar.gz
repo-cli-v0.1.0-aarch64-apple-darwin.tar.gz
repo-cli-v0.1.0-x86_64-pc-windows-msvc.zip
```

### 2.2 cargo-binstall

`cargo-binstall` is a drop-in replacement for `cargo install` that downloads pre-built binaries instead of compiling from source.

**How it works:**
1. Looks up the crate on crates.io to find the source repository URL
2. Searches the repository's GitHub Releases for matching binary assets
3. Falls back to `quickinstall` (a third-party binary hosting service)
4. Falls back to `cargo install` (compile from source) as last resort

**For maintainers:** Add metadata to `Cargo.toml` to hint binstall where to find binaries:
```toml
[package.metadata.binstall]
pkg-url = "{ repo }/releases/download/v{ version }/{ name }-v{ version }-{ target }.tar.gz"
bin-dir = "{ name }-v{ version }-{ target }/{ bin }"
pkg-fmt = "tgz"
```

**Signature verification:** cargo-binstall supports specifying a signing public key for package verification.

**User adoption:** cargo-binstall is widely adopted in the Rust developer community and increasingly the preferred way for Rust developers to install tools without long compile times.

### 2.3 cargo-dist

`cargo-dist` (by axodotdev) is the most comprehensive automated solution for Rust binary distribution as of 2026.

**Features (v0.30.x, stable 2025):**
- Generates complete GitHub Actions release workflows
- Builds binaries for all configured target triples
- Creates installer scripts:
  - Shell installer (`install.sh`) for Linux/macOS
  - PowerShell installer (`install.ps1`) for Windows
- Manages Homebrew tap formula updates
- Embeds checksum verification in installers (v0.26+)
- Supports custom signing keys
- Only publishes to Homebrew tap on stable releases (configurable)
- Self-hosting: just push a git tag to trigger a release

**Setup:**
```bash
cargo install cargo-dist
cargo dist init   # Interactive setup, writes to Cargo.toml [workspace.metadata.dist]
cargo dist plan   # Preview what will happen
git tag v0.1.0 && git push --tags  # Triggers release workflow
```

**Configuration in `Cargo.toml`:**
```toml
[workspace.metadata.dist]
cargo-dist-version = "0.30.3"
ci = ["github"]
installers = ["shell", "powershell", "homebrew"]
targets = ["x86_64-unknown-linux-musl", "aarch64-apple-darwin", "x86_64-apple-darwin", "x86_64-pc-windows-msvc"]
tap = "your-org/homebrew-tap"
publish-jobs = ["homebrew"]
```

**Current state (2026):** cargo-dist is the recommended approach for new Rust CLI tools. The axodotdev team maintains it actively and it powers many production tools.

### 2.4 cargo-release

`cargo-release` (crate-ci) handles the mechanical details of preparing a release:
- Version bumping in `Cargo.toml` files
- CHANGELOG heading updates
- Git tagging
- crates.io publishing

Often used in combination with `cargo-dist`: cargo-release handles version/changelog management and creates the tag; cargo-dist's GitHub Actions pick up the tag and build binaries.

---

## 3. Container Distribution (Docker)

### 3.1 Patterns for CLI Tools in Docker

Docker is less common as a primary distribution channel for CLI tools but can be valuable for:
- CI/CD pipelines that run the tool in containers
- Users who prefer containerized tools
- Server-side use cases

**Typical Dockerfile pattern for Rust CLI:**
```dockerfile
# Build stage
FROM rust:1.85-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin repo-cli

# Runtime stage - minimal image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libgit2-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/repo-cli /usr/local/bin/repo-cli
ENTRYPOINT ["repo-cli"]
```

**For fully static binaries (musl):**
```dockerfile
FROM rust:1.85-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl
FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/repo-cli /repo-cli
ENTRYPOINT ["/repo-cli"]
```

### 3.2 Docker Hub Publishing

- Free Docker Hub accounts allow unlimited public images
- GitHub Actions can automatically push to Docker Hub on release
- Multi-architecture images (linux/amd64, linux/arm64) can be published using `docker buildx`
- Image naming: `your-org/repo-cli:latest`, `your-org/repo-cli:0.1.0`

### 3.3 GitHub Container Registry (ghcr.io)

- Tightly integrated with GitHub repositories
- Free for public repositories
- Authentication uses existing GitHub tokens
- Often preferred over Docker Hub for open source projects

---

## 4. Installation Scripts

### 4.1 The curl-pipe-bash Pattern

The `curl | sh` pattern is widely used despite security concerns:

```bash
# As used by rustup, cargo-dist, mise, etc.
curl --proto '=https' --tlsv1.2 -LsSf https://your-site.com/install.sh | sh
```

**Security considerations:**
- Risk: Server could serve different content to piped vs. browser requests
- Risk: Partial execution if connection is interrupted mid-stream
- Mitigation: Use `--proto '=https' --tlsv1.2` to enforce TLS
- Mitigation: Structure script so functions are defined first and called at end (prevents partial execution)
- Mitigation: Checksum verification of downloaded binary inside the script
- Many major tools use this pattern (rustup, mise, deno, bun) and it is considered acceptable with proper mitigations

**cargo-dist's approach (best practice):** The generated shell installer:
1. Downloads the appropriate binary tarball for the detected platform
2. Verifies the SHA256 checksum before extraction (embedded in script as of v0.26)
3. Installs to `~/.cargo/bin` or a configurable location

### 4.2 Platform-Specific Installers

**Windows (PowerShell):**
```powershell
irm https://your-site.com/install.ps1 | iex
```

**macOS via Homebrew (preferred over curl|sh for macOS):**
```bash
brew install your-org/tap/repo-cli
```

---

## 5. CI/CD for Releases

### 5.1 GitHub Actions Matrix Builds

The standard approach for cross-platform Rust binary builds:

```yaml
name: Release
on:
  push:
    tags: ['v*']

jobs:
  build:
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-musl
            os: ubuntu-latest
          - target: aarch64-apple-darwin
            os: macos-latest
          - target: x86_64-apple-darwin
            os: macos-13  # Intel runner
          - target: x86_64-pc-windows-msvc
            os: windows-latest
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - run: cargo build --release --target ${{ matrix.target }}
      - uses: taiki-e/upload-rust-binary-action@v1
        with:
          bin: repo-cli
          target: ${{ matrix.target }}
          token: ${{ secrets.GITHUB_TOKEN }}
```

### 5.2 Key GitHub Actions for Rust Releases

| Action | Purpose |
|--------|---------|
| `dtolnay/rust-toolchain` | Install specific Rust toolchain |
| `taiki-e/upload-rust-binary-action` | Build and upload to GitHub Releases |
| `cargo-dist` generated workflow | Complete automated pipeline |
| `softprops/action-gh-release` | Create GitHub releases with assets |
| `crazy-max/ghaction-docker-buildx` | Multi-arch Docker builds |
| `mindsers/changelog-reader-action` | Extract changelog for release notes |

### 5.3 cargo-dist Generated Workflow

cargo-dist generates a complete `.github/workflows/release.yml` that handles:
- Building for all configured targets
- Creating the GitHub Release
- Uploading binary assets and checksums
- Updating the Homebrew tap formula
- Running the shell/PowerShell installer generation

This is the recommended approach as it requires zero manual workflow maintenance.

---

## 6. Case Studies: How Successful Rust CLI Tools Distribute

### 6.1 ripgrep (BurntSushi)

- **GitHub Releases:** Pre-built binaries for Linux (gnu + musl), macOS (x86_64 + aarch64), Windows (MSVC + GNU)
- **Homebrew Core:** `brew install ripgrep`
- **Chocolatey:** `choco install ripgrep`
- **Scoop:** `scoop install ripgrep`
- **Winget:** `winget install BurntSushi.ripgrep.MSVC`
- **cargo install:** `cargo install ripgrep`
- **cargo-binstall:** Supported via GitHub Releases detection
- **APT/RPM:** Available in Debian/Ubuntu repos, Fedora COPR
- **Key lesson:** Comprehensive coverage of all major package managers achieved over time; GitHub Releases are the foundation

### 6.2 bat (sharkdp)

- **GitHub Releases:** Binaries + `.deb`/`.rpm` packages as release assets
- **Homebrew Core:** `brew install bat`
- **Chocolatey, Scoop, Winget:** All supported
- **Key lesson:** Publishing `.deb` and `.rpm` as GitHub Release assets is a lightweight way to support Linux package manager users without maintaining a full repository

### 6.3 starship (starship-rs)

- **GitHub Releases:** Pre-built binaries
- **Homebrew Core:** `brew install starship`
- **cargo install:** `cargo install starship`
- **Winget, Scoop, Chocolatey:** All supported
- **Install script:** `curl -sS https://starship.rs/install.sh | sh`
- **Key lesson:** Install script + Homebrew + cargo-install covers 95% of the audience; Winget/Scoop/Chocolatey are additive

### 6.4 mise (jdx)

- **Install script:** Primary recommended install method: `curl https://mise.run | sh`
- **Homebrew:** `brew install mise`
- **cargo-binstall:** `cargo binstall mise`
- **Winget:** `winget install mise`
- **APT repository:** Maintains its own APT repository
- **Key lesson:** For tools targeting developers who don't necessarily have Rust installed, an install script as the primary method lowers friction significantly

### 6.5 zoxide (ajeetdsouza)

- **GitHub Releases:** Automated via cargo-dist-style workflow
- **Homebrew:** `brew install zoxide`
- **cargo install:** `cargo install zoxide`
- **Winget, Scoop, Chocolatey, APT:** All supported
- **Install script:** `curl -sSfL https://raw.githubusercontent.com/ajeetdsouza/zoxide/main/install.sh | sh`
- **Key lesson:** Even small tools can achieve comprehensive distribution coverage; each channel is relatively low maintenance once set up

### 6.6 just (casey)

- **GitHub Releases:** Pre-built binaries
- **Homebrew:** `brew install just`
- **cargo install:** `cargo install just`
- **cargo-binstall:** `cargo binstall just`
- **Winget, Scoop, Chocolatey, APT, RPM:** All major channels covered
- **Key lesson:** `just` targets a very broad audience (anyone who builds software); comprehensive coverage is essential

---

## 7. crates.io Publishing

### 7.1 Requirements for Publishing

To publish to crates.io, a crate must have:
- `name` - unique crate name
- `version` - semver version string
- `license` or `license-file` - SPDX expression (e.g., `MIT OR Apache-2.0`)
- `description` - short description
- `edition` - Rust edition

Recommended metadata:
- `readme` - path to README.md (displayed on crates.io)
- `homepage` - project URL
- `repository` - source repository URL
- `keywords` - up to 5 keywords for discoverability
- `categories` - up to 5 categories from crates.io taxonomy
- `documentation` - docs.rs URL (auto-generated if omitted)

Size limit: 10MB per `.crate` file.

### 7.2 Workspace Publishing

As of Cargo 1.90 (stable September 2025), workspace-level publishing is stable:
```bash
cargo publish --workspace   # Publishes all publishable crates in dependency order
cargo publish --dry-run     # Preview first
```

Individual crates can opt out of publishing:
```toml
[package]
publish = false  # For internal/library crates not meant for crates.io
```

### 7.3 Publishing Strategy for Repository Manager

Repository Manager has a workspace structure. Recommended approach:
- `repo-cli` - publish as a binary crate (the CLI tool users install)
- `repo-core` / `repo-agent` - publish as library crates if they have public API value
- Internal implementation crates - set `publish = false`

### 7.4 Trusted Publishing (no API tokens)

crates.io now supports Trusted Publishing for GitHub Actions (and GitLab CI), allowing crate publication without storing API tokens as secrets. Configure in crates.io profile settings, then:
```yaml
- uses: Swatinem/rust-cache@v2
- run: cargo publish
  env:
    CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
```

Or with OIDC-based trusted publishing (no token needed in newer versions).

### 7.5 cargo-release Integration

`cargo-release` automates the release process:
```bash
cargo release patch   # Bump patch version, update CHANGELOG, tag, push, publish
cargo release minor   # Same for minor version
```

Configure in `.config/release.toml` or `Cargo.toml`:
```toml
[workspace.metadata.release]
shared-version = true   # All crates share the same version
```

---

## 8. cargo-dist: Current State (2026)

### 8.1 Overview

cargo-dist v0.30.x is the current stable release series (latest: v0.30.3, December 2025). It is the most complete automated solution for Rust CLI distribution.

### 8.2 What cargo-dist Provides

1. **GitHub Actions workflow generation** - Complete release pipeline
2. **Multi-platform binary builds** - All major target triples
3. **Shell installer** - `install.sh` with embedded checksums
4. **PowerShell installer** - `install.ps1` for Windows
5. **Homebrew tap management** - Auto-generates and updates formula
6. **Checksum generation** - SHA256 for all artifacts
7. **Checksum verification** - Embedded in installers (v0.26+)
8. **Release notes integration** - From CHANGELOG.md or git commits

### 8.3 What cargo-dist Does NOT Currently Provide (as of 2026)

- Native Winget manifest automation (winget-releaser is a separate tool)
- Scoop manifest automation
- Chocolatey package creation
- APT/RPM repository management
- Docker image publishing

These gaps can be filled with companion GitHub Actions:
- `vedantmgoyal9/winget-releaser` for Winget
- Manual Scoop/Chocolatey manifests

### 8.4 Recommended cargo-dist Configuration for Repository Manager

```toml
[workspace.metadata.dist]
cargo-dist-version = "0.30.3"
ci = ["github"]
installers = ["shell", "powershell", "homebrew"]
targets = [
  "x86_64-unknown-linux-musl",
  "aarch64-unknown-linux-musl",
  "x86_64-apple-darwin",
  "aarch64-apple-darwin",
  "x86_64-pc-windows-msvc",
]
tap = "your-org/homebrew-tap"
publish-jobs = ["homebrew"]
publish-prereleases = false
pr-run-mode = "plan"  # Check releases on PRs without actually releasing
```

---

## 9. Cross-Compilation

### 9.1 The Cross-Compilation Challenge

Rust cross-compilation is straightforward for pure Rust code but becomes complex when C dependencies are involved. libgit2 (used by Repository Manager) is a C library, making this non-trivial.

### 9.2 Tools

**`cross` (cross-rs)**
- Docker-based cross-compilation for Rust
- Provides pre-configured images for many target triples
- Usage: `cross build --release --target aarch64-unknown-linux-gnu`
- Handles C dependencies automatically via Docker environments
- Best for Linux targets; limited macOS/Windows cross-compilation

**`cargo-zigbuild` (zig cc)**
- Uses Zig compiler as the C/C++ linker for cross-compilation
- Solves glibc versioning issues by targeting specific glibc versions
- Usage: `cargo zigbuild --target aarch64-unknown-linux-gnu.2.17`
- Better cross-compilation story than `cross` in many cases
- Supports macOS targets from Linux CI

**Native runners (GitHub Actions)**
- Build on the actual target platform (macos-latest for macOS)
- No cross-compilation required; most reliable
- More expensive (macOS runners cost more on GitHub Actions)
- **Recommended for production releases**

### 9.3 Target Triples for Major Platforms

| Platform | Target Triple | Notes |
|----------|---------------|-------|
| Linux x86_64 (glibc) | `x86_64-unknown-linux-gnu` | Most Linux distros |
| Linux x86_64 (static) | `x86_64-unknown-linux-musl` | Alpine, containers, maximum compatibility |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | AWS Graviton, Raspberry Pi 4+ |
| Linux ARM64 (static) | `aarch64-unknown-linux-musl` | Static ARM64 |
| macOS x86_64 | `x86_64-apple-darwin` | Intel Macs |
| macOS ARM64 | `aarch64-apple-darwin` | Apple Silicon (M1/M2/M3/M4) |
| Windows x86_64 MSVC | `x86_64-pc-windows-msvc` | Standard Windows |
| Windows x86_64 GNU | `x86_64-pc-windows-gnu` | MinGW-based |
| Windows ARM64 | `aarch64-pc-windows-msvc` | Surface Pro X, newer ARM Windows |

### 9.4 libgit2 Cross-Compilation Considerations

Repository Manager depends on `git2` (which binds to libgit2). Options:
1. **Use `cross`** - Docker images include libgit2 headers for many targets
2. **Vendor/bundle libgit2** - The `git2` crate can statically link libgit2 with the `vendored` feature
3. **Use native runners** - Avoid cross-compilation entirely
4. **Switch to pure-Rust git** - Libraries like `gix` (gitoxide) avoid the libgit2 C dependency entirely

The `vendored` feature is the most portable approach for binary distribution:
```toml
[dependencies]
git2 = { version = "0.20", features = ["vendored"] }
```

This statically links libgit2 into the binary, eliminating the runtime C library dependency.

---

## 10. Recommended Distribution Roadmap for Repository Manager

### Phase 1: Foundation (Pre-Release / Alpha)

1. **Ensure crates.io metadata** is complete in all `Cargo.toml` files
2. **Set up cargo-dist** with initial configuration
3. **Set up cargo-release** for version management
4. **Create a GitHub Release** manually for v0.1.0 with binaries built from CI
5. **Publish to crates.io** (at minimum `repo-cli`)
6. **Enable cargo-binstall support** via Cargo.toml metadata
7. **Enable `vendored` feature** for libgit2 to support static binaries

### Phase 2: Accessible Installation (Beta)

1. **Automate releases** with cargo-dist's generated GitHub Actions workflow
2. **Create a Homebrew tap** (`your-org/homebrew-tap`) managed by cargo-dist
3. **Create a Scoop bucket** for Windows users
4. **Add install script** to project website/README
5. **Submit Winget manifest** to `winget-pkgs`

### Phase 3: Broad Distribution (v1.0)

1. **Submit to homebrew-core** once usage thresholds are met
2. **Submit to nixpkgs** for NixOS users
3. **Publish `.deb` and `.rpm`** as GitHub Release assets
4. **Docker image** on ghcr.io for CI/CD use cases
5. **Chocolatey package** for enterprise Windows environments

### Priority Matrix

| Channel | Audience | Effort | Priority |
|---------|----------|--------|----------|
| cargo install | Rust developers | Already done | Done |
| GitHub Releases | All | Low (with cargo-dist) | High |
| cargo-binstall | Rust developers | Low | High |
| Homebrew tap | macOS/Linux devs | Low (cargo-dist) | High |
| Winget | Windows devs | Medium | High |
| Scoop | Windows devs | Low | Medium |
| crates.io | Rust developers | Low | Medium |
| Docker/ghcr.io | CI/DevOps | Medium | Medium |
| homebrew-core | macOS/Linux devs | Low-Medium | Low (until v1.0) |
| nixpkgs | NixOS users | Medium-High | Low |
| APT/RPM repos | Linux users | High | Low |
| Chocolatey | Windows enterprise | Medium | Low |

---

## Sources

- [Packaging and distributing a Rust tool - CLI Book](https://rust-cli.github.io/book/tutorial/packaging.html)
- [cargo-dist GitHub](https://github.com/axodotdev/cargo-dist)
- [cargo-dist Releases (v0.26.0, v0.30.x)](https://github.com/axodotdev/cargo-dist/releases)
- [cargo-binstall - Binary installation for Rust](https://github.com/cargo-bins/cargo-binstall)
- [Publishing on crates.io - The Cargo Book](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [crates.io development update - Rust Blog](https://blog.rust-lang.org/2026/01/21/crates-io-development-update/)
- [Publish all your crates everywhere all at once - Tweag](https://www.tweag.io/blog/2025-07-10-cargo-package-workspace/)
- [cargo-zigbuild - Compile with zig as linker](https://github.com/rust-cross/cargo-zigbuild)
- [Zig Makes Rust Cross-compilation Just Work](https://actually.fyi/posts/zig-makes-rust-cross-compilation-just-work/)
- [taiki-e/upload-rust-binary-action](https://github.com/taiki-e/upload-rust-binary-action)
- [crate-ci/cargo-release](https://github.com/crate-ci/cargo-release)
- [Packaging a Rust CLI Tool - Chris Woodruff](https://www.woodruff.dev/packaging-and-releasing-a-rust-cli-tool/)
- [Ivan Carvalho - Packaging Rust CLI to many places](https://ivaniscoding.github.io/posts/rustpackaging1/)
- [Installing mise](https://mise.jdx.dev/installing-mise.html)
- [zoxide GitHub](https://github.com/ajeetdsouza/zoxide)
- [bat GitHub](https://github.com/sharkdp/bat)
- [ripgrep GitHub](https://github.com/BurntSushi/ripgrep)
- [Rust on NixOS Wiki](https://nixos.wiki/wiki/Rust)
- [Deploy Rust Binaries with GitHub Actions - dzfrias](https://dzfrias.dev/blog/deploy-rust-cross-platform-github-actions/)
- [The Dangers of curl | bash](https://lukespademan.com/blog/the-dangers-of-curlbash/)
- [Distributing your own scripts via Homebrew](https://justin.searls.co/posts/how-to-distribute-your-own-scripts-via-homebrew/)
