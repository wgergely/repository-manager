# Rust CLI Ecosystem for Repository Orchestrator Tool

*Research Document: Implementation Technology Stack*
*Date: 2026-01-23*

## Executive Summary

This document evaluates the Rust ecosystem for building a cross-platform CLI orchestrator tool that manages agentic coding environments across multiple tools (Claude Code, Cursor, Gemini, etc.) with git worktree support. The tool needs robust CLI argument parsing, configuration management, git operations, template scaffolding, and cross-platform compatibility.

**Notable Crates by Category:**
- CLI Framework: **clap** (v4.x with derive macros), **argh** (minimal, fast compile)
- Interactive Prompts: **dialoguer** + **indicatif**, **inquire**
- Configuration: **figment** (layered), **config-rs** (runtime), direct serde
- Git Operations: **gix** (gitoxide, pure Rust), **git2** (libgit2 bindings)
- File System: **walkdir** + **fs_extra** + **notify**
- Templates: **tera** (Jinja2-like), **handlebars** (Mustache), **minijinja** (lightweight)
- Cross-platform Paths: **directories** (dirs-next)
- Process Execution: **tokio::process** (async), **xshell** (ergonomic scripting)
- Error Handling: **miette** (user-facing diagnostics), **thiserror** (error types), **anyhow** (context)
- Async Runtime: **tokio** (multi-threaded)

---

## 1. CLI Framework Evaluation

### 1.1 Comparison Matrix

| Crate | Version | Derive Macros | Subcommands | Completions | Maintained | Ecosystem |
|-------|---------|---------------|-------------|-------------|------------|-----------|
| **clap** | 4.x | Yes | Yes | Yes | Active | Dominant |
| argh | 0.1.x | Yes | Yes | No | Google | Minimal |
| structopt | 0.3.x | Yes | Yes | Limited | Deprecated | Legacy |
| pico-args | 0.5.x | No | Manual | No | Active | Minimal |
| lexopt | 0.3.x | No | Manual | No | Active | Minimal |

### 1.2 Detailed Analysis

#### clap (Command Line Argument Parser)

**Status:** Industry standard, actively maintained by the Rust CLI Working Group.

**Key Features:**
- Derive macro API (`#[derive(Parser)]`) for declarative argument definitions
- Builder API for programmatic construction
- Subcommand support with nesting
- Shell completion generation (bash, zsh, fish, PowerShell, elvish)
- Rich help formatting with colors
- Environment variable support
- Value validation and parsing
- Argument groups and conflicts

**Example Structure for Repository Manager:**
```rust
use clap::{Parser, Subcommand, Args};

#[derive(Parser)]
#[command(name = "repo-manager")]
#[command(about = "Orchestrate agentic coding environments")]
#[command(version, author)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new repository container
    Init(InitArgs),

    /// Create a new worktree
    Create(CreateArgs),

    /// Add agentic tool configuration
    Add(AddArgs),

    /// Remove worktree or configuration
    Remove(RemoveArgs),

    /// Install/setup agentic tools
    Install(InstallArgs),

    /// Sync configurations across worktrees
    Sync(SyncArgs),
}

#[derive(Args)]
struct InitArgs {
    /// Repository URL to clone
    #[arg(short, long)]
    url: Option<String>,

    /// Target directory
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Initialize for Claude Code
    #[arg(long)]
    claude: bool,

    /// Initialize for Gemini
    #[arg(long)]
    gemini: bool,

    /// Use worktree-based structure
    #[arg(long)]
    worktrees: bool,
}

#[derive(Args)]
struct CreateArgs {
    /// Branch name for new worktree
    name: String,

    /// Base branch to create from
    #[arg(short, long, default_value = "main")]
    from: String,

    /// Directory path for worktree
    #[arg(short, long)]
    path: Option<PathBuf>,
}
```

**Shell Completions:**
```rust
use clap_complete::{generate, Shell};

fn generate_completions(shell: Shell) {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "repo-manager", &mut std::io::stdout());
}
```

**Trade-offs:** clap offers the most complete feature set and ecosystem integration but has longer compile times. For projects where compile time is critical and advanced features are not needed, argh provides a lighter alternative.

#### argh (Google)

**Status:** Minimal alternative from Google, focused on compile-time optimization.

**Pros:**
- Faster compile times than clap
- Simple API
- Zero dependencies

**Cons:**
- Limited ecosystem integration
- No shell completions
- Less flexible validation
- Smaller community

**Use Case:** Appropriate for simple CLIs where compile time is critical or when zero dependencies is a requirement.

#### structopt

**Status:** Deprecated - merged into clap v3+.

**Migration:** All structopt users should migrate to clap derive macros. The API is nearly identical.

### 1.3 Interactive Prompts

For setup wizards and interactive configuration, clap alone is insufficient. Complement with:

#### dialoguer

**Purpose:** Interactive prompts (Input, Select, MultiSelect, Confirm, Password)

