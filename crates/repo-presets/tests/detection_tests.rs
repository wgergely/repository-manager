//! Integration tests for preset detection with realistic project structures.
//!
//! Tests that providers correctly identify project types based on actual
//! file structures, not just empty directories.

mod common;

use common::create_test_context;
use repo_presets::provider::PresetProvider;
use repo_presets::{
    ApplyReport, ApplyStatus, NodeProvider, PresetStatus, RustProvider, UvProvider,
};
use std::fs;
use tempfile::TempDir;

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
    assert!(
        report.is_detection_only(),
        "RustProvider.apply() must return DetectionOnly status, got: {:?}",
        report.status
    );
    assert_eq!(
        report.status,
        ApplyStatus::DetectionOnly,
        "Status must be DetectionOnly, not Success or Failed"
    );
    assert!(
        !report.is_success(),
        "RustProvider.apply() must NOT return Success — it does no real work"
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
    assert!(
        report.is_detection_only(),
        "NodeProvider.apply() must return DetectionOnly status, got: {:?}",
        report.status
    );
    assert_eq!(
        report.status,
        ApplyStatus::DetectionOnly,
        "Status must be DetectionOnly, not Success or Failed"
    );
    assert!(
        !report.is_success(),
        "NodeProvider.apply() must NOT return Success — it does no real work"
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

// ==========================================================================
// Type-system detection-only tests (P2 task requirements)
// ==========================================================================

/// Verify that DetectionOnly and Success are distinguishable via the type system.
/// This test MUST fail if detection_only() is swapped for success().
#[test]
fn test_detection_only_not_equal_to_success() {
    let detection = ApplyReport::detection_only(vec!["detected".to_string()]);
    let success = ApplyReport::success(vec!["applied".to_string()]);

    // Status enum values must differ
    assert_ne!(
        detection.status, success.status,
        "DetectionOnly and Success must be different ApplyStatus variants"
    );

    // Helper methods must give opposite results
    assert!(detection.is_detection_only());
    assert!(!detection.is_success());
    assert!(success.is_success());
    assert!(!success.is_detection_only());

    // Neither is a failure
    assert!(!detection.is_failure());
    assert!(!success.is_failure());
}

/// Verify that ApplyStatus::DetectionOnly is a distinct variant from Success and Failed.
#[test]
fn test_apply_status_variants_are_distinct() {
    let detection_only = ApplyStatus::DetectionOnly;
    let success = ApplyStatus::Success;
    let failed = ApplyStatus::Failed;

    assert_ne!(detection_only, success);
    assert_ne!(detection_only, failed);
    assert_ne!(success, failed);
}

/// NodeProvider.check() correctly detects a Node.js project with package.json.
#[tokio::test]
async fn test_node_provider_check_detects_package_json() {
    let temp = TempDir::new().unwrap();

    // Create package.json so check() detects Node
    fs::write(
        temp.path().join("package.json"),
        r#"{"name": "test-project", "version": "1.0.0"}"#,
    )
    .unwrap();

    let context = create_test_context(&temp);
    let provider = NodeProvider::new();

    let report = provider.check(&context).await.unwrap();

    // With package.json present, check() must NOT report Missing for package.json
    // (it may report Missing for node_modules or Broken for missing node binary,
    // but the key assertion is that it sees the package.json).
    let missing_package_json = report.status == PresetStatus::Missing
        && report
            .details
            .iter()
            .any(|d| d.contains("package.json not found"));
    assert!(
        !missing_package_json,
        "check() must detect package.json when it exists, got status={:?} details={:?}",
        report.status, report.details
    );
}

/// RustProvider.check() correctly detects a Rust project with Cargo.toml.
#[tokio::test]
async fn test_rust_provider_check_detects_cargo_toml() {
    let temp = TempDir::new().unwrap();

    // Create Cargo.toml so check() detects Rust
    fs::write(
        temp.path().join("Cargo.toml"),
        "[package]\nname = \"test-project\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();

    let context = create_test_context(&temp);
    let provider = RustProvider::new();

    let report = provider.check(&context).await.unwrap();

    // With Cargo.toml present, check() must not report Missing
    assert_ne!(
        report.status,
        PresetStatus::Missing,
        "check() must detect Cargo.toml when it exists — expected Healthy or Broken, got Missing"
    );
}

/// UvProvider.apply() does real work (runs uv venv), so it must return Success, not DetectionOnly.
/// Skip if uv is not installed.
#[tokio::test]
async fn test_uv_provider_apply_is_real_success() {
    let provider = UvProvider::new();

    // Check if uv is available; skip if not
    let temp = TempDir::new().unwrap();
    let context = create_test_context(&temp);

    let check_report = provider.check(&context).await.unwrap();
    if check_report.status == PresetStatus::Broken {
        eprintln!("Skipping test: uv not available on this system");
        return;
    }

    let report = provider.apply(&context).await.unwrap();

    // UvProvider does real work (creates a venv), so it must NOT be DetectionOnly
    assert!(
        !report.is_detection_only(),
        "UvProvider.apply() does real work and must NOT return DetectionOnly, got: {:?}",
        report.status
    );

    // It should be Success (assuming uv is available and can create a venv)
    assert!(
        report.is_success(),
        "UvProvider.apply() should return Success when it creates a venv, got: {:?}",
        report.status
    );

    // Verify the side effect: venv directory was actually created
    let venv_path = context.venv_path();
    assert!(
        venv_path.exists(),
        "UvProvider.apply() must create the venv directory at {}",
        venv_path
    );
}
