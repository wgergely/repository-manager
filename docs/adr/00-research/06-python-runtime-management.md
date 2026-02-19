# Research: Python Runtime Management from Non-Python Hosts

**Date:** 2026-02-19
**Researcher:** python-runtime-researcher (Opus agent)
**Source:** Web research

---

## 1. Python Discovery from Rust

### Cross-Platform Strategy

**Unix/macOS:**
- PEP 394: use `python3` (not `python`) as reliable command
- Pyenv shims in `~/.pyenv/shims/` intercept calls
- Search PATH for `python3`, `python3.x` executables

**Windows:**
- **py launcher** (PEP 397) is the standard, NOT PATH
- **PEP 514** registry schema: `HKEY_CURRENT_USER\Software\Python\<Company>\<Tag>`
- Registry keys: `InstallPath`, `ExecutablePath`
- User installations take precedence over system
- Microsoft Store Python registers via same PEP 514 mechanism

### Pitfalls
- `python` may point to Python 2 on older systems
- Pyenv shims mask system Python
- Windows Store Python is sandboxed
- Conda modifies PATH dynamically

### Recommendation
Follow uv's approach: managed installations -> virtual environments -> system PATH -> platform registries. On Windows, read PEP 514 keys via `winreg` crate.

---

## 2. uv (astral-sh/uv) - Key Prior Art

### Python Discovery Order
1. Managed Python installations (`UV_PYTHON_INSTALL_DIR`)
2. Virtual environments (`VIRTUAL_ENV`, `CONDA_PREFIX`, `.venv`)
3. System PATH (`python`, `python3`, `python3.x`)
4. Windows Registry (PEP 514)

### Version Request Syntax
Supports: `3`, `3.12`, `3.12.3`, `>=3.12,<3.13`, `cpython`, `pypy@3.11`

### Venv Creation (Key Insight)
uv creates venvs **entirely in Rust** (PEP 405) without invoking `python -m venv`. 80x faster. Creates directory structure, writes `pyvenv.cfg`, creates symlinks/copies.

### Python Download
Bundles distributions from python-build-standalone - prebuilt, relocatable CPython binaries.

### Recommendation
Delegate to `uv` as subprocess (`uv venv`, `uv pip install`) rather than reimplementing.

---

## 3. Pre-commit's Python Bootstrapping

- Per-hook isolated environments in `~/.cache/pre-commit/`
- Reads `.pre-commit-hooks.yaml` for `language` key
- Creates language-specific isolated env
- Reuses until config changes

### Recommendation
One venv per extension, cached persistently, keyed on extension version + Python version + dependency hash.

---

## 4. Python Version Constraint Formats

### PEP 440 Operators
- `~=V` - Compatible release
- `==V` - Exact match (supports wildcards)
- `!=V` - Exclusion
- `>=V`, `<=V`, `>V`, `<V` - Ordered comparison
- Combining: comma-separated = AND

### requires-python in pyproject.toml
```toml
[project]
requires-python = ">=3.8"
```

### Recommendation
Use PEP 440 for Python version constraints. The `pep440_rs` crate from uv project is available.

---

## 5. Vendored/Embedded Python Dependencies

### Patterns
1. **sys.path injection** - Prepend vendor dir to sys.path
2. **Import rewriting** - pip's approach (pip._vendor)
3. **Local wheel install** - `pip install --no-index --find-links=./vendor`
4. **PEP 302 import hooks** - Custom finders/loaders

### Recommendation
For vendored deps: `pip install --no-index --find-links=./vendor` into the extension's venv. Avoids network access.

---

## 6. Cross-Platform Venv Creation

| Aspect | Unix | Windows |
|--------|------|---------|
| Binary dir | `bin/` | `Scripts/` |
| Python binary | `bin/python3` (symlink) | `Scripts/python.exe` (copy) |
| Site-packages | `lib/pythonX.Y/site-packages/` | `Lib/site-packages/` |
| Path separator | `:` | `;` |

### Key Points
- Venvs are non-portable (absolute paths in shebangs/pyvenv.cfg)
- **Do NOT require activation** - invoke venv's Python directly
- `sys.prefix != sys.base_prefix` indicates running inside venv

---

## 7. MCP Server Process Management

### Stdio Transport (Primary)
- Client spawns server as subprocess
- JSON-RPC 2.0 over stdin/stdout, newline-delimited
- Server MUST NOT write non-MCP to stdout
- Server MAY write to stderr for logging

### Configuration Format (Claude Desktop)
```json
{
  "mcpServers": {
    "my-server": {
      "command": "python",
      "args": ["-m", "my_mcp_server"],
      "env": { "API_KEY": "value" }
    }
  }
}
```

### Gotchas
- Claude Desktop: completely isolated environment, env vars must be explicit
- Windows: `npx` needs `cmd /c` wrapper
- Servers should be stateless between messages

---

## 8. Rust Subprocess Patterns

### tokio::process::Command
```rust
let mut child = Command::new("/path/to/venv/bin/python")
    .args(&["-m", "extension_module"])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .kill_on_drop(true)
    .spawn()?;
```

### Key Patterns
- Separate tasks for reading and writing (avoid deadlocks)
- `kill_on_drop(true)` is essential
- Graceful shutdown: SIGTERM -> wait -> SIGKILL (no SIGTERM on Windows)
- `rmcp` crate provides `TokioChildProcess` transport for MCP

### Recommendations
1. `tokio::process::Command` with `kill_on_drop(true)`
2. Consider `rmcp` crate for MCP client
3. Process supervisor for health monitoring + restart
4. Stderr routed to logging system
5. Graceful shutdown via JSON-RPC notification -> SIGTERM -> SIGKILL