```rust
use dialoguer::{theme::ColorfulTheme, Select, Input, Confirm, MultiSelect};

fn setup_wizard() -> Result<Config> {
    let theme = ColorfulTheme::default();

    // Select agentic tools to configure
    let tools = vec!["Claude Code", "Cursor", "Windsurf", "Copilot", "Gemini"];
    let selections = MultiSelect::with_theme(&theme)
        .with_prompt("Select agentic tools to configure")
        .items(&tools)
        .interact()?;

    // Get project name
    let project_name: String = Input::with_theme(&theme)
        .with_prompt("Project name")
        .default("my-project".into())
        .interact_text()?;

    // Confirm worktree setup
    let use_worktrees = Confirm::with_theme(&theme)
        .with_prompt("Use git worktree-based structure?")
        .default(true)
        .interact()?;

    // Select directory structure pattern
    let patterns = vec!["Centralized", "Orphan Branch", "Submodule"];
    let pattern_idx = Select::with_theme(&theme)
        .with_prompt("Select container pattern")
        .items(&patterns)
        .default(0)
        .interact()?;

    Ok(Config { /* ... */ })
}
```

#### indicatif

**Purpose:** Progress bars and spinners for long-running operations

```rust
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};

fn clone_repository(url: &str) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    pb.set_message("Cloning repository...");
    pb.enable_steady_tick(Duration::from_millis(100));

    // Perform clone operation
    git_clone(url)?;

    pb.finish_with_message("Repository cloned successfully");
    Ok(())
}

fn sync_worktrees(worktrees: &[Worktree]) -> Result<()> {
    let mp = MultiProgress::new();

    for wt in worktrees {
        let pb = mp.add(ProgressBar::new(100));
        pb.set_style(ProgressStyle::default_bar()
            .template("{prefix:.bold} [{bar:40}] {pos}/{len}")
            .unwrap());
        pb.set_prefix(wt.name.clone());

        // Sync operation with progress updates
    }

    Ok(())
}
```

#### console

**Purpose:** Terminal styling, colors, and utilities (pairs well with dialoguer/indicatif)

```rust
use console::{style, Emoji, Term};

static SUCCESS: Emoji<'_, '_> = Emoji("✅ ", "");
static WARNING: Emoji<'_, '_> = Emoji("⚠️  ", "");
static ERROR: Emoji<'_, '_> = Emoji("❌ ", "");

fn print_status(message: &str, status: Status) {
    match status {
        Status::Success => println!("{} {}", SUCCESS, style(message).green()),
        Status::Warning => println!("{} {}", WARNING, style(message).yellow()),
        Status::Error => println!("{} {}", ERROR, style(message).red()),
    }
}
```

---

## 2. Configuration Management

### 2.1 Comparison Matrix

| Crate | Format Support | Layering | Env Vars | Type Safety | Profiles |
|-------|---------------|----------|----------|-------------|----------|
| **figment** | TOML, JSON, YAML, ENV | Yes | Yes | Strong | Yes |
| config | TOML, JSON, YAML, INI | Yes | Yes | Runtime | Yes |
| toml | TOML only | No | No | Strong | No |
| serde_yaml | YAML only | No | No | Strong | No |

### 2.2 Detailed Analysis

#### figment

**Status:** Modern, type-safe configuration with layering support.

**Key Features:**
- Multiple format support via providers
- Hierarchical configuration merging
- Environment variable integration
- Profile-based configuration
- Strong typing with serde

**Configuration Structure for Repository Manager:**
```rust
use figment::{Figment, providers::{Format, Toml, Json, Env, Serialized}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    /// Global settings
    global: GlobalConfig,

    /// Tool-specific configurations
    tools: ToolsConfig,

    /// Worktree settings
    worktrees: WorktreeConfig,

    /// Template paths
    templates: TemplateConfig,
}

#[derive(Debug, Deserialize, Serialize)]
struct GlobalConfig {
    /// Default container directory
    container_dir: PathBuf,

    /// Default branch name
    default_branch: String,

    /// Auto-sync on worktree creation
    auto_sync: bool,

    /// Verbose logging
    verbose: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct ToolsConfig {
    /// Claude Code settings
    claude: Option<ClaudeConfig>,

    /// Cursor settings
    cursor: Option<CursorConfig>,

    /// Windsurf settings
    windsurf: Option<WindsurfConfig>,

    /// GitHub Copilot settings
    copilot: Option<CopilotConfig>,

    /// Gemini Code Assist settings
    gemini: Option<GeminiConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ClaudeConfig {
    /// Enable Claude Code support
    enabled: bool,

    /// Path to CLAUDE.md template
    rules_template: Option<PathBuf>,

    /// MCP servers to configure
    mcp_servers: Vec<McpServerConfig>,

    /// Default permissions
    permissions: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct WorktreeConfig {
    /// Container structure pattern
    pattern: ContainerPattern,

    /// Worktree directory name
    worktrees_dir: String,

    /// Symlink strategy for configs
    symlink_configs: bool,
}

#[derive(Debug, Deserialize, Serialize)]
enum ContainerPattern {
    Centralized,
    OrphanBranch,
    Submodule,
}

// Configuration loading with layering
fn load_config() -> Result<Config, figment::Error> {
    let config: Config = Figment::new()
        // 1. Default values
        .merge(Serialized::defaults(Config::default()))
        // 2. System-wide config
        .merge(Toml::file("/etc/repo-manager/config.toml"))
        // 3. User config
        .merge(Toml::file(dirs::config_dir().unwrap().join("repo-manager/config.toml")))
        // 4. Project config
        .merge(Toml::file(".repo-manager.toml"))
        // 5. Environment variables (REPO_MANAGER_*)
        .merge(Env::prefixed("REPO_MANAGER_").split("_"))
        .extract()?;

    Ok(config)
}
```

