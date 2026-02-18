//! Integration tests for preset detection with realistic project structures.
//!
//! Tests that providers correctly identify project types based on actual
//! file structures, not just empty directories.

use repo_fs::{LayoutMode, NormalizedPath, WorkspaceLayout};
use repo_presets::provider::PresetProvider;
use repo_presets::{Context, NodeProvider, PresetStatus, RustProvider, UvProvider};
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

fn create_test_context(temp: &TempDir) -> Context {
    let layout = WorkspaceLayout {
        root: NormalizedPath::new(temp.path()),
        active_context: NormalizedPath::new(temp.path()),
        mode: LayoutMode::Classic,
    };
    Context::new(layout, HashMap::new())
}

// ==========================================================================
// Rust Provider Detection Tests
// ==========================================================================

#[tokio::test]
async fn test_rust_missing_without_cargo_toml() {
    let temp = TempDir::new().unwrap();
    let context = create_test_context(&temp);
    let provider = RustProvider::new();

    let report = provider.check(&context).await.unwrap();

    assert_eq!(
        report.status,
        PresetStatus::Missing,
        "Empty directory should report Rust as missing"
    );
    assert!(
        report.details.iter().any(|d| d.contains("Cargo.toml")),
        "Details should mention Cargo.toml, got: {:?}",
        report.details
    );
}

#[tokio::test]
async fn test_rust_detected_with_realistic_cargo_toml() {
    let temp = TempDir::new().unwrap();

    // Create a realistic Cargo.toml
    fs::write(
        temp.path().join("Cargo.toml"),
        r#"[package]
name = "my-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
"#,
    )
    .unwrap();

    // Create src/main.rs for a realistic project
    fs::create_dir(temp.path().join("src")).unwrap();
    fs::write(
        temp.path().join("src/main.rs"),
        "fn main() {\n    println!(\"Hello, world!\");\n}\n",
    )
    .unwrap();

    let context = create_test_context(&temp);
    let provider = RustProvider::new();

    let report = provider.check(&context).await.unwrap();

    // Since rustc IS available in this CI/test environment, should be Healthy
    if provider.check_rustc_available_sync() {
        assert_eq!(
            report.status,
            PresetStatus::Healthy,
            "Rust project with Cargo.toml and rustc should be healthy"
        );
    } else {
        // If rustc is not available, should be Broken (project exists but toolchain missing)
        assert_eq!(
            report.status,
            PresetStatus::Broken,
            "Rust project without rustc should be broken"
        );
    }
}

#[tokio::test]
async fn test_rust_not_confused_by_non_rust_files() {
    let temp = TempDir::new().unwrap();

    // Create files that look like a project but NOT Rust
    fs::write(temp.path().join("package.json"), "{}").unwrap();
    fs::write(temp.path().join("Makefile"), "all:\n\techo hello").unwrap();
    fs::write(
        temp.path().join("setup.py"),
        "from setuptools import setup\nsetup()",
    )
    .unwrap();

    let context = create_test_context(&temp);
    let provider = RustProvider::new();

    let report = provider.check(&context).await.unwrap();

    assert_eq!(
        report.status,
        PresetStatus::Missing,
        "Non-Rust project files should not trigger Rust detection"
    );
}

#[tokio::test]
async fn test_rust_apply_is_detection_only() {
    let temp = TempDir::new().unwrap();
    let context = create_test_context(&temp);
    let provider = RustProvider::new();

    let report = provider.apply(&context).await.unwrap();
    assert!(report.success);
    assert!(
        report
            .actions_taken
            .iter()
            .any(|a| a.contains("detection-only")),
        "Rust provider should indicate it's detection-only, got: {:?}",
        report.actions_taken
    );
}

// ==========================================================================
// Node Provider Detection Tests
// ==========================================================================

#[tokio::test]
async fn test_node_missing_without_package_json() {
    let temp = TempDir::new().unwrap();
    let context = create_test_context(&temp);
    let provider = NodeProvider::new();

    let report = provider.check(&context).await.unwrap();

    assert_eq!(
        report.status,
        PresetStatus::Missing,
        "Empty directory should report Node as missing"
    );
    assert!(
        report.details.iter().any(|d| d.contains("package.json")),
        "Details should mention package.json, got: {:?}",
        report.details
    );
}

