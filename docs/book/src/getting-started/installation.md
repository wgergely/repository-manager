# Installation

Repository Manager is distributed as a Rust binary called `repo`.

## Prerequisites

- [Rust](https://rustup.rs/) 1.70 or later (for building from source)
- Git

## From Source

Clone the repository and install the `repo` binary:

```bash
git clone https://github.com/wgergely/repository-manager
cd repository-manager
cargo install --path crates/repo-cli
```

After installation, the `repo` binary will be available on your `PATH`.

## Build Locally (Without Installing)

If you want to build without adding to your PATH:

```bash
git clone https://github.com/wgergely/repository-manager
cd repository-manager
cargo build --release
./target/release/repo --help
```

## Verify the Installation

```bash
repo --version
repo --help
```

## Shell Completions

Repository Manager can generate shell completions for Bash, Zsh, Fish, and PowerShell.

```bash
# Bash
repo completions bash > ~/.local/share/bash-completion/completions/repo

# Zsh
repo completions zsh > ~/.zfunc/_repo

# Fish
repo completions fish > ~/.config/fish/completions/repo.fish
```

After adding completions, restart your shell or source the completion file.

## Next Step

Once installed, follow the [Quick Start](quick-start.md) guide to initialize your first project.
