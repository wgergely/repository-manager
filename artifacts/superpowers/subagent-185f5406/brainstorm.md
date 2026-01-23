### Goal
- To define a robust, high-performance technical strategy for 1) programmatically managing Git repositories using worktrees in Rust, and 2) establishing a flexible plugin architecture for the command-line interface (CLI) to allow for future extensibility.

### Constraints
- **Technology Stack:** The core implementation must be in Rust for performance and safety.
- **Performance:** The solution must be fast, especially for Git operations, to ensure a smooth user experience. `gix` is preferred over `git2` for this reason.
- **Compatibility:** The plugin system must be cross-platform (Windows, macOS, Linux).
- **Extensibility:** The architecture should allow third parties to add new commands and functionality without modifying the core binary.

### Known context
- The project is in the initial design phase, defining the core architecture in `docs/design/01-architecture-spec.md`.
- No core logic has been implemented.
- The desired plugin functionality is analogous to systems like `nx` and `projen`.
    - **nx:** Employs a plugin model where "executors" and "generators" are distributed as packages. This is a direct parallel to the desired CLI plugin system, where external tools can be registered to execute tasks.
    - **projen:** Uses "project types" which are classes that codify project configurations. This is less about runtime plugins and more about advanced templating. Our CLI's plugins might *provide* projen-like templating capabilities.
- Research confirms `gix` is a modern, performance-focused Rust library for Git operations.
- Research on plugin systems highlights two primary models in Rust: separate binaries (or dynamic libraries) and WebAssembly (WASM).

### Risks
- **`gix` API Complexity:** `gix` is powerful but can be more complex and lower-level than alternatives like `git2`, potentially increasing development time.
- **Plugin ABI Stability (Binary Approach):** Rust does not have a stable Application Binary Interface (ABI). If using dynamic libraries, changes in Rust compiler versions between the core CLI and a plugin can cause crashes. This requires mitigation, such as using a C ABI (`#[repr(C)]`).
- **Performance & Maturity (WASM Approach):** WASM introduces a performance overhead compared to native code and is less mature for system-level tasks that might require deep OS integration (e.g., filesystem access, networking). The toolchain (e.g., WASI) is still evolving.
- **Security:** Loading external code (either as binaries or WASM modules) is a security risk. A clear trust and sandboxing model is required.

### Options (2–4)

#### Option 1: `gix` for Worktrees + Separate Binaries for Plugins

- **Summary:** Use `gix` for all Git worktree manipulations. The plugin system will be based on separate, standalone binaries that follow a specific command-line interface contract (e.g., `plugin-name --subcommand --json-input`). The core CLI discovers these plugins in a designated directory and invokes them as subprocesses, communicating via stdin/stdout.
- **Pros:**
    - **Native Performance:** Plugins run at full native speed.
    - **Simplicity & Maturity:** Invoking subprocesses is a simple, well-understood, and robust pattern (e.g., `git` itself works this way).
    - **Language Agnostic:** Any language can be used to write a plugin, as long as it compiles to a standalone executable.
    - **Total System Access:** Plugins are not sandboxed and have full access to the system, which is powerful for trusted, system-level tools.
- **Cons:**
    - **No Sandboxing:** A malicious or buggy plugin has full access to the user's system and the core application's data, posing a security risk.
    - **Discovery & Management:** Requires a clear mechanism for installing, upgrading, and discovering plugin binaries.
    - **Data Marshaling:** All communication happens via serializing data (e.g., to JSON) over stdin/stdout, which can be less efficient than in-memory calls.
- **Complexity / Risk:** Low complexity for the core implementation, but high risk from a security perspective if plugins are untrusted.

#### Option 2: `gix` for Worktrees + WASM for Plugins

- **Summary:** Use `gix` for Git operations. The plugin system will load and execute WASM modules using a runtime like Wasmer or Wasmtime. The core CLI exposes a specific set of host functions (the plugin API) that WASM modules can import to interact with the system and core application state in a controlled manner.
- **Pros:**
    - **Strong Security:** WASM provides a sandboxed environment, isolating plugin code from the host process and limiting its access to only the capabilities explicitly provided by the host (e.g., via WASI).
    - **Language Agnostic:** Plugins can be written in any language that compiles to WASM.
    - **Stable Interface:** The WASM binary format and API are standardized, avoiding the Rust ABI instability issues found with dynamic libraries.
    - **Portability:** A single `.wasm` plugin binary can run on any platform supported by the WASM runtime.
- **Cons:**
    - **Performance Overhead:** WASM execution is slower than native. Data transfer between the host and the WASM module can also be a bottleneck.
    - **Limited System Access:** The sandbox restricts what plugins can do. Accessing files, networks, or environment variables requires the host to explicitly and safely expose that functionality, which adds complexity.
    - **Toolchain Immaturity:** The ecosystem, particularly around the WebAssembly System Interface (WASI), is still evolving. Some system-level tasks may be difficult or impossible to implement.
- **Complexity / Risk:** High complexity due to the need to design a secure host API, manage the WASM runtime, and navigate the maturing WASI specification. Lower security risk during plugin execution.

### Recommendation
**Option 1: `gix` for Worktrees + Separate Binaries for Plugins.**

This approach is recommended for its simplicity, maturity, and performance. The `git` subcommand pattern is a proven model for extensible CLIs. While the lack of sandboxing is a valid concern, it can be mitigated by sourcing plugins from trusted locations and clearly communicating the security model to the user. This model provides the most power and flexibility for system-level automation, which is the primary goal of this tool. The performance penalty and system access limitations of WASM are too restrictive for the initial version of a developer-centric tool that will likely need to interact deeply with the file system and other local development tools.

### Acceptance criteria
- The CLI can successfully create a new Git worktree at a specified path from a given branch using `gix`.
- The CLI can list all active worktrees for the current repository.
- The CLI can remove an existing worktree.
- The CLI can discover executable files (e.g., `repo-manager-plugin-*`) in a predefined directory (`~/.config/repo-manager/plugins`).
- The CLI can execute a discovered plugin as a subprocess, passing arguments and receiving structured output (e.g., JSON) back.
- A sample plugin can be created that reads data from the core CLI and prints a result to stdout.
