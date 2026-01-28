# Phase C: Preset Providers Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Date:** 2026-01-27
**Priority:** Medium
**Estimated Tasks:** 6
**Dependencies:** Phase B (Core Completion)

---

## Goal

Expand the preset system with providers for common development environments: Conda, Node.js, and Rust. Each provider follows the established `PresetProvider` trait pattern.

---

## Prerequisites

- Phase B completed
- All tests passing: `cargo test --workspace`
- Understanding of `repo-presets` crate structure and `UvProvider` implementation

---

## Current State

The `repo-presets` crate has:
- `PresetProvider` trait (check/apply pattern)
- `UvProvider` for Python UV environments
- `Context` struct for provider execution

**Missing:**
- Conda/Mamba support for Python
- Node.js/npm/pnpm support
- Rust/Cargo support

---

## Task C.1: Implement CondaProvider

**Files:**
- Create: `crates/repo-presets/src/python/conda.rs`
- Modify: `crates/repo-presets/src/python/mod.rs`
- Modify: `crates/repo-presets/src/lib.rs`

**Step 1: Create conda.rs**

```rust
// crates/repo-presets/src/python/conda.rs
//! Conda/Mamba environment provider

use crate::context::Context;
use crate::error::{Error, Result};
use crate::provider::{ApplyReport, CheckReport, PresetProvider, PresetStatus};
use async_trait::async_trait;
use std::process::Command;

pub struct CondaProvider {
    /// Use mamba if available (faster)
    prefer_mamba: bool,
    /// Python version to install
    python_version: Option<String>,
}

impl CondaProvider {
    pub fn new() -> Self {
        Self {
            prefer_mamba: true,
            python_version: None,
        }
    }

    pub fn with_python_version(mut self, version: &str) -> Self {
        self.python_version = Some(version.to_string());
        self
    }

    fn conda_command(&self) -> &str {
        if self.prefer_mamba && which::which("mamba").is_ok() {
            "mamba"
        } else if which::which("conda").is_ok() {
            "conda"
        } else {
            "conda" // Will fail with helpful error
        }
    }

    fn env_path(&self, ctx: &Context) -> std::path::PathBuf {
        ctx.working_dir().join(".conda")
    }

    fn env_exists(&self, ctx: &Context) -> bool {
        self.env_path(ctx).join("conda-meta").exists()
    }
}

impl Default for CondaProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PresetProvider for CondaProvider {
    fn id(&self) -> &str {
        "env:python-conda"
    }

    async fn check(&self, ctx: &Context) -> Result<CheckReport> {
        // Check if conda is available
        let cmd = self.conda_command();
        if which::which(cmd).is_err() {
            return Ok(CheckReport {
                status: PresetStatus::Broken,
                details: vec![format!("{} not found in PATH", cmd)],
                remedial_action: None,
            });
        }

        // Check if environment exists
        if !self.env_exists(ctx) {
            return Ok(CheckReport {
                status: PresetStatus::Missing,
                details: vec!["Conda environment not found".to_string()],
                remedial_action: Some("Run 'repo sync' to create environment".to_string()),
            });
        }

        // Check Python version if specified
        if let Some(required_version) = &self.python_version {
            let python_path = self.env_path(ctx).join("bin/python");
            let output = Command::new(&python_path)
                .args(["--version"])
                .output();

            match output {
                Ok(output) => {
                    let version = String::from_utf8_lossy(&output.stdout);
                    if !version.contains(required_version) {
                        return Ok(CheckReport {
                            status: PresetStatus::Drifted,
                            details: vec![format!(
                                "Python version mismatch: expected {}, got {}",
                                required_version,
                                version.trim()
                            )],
                            remedial_action: Some("Run 'repo fix' to recreate environment".to_string()),
                        });
                    }
                }
                Err(e) => {
                    return Ok(CheckReport {
                        status: PresetStatus::Broken,
                        details: vec![format!("Failed to check Python version: {}", e)],
                        remedial_action: Some("Run 'repo fix' to repair environment".to_string()),
                    });
                }
            }
        }

        Ok(CheckReport {
            status: PresetStatus::Healthy,
            details: vec!["Conda environment is healthy".to_string()],
            remedial_action: None,
        })
    }

    async fn apply(&self, ctx: &Context) -> Result<ApplyReport> {
        let mut actions = Vec::new();
        let cmd = self.conda_command();

        // Check if conda is available
        if which::which(cmd).is_err() {
            return Err(Error::Provider(format!(
                "{} not found. Install conda or mamba first.",
                cmd
            )));
        }

        let env_path = self.env_path(ctx);

        // Create environment if it doesn't exist
        if !self.env_exists(ctx) {
            let mut args = vec![
                "create",
                "-p",
                env_path.to_str().unwrap(),
                "-y",
            ];

            if let Some(version) = &self.python_version {
                args.push("python");
                args.push(version);
            } else {
                args.push("python");
            }

            let output = Command::new(cmd)
                .args(&args)
                .current_dir(ctx.working_dir())
                .output()
                .map_err(|e| Error::Provider(format!("Failed to create conda env: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(Error::Provider(format!("Conda create failed: {}", stderr)));
            }

            actions.push(format!("Created conda environment at {}", env_path.display()));
        }

        // Create activation script
        let activate_script = ctx.working_dir().join(".repository/activate.sh");
        let script_content = format!(
            r#"#!/bin/bash
# Activate conda environment
eval "$(conda shell.bash hook)"
conda activate "{}"
"#,
            env_path.display()
        );

        std::fs::write(&activate_script, script_content)
            .map_err(|e| Error::Provider(format!("Failed to write activate script: {}", e)))?;

        actions.push("Created activation script".to_string());

        Ok(ApplyReport {
            success: true,
            actions,
            errors: Vec::new(),
        })
    }
}
```