**Example Configuration File (.repo-manager.toml):**
```toml
[global]
container_dir = "~/dev/containers"
default_branch = "main"
auto_sync = true
verbose = false

[worktrees]
pattern = "Centralized"
worktrees_dir = "worktrees"
symlink_configs = true

[tools.claude]
enabled = true
rules_template = "~/.config/repo-manager/templates/CLAUDE.md"
permissions = ["Bash", "Read", "Write", "Edit"]

[[tools.claude.mcp_servers]]
name = "filesystem"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem"]

[tools.cursor]
enabled = true
rules_template = "~/.config/repo-manager/templates/cursorrules"

[tools.copilot]
enabled = false

[tools.gemini]
enabled = false
```

#### config-rs

**Status:** Mature, widely used, but less type-safe than figment.

**Use Case:** Legacy projects or when runtime configuration is preferred.

#### Direct serde with toml/serde_yaml

**Use Case:** Simple, single-file configurations without layering needs.

### 2.3 Trade-offs

**figment** strengths:
- Strong compile-time type checking
- Layering support (system -> user -> project -> env)
- Multiple format support
- Profile-based configuration

**config-rs** strengths:
- Mature, widely used
- Runtime flexibility
- Well-documented

**Direct serde** strengths:
- Simplest approach for single-file configs
- No additional dependencies beyond serde
- Full control over format parsing

---

## 3. Git Operations

### 3.1 Comparison Matrix

| Crate | Implementation | Performance | Safety | Features | Size |
|-------|----------------|-------------|--------|----------|------|
| **gix** | Pure Rust | Excellent | Memory-safe | Growing | Large |
| git2 | libgit2 bindings | Good | C FFI | Complete | Medium |

### 3.2 Detailed Analysis

#### gix (gitoxide)

**Status:** Pure Rust implementation, rapidly maturing, used by cargo.

**Advantages:**
- Memory safe (no C dependencies)
- Excellent performance (often faster than libgit2)
- Cross-compilation friendly
- Active development by Byron (sponsored by GitButler)
- Used in production by cargo

**Worktree Operations with gix:**
```rust
use gix::{Repository, progress::Discard};
use std::path::Path;

struct GitOperations {
    repo: Repository,
}

impl GitOperations {
    fn open(path: &Path) -> Result<Self> {
        let repo = gix::open(path)?;
        Ok(Self { repo })
    }

    fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>> {
        let worktrees = self.repo.worktrees()?;
        let mut result = Vec::new();

        for wt in worktrees {
            let wt = wt?;
            result.push(WorktreeInfo {
                name: wt.id().to_string(),
                path: wt.base()?.to_path_buf(),
                branch: wt.head_ref()?.map(|r| r.name().to_string()),
                locked: wt.is_locked(),
            });
        }

        Ok(result)
    }

    fn create_worktree(&self, name: &str, branch: &str, path: &Path) -> Result<()> {
        // gix worktree creation API
        let reference = self.repo.find_reference(branch)?;
        self.repo.worktree_add(name, path, reference)?;
        Ok(())
    }

    fn remove_worktree(&self, name: &str, force: bool) -> Result<()> {
        let worktree = self.repo.find_worktree(name)?;
        if force || !worktree.is_locked() {
            worktree.remove()?;
        }
        Ok(())
    }

    fn clone_bare(url: &str, path: &Path) -> Result<Repository> {
        let mut prepare = gix::prepare_clone_bare(url, path)?;
        let (repo, _) = prepare.fetch_then_checkout(Discard, &std::sync::atomic::AtomicBool::new(false))?;
        Ok(repo)
    }

    fn get_common_dir(&self) -> PathBuf {
        self.repo.common_dir().to_path_buf()
    }

    fn is_worktree(&self) -> bool {
        self.repo.is_worktree()
    }
}

#[derive(Debug)]
struct WorktreeInfo {
    name: String,
    path: PathBuf,
    branch: Option<String>,
    locked: bool,
}
```

#### git2 (libgit2 bindings)

**Status:** Mature, complete feature set, C library dependency.

**Advantages:**
- Complete git feature coverage
- Well-documented
- Battle-tested in production

**Disadvantages:**
- C library dependency (libgit2)
- Cross-compilation complexity
- Potential memory safety issues at FFI boundary

**Use Case:** When you need features not yet in gix, or working with existing git2 codebases.

```rust
use git2::{Repository, WorktreePruneOptions};

fn git2_worktree_operations(repo_path: &Path) -> Result<()> {
    let repo = Repository::open(repo_path)?;

    // List worktrees
    let worktrees = repo.worktrees()?;
    for name in worktrees.iter() {
        if let Some(name) = name {
            let wt = repo.find_worktree(name)?;
            println!("Worktree: {} at {:?}", name, wt.path());
        }
    }

    // Create worktree
    let reference = repo.find_branch("feature", git2::BranchType::Local)?;
    repo.worktree("feature-wt", Path::new("./feature"), Some(&mut WorktreeAddOptions::new()))?;

    Ok(())
}
```

### 3.3 Recommendation

**Primary:** Use **gix** for new development - it's the future of Rust git tooling.

**Fallback:** Keep **git2** as optional dependency for features not yet in gix.

```toml
[dependencies]
gix = { version = "0.60", default-features = false, features = ["worktree", "clone", "status"] }

[dependencies.git2]
version = "0.18"
optional = true

[features]
libgit2 = ["git2"]
```

