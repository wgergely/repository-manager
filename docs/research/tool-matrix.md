# Tool Matrix - Source of Truth

> **Purpose:** Canonical reference for all tools supported by Repository Manager.
> **Last Updated:** 2026-01-29
> **Status:** Research Complete (6/7 tools researched, JetBrains pending)

## Overview

Repository Manager supports **13 tools** across **4 categories**. This matrix documents installation methods, configuration paths, versioning models, and Docker feasibility for each tool.

## Summary Table

| Tool | Category | Headless | Docker Feasibility | API Required |
|------|----------|----------|-------------------|--------------|
| VS Code | IDE | Yes | Trivial | No |
| Cursor | IDE | **Yes (CLI)** | **Moderate** | Anthropic |
| Zed | IDE | **No** | **Difficult** | Anthropic |
| JetBrains | IDE | Yes | Moderate | Provider-dependent |
| Windsurf | IDE | **No** | **Complex** | Codeium |
| Antigravity | IDE | **Unknown** | **Complex** | **Gemini (Google)** |
| Claude | CLI Agent | Yes | Trivial | Anthropic |
| Aider | CLI Agent | Yes | Trivial | Configurable |
| Gemini CLI | CLI Agent | Yes | Trivial | **Google (free tier)** |
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
| **Description** | AI-first code editor (VS Code fork) with dedicated CLI |
| **Config Path** | `.cursorrules` |
| **Config Format** | Plain text |
| **Additional Paths** | `.cursor/rules/` (directory) |
| **Capabilities** | Custom instructions, rules directory |
| **Installation** | `curl https://cursor.com/install -fsS \| bash` |
| **Headless Mode** | **Yes** - CLI with print mode (`-p`) for non-interactive use |
| **Version Command** | Unknown (CLI is separate from editor) |
| **Versioning Model** | Auto-updating editor, CLI in beta |
| **Pin Strategy** | Snapshot container images by date |
| **Docker Feasibility** | **Moderate** - CLI usable, GUI needs Xvfb |
| **API Required** | Anthropic API key (built-in or user-provided) |

**RESEARCH COMPLETE:**
- [x] Does Cursor expose a CLI? **Yes - `agent` command with full CLI**
- [x] Can `.cursorrules` loading be tested? **Yes via CLI**
- [x] Headless mode? **Yes - print mode (`-p`) for scripts/CI**
- [ ] Update mechanism - can it be disabled? Still unknown

**CLI Commands:**
```bash
# Installation
curl https://cursor.com/install -fsS | bash          # Linux/macOS
irm 'https://cursor.com/install?win32=true' | iex    # Windows

# Core usage
agent                        # Start interactive session
agent "[prompt]"             # Begin with initial instruction
agent ls                     # List previous conversations
agent resume                 # Resume latest chat
agent --resume="chat-id"     # Resume specific session

# Modes
agent --mode=plan            # Plan mode (design before coding)
agent --mode=ask             # Ask mode (explore without changes)
agent -p "[prompt]"          # Print mode (non-interactive for CI)
agent -p --force "[prompt]"  # Print mode with auto file changes
```

**Key Features (Jan 2026):**
- Cloud Agents: Prepend `&` to push conversation to cloud
- Model selection: `--model` flag
- Non-interactive: `-p` flag for scripts/CI pipelines

