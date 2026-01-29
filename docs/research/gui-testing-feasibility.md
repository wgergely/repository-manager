# GUI Testing Feasibility Research

> **Purpose:** Investigate what's actually possible for testing GUI-based tools in Docker containers.
> **Last Updated:** 2026-01-29
> **Status:** Research Draft - Requires Hands-On Verification

## Overview

Several tools in our matrix are GUI applications that present unique challenges for containerized testing:
- **Cursor** - Electron-based (VS Code fork)
- **Zed** - Native Rust, GPU-accelerated
- **Windsurf** - Likely Electron-based (Codeium's editor)
- **JetBrains IDEs** - Heavy Java applications
- **VS Code** - Electron-based (but has good headless support)

This document researches the feasibility of running these tools headlessly in Docker.

---

## Testing Approaches Overview

| Approach | Description | Complexity | Visual Debug |
|----------|-------------|------------|--------------|
| Native headless | Tool's built-in headless mode | Low | No |
| Xvfb | Virtual framebuffer | Medium | No |
| Xvfb + VNC | Virtual framebuffer + remote view | Medium | Yes |
| Xvfb + screenshot | Capture screenshots for assertion | Medium | Partial |
| Playwright | Browser automation for Electron | High | Yes |
| CLI-only | Test only CLI/config aspects | Low | N/A |

---

## Approach 1: Native Headless Modes

Some tools have built-in support for running without a display.

### VS Code

**Status: Well Supported**

VS Code has excellent headless support:

```bash
# Run without GUI
code --headless

# Install extensions headlessly
code --install-extension ms-python.python --force

# Extension testing
code --extensionTestsPath=/path/to/tests --extensionDevelopmentPath=/path/to/extension

# CLI operations (work without display)
code --list-extensions
code --version
code --diff file1 file2
```

**For our purposes:** VS Code headless is sufficient for testing that:
- Extensions install correctly
- Config files are read (via extension tests)
- MCP servers can be configured

### JetBrains IDEs

**Status: Partially Supported**

JetBrains offers:

1. **Headless inspections:**
   ```bash
   ./idea.sh inspect /project /output -v2 -d /path/to/inspections
   ```

2. **IDE Starter (for plugin testing):**
   - Official framework for automated testing
   - GitHub: `JetBrains/intellij-ide-starter`
   - Can run IDE headlessly for plugin tests

3. **Remote Development (Gateway):**
   - Server component can run headless
   - But not useful for our testing scenario

**Research needed:**
- [ ] Does IDE Starter work in Docker?
- [ ] Can we test `.aiassistant/rules/` loading programmatically?
- [ ] Memory/CPU requirements for headless JetBrains?

### Cursor

**Status: Unknown - Research Needed**

As a VS Code fork, Cursor *might* inherit some headless capabilities:

```bash
# Potential commands to investigate
cursor --headless            # Does this exist?
cursor --install-extension   # Does this exist?
cursor --version             # Does this exist?
```

**Research tasks:**
- [ ] Download Cursor and inspect CLI options
- [ ] Check if Cursor exposes `code`-like CLI
- [ ] Test if `.cursorrules` can be validated without full GUI

### Zed

**Status: Likely Not Supported**

Zed is GPU-accelerated and designed for visual performance:

```bash
# Known CLI options
zed --version
zed /path/to/project   # Opens GUI
```

From Zed's architecture:
- Uses GPU rendering (Metal on macOS, Vulkan on Linux)
- No documented headless mode
- May crash without display

**Research tasks:**
- [ ] Test if Zed runs with software rendering (LIBGL_ALWAYS_SOFTWARE=1)
- [ ] Test Zed with Xvfb (virtual framebuffer)
- [ ] Check if config validation can be done via CLI

### Windsurf

**Status: Unknown - Research Needed**

Likely a VS Code fork (Codeium product):

**Research tasks:**
- [ ] Confirm if Windsurf is Electron/VS Code based
- [ ] Check for CLI options
- [ ] Test headless operation

---

## Approach 2: Xvfb (Virtual Framebuffer)

For tools without native headless mode, Xvfb provides a virtual display.

### How Xvfb Works

```
┌─────────────────────────────────────────┐
│              Container                   │
│                                         │
│  ┌─────────┐        ┌──────────────┐   │
│  │  Xvfb   │◄──────►│  GUI Tool    │   │
│  │ :99     │        │ (Cursor/Zed) │   │
│  └─────────┘        └──────────────┘   │
│       │                                 │
│       ▼ (no physical output)            │
│    Memory                               │
└─────────────────────────────────────────┘
```

### Basic Setup

```dockerfile
# Install Xvfb
RUN apt-get install -y xvfb

# Start Xvfb before the application
ENV DISPLAY=:99
CMD Xvfb :99 -screen 0 1920x1080x24 & sleep 1 && /app/gui-tool
```

### With Window Manager

Some tools require a window manager:

```dockerfile
RUN apt-get install -y xvfb fluxbox

CMD Xvfb :99 -screen 0 1920x1080x24 & \
    sleep 1 && \
    fluxbox & \
    sleep 1 && \
    /app/gui-tool
```

### Potential Issues

| Issue | Symptom | Mitigation |
|-------|---------|------------|
| No GPU | Crash on startup | Software rendering, mesa |
| Missing fonts | Broken text | Install fonts-liberation |
| D-Bus missing | Startup errors | Install dbus, run daemon |
| Missing audio | Warnings/errors | Pulse audio stub or ignore |
| Slow rendering | Timeout failures | Increase timeouts |

### GPU Software Rendering

For GPU-requiring tools, force software rendering:

```dockerfile
RUN apt-get install -y mesa-utils libgl1-mesa-dri

ENV LIBGL_ALWAYS_SOFTWARE=1
ENV MESA_GL_VERSION_OVERRIDE=4.5
```

**Limitation:** May not work for Vulkan-based tools (Zed).

---

## Approach 3: Xvfb + VNC (Debugging)

Add VNC for visual debugging during test development:

```dockerfile
RUN apt-get install -y xvfb x11vnc

# Store VNC password
RUN mkdir -p /root/.vnc && \
    x11vnc -storepasswd "testpass" /root/.vnc/passwd

CMD Xvfb :99 -screen 0 1920x1080x24 & \
    sleep 1 && \
    x11vnc -display :99 -forever -usepw -rfbport 5900 & \
    /app/gui-tool
```

**Usage:**
```bash
# Run container with port exposed
docker run -p 5901:5900 repo-test/cursor

# Connect VNC client to localhost:5901
```

**VNC Clients:**
- macOS: Built-in Screen Sharing (vnc://localhost:5901)
- Windows: RealVNC, TightVNC
- Linux: Remmina, vncviewer
- Browser: noVNC (web-based)

### noVNC (Web-Based)

For easy access without VNC client:

```dockerfile
# Add noVNC
RUN git clone https://github.com/novnc/noVNC.git /opt/novnc && \
    git clone https://github.com/novnc/websockify.git /opt/novnc/utils/websockify

# Expose web port
EXPOSE 6080

CMD Xvfb :99 & \
    x11vnc -display :99 -forever -rfbport 5900 & \
    /opt/novnc/utils/novnc_proxy --vnc localhost:5900 --listen 6080 & \
    /app/gui-tool
```

Access via browser: `http://localhost:6080/vnc.html`

---

## Approach 4: Playwright for Electron Apps

For Electron-based tools (Cursor, Windsurf, VS Code), Playwright can automate the GUI.

### How It Works

```
┌─────────────────────────────────────────────┐
│                Container                     │
│                                             │
│  ┌────────────┐      ┌──────────────────┐  │
│  │ Playwright │─────►│   Electron App   │  │
│  │   Script   │ CDP  │  (Cursor/etc)    │  │
│  └────────────┘      └──────────────────┘  │
└─────────────────────────────────────────────┘
```

### Setup

```dockerfile
RUN npm install -g playwright
RUN npx playwright install-deps
```

### Example Test

```javascript
// test-cursor-config.js
const { _electron: electron } = require('playwright');

(async () => {
  // Launch Electron app
  const app = await electron.launch({
    executablePath: '/opt/cursor/cursor',
    args: ['/workspace/test-repo']
  });

  // Get main window
  const window = await app.firstWindow();

  // Wait for app to load
  await window.waitForLoadState('domcontentloaded');

  // Check if .cursorrules was loaded
  // (implementation depends on how Cursor exposes this)

  await app.close();
})();
```

### Limitations

- Requires understanding of app's internal structure
- Electron apps must expose Chrome DevTools Protocol
- May break with app updates
- Complex to maintain

---

## Per-Tool Feasibility Assessment

### VS Code

| Aspect | Feasibility | Notes |
|--------|-------------|-------|
| Installation | Easy | Well-documented apt/deb install |
| Headless run | Easy | Native `--headless` flag |
| Extension install | Easy | `code --install-extension` |
| Config testing | Medium | Via extension test framework |
| MCP testing | Medium | Requires extension that uses MCP |

**Verdict:** **Trivial** - Full support for our testing needs.

### Cursor

| Aspect | Feasibility | Notes |
|--------|-------------|-------|
| Installation | Medium | AppImage or .deb available |
| Headless run | Unknown | Needs research |
| Config testing | Unknown | Needs research |
| Xvfb fallback | Likely works | If it's Electron-based |
| Playwright | Likely works | If it's Electron-based |

**Verdict:** **Research needed** - Start with Xvfb approach.

**Research Plan:**
1. Download and install Cursor in Docker
2. Test with Xvfb - does it launch?
3. Check for CLI options (`cursor --help`)
4. If no CLI, test with Playwright
5. Document minimum viable test

### Zed

| Aspect | Feasibility | Notes |
|--------|-------------|-------|
| Installation | Easy | Binary download |
| Headless run | Unlikely | GPU-focused design |
| Config testing | Unknown | May have CLI config validation |
| Xvfb fallback | Uncertain | GPU requirement may block |
| Software render | Uncertain | Vulkan may not have SW fallback |

**Verdict:** **Complex** - May require alternative approach.

**Research Plan:**
1. Install Zed in Docker
2. Test with `LIBGL_ALWAYS_SOFTWARE=1`
3. Test with Xvfb
4. Check Zed CLI for config validation options
5. If all fail, consider config-file-only testing

**Alternative:** Test only that:
- Config files are generated correctly (file content validation)
- Zed binary exists and runs `--version`
- Skip full integration testing

### Windsurf

| Aspect | Feasibility | Notes |
|--------|-------------|-------|
| Installation | Unknown | Need to find download |
| Headless run | Unknown | Likely VS Code fork |
| Config testing | Unknown | Needs research |

**Verdict:** **Research needed** - Same approach as Cursor.

### JetBrains

| Aspect | Feasibility | Notes |
|--------|-------------|-------|
| Installation | Easy | tar.gz archives available |
| Headless run | Partial | IDE Starter framework |
| Plugin testing | Medium | Official test framework exists |
| Resource usage | High | JVM needs 2-4GB RAM |

**Verdict:** **Moderate** - Possible but resource-intensive.

**Research Plan:**
1. Test IDE Starter in Docker
2. Measure memory requirements
3. Create minimal plugin test for config loading
4. Document setup process

---

## Minimum Viable Test Strategy

For each tool, define the minimum we need to verify:

| Tool | Level 1: Config | Level 2: Detection | Level 3: Integration |
|------|-----------------|-------------------|---------------------|
| VS Code | File generated | Tool runs `--version` | Extension loads config |
| Cursor | File generated | Binary exists | Xvfb + launches |
| Zed | File generated | `--version` works | Best effort |
| Windsurf | File generated | Binary exists | Xvfb + launches |
| JetBrains | Files generated | IDE installs | IDE Starter test |

**Fallback strategy:** If Level 3 is not achievable for a tool, document it and test only Level 1+2.

---

## Required Docker Dependencies

### Minimal X11 Setup

```dockerfile
RUN apt-get update && apt-get install -y \
    xvfb \
    libx11-6 \
    libxext6 \
    libxrender1 \
    && rm -rf /var/lib/apt/lists/*
```

### Full GUI Setup

```dockerfile
RUN apt-get update && apt-get install -y \
    xvfb \
    x11vnc \
    fluxbox \
    libx11-6 \
    libxext6 \
    libxrender1 \
    libxtst6 \
    libxi6 \
    libxrandr2 \
    libxcomposite1 \
    libxcursor1 \
    libxdamage1 \
    libxfixes3 \
    libxss1 \
    libgconf-2-4 \
    libnss3 \
    libasound2 \
    libatk1.0-0 \
    libgtk-3-0 \
    libgbm1 \
    libdrm2 \
    dbus \
    fonts-liberation \
    fonts-noto-color-emoji \
    && rm -rf /var/lib/apt/lists/*
```

### Software Rendering

```dockerfile
RUN apt-get update && apt-get install -y \
    mesa-utils \
    libgl1-mesa-dri \
    libgl1-mesa-glx \
    && rm -rf /var/lib/apt/lists/*

ENV LIBGL_ALWAYS_SOFTWARE=1
```

---

## Research Checklist

### Immediate (Before Implementation)

- [ ] Install Cursor in Docker container
- [ ] Test Cursor with Xvfb
- [ ] Check Cursor CLI options
- [ ] Install Zed in Docker container
- [ ] Test Zed with software rendering
- [ ] Test Zed with Xvfb
- [ ] Find Windsurf installation method
- [ ] Test JetBrains IDE Starter in Docker

### Per-Tool Documentation

For each tool, document:
1. Exact installation commands
2. Xvfb compatibility (yes/no/partial)
3. Minimum test that works
4. Resource requirements (RAM, CPU)
5. Known issues and workarounds

### Success Criteria

A tool is "testable" if we can:
1. Install it non-interactively
2. Start it (headless or Xvfb)
3. Verify it reads configuration files
4. Exit cleanly

---

## References

- [VS Code CLI Reference](https://code.visualstudio.com/docs/editor/command-line)
- [Playwright Electron](https://playwright.dev/docs/api/class-electron)
- [JetBrains IDE Starter](https://github.com/JetBrains/intellij-ide-starter)
- [Xvfb Documentation](https://www.x.org/releases/X11R7.6/doc/man/man1/Xvfb.1.xhtml)
- [Docker GUI Apps](https://www.baeldung.com/ops/docker-container-gui-applications)

---

## Next Steps

1. Create proof-of-concept containers for each GUI tool
2. Document what works and what doesn't
3. Update this document with findings
4. Adjust docker-architecture.md based on findings
5. Create decision records for tool-specific strategies