---

## 4. File System Operations

### 4.1 Core Crates

| Crate | Purpose | Cross-platform | Performance |
|-------|---------|----------------|-------------|
| **walkdir** | Directory traversal | Yes | Excellent |
| **fs_extra** | Copy/move operations | Yes | Good |
| **notify** | File watching | Yes | Excellent |
| **tempfile** | Temporary files | Yes | Good |
| **remove_dir_all** | Robust deletion | Yes | Good |

### 4.2 Implementation Patterns

#### Directory Traversal with walkdir

```rust
use walkdir::{WalkDir, DirEntry};
use std::path::Path;

fn find_config_files(root: &Path) -> Vec<PathBuf> {
    let config_patterns = [
        "CLAUDE.md",
        ".cursorrules",
        ".windsurfrules",
        ".github/copilot-instructions.md",
    ];

    WalkDir::new(root)
        .max_depth(3)
        .into_iter()
        .filter_entry(|e| !is_hidden(e) && !is_ignored(e))
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy();
            config_patterns.iter().any(|p| name == *p || e.path().ends_with(p))
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with('.') && s != "." && s != "..")
        .unwrap_or(false)
}

fn is_ignored(entry: &DirEntry) -> bool {
    let ignored = ["node_modules", "target", ".git", "__pycache__"];
    entry.file_name()
        .to_str()
        .map(|s| ignored.contains(&s))
        .unwrap_or(false)
}
```

#### File Operations with fs_extra

```rust
use fs_extra::{dir, file};
use std::path::Path;

fn setup_worktree_configs(container: &Path, worktree: &Path, tools: &[Tool]) -> Result<()> {
    let agentic_dir = container.join(".agentic");

    for tool in tools {
        match tool {
            Tool::Claude => {
                // Symlink .claude directory
                let src = agentic_dir.join("claude");
                let dst = worktree.join(".claude");
                create_symlink(&src, &dst)?;
            }
            Tool::Cursor => {
                // Copy .cursorrules (Cursor doesn't follow symlinks well)
                let src = agentic_dir.join("cursor/.cursorrules");
                let dst = worktree.join(".cursorrules");
                file::copy(&src, &dst, &file::CopyOptions::new())?;
            }
            // ...
        }
    }

    Ok(())
}

fn copy_template_directory(src: &Path, dst: &Path) -> Result<u64> {
    let options = dir::CopyOptions {
        overwrite: false,
        skip_exist: true,
        copy_inside: true,
        content_only: true,
        ..Default::default()
    };

    dir::copy(src, dst, &options)
}
```

#### File Watching with notify

```rust
use notify::{Watcher, RecursiveMode, Event, Config};
use std::sync::mpsc::channel;
use std::time::Duration;

fn watch_config_changes(paths: &[PathBuf]) -> Result<()> {
    let (tx, rx) = channel();

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res {
            tx.send(event).unwrap();
        }
    })?;

    for path in paths {
        watcher.watch(path, RecursiveMode::NonRecursive)?;
    }

    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(event) => {
                println!("Config changed: {:?}", event.paths);
                sync_configs(&event.paths)?;
            }
            Err(_) => continue,
        }
    }
}
```

### 4.3 Cross-Platform Path Handling

#### directories (dirs-next)

```rust
use directories::{ProjectDirs, UserDirs, BaseDirs};

fn get_config_paths() -> ConfigPaths {
    let project_dirs = ProjectDirs::from("com", "example", "repo-manager")
        .expect("Failed to determine project directories");

    ConfigPaths {
        // ~/.config/repo-manager/ (Linux)
        // ~/Library/Application Support/com.example.repo-manager/ (macOS)
        // C:\Users\<User>\AppData\Roaming\example\repo-manager\ (Windows)
        config_dir: project_dirs.config_dir().to_path_buf(),

        // ~/.local/share/repo-manager/ (Linux)
        // ~/Library/Application Support/com.example.repo-manager/ (macOS)
        // C:\Users\<User>\AppData\Roaming\example\repo-manager\data\ (Windows)
        data_dir: project_dirs.data_dir().to_path_buf(),

        // ~/.cache/repo-manager/ (Linux)
        // ~/Library/Caches/com.example.repo-manager/ (macOS)
        // C:\Users\<User>\AppData\Local\example\repo-manager\cache\ (Windows)
        cache_dir: project_dirs.cache_dir().to_path_buf(),
    }
}

fn get_user_dirs() -> Option<UserDirectories> {
    let user_dirs = UserDirs::new()?;

    Some(UserDirectories {
        home: user_dirs.home_dir().to_path_buf(),
        documents: user_dirs.document_dir().map(|p| p.to_path_buf()),
        downloads: user_dirs.download_dir().map(|p| p.to_path_buf()),
    })
}
```

---

## 5. Template/Scaffolding

### 5.1 Comparison Matrix

| Crate | Syntax | Performance | Features | Learning Curve |
|-------|--------|-------------|----------|----------------|
| **tera** | Jinja2-like | Good | Rich | Low |
| handlebars | Mustache-like | Good | Logic-less | Low |
| askama | Rust-native | Excellent | Compile-time | Medium |
| minijinja | Jinja2 | Excellent | Lightweight | Low |

### 5.2 Template Engine Selection