**Step 2: Add which dependency**

```bash
cargo add which -p repo-presets
```

**Step 3: Export from python/mod.rs**

```rust
mod conda;
pub use conda::CondaProvider;
```

**Step 4: Export from lib.rs**

```rust
pub use python::{CondaProvider, UvProvider};
```

**Step 5: Add tests**

Create `crates/repo-presets/tests/conda_tests.rs`:

```rust
use repo_presets::{CondaProvider, Context, PresetProvider};
use tempfile::tempdir;

#[tokio::test]
async fn test_conda_provider_check_missing() {
    let dir = tempdir().unwrap();
    let ctx = Context::new(dir.path().to_path_buf());
    let provider = CondaProvider::new();

    let report = provider.check(&ctx).await.unwrap();
    // Will be Missing or Broken depending on conda availability
    assert!(matches!(
        report.status,
        repo_presets::PresetStatus::Missing | repo_presets::PresetStatus::Broken
    ));
}
```

**Step 6: Commit**

```bash
git add crates/repo-presets/
git commit -m "feat(repo-presets): implement CondaProvider

Adds conda/mamba environment management with:
- Auto-detection of mamba vs conda
- Python version specification
- Activation script generation

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task C.2: Implement NodeProvider

**Files:**
- Create: `crates/repo-presets/src/node/mod.rs`
- Create: `crates/repo-presets/src/node/npm.rs`
- Modify: `crates/repo-presets/src/lib.rs`

**Step 1: Create node module structure**

```rust
// crates/repo-presets/src/node/mod.rs
mod npm;

pub use npm::NodeProvider;
```

**Step 2: Implement NodeProvider**

```rust
// crates/repo-presets/src/node/npm.rs
//! Node.js environment provider

use crate::context::Context;
use crate::error::{Error, Result};
use crate::provider::{ApplyReport, CheckReport, PresetProvider, PresetStatus};
use async_trait::async_trait;
use std::process::Command;

/// Package manager preference
#[derive(Debug, Clone, Copy, Default)]
pub enum PackageManager {
    #[default]
    Npm,
    Pnpm,
    Yarn,
}

pub struct NodeProvider {
    package_manager: PackageManager,
    node_version: Option<String>,
}

impl NodeProvider {
    pub fn new() -> Self {
        Self {
            package_manager: PackageManager::default(),
            node_version: None,
        }
    }

    pub fn with_package_manager(mut self, pm: PackageManager) -> Self {
        self.package_manager = pm;
        self
    }

    pub fn with_node_version(mut self, version: &str) -> Self {
        self.node_version = Some(version.to_string());
        self
    }

    fn pm_command(&self) -> &str {
        match self.package_manager {
            PackageManager::Npm => "npm",
            PackageManager::Pnpm => "pnpm",
            PackageManager::Yarn => "yarn",
        }
    }

    fn has_package_json(&self, ctx: &Context) -> bool {
        ctx.working_dir().join("package.json").exists()
    }

    fn has_node_modules(&self, ctx: &Context) -> bool {
        ctx.working_dir().join("node_modules").exists()
    }

    fn has_lockfile(&self, ctx: &Context) -> bool {
        let dir = ctx.working_dir();
        dir.join("package-lock.json").exists()
            || dir.join("pnpm-lock.yaml").exists()
            || dir.join("yarn.lock").exists()
    }
}

impl Default for NodeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PresetProvider for NodeProvider {
    fn id(&self) -> &str {
        "env:node"
    }

