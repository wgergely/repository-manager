# Per-Tool Versioning Research

> **Purpose:** Document versioning models, update mechanisms, and pinning strategies for each tool.
> **Last Updated:** 2026-01-29
> **Status:** Research Draft - Requires Per-Tool Verification

## Overview

Tools in our matrix have fundamentally different versioning approaches:

| Category | Versioning Model | Tools |
|----------|-----------------|-------|
| Traditional SemVer | Explicit versions, manual updates | JetBrains, VS Code |
| Package Manager | SemVer via npm/pip, explicit install | Claude CLI, Aider, Gemini CLI |
| Auto-Updating | Silent background updates | Cursor, Zed, Windsurf |
| Extension-Bound | Version tied to host IDE | Cline, Roo, Copilot, Amazon Q |

---

## Versioning Categories

### Traditional SemVer

**Characteristics:**
- Explicit version numbers (e.g., 2024.1.3)
- Manual update process
- Changelogs published
- Old versions downloadable
- Breaking changes documented

**Docker Strategy:**
- Pin to specific version in Dockerfile
- Update manually when needed
- Test against N recent versions for compatibility matrix

### Package Manager SemVer

**Characteristics:**
- Published to npm/pip/cargo
- SemVer with constraints (^1.0.0, ~1.0.0)
- Lock files (package-lock.json, requirements.txt)
- Easy to pin and upgrade

**Docker Strategy:**
- Pin exact version in Dockerfile (`npm install pkg@1.2.3`)
- Use lock files in container
- Automated dependency update PRs (Dependabot/Renovate)

### Auto-Updating

**Characteristics:**
- Updates happen silently in background
- No version selection in UI
- Difficult or impossible to pin
- May change behavior without notice
- Often no public changelog

**Docker Strategy:**
- Snapshot images with build date tag
- Accept that version cannot be pinned
- Rebuild images regularly to get updates
- Consider disabling auto-update if possible

### Extension-Bound

**Characteristics:**
- Version tied to host IDE version
- Published to extension marketplaces
- May have separate extension version
- Update policy controlled by IDE settings

**Docker Strategy:**
- Pin both IDE version and extension version
- Use `code --install-extension ext@version` where supported
- Document compatible IDE + extension version pairs

---

## Per-Tool Versioning Details

### VS Code

| Property | Value |
|----------|-------|
| **Versioning Model** | Traditional SemVer |
| **Version Format** | Major.Minor.Patch (e.g., 1.85.2) |
| **Release Cadence** | Monthly (stable), Weekly (insiders) |
| **Changelog** | https://code.visualstudio.com/updates |
| **Version Command** | `code --version` |
| **Download Archive** | Yes - all versions available |
| **Pin Strategy** | Download specific .deb/tar.gz |

**Version Pinning:**
```dockerfile
# Pin to specific version
ARG VSCODE_VERSION=1.85.2
RUN wget "https://update.code.visualstudio.com/${VSCODE_VERSION}/linux-deb-x64/stable" -O code.deb \
    && dpkg -i code.deb
```

**Update Policy:**
- Stable monthly releases
- Check monthly for updates
- Test with new version before updating Docker image

---

### Cursor

| Property | Value |
|----------|-------|
| **Versioning Model** | Auto-Updating |
| **Version Format** | Unknown - research needed |
| **Release Cadence** | Continuous |
| **Changelog** | No public changelog found |
| **Version Command** | Research needed |
| **Download Archive** | Research needed |
| **Pin Strategy** | Snapshot by date |

**Research Needed:**
- [ ] How to check Cursor version
- [ ] Is there a version history/changelog?
- [ ] Can auto-update be disabled?
- [ ] Are old versions downloadable?

**Provisional Docker Strategy:**
```dockerfile
# Tag images by build date
# repo-test/cursor:2026-01-29
RUN wget https://download.cursor.sh/linux/appImage/x64 -O /opt/cursor.AppImage
LABEL build.date="2026-01-29"
```

**Update Policy:**
- Rebuild weekly to capture updates
- Tag images with build date
- Maintain recent snapshots for regression testing

---

### Zed

| Property | Value |
|----------|-------|
| **Versioning Model** | Auto-Updating (with channels) |
| **Version Format** | v0.XXX.Y (e.g., v0.123.4) |
| **Release Cadence** | Frequent (Preview channel daily) |
| **Changelog** | https://zed.dev/releases |
| **Version Command** | `zed --version` |
| **Download Archive** | Yes - GitHub releases |
| **Pin Strategy** | Download specific release from GitHub |

**Version Pinning:**
```dockerfile
ARG ZED_VERSION=0.123.4
RUN wget "https://github.com/zed-industries/zed/releases/download/v${ZED_VERSION}/zed-linux-x86_64.tar.gz" \
    && tar -xzf zed-linux-x86_64.tar.gz
```

**Update Policy:**
- Check GitHub releases monthly
- Test new versions before updating
- Zed is rapidly evolving - expect breaking changes