#### tera (Recommended)

**Status:** Mature, Jinja2-compatible, widely used in Rust web frameworks.

**Advantages:**
- Familiar Jinja2 syntax
- Rich feature set (filters, tests, inheritance)
- Good error messages
- Active maintenance

```rust
use tera::{Tera, Context};
use std::path::Path;

struct TemplateEngine {
    tera: Tera,
}

impl TemplateEngine {
    fn new(template_dir: &Path) -> Result<Self> {
        let pattern = template_dir.join("**/*").to_string_lossy().to_string();
        let tera = Tera::new(&pattern)?;
        Ok(Self { tera })
    }

    fn render_claude_md(&self, ctx: &ProjectContext) -> Result<String> {
        let mut context = Context::new();
        context.insert("project_name", &ctx.name);
        context.insert("tech_stack", &ctx.tech_stack);
        context.insert("coding_style", &ctx.coding_style);
        context.insert("commands", &ctx.commands);
        context.insert("architecture", &ctx.architecture);

        self.tera.render("CLAUDE.md.tera", &context)
            .map_err(Into::into)
    }

    fn render_cursorrules(&self, ctx: &ProjectContext) -> Result<String> {
        let mut context = Context::new();
        context.insert("project", ctx);

        self.tera.render("cursorrules.tera", &context)
            .map_err(Into::into)
    }
}
```

**Example Template (CLAUDE.md.tera):**
```jinja2
# {{ project_name }}

## Project Overview
{{ description | default(value="A software project.") }}

## Tech Stack
{% for tech in tech_stack %}
- {{ tech }}
{% endfor %}

## Code Style
{% for rule in coding_style %}
- {{ rule }}
{% endfor %}

## Commands
{% for cmd in commands %}
- **{{ cmd.name }}**: `{{ cmd.command }}`
{% endfor %}

## Architecture
{{ architecture | default(value="Standard project structure.") }}

{% if custom_sections %}
{% for section in custom_sections %}
## {{ section.title }}
{{ section.content }}
{% endfor %}
{% endif %}
```

#### handlebars

**Use Case:** When you want logic-less templates (separation of concerns).

```rust
use handlebars::Handlebars;

fn render_with_handlebars() -> Result<String> {
    let mut hb = Handlebars::new();
    hb.register_template_file("claude", "templates/CLAUDE.md.hbs")?;

    let data = serde_json::json!({
        "project_name": "My Project",
        "tech_stack": ["Rust", "TypeScript"],
    });

    hb.render("claude", &data).map_err(Into::into)
}
```

#### minijinja

**Use Case:** When you need lightweight Jinja2 without tera's full feature set.

```rust
use minijinja::{Environment, context};

fn render_with_minijinja() -> Result<String> {
    let mut env = Environment::new();
    env.add_template("claude", include_str!("templates/CLAUDE.md.j2"))?;

    let template = env.get_template("claude")?;
    template.render(context! {
        project_name => "My Project",
        tech_stack => ["Rust", "TypeScript"],
    }).map_err(Into::into)
}
```

### 5.3 Recommendation

Use **tera** for its:
- Familiar Jinja2 syntax (widely known)
- Template inheritance for shared base templates
- Rich filter and macro support
- Good error reporting

---

## 6. Process Execution

### 6.1 Comparison Matrix

| Crate | Async | Ergonomics | Shell Integration | Use Case |
|-------|-------|------------|-------------------|----------|
| std::process | No | Low | Manual | Basic |
| **tokio::process** | Yes | Medium | Manual | Async apps |
| **xshell** | No | High | Built-in | Scripts |
| duct | No | High | Piping | Pipelines |

### 6.2 Implementation Patterns

#### tokio::process for Async Operations

```rust
use tokio::process::Command;
use std::process::Stdio;

async fn run_git_command(args: &[&str], cwd: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .wait_with_output()
        .await?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("Git command failed: {}", stderr))
    }
}

async fn parallel_worktree_operations(worktrees: &[Worktree]) -> Result<()> {
    let handles: Vec<_> = worktrees
        .iter()
        .map(|wt| {
            let path = wt.path.clone();
            tokio::spawn(async move {
                run_git_command(&["fetch", "--all"], &path).await
            })
        })
        .collect();

    for handle in handles {
        handle.await??;
    }

    Ok(())
}
```

#### xshell for Scripted Operations

```rust
use xshell::{cmd, Shell};

fn setup_container(sh: &Shell, url: &str, name: &str) -> Result<()> {
    // xshell provides ergonomic command building
    sh.create_dir(name)?;
    sh.change_dir(name);

    // Clone as bare
    cmd!(sh, "git clone --bare {url} .git").run()?;

    // Configure for worktrees
    cmd!(sh, "git config core.bare false").run()?;

    // Create initial worktree
    cmd!(sh, "git worktree add main main").run()?;

    // Create .agentic directory
    sh.create_dir(".agentic")?;
    sh.create_dir(".agentic/claude")?;
    sh.create_dir(".agentic/cursor")?;
    sh.create_dir(".agentic/shared")?;

    Ok(())
}

fn sync_worktree_configs(sh: &Shell, container: &Path, worktree: &Path) -> Result<()> {
    let agentic = container.join(".agentic");

    // Create symlinks (Unix) or junctions (Windows)
    #[cfg(unix)]
    {
        let claude_src = agentic.join("claude");
        let claude_dst = worktree.join(".claude");
        cmd!(sh, "ln -sf {claude_src} {claude_dst}").run()?;
    }

    #[cfg(windows)]
    {
        let claude_src = agentic.join("claude");
        let claude_dst = worktree.join(".claude");
        cmd!(sh, "mklink /J {claude_dst} {claude_src}").run()?;
    }

    Ok(())
}
```