**Sources:** [Cursor CLI Docs](https://cursor.com/docs/cli/overview), [Cursor Changelog](https://cursor.com/changelog/cli-jan-16-2026)

---

### Zed

| Property | Value |
|----------|-------|
| **Slug** | `zed` |
| **Description** | High-performance GPU-accelerated editor (Vulkan) |
| **Config Path** | `.zed/settings.json`, `.rules` |
| **Config Format** | JSON, Plain text |
| **Additional Paths** | None documented |
| **Capabilities** | Custom instructions, MCP servers |
| **Installation** | `curl -f https://zed.dev/install.sh \| sh` |
| **Headless Mode** | **No** - requires Vulkan GPU, no headless mode |
| **Version Command** | `zed --version` |
| **Versioning Model** | Auto-updating, Preview/Stable channels |
| **Pin Strategy** | Download specific release from GitHub |
| **Docker Feasibility** | **Difficult** - Vulkan GPU required, no software fallback |
| **API Required** | Anthropic (for AI features) |

**RESEARCH COMPLETE:**
- [x] Can Zed run without GPU? **No - requires Vulkan. Will fail with "NoSupportedDeviceFound"**
- [x] Is there a headless mode? **No - not documented**
- [x] Config loading via CLI? **No - editor must launch**
- [x] Without display? **Will fail to open window**

**GPU Requirements:**
- Requires Vulkan-compatible GPU
- Error without GPU: `Zed failed to open a window: NoSupportedDeviceFound`
- Software rendering (llvmpipe) may work but has issues
- amdvlk driver problematic - use vulkan-radeon instead

**Environment Variables:**
```bash
ZED_CHANNEL=preview|stable     # Build channel
ZED_DEVICE_ID=0x2484           # Force specific GPU (hex)
ZED_PATH_SAMPLE_COUNT=0        # Fix AMD GPU crashes
DRI_PRIME=1                    # Force discrete GPU
MESA_VK_DEVICE_SELECT=list     # List available GPUs
```

**Installation Commands:**
```bash
# Official script (stable)
curl -f https://zed.dev/install.sh | sh

# Preview channel
curl -f https://zed.dev/install.sh | ZED_CHANNEL=preview sh

# Manual download
wget https://github.com/zed-industries/zed/releases/download/vX.Y.Z/zed-linux-x86_64.tar.gz
```

**Docker Strategy:** Config-file-only testing recommended. Full integration testing may require GPU passthrough or acceptance of limited coverage.

**Sources:** [Zed Linux Docs](https://zed.dev/docs/linux)

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
| **Description** | AI-native IDE by Codeium with Cascade agent |
| **Config Path** | `.windsurfrules` |
| **Config Format** | Plain text |
| **Additional Paths** | None documented |
| **Capabilities** | Custom instructions, Cascade AI agent |
| **Installation** | apt repository (Debian/Ubuntu) or rpm (Fedora/CentOS) |
| **Headless Mode** | **No** - no CLI documented |
| **Version Command** | Unknown |
| **Versioning Model** | Auto-updating via repository |
| **Pin Strategy** | Snapshot container images by date |
| **Docker Feasibility** | **Complex** - GUI app, needs Xvfb |
| **API Required** | Codeium account (free tier available) |

**RESEARCH COMPLETE:**
- [x] Is Windsurf a VS Code fork? **Likely yes** (similar architecture)
- [x] Official Linux installation? **Yes - apt/rpm repositories**
- [x] CLI mode? **No - no CLI documented**
- [x] Update mechanism? **Auto-updating via system package manager**

**Installation Commands (Debian/Ubuntu):**
```bash
curl -fsSL "https://windsurf-stable.codeiumdata.com/wVxQEIWkwPUEAGf3/windsurf.gpg" | sudo gpg --dearmor -o /usr/share/keyrings/windsurf-stable-archive-keyring.gpg
echo "deb [signed-by=/usr/share/keyrings/windsurf-stable-archive-keyring.gpg arch=amd64] https://windsurf-stable.codeiumdata.com/wVxQEIWkwPUEAGf3/apt stable main" | sudo tee /etc/apt/sources.list.d/windsurf.list > /dev/null
sudo apt update && sudo apt install windsurf
```

**Installation Commands (Fedora/CentOS 8+):**
```bash
sudo rpm --import https://windsurf-stable.codeiumdata.com/wVxQEIWkwPUEAGf3/yum/RPM-GPG-KEY-windsurf
echo -e "[windsurf]\nname=Windsurf Repository\nbaseurl=https://windsurf-stable.codeiumdata.com/wVxQEIWkwPUEAGf3/yum/repo/\nenabled=1\nautorefresh=1\ngpgcheck=1\ngpgkey=https://windsurf-stable.codeiumdata.com/wVxQEIWkwPUEAGf3/yum/RPM-GPG-KEY-windsurf" | sudo tee /etc/yum.repos.d/windsurf.repo > /dev/null
sudo dnf install windsurf
```

**Docker Strategy:** Requires Xvfb for GUI testing. Consider config-file-only testing as fallback.

**Sources:** [Windsurf Download](https://windsurf.com/editor/download-linux)

---

### Antigravity (Google Antigravity)

| Property | Value |
|----------|-------|
| **Slug** | `antigravity` |
| **Description** | **Google's agentic IDE** - announced Nov 2025 with Gemini 3 |
| **Config Path** | `.agent/rules.md` (workspace rules) |
| **Config Format** | Markdown |
| **Additional Paths** | `.agent/` directory, Global rules via UI |
| **Capabilities** | Custom instructions, rules directory, Skills system, MCP |
| **Installation** | Download from `antigravity.google/download` |
| **Headless Mode** | **Unknown** - likely no (agentic IDE) |
| **Version Command** | Unknown |
| **Versioning Model** | Unknown - public preview |
| **Pin Strategy** | Snapshot container images by date |
| **Docker Feasibility** | **Complex** - GUI app, needs Xvfb research |
| **API Required** | **Gemini 3** (Pro, Flash, Deep Think) - free tier available |

**RESEARCH COMPLETE - MAJOR FINDING:**
- [x] What is Antigravity? **Google's new agentic IDE, announced November 18, 2025**
- [x] Where to install? **antigravity.google/download**
- [x] Platforms? **macOS, Windows, Linux**
- [x] Documentation? **[Google Codelabs](https://codelabs.developers.google.com/getting-started-google-antigravity)**

**Key Features:**
- **Agent Manager:** Spawn, orchestrate, and observe multiple AI agents
- **Autonomous execution:** Agents plan, execute, verify across editor/terminal/browser
- **Artifact system:** Task lists, implementation plans, screenshots, browser recordings
- **Knowledge base:** Save context and code snippets
- **Skills system:** Codify best practices into executable assets

**Model Support:**
- Gemini 3 Pro (generous free rate limits)
- Anthropic Claude Sonnet 4.5
- OpenAI GPT-OSS

**Rules Configuration:**
- **Global rules:** Settings → Customizations → "+ Global"
- **Workspace rules:** Settings → Customizations → "+ Workspace"
- **Skills:** Advanced rules for specific tasks (database-migration, etc.)

**Docker Strategy:** Likely requires Xvfb. Research needed on whether agents can be run via API.

**Sources:** [Google Developers Blog](https://developers.googleblog.com/build-with-google-antigravity-our-new-agentic-development-platform/), [Codecademy Tutorial](https://www.codecademy.com/article/how-to-set-up-and-use-google-antigravity)

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
| **Description** | Google's open-source AI coding assistant CLI |
| **Config Path** | `GEMINI.md` |
| **Config Format** | Markdown |
| **Additional Paths** | None documented |
| **Capabilities** | Custom instructions, MCP support, Google Search grounding |
| **Installation** | `npm install -g @google/gemini-cli` |
| **Headless Mode** | Yes - CLI native |
| **Version Command** | `gemini --version` |
| **Versioning Model** | npm semver, weekly releases |
| **Pin Strategy** | `npm install -g @google/gemini-cli@x.y.z` |
| **Docker Feasibility** | **Trivial** - pure CLI |
| **API Required** | Google Account (free tier: 60 req/min, 1000 req/day) |

**RESEARCH COMPLETE:**
- [x] npm package name: **`@google/gemini-cli`** (current version 0.24.0)
- [x] Publicly available? **Yes - open source Apache 2.0**
- [x] Authentication: **Google Account (personal recommended, not Workspace)**

**Installation Commands:**
```bash
# Global install (recommended)
npm install -g @google/gemini-cli

# Run directly with npx
npx @google/gemini-cli

# In conda environment
conda create -y -n gemini_env -c conda-forge nodejs
conda activate gemini_env
npm install -g @google/gemini-cli

# Verify
gemini --version
```

**Key Features:**
- Gemini 2.5 Pro with 1M token context window
- Built-in tools: Google Search, file ops, shell commands, web fetch
- MCP (Model Context Protocol) support
- Terminal-first design

**Release Schedule:**
- **Preview:** UTC 2359 Tuesdays (may have regressions)
- **Stable:** UTC 2000 Tuesdays (validated)

**Free Tier Limits:**
- 60 model requests per minute
- 1,000 requests per day

**Sources:** [npm package](https://www.npmjs.com/package/@google/gemini-cli), [GitHub](https://github.com/google-gemini/gemini-cli), [Gemini CLI Docs](https://geminicli.com/docs/)

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
| **Description** | AI dev team in your editor (formerly Roo Cline) |
| **Config Path** | `.roo/rules/` |
| **Config Format** | Markdown directory |
| **Additional Paths** | `.roomodes` (YAML/JSON) |
| **Capabilities** | Custom instructions, MCP servers, rules directory |
| **Installation** | `code --install-extension RooVeterinaryInc.roo-cline` |
| **Headless Mode** | Via VS Code headless |
| **Version Command** | `code --list-extensions --show-versions` |
| **Versioning Model** | VS Code marketplace, semver |
| **Pin Strategy** | Install specific extension version |
| **Docker Feasibility** | **Moderate** - requires VS Code headless |
| **API Required** | Configurable (OpenRouter, Anthropic, OpenAI, Gemini, Bedrock, local) |

**RESEARCH COMPLETE:**
- [x] VS Code extension ID: **`RooVeterinaryInc.roo-cline`**
- [ ] `.roomodes` behavior: Still needs documentation review

**Installation Commands:**
```bash
# VS Code Marketplace (recommended)
code --install-extension RooVeterinaryInc.roo-cline

# Open VSX (for VSCodium)
# Available at: https://open-vsx.org/extension/RooVeterinaryInc/roo-cline
```

**Supported AI Providers:**
- OpenRouter
- Anthropic
- Glama
- OpenAI
- Google Gemini
- AWS Bedrock
- Azure
- GCP Vertex
- Local models (LM Studio, Ollama) - anything OpenAI-compatible

**Approval Modes:**
- Manual Approval: Review every step
- Autonomous/Auto-Approve: Uninterrupted workflows
- Hybrid: Auto-approve safe actions, confirm risky ones

**History:** Renamed from "Roo Cline" to "Roo Code" after 50,000+ installations.

**Sources:** [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=RooVeterinaryInc.roo-cline), [Roo Code Docs](https://docs.roocode.com/), [GitHub](https://github.com/RooCodeInc/Roo-Code)

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
| **Moderate** | JetBrains, Cline, Roo, Copilot, Amazon Q, **Cursor** | Requires headless IDE setup or CLI |
| **Complex** | Windsurf, Antigravity | GUI apps, need Xvfb |
| **Difficult** | **Zed** | Requires Vulkan GPU - no software fallback |

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

- [x] **Cursor:** Has full CLI with headless print mode (`-p`), `agent` command
- [x] **Zed:** Requires Vulkan GPU, NO headless mode, Docker difficult
- [x] **Windsurf:** Has apt/rpm repos for Linux, NO CLI, needs Xvfb
- [x] **Antigravity:** Is **Google Antigravity** - major Google IDE announced Nov 2025
- [x] **Gemini CLI:** `@google/gemini-cli` npm package, free tier available
- [x] **Roo Code:** Extension ID is `RooVeterinaryInc.roo-cline`
- [ ] JetBrains: headless SDK documentation review (still pending)
