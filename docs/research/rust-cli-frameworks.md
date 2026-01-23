# Rust CLI Frameworks

Evaluation of CLI frameworks for building the repo-manager tool.

## Recommendation: clap v4

**clap** is the industry standard, actively maintained by the Rust CLI Working Group.

## Comparison Matrix

| Crate | Version | Derive Macros | Subcommands | Completions | Ecosystem |
|-------|---------|---------------|-------------|-------------|-----------|
| **clap** | 4.x | Yes | Yes | Yes | Dominant |
| argh | 0.1.x | Yes | Yes | No | Minimal |
| structopt | 0.3.x | Yes | Yes | Limited | Deprecated |
| pico-args | 0.5.x | No | Manual | No | Minimal |

## clap Example

```rust
use clap::{Parser, Subcommand, Args};

#[derive(Parser)]
#[command(name = "repo-manager")]
#[command(about = "Orchestrate agentic coding environments")]
#[command(version, author)]
struct Cli {
    #[arg(short, long, global = true)]
    verbose: bool,

    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init(InitArgs),
    Create(CreateArgs),
    Sync(SyncArgs),
}

#[derive(Args)]
struct InitArgs {
    #[arg(short, long)]
    url: Option<String>,

    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(long)]
    claude: bool,

    #[arg(long)]
    worktrees: bool,
}
```

## Shell Completions

```rust
use clap_complete::{generate, Shell};

fn generate_completions(shell: Shell) {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "repo-manager", &mut std::io::stdout());
}
```

## Interactive Prompts: dialoguer + indicatif

For setup wizards:

```rust
use dialoguer::{theme::ColorfulTheme, Select, Input, Confirm, MultiSelect};
use indicatif::{ProgressBar, ProgressStyle};

fn setup_wizard() -> Result<Config> {
    let theme = ColorfulTheme::default();

    let tools = vec!["Claude Code", "Cursor", "Windsurf", "Copilot"];
    let selections = MultiSelect::with_theme(&theme)
        .with_prompt("Select agentic tools to configure")
        .items(&tools)
        .interact()?;

    let project_name: String = Input::with_theme(&theme)
        .with_prompt("Project name")
        .default("my-project".into())
        .interact_text()?;

    Ok(Config { /* ... */ })
}
```

Progress bars:

```rust
fn clone_repository(url: &str) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    pb.set_message("Cloning repository...");
    pb.enable_steady_tick(Duration::from_millis(100));

    // Perform clone
    git_clone(url)?;

    pb.finish_with_message("Repository cloned successfully");
    Ok(())
}
```

## Cargo Dependencies

```toml
[dependencies]
clap = { version = "4.5", features = ["derive", "env", "string"] }
clap_complete = "4.5"
dialoguer = { version = "0.11", features = ["fuzzy-select"] }
indicatif = "0.17"
console = "0.15"
```

---

*Last updated: 2026-01-23*
*Status: Complete*