### 6.3 Recommendation

- Use **tokio::process** for async operations (parallel git fetches, etc.)
- Use **xshell** for setup scripts and one-off operations
- Combine both based on context

---

## 7. Error Handling

### 7.1 Comparison Matrix

| Crate | Purpose | User-Friendly | Backtraces | Use Case |
|-------|---------|---------------|------------|----------|
| **miette** | Diagnostics | Excellent | Yes | CLI apps |
| **thiserror** | Error types | N/A | N/A | Libraries |
| **anyhow** | Error context | Good | Yes | Applications |
| eyre | Error handling | Good | Yes | Applications |

### 7.2 Recommended Pattern: miette + thiserror

#### Define Errors with thiserror

```rust
use thiserror::Error;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum RepoManagerError {
    #[error("Repository not found at {path}")]
    RepoNotFound { path: PathBuf },

    #[error("Worktree '{name}' already exists")]
    WorktreeExists { name: String },

    #[error("Configuration file not found: {path}")]
    ConfigNotFound { path: PathBuf },

    #[error("Invalid container structure: {reason}")]
    InvalidContainer { reason: String },

    #[error("Git operation failed: {operation}")]
    GitError {
        operation: String,
        #[source]
        source: gix::Error,
    },

    #[error("Template rendering failed: {template}")]
    TemplateError {
        template: String,
        #[source]
        source: tera::Error,
    },

    #[error("I/O error: {context}")]
    IoError {
        context: String,
        #[source]
        source: std::io::Error,
    },
}
```

#### User-Friendly Output with miette

```rust
use miette::{Diagnostic, SourceSpan, NamedSource};

#[derive(Error, Debug, Diagnostic)]
#[error("Configuration parsing failed")]
#[diagnostic(
    code(repo_manager::config::parse_error),
    help("Check that your configuration file is valid TOML")
)]
pub struct ConfigParseError {
    #[source_code]
    pub src: NamedSource<String>,

    #[label("error occurred here")]
    pub span: SourceSpan,

    #[source]
    pub source: toml::de::Error,
}

// Usage in main
fn main() -> miette::Result<()> {
    // miette will format errors beautifully
    let config = load_config()?;
    run(config)?;
    Ok(())
}

// Example error output:
// Error: Configuration parsing failed
//
//   × expected a value
//    ╭─[config.toml:15:1]
//  15 │ invalid_key =
//    ·               ▲
//    ·               ╰── error occurred here
//    ╰────
//   help: Check that your configuration file is valid TOML
```

#### Context-Rich Errors with anyhow (Internal)

```rust
use anyhow::{Context, Result};

fn clone_repository(url: &str, path: &Path) -> Result<()> {
    std::fs::create_dir_all(path)
        .with_context(|| format!("Failed to create directory: {}", path.display()))?;

    gix::prepare_clone(url, path)
        .with_context(|| format!("Failed to clone repository: {}", url))?
        .fetch_then_checkout(/* ... */)
        .with_context(|| "Failed to complete clone operation")?;

    Ok(())
}
```

### 7.3 Recommendation

- **miette** for user-facing error display in CLI
- **thiserror** for defining error types in library code
- **anyhow** for internal error propagation with context

---

## 8. Architecture Patterns

### 8.1 Plugin System Patterns

#### Trait-Based Static Dispatch (Recommended)

```rust
/// Provider trait for agentic tool integrations
pub trait AgenticProvider: Send + Sync {
    /// Provider name
    fn name(&self) -> &'static str;

    /// Initialize configuration in a directory
    fn init(&self, path: &Path, config: &ToolConfig) -> Result<()>;

    /// Get configuration file paths for this provider
    fn config_paths(&self, root: &Path) -> Vec<PathBuf>;

    /// Sync configuration to a worktree
    fn sync_to_worktree(&self, source: &Path, worktree: &Path) -> Result<()>;

    /// Validate existing configuration
    fn validate(&self, path: &Path) -> Result<ValidationResult>;
}

/// Claude Code provider implementation
pub struct ClaudeProvider {
    templates: TemplateEngine,
}

impl AgenticProvider for ClaudeProvider {
    fn name(&self) -> &'static str {
        "claude"
    }

    fn init(&self, path: &Path, config: &ToolConfig) -> Result<()> {
        let claude_dir = path.join(".claude");
        std::fs::create_dir_all(&claude_dir)?;

        // Create CLAUDE.md
        let content = self.templates.render_claude_md(config)?;
        std::fs::write(path.join("CLAUDE.md"), content)?;

        // Create settings.json
        let settings = serde_json::to_string_pretty(&config.settings)?;
        std::fs::write(claude_dir.join("settings.json"), settings)?;

        Ok(())
    }

    fn config_paths(&self, root: &Path) -> Vec<PathBuf> {
        vec![
            root.join("CLAUDE.md"),
            root.join(".claude"),
        ]
    }

    fn sync_to_worktree(&self, source: &Path, worktree: &Path) -> Result<()> {
        // Symlink .claude directory
        let src = source.join(".agentic/claude");
        let dst = worktree.join(".claude");
        create_symlink(&src, &dst)?;

        // Symlink CLAUDE.md
        let src = source.join(".agentic/CLAUDE.md");
        let dst = worktree.join("CLAUDE.md");
        create_symlink(&src, &dst)?;

        Ok(())
    }

    fn validate(&self, path: &Path) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        if !path.join("CLAUDE.md").exists() {
            result.add_warning("CLAUDE.md not found");
        }

        if !path.join(".claude").exists() {
            result.add_warning(".claude directory not found");
        }

        Ok(result)
    }
}

/// Provider registry for runtime lookup
pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn AgenticProvider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            providers: HashMap::new(),
        };

        // Register built-in providers
        registry.register(Box::new(ClaudeProvider::new()));
        registry.register(Box::new(CursorProvider::new()));
        registry.register(Box::new(CopilotProvider::new()));
        registry.register(Box::new(GeminiProvider::new()));

        registry
    }

    pub fn register(&mut self, provider: Box<dyn AgenticProvider>) {
        self.providers.insert(provider.name().to_string(), provider);
    }

    pub fn get(&self, name: &str) -> Option<&dyn AgenticProvider> {
        self.providers.get(name).map(|p| p.as_ref())
    }

    pub fn all(&self) -> impl Iterator<Item = &dyn AgenticProvider> {
        self.providers.values().map(|p| p.as_ref())
    }
}
```

