# Tool Matrix - Source of Truth

> **Purpose:** Canonical reference for all tools supported by Repository Manager.
> **Last Updated:** 2026-01-29
> **Status:** Research Draft

## Overview

Repository Manager supports **13 tools** across **4 categories**. This matrix documents installation methods, configuration paths, versioning models, and Docker feasibility for each tool.

## Summary Table

| Tool | Category | Headless | Docker Feasibility | API Required |
|------|----------|----------|-------------------|--------------|
| VS Code | IDE | Yes | Trivial | No |
| Cursor | IDE | Research | Complex | Anthropic |
| Zed | IDE | Research | Complex | Anthropic |
| JetBrains | IDE | Yes | Moderate | Provider-dependent |
| Windsurf | IDE | Research | Complex | Provider-dependent |
| Antigravity | IDE | Research | Unknown | Unknown |
| Claude | CLI Agent | Yes | Trivial | Anthropic |
| Aider | CLI Agent | Yes | Trivial | Configurable |
| Gemini CLI | CLI Agent | Yes | Trivial | Google |
| Cline | Autonomous | Yes* | Moderate | Configurable |
| Roo Code | Autonomous | Yes* | Moderate | Configurable |
| GitHub Copilot | Copilot | Yes* | Moderate | GitHub |
| Amazon Q | Copilot | Yes* | Moderate | AWS |

*Requires VS Code headless mode

---

## Category: IDEs (6 tools)

### VS Code

| Property | Value |
|----------|-------|
| **Slug** | `vscode` |
| **Description** | Microsoft's open-source code editor |
| **Config Path** | `.vscode/settings.json` |
| **Config Format** | JSON |
| **Additional Paths** | `.vscode/extensions.json`, `.vscode/tasks.json` |
| **Capabilities** | Custom instructions (via settings), extensions |
| **Installation** | apt, snap, .deb package, or official binary |
| **Headless Mode** | Yes - `code --headless`, `code-server` |
| **Version Command** | `code --version` |
| **Versioning Model** | Monthly releases, semver (e.g., 1.85.0) |
| **Pin Strategy** | Download specific version from releases |
| **Docker Feasibility** | **Trivial** - well-documented headless support |
| **API Required** | No (editor itself needs no API) |

**Installation Commands:**
```bash
# Ubuntu/Debian
curl -fsSL https://packages.microsoft.com/keys/microsoft.asc | gpg --dearmor > packages.microsoft.gpg
install -o root -g root -m 644 packages.microsoft.gpg /usr/share/keyrings/
echo "deb [arch=amd64 signed-by=/usr/share/keyrings/packages.microsoft.gpg] https://packages.microsoft.com/repos/vscode stable main" > /etc/apt/sources.list.d/vscode.list
apt update && apt install code

# Or direct download
wget "https://code.visualstudio.com/sha/download?build=stable&os=linux-deb-x64" -O code.deb
dpkg -i code.deb
```

**Headless Testing:**
```bash
# Install extension headlessly
code --install-extension ms-python.python --force

# Run extension tests
code --extensionTestsPath=/path/to/tests
```

---

### Cursor

| Property | Value |
|----------|-------|
| **Slug** | `cursor` |
| **Description** | AI-first code editor (VS Code fork) |
| **Config Path** | `.cursorrules` |
| **Config Format** | Plain text |
| **Additional Paths** | `.cursor/rules/` (directory) |
| **Capabilities** | Custom instructions, rules directory |
| **Installation** | AppImage, .deb, or binary download |
| **Headless Mode** | **Research needed** - may inherit VS Code headless |
| **Version Command** | Unknown - research needed |
| **Versioning Model** | Auto-updating, no public version history |
| **Pin Strategy** | Snapshot container images by date |
| **Docker Feasibility** | **Complex** - GUI app, needs Xvfb research |
| **API Required** | Anthropic API key (built-in) |

**Research Questions:**
- [ ] Does Cursor expose a CLI similar to VS Code's `code` command?
- [ ] Can `.cursorrules` loading be tested without full GUI?
- [ ] Is there an official Docker image or headless mode?
- [ ] What's the update mechanism - can it be disabled?

**Installation Commands (to verify):**
```bash
# Download AppImage
wget https://download.cursor.sh/linux/appImage/x64 -O cursor.AppImage
chmod +x cursor.AppImage

# Or .deb
wget https://download.cursor.sh/linux/deb/x64 -O cursor.deb
dpkg -i cursor.deb
```