    async fn check(&self, ctx: &Context) -> Result<CheckReport> {
        // Check if Node is available
        if which::which("node").is_err() {
            return Ok(CheckReport {
                status: PresetStatus::Broken,
                details: vec!["Node.js not found in PATH".to_string()],
                remedial_action: Some("Install Node.js first".to_string()),
            });
        }

        // Check package manager
        let pm = self.pm_command();
        if which::which(pm).is_err() {
            return Ok(CheckReport {
                status: PresetStatus::Broken,
                details: vec![format!("{} not found in PATH", pm)],
                remedial_action: Some(format!("Install {} first", pm)),
            });
        }

        // Check for package.json
        if !self.has_package_json(ctx) {
            return Ok(CheckReport {
                status: PresetStatus::Missing,
                details: vec!["No package.json found".to_string()],
                remedial_action: Some("Run 'npm init' or 'repo sync' to create package.json".to_string()),
            });
        }

        // Check for node_modules
        if !self.has_node_modules(ctx) {
            return Ok(CheckReport {
                status: PresetStatus::Missing,
                details: vec!["node_modules not found".to_string()],
                remedial_action: Some("Run 'npm install' or 'repo sync'".to_string()),
            });
        }

        Ok(CheckReport {
            status: PresetStatus::Healthy,
            details: vec!["Node.js environment is healthy".to_string()],
            remedial_action: None,
        })
    }