#### Enum-Based Static Dispatch (Simple Alternative)

```rust
#[derive(Clone, Copy, Debug)]
pub enum Tool {
    Claude,
    Cursor,
    Windsurf,
    Copilot,
    Gemini,
}

impl Tool {
    pub fn config_files(&self) -> &'static [&'static str] {
        match self {
            Tool::Claude => &["CLAUDE.md", ".claude/"],
            Tool::Cursor => &[".cursorrules", ".cursor/"],
            Tool::Windsurf => &[".windsurfrules", ".windsurf/"],
            Tool::Copilot => &[".github/copilot-instructions.md"],
            Tool::Gemini => &[".gemini/"],
        }
    }

    pub fn init(&self, path: &Path, ctx: &ProjectContext) -> Result<()> {
        match self {
            Tool::Claude => init_claude(path, ctx),
            Tool::Cursor => init_cursor(path, ctx),
            Tool::Windsurf => init_windsurf(path, ctx),
            Tool::Copilot => init_copilot(path, ctx),
            Tool::Gemini => init_gemini(path, ctx),
        }
    }
}
```

### 8.2 Dynamic Dispatch with dyn Traits

Use when:
- Plugin count is unknown at compile time
- External plugin loading needed
- Highly extensible architecture required

```rust
pub struct DynamicProviderRegistry {
    providers: Vec<Box<dyn AgenticProvider>>,
}

impl DynamicProviderRegistry {
    pub fn load_from_directory(&mut self, path: &Path) -> Result<()> {
        // Load dynamic libraries (.so/.dll) implementing AgenticProvider
        // Requires unsafe FFI
        unimplemented!("Dynamic loading requires libloading crate")
    }
}
```

### 8.3 Recommendation

For the repository manager:
1. Use **trait-based static dispatch** for built-in providers
2. Keep door open for dynamic plugins via `Box<dyn AgenticProvider>`
3. Use enum dispatch for simple, known-at-compile-time cases

---

## 9. Performance Considerations

### 9.1 Parallel File Operations

```rust
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

fn parallel_sync(worktrees: &[PathBuf], configs: &[ConfigFile]) -> Result<SyncResult> {
    let success_count = AtomicUsize::new(0);
    let error_count = AtomicUsize::new(0);

    worktrees.par_iter().for_each(|wt| {
        match sync_worktree(wt, configs) {
            Ok(_) => { success_count.fetch_add(1, Ordering::Relaxed); }
            Err(_) => { error_count.fetch_add(1, Ordering::Relaxed); }
        }
    });

    Ok(SyncResult {
        success: success_count.load(Ordering::Relaxed),
        errors: error_count.load(Ordering::Relaxed),
    })
}
```

### 9.2 Async vs Sync Decision Matrix

| Operation | Recommendation | Reason |
|-----------|----------------|--------|
| Git clone | Async | Network-bound |
| Git fetch | Async | Network-bound |
| File copy | Sync + rayon | CPU/IO bound, rayon handles parallelism |
| Template render | Sync | CPU-bound, fast |
| Config parse | Sync | Fast, small files |
| Process execution | Async | Waiting on external process |
| User prompts | Sync | Interactive, blocking |

### 9.3 Memory-Mapped Files

For large file operations:

```rust
use memmap2::Mmap;

fn read_large_config(path: &Path) -> Result<String> {
    let file = std::fs::File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let content = std::str::from_utf8(&mmap)?;
    Ok(content.to_string())
}
```

**When to use:**
- Files > 1MB
- Frequent random access
- Memory-constrained environments

**When NOT to use:**
- Small config files (< 100KB)
- Sequential reads
- Files that may be modified during read

---

## 10. Recommended Cargo.toml