---

### Zed

| Property | Value |
|----------|-------|
| **Slug** | `zed` |
| **Description** | High-performance GPU-accelerated editor |
| **Config Path** | `.zed/settings.json`, `.rules` |
| **Config Format** | JSON, Plain text |
| **Additional Paths** | None documented |
| **Capabilities** | Custom instructions, MCP servers |
| **Installation** | Official script, binary, or Homebrew |
| **Headless Mode** | **Research needed** - GPU requirement may block |
| **Version Command** | `zed --version` |
| **Versioning Model** | Auto-updating, Preview/Stable channels |
| **Pin Strategy** | Download specific release from GitHub |
| **Docker Feasibility** | **Complex** - GPU-accelerated, may not run headless |
| **API Required** | Anthropic (for AI features) |

**Research Questions:**
- [ ] Can Zed run without GPU acceleration?
- [ ] Is there a headless or server mode?
- [ ] Can config loading be tested via CLI?
- [ ] What happens when Zed launches without display?

**Installation Commands:**
```bash
# Official script
curl -f https://zed.dev/install.sh | sh

# Or download binary
wget https://github.com/zed-industries/zed/releases/download/v0.xxx/zed-linux-x86_64.tar.gz
tar -xzf zed-linux-x86_64.tar.gz
```

---

### JetBrains (AI Assistant)

| Property | Value |
|----------|-------|
| **Slug** | `jetbrains` |
| **Description** | JetBrains IDEs with AI Assistant plugin |
| **Config Path** | `.aiassistant/rules/` |
| **Config Format** | Markdown directory |
| **Additional Paths** | `.aiignore` |
| **Capabilities** | Custom instructions, MCP servers, rules directory |
| **Installation** | JetBrains Toolbox or direct download |
| **Headless Mode** | Yes - IntelliJ Headless SDK for plugin testing |
| **Version Command** | `idea.sh --version` (or specific IDE) |
| **Versioning Model** | Traditional semver, annual major releases |
| **Pin Strategy** | Download specific version archive |
| **Docker Feasibility** | **Moderate** - headless SDK exists but complex |
| **API Required** | JetBrains AI subscription or provider API |

**Research Questions:**
- [ ] Which IDEs support AI Assistant? (IntelliJ, PyCharm, WebStorm, etc.)
- [ ] How does the headless SDK work for plugin testing?
- [ ] Can we test config loading without full IDE?
- [ ] Memory/resource requirements for containerized JetBrains?

**Installation Commands:**
```bash
# Download IntelliJ Community (example)
wget https://download.jetbrains.com/idea/ideaIC-2024.1.tar.gz
tar -xzf ideaIC-2024.1.tar.gz

# Headless mode
./idea.sh inspect /project /output -v2
```

---

### Windsurf

| Property | Value |
|----------|-------|
| **Slug** | `windsurf` |
| **Description** | AI-native code editor by Codeium |
| **Config Path** | `.windsurfrules` |
| **Config Format** | Plain text |
| **Additional Paths** | None documented |
| **Capabilities** | Custom instructions |
| **Installation** | Binary download |
| **Headless Mode** | **Research needed** - likely VS Code fork |
| **Version Command** | Unknown - research needed |
| **Versioning Model** | Auto-updating |
| **Pin Strategy** | Snapshot container images by date |
| **Docker Feasibility** | **Complex** - GUI app, needs Xvfb research |
| **API Required** | Codeium account (built-in) |

**Research Questions:**
- [ ] Is Windsurf a VS Code fork? (affects headless strategy)
- [ ] Official installation method for Linux?
- [ ] Can `.windsurfrules` be tested via CLI?
- [ ] What's the update mechanism?

---

### Antigravity

| Property | Value |
|----------|-------|
| **Slug** | `antigravity` |
| **Description** | AI coding assistant (emerging tool) |
| **Config Path** | `.agent/rules.md` |
| **Config Format** | Markdown |
| **Additional Paths** | `.agent/` directory |
| **Capabilities** | Custom instructions, rules directory |
| **Installation** | **Unknown - research needed** |
| **Headless Mode** | **Unknown** |
| **Version Command** | Unknown |
| **Versioning Model** | Unknown |
| **Pin Strategy** | Unknown |
| **Docker Feasibility** | **Unknown** - requires research |
| **API Required** | Unknown |

