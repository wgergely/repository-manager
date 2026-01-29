# Rust Registry and Plugin Patterns Research

**Date:** 2026-01-29
**Purpose:** Inform the tool registry system overhaul for repo-tools
**Status:** Complete

---

## Executive Summary

This document captures research on modern Rust patterns for building extensible registry systems. The findings inform the design of a unified tool registration system that eliminates code duplication while maintaining performance and extensibility.

**Key Recommendation:** Hybrid architecture using `enum_dispatch` for built-in tools (10x performance) and trait objects for user-defined extensions.

---

## 1. Global Registration Patterns

### 1.1 inventory Crate

**Source:** [github.com/dtolnay/inventory](https://github.com/dtolnay/inventory)

The `inventory` crate provides distributed plugin registration without a central list.

**How it works:**
- Uses runtime initialization functions similar to `__attribute__((constructor))` in C
- Each `inventory::submit!` call registers items before `main()` executes
- Items collected by type into an iterable registry

**Pros:**
- Decentralized registration (each module self-registers)
- Works with dynamic loading (`dlopen`)
- No central coordination needed

**Cons:**
- Requires running code before `main()`
- Poor embedded/no_std support
- Slight runtime overhead at startup

### 1.2 linkme Crate

**Source:** [github.com/dtolnay/linkme](https://github.com/dtolnay/linkme)

Alternative to inventory using linker sections.

**How it works:**
- Uses linker tricks to create sections containing registered elements
- All work happens at compile/link time, not runtime
- Results in a true `&'static [T]` slice

**Pros:**
- No runtime initialization
- Better embedded support (cortex-m tested)
- True slice semantics

**Cons:**
- No dynamic library support
- Requires const expressions
- More linker-dependent

### 1.3 Emerging Consensus

**Source:** [Global Registration Blog](https://donsz.nl/blog/global-registration/)

> "Crate-local registries with explicit cross-crate imports represent the superior design, avoiding versioning conflicts and dependency surprises inherent in truly global registration schemes."

The Rust community is moving away from magic global registration toward explicit patterns.

---

## 2. Dispatch Performance

### 2.1 enum_dispatch Crate

**Source:** [crates.io/crates/enum_dispatch](https://crates.io/crates/enum_dispatch)

Provides near drop-in replacement for dynamic dispatch with up to 10x speedup.

**Benchmarks (from crate documentation):**
| Method | Time (ns/iter) | Relative |
|--------|---------------|----------|
| enum_dispatch | 479,630 | 1x |
| Box<dyn Trait> | 5,900,191 | ~12x slower |

**Why it's faster:**
- Vec of enums rather than addresses (half the indirection)
- No vtable lookups
- Compiler can inline and optimize within match arms

### 2.2 When to Use Each Approach

**Source:** [Enum or Trait Object - Possible Rust](https://www.possiblerust.com/guide/enum-or-trait-object)

| Scenario | Recommendation |
|----------|----------------|
| Closed set of types known at compile time | **Enum dispatch** |
| Internal delegation only | **Enum dispatch** |
| External extensibility needed | **Trait objects** |
| User-defined types | **Trait objects** |
| Plugin system | **Trait objects** |

**Key insight:** "If the need for delegation is only internal, you're likely better off with an enum. It's faster, subject to fewer rules, and makes it easy to see all variants."

---

## 3. Framework Patterns

### 3.1 Bevy Plugin System

**Source:** [Bevy Cheat Book - Plugins](https://bevy-cheatbook.github.io/programming/plugins.html)

Bevy uses explicit registration with no global magic:

```rust
App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins((MyPlugin, AnotherPlugin))
    .run();
```

**Key patterns:**
- Plugins implement a `Plugin` trait with `build(&self, app: &mut App)`
- `PluginGroup` combines related plugins
- Configuration via `.set()` method on plugin groups
- Disabling via `.disable::<T>()`

**Why it works:** Explicit is better than implicit. All registration visible in one place.

### 3.2 Nushell Plugin Architecture

**Source:** [Nushell Contributor Book - Plugins](https://www.nushell.sh/contributor-book/plugins.html)

Nushell distinguishes between in-process and external plugins:

**In-process (Rust):**
- Implement `Plugin` trait
- Register commands via `PluginCommand` trait
- Direct function calls, no serialization overhead

**External (any language):**
- Standalone executables
- Communicate via stdin/stdout
- JSON or MsgPack serialization
- Registry file tracks known plugins (`plugin add`, `plugin use`)

**Key insight:** Two-tier architecture allows both performance (Rust) and flexibility (any language).

### 3.3 Axum/Tower Middleware

**Source:** [Axum Documentation](https://docs.rs/axum/latest/axum/)

Axum uses Tower's `Service` and `Layer` traits for middleware:

```rust
let app = Router::new()
    .route("/", get(handler))
    .layer(TraceLayer::new())
    .layer(CompressionLayer::new());
```

**Key patterns:**
- Composable layers wrap services
- Each layer is a separate type
- No global registration, explicit stacking
- Entire Tower ecosystem available

### 3.4 Cargo Registry

**Source:** [Cargo Book - Registries](https://doc.rust-lang.org/cargo/reference/registries.html)

Cargo's registry system supports multiple sources with precedence:

```
$HOME/.cargo/registry/
├── index/       # Package metadata (one file per crate)
├── cache/       # Downloaded .crate tarballs
└── src/         # Unpacked source code
```

**Key patterns:**
- Index-based lookup (efficient resolution)
- Local caching of remote resources
- Source replacement (vendoring, mirroring)
- Multiple registries with precedence

---

## 4. Component Architecture

### 4.1 Trait-Based Components

**Source:** [Component-Based Architecture in Rust](https://vadosware.io/post/a-pattern-for-component-based-program-architecture-in-rust/)

Pattern for building modular systems:

```rust
trait Component {
    fn get_name(&self) -> &str;
    fn start(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
}
```

**Extensibility through trait composition:**
- `Configurable<C>` - Runtime configuration
- `FileConfigurable<C>` - File-based config
- `HandlesSimpleRequests` - Request/response with associated types

**Key insight:** Specialized traits as supertraits maintain flexibility while establishing clear contracts.

### 4.2 Hexagonal Architecture

**Source:** [Rust Design Patterns](https://rust-unofficial.github.io/patterns/)

Ports and adapters pattern in Rust:
- Traits define ports (interfaces)
- Concrete types implement adapters
- Core logic depends only on traits
- Easy to swap implementations for testing

---

## 5. Dependency Injection

### 5.1 Shaku

**Source:** [github.com/AzureMarker/shaku](https://github.com/AzureMarker/shaku)

Compile-time dependency injection for Rust:

```rust
#[derive(Component)]
#[shaku(interface = IOutput)]
struct ConsoleOutput;

#[derive(Component)]
#[shaku(interface = IWriter)]
struct TodayWriter {
    #[shaku(inject)]
    output: Arc<dyn IOutput>,
}
```

**Key features:**
- Compile-time dependency checking
- Module-based service grouping
- Thread-safe by default
- Integrations for Rocket, Axum

**Trade-off:** Adds complexity; best for large applications with many dependencies.

---

## 6. Recommendations for repo-tools

Based on this research, the recommended architecture for the tool registry:

### 6.1 Use enum_dispatch for Built-ins

The 13 built-in tools are a closed set known at compile time. Using `enum_dispatch` provides:
- 10x performance improvement over trait objects
- Compiler optimizations (inlining within match arms)
- Type safety with exhaustive matching

### 6.2 Use Trait Objects for Extensions

User-defined tools (via TOML schemas) need runtime flexibility:
- Can't know all tools at compile time
- Users shouldn't need to recompile
- Trait objects provide necessary extensibility

### 6.3 Explicit Registration (Bevy-style)

Avoid global/magic registration:
- Single source of truth in one file
- Feature flags via standard `#[cfg]` attributes
- Clear, auditable registration code

### 6.4 Layered Configuration (Cargo-style)

Multiple sources with precedence:
1. Built-in defaults (code)
2. Project configuration (`.repository/tools.toml`)
3. Environment variables (`REPO_TOOL_*`)

### 6.5 No New Dependencies for Core

The recommended pattern uses:
- `enum_dispatch` (small, well-maintained)
- Standard Rust features (`#[cfg]`, traits, enums)
- Existing `figment` for config layering

---

## 7. Sources

### Crates and Libraries
- [inventory](https://github.com/dtolnay/inventory) - Typed distributed plugin registration
- [linkme](https://github.com/dtolnay/linkme) - Safe cross-platform linker shenanigans
- [enum_dispatch](https://crates.io/crates/enum_dispatch) - Near drop-in replacement for dynamic dispatch
- [shaku](https://github.com/AzureMarker/shaku) - Compile-time dependency injection

### Documentation and Guides
- [Global Registration Blog](https://donsz.nl/blog/global-registration/) - Analysis of registration patterns
- [Enum or Trait Object](https://www.possiblerust.com/guide/enum-or-trait-object) - Decision guide
- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/) - Official patterns book
- [Bevy Plugins](https://bevy-cheatbook.github.io/programming/plugins.html) - Plugin system design

### Framework References
- [Nushell Plugins](https://www.nushell.sh/contributor-book/plugins.html) - Two-tier plugin architecture
- [Axum Documentation](https://docs.rs/axum/latest/axum/) - Tower-based middleware
- [Cargo Registries](https://doc.rust-lang.org/cargo/reference/registries.html) - Index and source management

### Performance Analysis
- [Rust Dispatch Explained](https://www.somethingsblog.com/2025/04/20/rust-dispatch-explained-when-enums-beat-dyn-trait/) - When enums beat dyn Trait
- [enum_dispatch Benchmarks](https://crates.io/crates/enum_dispatch) - 10x performance claims

---

*Research completed: 2026-01-29*