**Research Needed:**
- [ ] How to disable auto-update in Zed
- [ ] Stable vs Preview channel behavior

---

### JetBrains (IntelliJ IDEA)

| Property | Value |
|----------|-------|
| **Versioning Model** | Traditional SemVer |
| **Version Format** | YYYY.N.P (e.g., 2024.1.3) |
| **Release Cadence** | ~3 major releases per year |
| **Changelog** | https://www.jetbrains.com/idea/whatsnew/ |
| **Version Command** | `./idea.sh --version` |
| **Download Archive** | Yes - all versions available |
| **Pin Strategy** | Download specific archive |

**Version Pinning:**
```dockerfile
ARG IDEA_VERSION=2024.1
ARG IDEA_BUILD=241.14494.240
RUN wget "https://download.jetbrains.com/idea/ideaIC-${IDEA_VERSION}.tar.gz" \
    && tar -xzf ideaIC-${IDEA_VERSION}.tar.gz
```

**Archive URL Pattern:**
- Community: `https://download.jetbrains.com/idea/ideaIC-YYYY.N.tar.gz`
- Ultimate: `https://download.jetbrains.com/idea/ideaIU-YYYY.N.tar.gz`

**Update Policy:**
- Update quarterly with major releases
- LTS versions available for enterprises
- Test AI Assistant plugin compatibility

---

### Windsurf

| Property | Value |
|----------|-------|
| **Versioning Model** | Unknown - research needed |
| **Version Format** | Unknown |
| **Release Cadence** | Unknown |
| **Changelog** | Unknown |
| **Version Command** | Unknown |
| **Download Archive** | Unknown |
| **Pin Strategy** | Unknown |

**Research Needed:**
- [ ] Find Windsurf documentation
- [ ] Determine if VS Code fork (may inherit versioning)
- [ ] Find download/installation method
- [ ] Check for version command

---

### Antigravity

| Property | Value |
|----------|-------|
| **Versioning Model** | Unknown |
| **All Properties** | Requires full research |

**Research Needed:**
- [ ] What is Antigravity? (product, company, open source?)
- [ ] Where to download?
- [ ] How versioning works?

---

### Claude CLI

| Property | Value |
|----------|-------|
| **Versioning Model** | Package Manager (npm) |
| **Version Format** | SemVer (e.g., 1.0.0) |
| **Release Cadence** | Frequent (weekly-ish) |
| **Changelog** | npm page / GitHub releases |
| **Version Command** | `claude --version` |
| **Package** | `@anthropic-ai/claude-code` |
| **Pin Strategy** | npm version pinning |

**Version Pinning:**
```dockerfile
# Pin exact version
RUN npm install -g @anthropic-ai/claude-code@1.0.0

# Or use package.json with lock file
COPY package.json package-lock.json ./
RUN npm ci
```

**Check Latest Version:**
```bash
npm view @anthropic-ai/claude-code version
```

**Update Policy:**
- Monitor npm for new releases
- Test updates in isolation before deploying
- Use Dependabot/Renovate for automated update PRs

---

### Aider

| Property | Value |
|----------|-------|
| **Versioning Model** | Package Manager (pip) |
| **Version Format** | SemVer (e.g., 0.50.0) |
| **Release Cadence** | Very frequent (multiple per week) |
| **Changelog** | https://aider.chat/HISTORY.html |
| **Version Command** | `aider --version` |
| **Package** | `aider-chat` |
| **Pin Strategy** | pip version pinning |

**Version Pinning:**
```dockerfile
# Pin exact version
RUN pip install aider-chat==0.50.0

# Or use requirements.txt
COPY requirements.txt ./
RUN pip install -r requirements.txt
```

**Check Latest Version:**
```bash
pip index versions aider-chat
```

**Update Policy:**
- Aider updates very frequently
- Pin to stable version, update monthly
- Check HISTORY.html for breaking changes

---

### Gemini CLI

| Property | Value |
|----------|-------|
| **Versioning Model** | Package Manager (npm) - assumed |
| **Version Format** | Unknown |
| **Release Cadence** | Unknown |
| **Changelog** | Unknown |
| **Version Command** | Unknown |
| **Package** | Unknown - research needed |
| **Pin Strategy** | npm version pinning (assumed) |

**Research Needed:**
- [ ] Confirm npm package name
- [ ] Verify installation method
- [ ] Check version command
- [ ] Find documentation

---

### Cline (VS Code Extension)

| Property | Value |
|----------|-------|
| **Versioning Model** | Extension-Bound |
| **Version Format** | SemVer (e.g., 2.1.0) |
| **Release Cadence** | Frequent |
| **Changelog** | VS Code Marketplace |
| **Version Command** | `code --list-extensions --show-versions` |
| **Extension ID** | `saoudrizwan.claude-dev` |
| **Pin Strategy** | Install specific version |

**Version Pinning:**
```dockerfile
# Install specific version
RUN code --install-extension saoudrizwan.claude-dev@2.1.0 --force
```

**Note:** VS Code extension version pinning via CLI may not work for all extensions. Research needed.