```toml
[package]
name = "repo-manager"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
description = "Orchestrate agentic coding environments"
license = "MIT OR Apache-2.0"
repository = "https://github.com/example/repo-manager"

[dependencies]
# CLI Framework
clap = { version = "4.5", features = ["derive", "env", "string"] }
clap_complete = "4.5"

# Interactive Prompts
dialoguer = { version = "0.11", features = ["fuzzy-select"] }
indicatif = "0.17"
console = "0.15"

# Configuration
figment = { version = "0.10", features = ["toml", "json", "yaml", "env"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Git Operations
gix = { version = "0.60", default-features = false, features = [
    "worktree",
    "clone",
    "status",
    "revision",
    "index",
] }

# File System
walkdir = "2.5"
fs_extra = "1.3"
notify = "6.1"
tempfile = "3.10"
remove_dir_all = "0.8"

# Cross-platform Paths
directories = "5.0"

# Templates
tera = "1.19"

# Process Execution
xshell = "0.2"

# Async Runtime
tokio = { version = "1.36", features = ["full"] }

# Error Handling
miette = { version = "7.2", features = ["fancy"] }
thiserror = "1.0"
anyhow = "1.0"

# Parallel Processing
rayon = "1.9"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[dev-dependencies]
assert_fs = "1.1"
predicates = "3.1"
assert_cmd = "2.0"
insta = { version = "1.38", features = ["yaml"] }

[features]
default = []
# Enable libgit2 fallback
libgit2 = ["gix/git2"]

[profile.release]
lto = true
codegen-units = 1
strip = true
panic = "abort"

[profile.dev]
# Faster builds in development
opt-level = 0
debug = true

[profile.dev.package."*"]
# Optimize dependencies even in dev builds
opt-level = 2
```

---

## 11. Testing Strategy

### 11.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_loading() {
        let config = r#"
            [global]
            default_branch = "main"

            [tools.claude]
            enabled = true
        "#;

        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("config.toml"), config).unwrap();

        let loaded = load_config_from(temp.path()).unwrap();
        assert_eq!(loaded.global.default_branch, "main");
        assert!(loaded.tools.claude.unwrap().enabled);
    }

    #[test]
    fn test_provider_registration() {
        let registry = ProviderRegistry::new();
        assert!(registry.get("claude").is_some());
        assert!(registry.get("cursor").is_some());
        assert!(registry.get("unknown").is_none());
    }
}
```

### 11.2 Integration Tests

```rust
// tests/integration/cli.rs
use assert_cmd::Command;
use predicates::prelude::*;
use assert_fs::prelude::*;

#[test]
fn test_init_command() {
    let temp = assert_fs::TempDir::new().unwrap();

    Command::cargo_bin("repo-manager")
        .unwrap()
        .args(["init", "--claude", "--worktrees"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized"));

    temp.child(".agentic").assert(predicate::path::is_dir());
    temp.child(".agentic/claude").assert(predicate::path::is_dir());
}

#[test]
fn test_create_worktree() {
    let temp = setup_test_container();

    Command::cargo_bin("repo-manager")
        .unwrap()
        .args(["create", "feature-test", "--from", "main"])
        .current_dir(temp.path())
        .assert()
        .success();

    temp.child("worktrees/feature-test").assert(predicate::path::is_dir());
    temp.child("worktrees/feature-test/.claude").assert(predicate::path::is_symlink());
}
```

### 11.3 Snapshot Tests

```rust
use insta::assert_yaml_snapshot;

#[test]
fn test_template_rendering() {
    let ctx = ProjectContext {
        name: "test-project".into(),
        tech_stack: vec!["Rust".into(), "TypeScript".into()],
        // ...
    };

    let engine = TemplateEngine::new("templates/").unwrap();
    let output = engine.render_claude_md(&ctx).unwrap();

    assert_yaml_snapshot!(output);
}
```

---

## 12. Summary and Recommendations

### Recommended Stack

| Category | Primary | Alternative |
|----------|---------|-------------|
| CLI Framework | clap v4 | - |
| Interactive | dialoguer + indicatif | inquire |
| Config | figment | config-rs |
| Git | gix | git2 (fallback) |
| File System | walkdir + fs_extra | - |
| Templates | tera | minijinja |
| Paths | directories | dirs |
| Process | tokio::process + xshell | - |
| Errors | miette + thiserror | anyhow + eyre |
| Async | tokio | - |
| Parallel | rayon | - |

### Architecture Decisions

1. **Use trait-based providers** for extensibility without runtime overhead
2. **Prefer static dispatch** for built-in tools, support dynamic for plugins
3. **Async for I/O-bound operations**, sync + rayon for CPU-bound
4. **Layer errors**: thiserror for types, miette for display, anyhow for context
5. **Hierarchical configuration** with figment for system -> user -> project -> env

### Cross-Platform Considerations

1. **Symlinks**: Use `std::os::unix::fs::symlink` / `std::os::windows::fs::symlink_dir`
2. **Paths**: Always use `directories` crate for XDG/Windows paths
3. **Line endings**: Use `\n` internally, convert on output if needed
4. **Process execution**: Test on all platforms, handle command differences

### Next Steps

1. Scaffold project structure with recommended dependencies
2. Implement core CLI with clap
3. Build provider trait and registry
4. Add Claude and Cursor providers first
5. Implement worktree management with gix
6. Add interactive setup wizard
7. Test cross-platform (Windows, macOS, Linux)

---

*Document Status: Complete*
*Last Updated: 2026-01-23*
*Branch: research-docs*