**Research Questions:**
- [ ] What is Antigravity exactly? (company, product, open source?)
- [ ] Where is it installed from?
- [ ] What platforms are supported?
- [ ] Is there any documentation?

---

## Category: CLI Agents (3 tools)

### Claude (Claude Code)

| Property | Value |
|----------|-------|
| **Slug** | `claude` |
| **Description** | Anthropic's official CLI for Claude |
| **Config Path** | `CLAUDE.md` |
| **Config Format** | Markdown |
| **Additional Paths** | `.claude/`, `.claude/rules/`, `.claude/settings.json` |
| **Capabilities** | Custom instructions, MCP servers, rules directory |
| **Installation** | npm: `@anthropic-ai/claude-code` |
| **Headless Mode** | Yes - CLI native |
| **Version Command** | `claude --version` |
| **Versioning Model** | npm semver, frequent updates |
| **Pin Strategy** | `npm install @anthropic-ai/claude-code@x.y.z` |
| **Docker Feasibility** | **Trivial** - pure CLI |
| **API Required** | `ANTHROPIC_API_KEY` |

**Installation Commands:**
```bash
npm install -g @anthropic-ai/claude-code
# or
npx @anthropic-ai/claude-code
```

**Test Commands:**
```bash
# Verify installation
claude --version

# Test with mock/real API
ANTHROPIC_API_KEY=xxx claude --print-system-prompt
```

---

### Aider

| Property | Value |
|----------|-------|
| **Slug** | `aider` |
| **Description** | AI pair programming in terminal |
| **Config Path** | `.aider.conf.yml` |
| **Config Format** | YAML |
| **Additional Paths** | `CONVENTIONS.md`, `.aider/` |
| **Capabilities** | Custom instructions, multi-model support |
| **Installation** | pip/pipx: `aider-chat` |
| **Headless Mode** | Yes - CLI native |
| **Version Command** | `aider --version` |
| **Versioning Model** | pip semver, very active development |
| **Pin Strategy** | `pip install aider-chat==x.y.z` |
| **Docker Feasibility** | **Trivial** - pure CLI |
| **API Required** | Configurable (Anthropic, OpenAI, Ollama, etc.) |

**Installation Commands:**
```bash
pip install aider-chat
# or
pipx install aider-chat
```

**Test Commands:**
```bash
# Verify installation
aider --version

# Test config loading
aider --show-config
```

---

### Gemini CLI

| Property | Value |
|----------|-------|
| **Slug** | `gemini` |
| **Description** | Google's AI coding assistant CLI |
| **Config Path** | `GEMINI.md` |
| **Config Format** | Markdown |
| **Additional Paths** | None documented |
| **Capabilities** | Custom instructions |
| **Installation** | npm (package name TBD - research needed) |
| **Headless Mode** | Yes - CLI native |
| **Version Command** | `gemini --version` (TBD) |
| **Versioning Model** | npm semver (assumed) |
| **Pin Strategy** | npm version pinning |
| **Docker Feasibility** | **Trivial** - pure CLI |
| **API Required** | Google Cloud credentials |

**Research Questions:**
- [ ] What is the exact npm package name?
- [ ] Is it publicly available or requires Google Cloud setup?
- [ ] What authentication method does it use?

---

## Category: Autonomous Agents (2 tools)

*These are VS Code extensions that act as autonomous coding agents.*

### Cline

| Property | Value |
|----------|-------|
| **Slug** | `cline` |
| **Description** | Autonomous coding agent (VS Code extension) |
| **Config Path** | `.clinerules` |
| **Config Format** | Plain text or directory |
| **Additional Paths** | `.clinerules/` (directory) |
| **Capabilities** | Custom instructions, rules directory |
| **Installation** | VS Code extension: `saoudrizwan.claude-dev` |
| **Headless Mode** | Via VS Code headless |
| **Version Command** | `code --list-extensions --show-versions` |
| **Versioning Model** | VS Code marketplace, semver |
| **Pin Strategy** | Install specific extension version |
| **Docker Feasibility** | **Moderate** - requires VS Code headless |
| **API Required** | Configurable (Anthropic, OpenAI, etc.) |

**Installation Commands:**
```bash
code --install-extension saoudrizwan.claude-dev
```

---

### Roo Code