#[tokio::test]
async fn test_node_detected_with_realistic_package_json() {
    let temp = TempDir::new().unwrap();

    // Create a realistic package.json
    fs::write(
        temp.path().join("package.json"),
        r#"{
  "name": "my-app",
  "version": "1.0.0",
  "dependencies": {
    "express": "^4.18.2",
    "lodash": "^4.17.21"
  },
  "scripts": {
    "start": "node index.js",
    "test": "jest"
  }
}"#,
    )
    .unwrap();

    // Create node_modules (simulating installed deps)
    fs::create_dir(temp.path().join("node_modules")).unwrap();

    let context = create_test_context(&temp);
    let provider = NodeProvider::new();

    let report = provider.check(&context).await.unwrap();

    if provider.check_node_available_sync() {
        assert_eq!(
            report.status,
            PresetStatus::Healthy,
            "Node project with package.json, node_modules, and node binary should be healthy"
        );
    } else {
        // Node not available on this system
        assert_eq!(
            report.status,
            PresetStatus::Broken,
            "Node project without node binary should be broken"
        );
    }
}

#[tokio::test]
async fn test_node_package_json_without_node_modules() {
    let temp = TempDir::new().unwrap();

    // Create package.json but NO node_modules
    fs::write(
        temp.path().join("package.json"),
        r#"{"name": "test", "version": "1.0.0"}"#,
    )
    .unwrap();

    let context = create_test_context(&temp);
    let provider = NodeProvider::new();

    let report = provider.check(&context).await.unwrap();

    if provider.check_node_available_sync() {
        // Node available but no node_modules -> Missing (deps not installed)
        assert_eq!(
            report.status,
            PresetStatus::Missing,
            "Package.json without node_modules should be missing"
        );
        assert!(
            report
                .details
                .iter()
                .any(|d| d.contains("Dependencies") || d.contains("node_modules")),
            "Details should mention missing dependencies, got: {:?}",
            report.details
        );
    } else {
        // Node not available -> Broken
        assert_eq!(report.status, PresetStatus::Broken);
    }
}

#[tokio::test]
async fn test_node_apply_is_detection_only() {
    let temp = TempDir::new().unwrap();
    let context = create_test_context(&temp);
    let provider = NodeProvider::new();

    let report = provider.apply(&context).await.unwrap();
    assert!(report.success);
    assert!(
        report
            .actions_taken
            .iter()
            .any(|a| a.contains("detection-only")),
        "Node provider should indicate it's detection-only, got: {:?}",
        report.actions_taken
    );
}

// ==========================================================================
// Multi-Provider Detection (same project directory)
// ==========================================================================

#[tokio::test]
async fn test_polyglot_project_detects_multiple_presets() {
    let temp = TempDir::new().unwrap();

    // Create a project that has BOTH Rust and Node.js components
    fs::write(
        temp.path().join("Cargo.toml"),
        "[package]\nname = \"full-stack\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        temp.path().join("package.json"),
        r#"{"name": "frontend", "version": "1.0.0"}"#,
    )
    .unwrap();
    // Create node_modules so Node detection sees installed dependencies
    fs::create_dir(temp.path().join("node_modules")).unwrap();

    let context = create_test_context(&temp);

    let rust_provider = RustProvider::new();
    let node_provider = NodeProvider::new();

    let rust_report = rust_provider.check(&context).await.unwrap();
    let node_report = node_provider.check(&context).await.unwrap();

    // Rust should NOT be Missing (Cargo.toml exists)
    assert_ne!(
        rust_report.status,
        PresetStatus::Missing,
        "Polyglot project should detect Rust (Cargo.toml present)"
    );

    // Node should detect the package.json (not Missing for package.json)
    // But node_modules is missing, so it depends on node availability
    if node_provider.check_node_available_sync() {
        assert_ne!(
            node_report.status,
            PresetStatus::Missing,
            "Polyglot project with package.json and node available should detect Node"
        );
    }
}

#[tokio::test]
async fn test_empty_project_all_providers_missing() {
    let temp = TempDir::new().unwrap();
    let context = create_test_context(&temp);

    let rust_provider = RustProvider::new();
    let node_provider = NodeProvider::new();
    let uv_provider = UvProvider::new();

    let rust_report = rust_provider.check(&context).await.unwrap();
    let node_report = node_provider.check(&context).await.unwrap();
    let uv_report = uv_provider.check(&context).await.unwrap();

    // All should be non-healthy for an empty directory
    assert_ne!(rust_report.status, PresetStatus::Healthy);
    assert_ne!(node_report.status, PresetStatus::Healthy);
    assert_ne!(uv_report.status, PresetStatus::Healthy);
}