    async fn apply(&self, ctx: &Context) -> Result<ApplyReport> {
        let mut actions = Vec::new();
        let pm = self.pm_command();

        // Check if Node and package manager are available
        if which::which("node").is_err() {
            return Err(Error::Provider("Node.js not found".to_string()));
        }

        if which::which(pm).is_err() {
            return Err(Error::Provider(format!("{} not found", pm)));
        }

        // Create package.json if missing
        if !self.has_package_json(ctx) {
            let package_json = serde_json::json!({
                "name": ctx.working_dir().file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("project"),
                "version": "0.1.0",
                "private": true
            });

            let path = ctx.working_dir().join("package.json");
            std::fs::write(&path, serde_json::to_string_pretty(&package_json)?)
                .map_err(|e| Error::Provider(format!("Failed to write package.json: {}", e)))?;

            actions.push("Created package.json".to_string());
        }

        // Run install if node_modules missing
        if !self.has_node_modules(ctx) {
            let output = Command::new(pm)
                .arg("install")
                .current_dir(ctx.working_dir())
                .output()
                .map_err(|e| Error::Provider(format!("Failed to run {} install: {}", pm, e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(Error::Provider(format!("{} install failed: {}", pm, stderr)));
            }

            actions.push(format!("Ran {} install", pm));
        }

        Ok(ApplyReport {
            success: true,
            actions,
            errors: Vec::new(),
        })
    }
}
```

**Step 3-6:** Export, test, commit (similar pattern)

---

## Task C.3: Implement RustProvider

**Files:**
- Create: `crates/repo-presets/src/rust_preset/mod.rs`
- Create: `crates/repo-presets/src/rust_preset/cargo.rs`
- Modify: `crates/repo-presets/src/lib.rs`

**Step 1: Create rust_preset module**

```rust
// crates/repo-presets/src/rust_preset/cargo.rs
//! Rust/Cargo environment provider

use crate::context::Context;
use crate::error::{Error, Result};
use crate::provider::{ApplyReport, CheckReport, PresetProvider, PresetStatus};
use async_trait::async_trait;
use std::process::Command;

pub struct RustProvider {
    /// Toolchain to use (stable, nightly, etc.)
    toolchain: String,
    /// Additional components to install
    components: Vec<String>,
}

impl RustProvider {
    pub fn new() -> Self {
        Self {
            toolchain: "stable".to_string(),
            components: vec!["clippy".to_string(), "rustfmt".to_string()],
        }
    }

    pub fn with_toolchain(mut self, toolchain: &str) -> Self {
        self.toolchain = toolchain.to_string();
        self
    }

    pub fn with_component(mut self, component: &str) -> Self {
        self.components.push(component.to_string());
        self
    }

    fn has_cargo_toml(&self, ctx: &Context) -> bool {
        ctx.working_dir().join("Cargo.toml").exists()
    }
}

impl Default for RustProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PresetProvider for RustProvider {
    fn id(&self) -> &str {
        "env:rust"
    }

    async fn check(&self, ctx: &Context) -> Result<CheckReport> {
        // Check if rustup is available
        if which::which("rustup").is_err() {
            return Ok(CheckReport {
                status: PresetStatus::Broken,
                details: vec!["rustup not found in PATH".to_string()],
                remedial_action: Some("Install rustup from https://rustup.rs".to_string()),
            });
        }

        // Check if cargo is available
        if which::which("cargo").is_err() {
            return Ok(CheckReport {
                status: PresetStatus::Broken,
                details: vec!["cargo not found in PATH".to_string()],
                remedial_action: Some("Run 'rustup install stable'".to_string()),
            });
        }

        // Check for Cargo.toml
        if !self.has_cargo_toml(ctx) {
            return Ok(CheckReport {
                status: PresetStatus::Missing,
                details: vec!["No Cargo.toml found".to_string()],
                remedial_action: Some("Run 'cargo init' or 'repo sync'".to_string()),
            });
        }

        // Check toolchain
        let output = Command::new("rustup")
            .args(["show", "active-toolchain"])
            .output();

        if let Ok(output) = output {
            let toolchain = String::from_utf8_lossy(&output.stdout);
            if !toolchain.contains(&self.toolchain) {
                return Ok(CheckReport {
                    status: PresetStatus::Drifted,
                    details: vec![format!(
                        "Toolchain mismatch: expected {}, active is {}",
                        self.toolchain,
                        toolchain.trim()
                    )],
                    remedial_action: Some("Run 'repo fix' to switch toolchain".to_string()),
                });
            }
        }

        Ok(CheckReport {
            status: PresetStatus::Healthy,
            details: vec!["Rust environment is healthy".to_string()],
            remedial_action: None,
        })
    }

    async fn apply(&self, ctx: &Context) -> Result<ApplyReport> {
        let mut actions = Vec::new();

        // Check if rustup is available
        if which::which("rustup").is_err() {
            return Err(Error::Provider("rustup not found".to_string()));
        }

        // Ensure toolchain is installed
        let output = Command::new("rustup")
            .args(["toolchain", "install", &self.toolchain])
            .output()
            .map_err(|e| Error::Provider(format!("Failed to install toolchain: {}", e)))?;

        if output.status.success() {
            actions.push(format!("Installed {} toolchain", self.toolchain));
        }

        // Install components
        for component in &self.components {
            let output = Command::new("rustup")
                .args(["component", "add", component, "--toolchain", &self.toolchain])
                .output()
                .map_err(|e| Error::Provider(format!("Failed to add {}: {}", component, e)))?;

            if output.status.success() {
                actions.push(format!("Installed {} component", component));
            }
        }

        // Create rust-toolchain.toml if missing
        let toolchain_file = ctx.working_dir().join("rust-toolchain.toml");
        if !toolchain_file.exists() {
            let content = format!(
                r#"[toolchain]
channel = "{}"
components = {:?}
"#,
                self.toolchain, self.components
            );

            std::fs::write(&toolchain_file, content)
                .map_err(|e| Error::Provider(format!("Failed to write rust-toolchain.toml: {}", e)))?;

            actions.push("Created rust-toolchain.toml".to_string());
        }

        Ok(ApplyReport {
            success: true,
            actions,
            errors: Vec::new(),
        })
    }
}
```

**Step 2-6:** Export, test, commit (similar pattern)

---

## Task C.4: Implement EditorConfigProvider

**Files:**
- Create: `crates/repo-presets/src/config/mod.rs`
- Create: `crates/repo-presets/src/config/editorconfig.rs`

A simpler provider that just generates an `.editorconfig` file.

---

## Task C.5: Implement GitIgnoreProvider

**Files:**
- Create: `crates/repo-presets/src/config/gitignore.rs`

Generates appropriate `.gitignore` based on detected languages.

---

## Task C.6: Integration Tests

**Files:**
- Create: `crates/repo-presets/tests/integration_tests.rs`

Test the full workflow of check -> apply -> check for each provider.

---

## Verification

```bash
# Run all preset tests
cargo test -p repo-presets

# Manual verification (if environments available)
mkdir /tmp/test-node && cd /tmp/test-node
repo init --presets node
ls node_modules  # Should exist

mkdir /tmp/test-rust && cd /tmp/test-rust
repo init --presets rust
cat rust-toolchain.toml  # Should exist
```

---

## Summary

| Task | Description | Risk | Effort |
|------|-------------|------|--------|
| C.1 | CondaProvider | Medium | Medium |
| C.2 | NodeProvider | Low | Medium |
| C.3 | RustProvider | Low | Medium |
| C.4 | EditorConfigProvider | Low | Low |
| C.5 | GitIgnoreProvider | Low | Low |
| C.6 | Integration tests | Low | Low |

**Total Effort:** ~1 day of focused work