| Property | Value |
|----------|-------|
| **Slug** | `roo` |
| **Description** | Fork of Cline with additional features |
| **Config Path** | `.roo/rules/` |
| **Config Format** | Markdown directory |
| **Additional Paths** | `.roomodes` (YAML/JSON) |
| **Capabilities** | Custom instructions, MCP servers, rules directory |
| **Installation** | VS Code extension (marketplace ID TBD) |
| **Headless Mode** | Via VS Code headless |
| **Version Command** | `code --list-extensions --show-versions` |
| **Versioning Model** | VS Code marketplace, semver |
| **Pin Strategy** | Install specific extension version |
| **Docker Feasibility** | **Moderate** - requires VS Code headless |
| **API Required** | Configurable (Anthropic, OpenAI, etc.) |

**Research Questions:**
- [ ] What is the VS Code marketplace extension ID?
- [ ] How does `.roomodes` affect behavior?

---

## Category: Copilot-style Assistants (2 tools)

### GitHub Copilot

| Property | Value |
|----------|-------|
| **Slug** | `copilot` |
| **Description** | GitHub's AI pair programmer |
| **Config Path** | `.github/copilot-instructions.md` |
| **Config Format** | Markdown |
| **Additional Paths** | `.github/instructions/` (directory) |
| **Capabilities** | Custom instructions, rules directory |
| **Installation** | VS Code/JetBrains extension |
| **Headless Mode** | Via host IDE headless |
| **Version Command** | Via extension version |
| **Versioning Model** | Extension marketplace versions |
| **Pin Strategy** | Install specific extension version |
| **Docker Feasibility** | **Moderate** - requires IDE headless |
| **API Required** | GitHub Copilot subscription + `GITHUB_TOKEN` |

**Installation Commands:**
```bash
# VS Code
code --install-extension GitHub.copilot
```

---

### Amazon Q

| Property | Value |
|----------|-------|
| **Slug** | `amazonq` |
| **Description** | AWS AI coding assistant |
| **Config Path** | `.amazonq/rules/` |
| **Config Format** | Markdown directory |
| **Additional Paths** | None documented |
| **Capabilities** | Custom instructions, rules directory |
| **Installation** | VS Code/JetBrains extension |
| **Headless Mode** | Via host IDE headless |
| **Version Command** | Via extension version |
| **Versioning Model** | Extension marketplace versions |
| **Pin Strategy** | Install specific extension version |
| **Docker Feasibility** | **Moderate** - requires IDE headless |
| **API Required** | AWS credentials (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`) |

**Installation Commands:**
```bash
# VS Code
code --install-extension AmazonWebServices.amazon-q-vscode
```

---

## Docker Feasibility Summary

| Feasibility | Tools | Notes |
|-------------|-------|-------|
| **Trivial** | Claude, Aider, Gemini CLI, VS Code | Pure CLI or well-documented headless |
| **Moderate** | JetBrains, Cline, Roo, Copilot, Amazon Q | Requires headless IDE setup |
| **Complex** | Cursor, Zed, Windsurf | GUI apps, need Xvfb/VNC research |
| **Unknown** | Antigravity | Requires full research |

---

## Version Matrix Template

For certification runs, track tested versions:

| Tool | Minimum Tested | Maximum Tested | Last Certified | Notes |
|------|---------------|----------------|----------------|-------|
| Claude CLI | TBD | TBD | TBD | |
| Aider | TBD | TBD | TBD | |
| VS Code | TBD | TBD | TBD | |
| ... | ... | ... | ... | |

---

## Appendix: Configuration Format Reference

| Format | Tools Using | Parser |
|--------|------------|--------|
| JSON | VS Code, Zed | `serde_json` |
| YAML | Aider, Roo (.roomodes) | `serde_yaml` |
| TOML | (internal config) | `toml` |
| Markdown | Claude, Copilot, JetBrains, Amazon Q | `tree-sitter-md` |
| Plain Text | Cursor, Cline, Windsurf, Zed (.rules) | Direct read |

---

## Research Tracking

- [ ] Cursor: headless mode, CLI, installation verification
- [ ] Zed: GPU requirements, headless feasibility
- [ ] Windsurf: VS Code fork status, installation
- [ ] Antigravity: basic product research
- [ ] Gemini CLI: npm package name, auth method
- [ ] Roo Code: marketplace extension ID
- [ ] JetBrains: headless SDK documentation review