**Update Policy:**
- Check marketplace monthly
- Test with new versions before updating

---

### Roo Code (VS Code Extension)

| Property | Value |
|----------|-------|
| **Versioning Model** | Extension-Bound |
| **Version Format** | SemVer |
| **Extension ID** | Research needed |
| **Other Properties** | Similar to Cline |

**Research Needed:**
- [ ] Confirm VS Code marketplace extension ID
- [ ] Verify version pinning works

---

### GitHub Copilot (VS Code Extension)

| Property | Value |
|----------|-------|
| **Versioning Model** | Extension-Bound |
| **Version Format** | SemVer |
| **Extension ID** | `GitHub.copilot` |
| **Other Properties** | Similar to Cline |

**Version Pinning:**
```dockerfile
RUN code --install-extension GitHub.copilot@1.100.0 --force
```

---

### Amazon Q (VS Code Extension)

| Property | Value |
|----------|-------|
| **Versioning Model** | Extension-Bound |
| **Version Format** | SemVer |
| **Extension ID** | `AmazonWebServices.amazon-q-vscode` |
| **Other Properties** | Similar to Cline |

**Version Pinning:**
```dockerfile
RUN code --install-extension AmazonWebServices.amazon-q-vscode@1.0.0 --force
```

---

## Compatibility Matrix Template

Track tested version combinations:

| Tool | Minimum | Latest Tested | Current Stable | Status |
|------|---------|--------------|----------------|--------|
| VS Code | 1.80.0 | 1.85.2 | 1.85.2 | ✅ |
| Cursor | - | 2026-01-29 | Auto-updating | ⚠️ |
| Zed | 0.120.0 | 0.123.4 | 0.123.4 | ⚠️ |
| JetBrains | 2023.3 | 2024.1.3 | 2024.1.3 | ✅ |
| Claude CLI | 0.1.0 | 1.0.0 | 1.0.0 | ✅ |
| Aider | 0.45.0 | 0.50.0 | 0.50.0 | ✅ |

**Status Legend:**
- ✅ Fully tested, version pinnable
- ⚠️ Testing possible, version control limited
- ❌ Cannot test or version control
- ❓ Research needed

---

## Version Update Workflow

### For Package Manager Tools

```bash
# Check for updates
npm outdated @anthropic-ai/claude-code
pip list --outdated | grep aider

# Update Dockerfile
# ARG CLAUDE_VERSION=1.0.0 → ARG CLAUDE_VERSION=1.1.0

# Rebuild and test
docker compose build claude
docker compose run --rm claude claude --version
```

### For Auto-Updating Tools

```bash
# Rebuild to get latest
docker compose build --no-cache cursor

# Tag with date
docker tag repo-test/cursor:latest repo-test/cursor:2026-01-29

# Test
docker compose run --rm cursor
```

### For Traditional Tools

```bash
# Check release notes
# Update version ARG
# Rebuild and test
```

---

## Automation Opportunities

### Dependabot (npm/pip packages)

```yaml
# .github/dependabot.yml
version: 2
updates:
  - package-ecosystem: "npm"
    directory: "/docker/cli/claude"
    schedule:
      interval: "weekly"

  - package-ecosystem: "pip"
    directory: "/docker/cli/aider"
    schedule:
      interval: "weekly"
```

### Renovate (more flexible)

```json
{
  "extends": ["config:base"],
  "packageRules": [
    {
      "matchPackageNames": ["@anthropic-ai/claude-code"],
      "groupName": "Claude CLI"
    },
    {
      "matchPackageNames": ["aider-chat"],
      "groupName": "Aider"
    }
  ]
}
```

### Version Check Script

```bash
#!/bin/bash
# check-versions.sh

echo "Checking tool versions..."

# npm packages
echo "Claude CLI:"
npm view @anthropic-ai/claude-code version

# pip packages
echo "Aider:"
pip index versions aider-chat | head -1

# GitHub releases
echo "Zed:"
curl -s https://api.github.com/repos/zed-industries/zed/releases/latest | jq -r .tag_name

echo "VS Code:"
curl -s https://api.github.com/repos/microsoft/vscode/releases/latest | jq -r .tag_name
```

---

## Research Checklist

### Immediate

- [ ] Verify Claude CLI npm package name and current version
- [ ] Verify Gemini CLI npm package name
- [ ] Find Windsurf installation/versioning info
- [ ] Research Antigravity completely
- [ ] Test VS Code extension version pinning via CLI

### Per-Tool Documentation

For each tool, document:
1. Exact version command
2. How to check for updates
3. How to download specific versions
4. Breaking change history (if any)

---

## References

- [npm package versioning](https://docs.npmjs.com/about-semantic-versioning)
- [pip version specifiers](https://peps.python.org/pep-0440/)
- [VS Code release notes](https://code.visualstudio.com/updates)
- [JetBrains release archive](https://www.jetbrains.com/idea/download/other.html)
- [Zed releases](https://github.com/zed-industries/zed/releases)
- [Aider changelog](https://aider.chat/HISTORY.html)
