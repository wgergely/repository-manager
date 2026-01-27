//! Integration tests for repo-core
//!
//! These tests verify the complete workflow of the repo-core crate,
//! ensuring all components work together correctly.

use pretty_assertions::assert_eq;
use repo_core::backend::{ModeBackend, StandardBackend, WorktreeBackend};
use repo_core::config::{ConfigResolver, RuntimeContext};
use repo_core::ledger::{Intent, Ledger, Projection, ProjectionKind};
use repo_core::sync::{CheckStatus, SyncEngine};
use repo_core::Mode;
use repo_fs::NormalizedPath;
use serde_json::json;
use std::fs;
use tempfile::TempDir;
use uuid::Uuid;

// =============================================================================
// Test 1: Complete Workflow
// =============================================================================

/// Test the complete workflow from configuration to sync
///
/// This test verifies:
/// 1. Create temp directory with .git structure
/// 2. Create .repository/config.toml with presets and tools
/// 3. Resolve configuration using ConfigResolver
/// 4. Verify mode, presets, and tools are parsed correctly
/// 5. Generate RuntimeContext and verify JSON structure
/// 6. Create SyncEngine and run sync()
/// 7. Verify sync succeeds
/// 8. Run check() and verify healthy status
#[test]
fn test_complete_workflow() {
    // Step 1: Create temp directory with .git structure
    let temp = TempDir::new().expect("Failed to create temp dir");
    fs::create_dir(temp.path().join(".git")).expect("Failed to create .git");
    fs::write(temp.path().join(".git/HEAD"), "ref: refs/heads/main\n").expect("Failed to write HEAD");
    fs::create_dir_all(temp.path().join(".git/refs/heads")).expect("Failed to create refs/heads");

    // Step 2: Create .repository/config.toml with presets and tools
    let repo_dir = temp.path().join(".repository");
    fs::create_dir_all(&repo_dir).expect("Failed to create .repository");

    let config_content = r#"
tools = ["cargo", "rustfmt", "clippy"]
rules = ["no-unsafe", "no-unwrap"]

[core]
mode = "standard"

[presets."env:rust"]
version = "1.75"
edition = "2021"
profile = "release"

[presets."env:python"]
version = "3.12"
provider = "uv"

[presets."tool:linter"]
enabled = true
strict = true

[presets."config:editor"]
theme = "dark"
font_size = 14
"#;
    fs::write(repo_dir.join("config.toml"), config_content).expect("Failed to write config.toml");

    // Step 3: Resolve configuration using ConfigResolver
    let root = NormalizedPath::new(temp.path());
    let resolver = ConfigResolver::new(root.clone());
    let config = resolver.resolve().expect("Failed to resolve config");

    // Step 4: Verify mode, presets, and tools are parsed correctly
    assert_eq!(config.mode, "standard");
    assert_eq!(config.tools, vec!["cargo", "rustfmt", "clippy"]);
    assert_eq!(config.rules, vec!["no-unsafe", "no-unwrap"]);

    // Verify presets are parsed
    assert!(config.presets.contains_key("env:rust"));
    assert!(config.presets.contains_key("env:python"));
    assert!(config.presets.contains_key("tool:linter"));
    assert!(config.presets.contains_key("config:editor"));

    // Verify preset values
    let rust_preset = &config.presets["env:rust"];
    assert_eq!(rust_preset["version"], "1.75");
    assert_eq!(rust_preset["edition"], "2021");

    let python_preset = &config.presets["env:python"];
    assert_eq!(python_preset["version"], "3.12");
    assert_eq!(python_preset["provider"], "uv");

    // Step 5: Generate RuntimeContext and verify JSON structure
    let context = RuntimeContext::from_resolved(&config);

    // Verify runtime (env: presets become runtime)
    assert!(context.has_runtime());
    assert!(context.runtime.contains_key("rust"));
    assert!(context.runtime.contains_key("python"));
    assert_eq!(context.get_runtime("rust").unwrap()["version"], "1.75");
    assert_eq!(context.get_runtime("python").unwrap()["provider"], "uv");

    // Verify capabilities (tool: and config: become capabilities)
    assert!(context.has_capabilities());
    assert!(context.has_capability("tool:linter"));
    assert!(context.has_capability("config:editor"));

    // Verify JSON structure
    let json_output = context.to_json();
    assert!(json_output["runtime"].is_object());
    assert!(json_output["capabilities"].is_array());
    assert_eq!(json_output["runtime"]["rust"]["edition"], "2021");

    // Step 6: Create SyncEngine and run sync()
    let engine = SyncEngine::new(root.clone(), Mode::Standard).expect("Failed to create SyncEngine");

    // Step 7: Verify sync succeeds
    let sync_report = engine.sync().expect("Sync failed");
    assert!(sync_report.success);

    // Verify ledger was created
    let ledger_path = engine.ledger_path();
    assert!(ledger_path.exists(), "Ledger file should be created by sync");

    // Step 8: Run check() and verify healthy status
    let check_report = engine.check().expect("Check failed");
    assert_eq!(check_report.status, CheckStatus::Healthy);
    assert!(check_report.drifted.is_empty());
    assert!(check_report.missing.is_empty());
}

// =============================================================================
// Test 2: Mode Backends
// =============================================================================

/// Test StandardBackend and WorktreeBackend
///
/// This test verifies:
/// - StandardBackend:
///   - Create temp dir with .git
///   - Verify config_root() returns root.join(".repository")
///   - Verify working_dir() returns root
/// - WorktreeBackend:
///   - Create temp dir with .gt and main/
///   - Verify config_root() returns container.join(".repository") (shared)
///   - Verify working_dir() returns the worktree path
#[test]
fn test_mode_backends() {
    // =========================================================================
    // Test StandardBackend
    // =========================================================================

    // Create temp dir with .git
    let standard_temp = TempDir::new().expect("Failed to create temp dir");
    fs::create_dir(standard_temp.path().join(".git")).expect("Failed to create .git");
    fs::write(standard_temp.path().join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
    fs::create_dir_all(standard_temp.path().join(".git/refs/heads")).unwrap();

    let standard_root = NormalizedPath::new(standard_temp.path());
    let standard_backend = StandardBackend::new(standard_root.clone()).expect("Failed to create StandardBackend");

    // Verify config_root() returns root.join(".repository")
    let expected_config_root = standard_root.join(".repository");
    assert_eq!(
        standard_backend.config_root().as_str(),
        expected_config_root.as_str(),
        "StandardBackend config_root should be root/.repository"
    );

    // Verify working_dir() returns root
    assert_eq!(
        standard_backend.working_dir().as_str(),
        standard_root.as_str(),
        "StandardBackend working_dir should be the root"
    );

    // =========================================================================
    // Test WorktreeBackend
    // =========================================================================

    // Create temp dir with .gt and main/
    let worktree_temp = TempDir::new().expect("Failed to create temp dir");
    fs::create_dir(worktree_temp.path().join(".gt")).expect("Failed to create .gt");
    fs::create_dir(worktree_temp.path().join("main")).expect("Failed to create main/");
    fs::write(worktree_temp.path().join(".gt/HEAD"), "ref: refs/heads/main\n").unwrap();
    fs::create_dir_all(worktree_temp.path().join(".gt/refs/heads")).unwrap();

    // Create .git file in main that points to .gt
    fs::write(
        worktree_temp.path().join("main/.git"),
        format!("gitdir: {}/.gt", worktree_temp.path().display()),
    )
    .expect("Failed to write main/.git");

    let container = NormalizedPath::new(worktree_temp.path());
    let main_worktree = NormalizedPath::new(worktree_temp.path().join("main"));
    let worktree_backend = WorktreeBackend::new(container.clone()).expect("Failed to create WorktreeBackend");

    // Verify config_root() returns container.join(".repository") (shared)
    let expected_container_config = container.join(".repository");
    assert_eq!(
        worktree_backend.config_root().as_str(),
        expected_container_config.as_str(),
        "WorktreeBackend config_root should be container/.repository (shared)"
    );

    // Verify working_dir() returns the worktree path
    assert_eq!(
        worktree_backend.working_dir().as_str(),
        main_worktree.as_str(),
        "WorktreeBackend working_dir should be the main worktree"
    );

    // Test with specific worktree
    let feature_worktree_path = worktree_temp.path().join("feature-x");
    fs::create_dir(&feature_worktree_path).expect("Failed to create feature-x/");
    fs::write(
        feature_worktree_path.join(".git"),
        format!("gitdir: {}/.gt/worktrees/feature-x", worktree_temp.path().display()),
    )
    .expect("Failed to write feature-x/.git");

    let feature_worktree = NormalizedPath::new(&feature_worktree_path);
    let feature_backend =
        WorktreeBackend::with_worktree(container.clone(), feature_worktree.clone())
            .expect("Failed to create WorktreeBackend with specific worktree");

    // Config root should still be at container level (shared)
    assert_eq!(
        feature_backend.config_root().as_str(),
        expected_container_config.as_str(),
        "WorktreeBackend config_root should always be at container level"
    );

    // Working dir should be the feature worktree
    assert_eq!(
        feature_backend.working_dir().as_str(),
        feature_worktree.as_str(),
        "WorktreeBackend working_dir should be the specified worktree"
    );
}

// =============================================================================
// Test 3: Ledger Persistence
// =============================================================================

/// Test Ledger save and load functionality
///
/// This test verifies:
/// - Create a Ledger with an Intent and Projection
/// - Save to temp file
/// - Load back
/// - Verify all data is preserved
#[test]
fn test_ledger_persistence() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let ledger_path = temp.path().join("ledger.toml");

    // Create a Ledger with an Intent and Projection
    let mut ledger = Ledger::new();

    let fixed_uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    let marker_uuid = Uuid::parse_str("660e8400-e29b-41d4-a716-446655440001").unwrap();

    let mut intent = Intent::with_uuid(
        "rule:python/style/snake-case".to_string(),
        fixed_uuid,
        json!({
            "severity": "warning",
            "autofix": true,
            "exclude": ["test_*.py", "conftest.py"]
        }),
    );

    // Add TextBlock projection
    intent.add_projection(Projection::text_block(
        "cursor".to_string(),
        std::path::PathBuf::from(".cursor/rules/python-style.mdc"),
        marker_uuid,
        "abc123def456".to_string(),
    ));

    // Add JsonKey projection
    intent.add_projection(Projection::json_key(
        "vscode".to_string(),
        std::path::PathBuf::from(".vscode/settings.json"),
        "python.linting.pylintEnabled".to_string(),
        json!(true),
    ));

    // Add FileManaged projection
    intent.add_projection(Projection::file_managed(
        "ruff".to_string(),
        std::path::PathBuf::from("pyproject.toml"),
        "sha256-checksum-here".to_string(),
    ));

    ledger.add_intent(intent);

    // Add a second intent
    let mut intent2 = Intent::new("rule:rust/style/naming".to_string(), json!({"strict": true}));
    intent2.add_projection(Projection::file_managed(
        "rustfmt".to_string(),
        std::path::PathBuf::from("rustfmt.toml"),
        "rustfmt-checksum".to_string(),
    ));
    let intent2_uuid = intent2.uuid;
    ledger.add_intent(intent2);

    // Save to temp file
    ledger.save(&ledger_path).expect("Failed to save ledger");
    assert!(ledger_path.exists(), "Ledger file should exist after save");

    // Load back
    let loaded = Ledger::load(&ledger_path).expect("Failed to load ledger");

    // Verify all data is preserved
    assert_eq!(loaded.intents().len(), 2, "Should have 2 intents");

    // Verify first intent
    let loaded_intent = loaded.get_intent(fixed_uuid).expect("Intent should exist");
    assert_eq!(loaded_intent.id, "rule:python/style/snake-case");
    assert_eq!(loaded_intent.uuid, fixed_uuid);
    assert_eq!(loaded_intent.args["severity"], "warning");
    assert_eq!(loaded_intent.args["autofix"], true);
    assert!(loaded_intent.args["exclude"].is_array());
    assert_eq!(loaded_intent.args["exclude"][0], "test_*.py");

    // Verify projections
    assert_eq!(loaded_intent.projections().len(), 3, "Should have 3 projections");

    // Verify TextBlock projection
    let text_block_proj = loaded_intent
        .projections()
        .iter()
        .find(|p| p.tool == "cursor")
        .expect("Should have cursor projection");
    assert_eq!(
        text_block_proj.file,
        std::path::PathBuf::from(".cursor/rules/python-style.mdc")
    );
    match &text_block_proj.kind {
        ProjectionKind::TextBlock { marker, checksum } => {
            assert_eq!(*marker, marker_uuid);
            assert_eq!(checksum, "abc123def456");
        }
        _ => panic!("Expected TextBlock projection kind"),
    }

    // Verify JsonKey projection
    let json_key_proj = loaded_intent
        .projections()
        .iter()
        .find(|p| p.tool == "vscode")
        .expect("Should have vscode projection");
    match &json_key_proj.kind {
        ProjectionKind::JsonKey { path, value } => {
            assert_eq!(path, "python.linting.pylintEnabled");
            assert_eq!(*value, json!(true));
        }
        _ => panic!("Expected JsonKey projection kind"),
    }

    // Verify FileManaged projection
    let file_managed_proj = loaded_intent
        .projections()
        .iter()
        .find(|p| p.tool == "ruff")
        .expect("Should have ruff projection");
    match &file_managed_proj.kind {
        ProjectionKind::FileManaged { checksum } => {
            assert_eq!(checksum, "sha256-checksum-here");
        }
        _ => panic!("Expected FileManaged projection kind"),
    }

    // Verify second intent
    let loaded_intent2 = loaded.get_intent(intent2_uuid).expect("Second intent should exist");
    assert_eq!(loaded_intent2.id, "rule:rust/style/naming");
    assert_eq!(loaded_intent2.args["strict"], true);
    assert_eq!(loaded_intent2.projections().len(), 1);

    // Verify find_by_rule works after load
    let python_intents = loaded.find_by_rule("rule:python/style/snake-case");
    assert_eq!(python_intents.len(), 1);

    let rust_intents = loaded.find_by_rule("rule:rust/style/naming");
    assert_eq!(rust_intents.len(), 1);
}

// =============================================================================
// Test 4: Config Hierarchy
// =============================================================================

/// Test configuration hierarchy and deep merge
///
/// This test verifies:
/// - Create .repository/config.toml with base presets
/// - Create .repository/config.local.toml with overrides
/// - Resolve configuration
/// - Verify local overrides take precedence
/// - Verify deep merge preserves non-overridden values
#[test]
fn test_config_hierarchy() {
    let temp = TempDir::new().expect("Failed to create temp dir");

    // Create .repository directory
    let repo_dir = temp.path().join(".repository");
    fs::create_dir_all(&repo_dir).expect("Failed to create .repository");

    // Create .repository/config.toml with base presets
    let base_config = r#"
tools = ["cargo", "rustfmt"]
rules = ["base-rule-1", "base-rule-2"]

[core]
mode = "standard"

[presets."env:python"]
version = "3.11"
provider = "pyenv"
venv_name = ".venv"
requirements_file = "requirements.txt"

[presets."env:node"]
version = "18"
manager = "nvm"

[presets."tool:linter"]
enabled = true
strict = false
max_line_length = 80

[presets."config:editor"]
theme = "light"
font_size = 12
tab_size = 4
"#;
    fs::write(repo_dir.join("config.toml"), base_config).expect("Failed to write config.toml");

    // Create .repository/config.local.toml with overrides
    let local_config = r#"
tools = ["clippy", "python"]
rules = ["local-rule"]

[core]
mode = "worktree"

[presets."env:python"]
version = "3.12"
provider = "uv"

[presets."tool:linter"]
strict = true
max_line_length = 120

[presets."tool:formatter"]
enabled = true
style = "black"
"#;
    fs::write(repo_dir.join("config.local.toml"), local_config)
        .expect("Failed to write config.local.toml");

    // Resolve configuration
    let root = NormalizedPath::new(temp.path());
    let resolver = ConfigResolver::new(root);
    let config = resolver.resolve().expect("Failed to resolve config");

    // Verify local overrides take precedence for scalar values
    assert_eq!(config.mode, "worktree", "Local mode should override base");

    // Verify deep merge preserves non-overridden values in presets

    // Python preset: overridden values + preserved values
    let python = &config.presets["env:python"];
    assert_eq!(python["version"], "3.12", "Local version should override");
    assert_eq!(python["provider"], "uv", "Local provider should override");
    assert_eq!(
        python["venv_name"], ".venv",
        "Base venv_name should be preserved"
    );
    assert_eq!(
        python["requirements_file"], "requirements.txt",
        "Base requirements_file should be preserved"
    );

    // Node preset: only in base, should be preserved
    assert!(
        config.presets.contains_key("env:node"),
        "Base-only preset should be preserved"
    );
    let node = &config.presets["env:node"];
    assert_eq!(node["version"], "18");
    assert_eq!(node["manager"], "nvm");

    // Linter preset: partial override
    let linter = &config.presets["tool:linter"];
    assert_eq!(linter["enabled"], true, "Base enabled should be preserved");
    assert_eq!(linter["strict"], true, "Local strict should override");
    assert_eq!(
        linter["max_line_length"], 120,
        "Local max_line_length should override"
    );

    // Editor preset: only in base, should be preserved
    assert!(config.presets.contains_key("config:editor"));
    let editor = &config.presets["config:editor"];
    assert_eq!(editor["theme"], "light");
    assert_eq!(editor["font_size"], 12);
    assert_eq!(editor["tab_size"], 4);

    // Formatter preset: only in local, should be added
    assert!(
        config.presets.contains_key("tool:formatter"),
        "Local-only preset should be added"
    );
    let formatter = &config.presets["tool:formatter"];
    assert_eq!(formatter["enabled"], true);
    assert_eq!(formatter["style"], "black");

    // Verify tools are merged (unique values from both)
    assert!(
        config.tools.contains(&"cargo".to_string()),
        "Base tool should be preserved"
    );
    assert!(
        config.tools.contains(&"rustfmt".to_string()),
        "Base tool should be preserved"
    );
    assert!(
        config.tools.contains(&"clippy".to_string()),
        "Local tool should be added"
    );
    assert!(
        config.tools.contains(&"python".to_string()),
        "Local tool should be added"
    );

    // Verify rules are merged (unique values from both)
    assert!(config.rules.contains(&"base-rule-1".to_string()));
    assert!(config.rules.contains(&"base-rule-2".to_string()));
    assert!(config.rules.contains(&"local-rule".to_string()));

    // Verify RuntimeContext is correctly generated from merged config
    let context = RuntimeContext::from_resolved(&config);

    // env: presets become runtime
    assert!(context.runtime.contains_key("python"));
    assert!(context.runtime.contains_key("node"));
    assert_eq!(context.runtime["python"]["version"], "3.12");
    assert_eq!(context.runtime["node"]["version"], "18");

    // tool: and config: become capabilities
    assert!(context.capabilities.contains(&"tool:linter".to_string()));
    assert!(context.capabilities.contains(&"tool:formatter".to_string()));
    assert!(context.capabilities.contains(&"config:editor".to_string()));
}

// =============================================================================
// Additional Integration Tests
// =============================================================================

/// Test SyncEngine with ledger containing projections that all pass check
#[test]
fn test_sync_engine_complete_check_flow() {
    let temp = TempDir::new().expect("Failed to create temp dir");

    // Create .git structure
    fs::create_dir(temp.path().join(".git")).unwrap();
    fs::write(temp.path().join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
    fs::create_dir_all(temp.path().join(".git/refs/heads")).unwrap();

    // Create actual config files that will match projections
    let vscode_dir = temp.path().join(".vscode");
    fs::create_dir_all(&vscode_dir).unwrap();
    fs::write(
        vscode_dir.join("settings.json"),
        r#"{"editor": {"fontSize": 14, "tabSize": 2}}"#,
    )
    .unwrap();

    // Create a managed file with known checksum
    let managed_content = r#"{"managed": true}"#;
    fs::write(temp.path().join("managed.json"), managed_content).unwrap();

    // Compute checksum for managed file
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(managed_content.as_bytes());
    let managed_checksum = format!("{:x}", hasher.finalize());

    // Create a file with marker for text block
    let marker = Uuid::new_v4();
    let cursor_dir = temp.path().join(".cursor").join("rules");
    fs::create_dir_all(&cursor_dir).unwrap();
    fs::write(
        cursor_dir.join("test.mdc"),
        format!(
            "# Test Rule\n<!-- BEGIN {} -->\nRule content here\n<!-- END {} -->\n",
            marker, marker
        ),
    )
    .unwrap();

    // Create .repository directory and ledger
    let repo_dir = temp.path().join(".repository");
    fs::create_dir_all(&repo_dir).unwrap();

    let mut ledger = Ledger::new();
    let mut intent = Intent::new("rule:test/complete".to_string(), json!({}));

    // Add all projection types that should pass
    intent.add_projection(Projection::json_key(
        "vscode".to_string(),
        std::path::PathBuf::from(".vscode/settings.json"),
        "editor.fontSize".to_string(),
        json!(14),
    ));
    intent.add_projection(Projection::file_managed(
        "custom".to_string(),
        std::path::PathBuf::from("managed.json"),
        managed_checksum,
    ));
    intent.add_projection(Projection::text_block(
        "cursor".to_string(),
        std::path::PathBuf::from(".cursor/rules/test.mdc"),
        marker,
        "block-checksum".to_string(),
    ));

    ledger.add_intent(intent);
    ledger.save(&repo_dir.join("ledger.toml")).unwrap();

    // Create SyncEngine and verify everything
    let root = NormalizedPath::new(temp.path());
    let engine = SyncEngine::new(root, Mode::Standard).unwrap();

    // Check should report healthy
    let report = engine.check().unwrap();
    assert_eq!(
        report.status,
        CheckStatus::Healthy,
        "All projections should be healthy"
    );
    assert!(report.drifted.is_empty());
    assert!(report.missing.is_empty());

    // Sync should succeed
    let sync_report = engine.sync().unwrap();
    assert!(sync_report.success);

    // Fix should also succeed
    let fix_report = engine.fix().unwrap();
    assert!(fix_report.success);
}

/// Test Mode parsing and display
#[test]
fn test_mode_roundtrip() {
    // Test Standard mode
    let standard: Mode = "standard".parse().unwrap();
    assert_eq!(standard, Mode::Standard);
    assert_eq!(standard.to_string(), "standard");
    assert!(!standard.supports_parallel_worktrees());

    // Test Worktrees mode
    let worktrees: Mode = "worktrees".parse().unwrap();
    assert_eq!(worktrees, Mode::Worktrees);
    assert_eq!(worktrees.to_string(), "worktrees");
    assert!(worktrees.supports_parallel_worktrees());

    // Test alternative strings
    assert_eq!("default".parse::<Mode>().unwrap(), Mode::Standard);
    assert_eq!("worktree".parse::<Mode>().unwrap(), Mode::Worktrees);
    assert_eq!("container".parse::<Mode>().unwrap(), Mode::Worktrees);

    // Test default (worktrees per spec)
    assert_eq!(Mode::default(), Mode::Worktrees);
}

/// Test that config resolution works when no config files exist
#[test]
fn test_config_resolution_with_no_files() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let root = NormalizedPath::new(temp.path());

    let resolver = ConfigResolver::new(root);

    // has_config and has_local_overrides should return false
    assert!(!resolver.has_config());
    assert!(!resolver.has_local_overrides());

    // resolve should return defaults (worktrees per spec)
    let config = resolver.resolve().expect("Should resolve with defaults");
    assert_eq!(config.mode, "worktrees");
    assert!(config.presets.is_empty());
    assert!(config.tools.is_empty());
    assert!(config.rules.is_empty());
}

/// Test RuntimeContext edge cases
#[test]
fn test_runtime_context_edge_cases() {
    use repo_core::config::ResolvedConfig;
    use std::collections::HashMap;

    // Test with only env presets (no capabilities)
    let mut presets = HashMap::new();
    presets.insert("env:python".to_string(), json!({"version": "3.12"}));
    presets.insert("env:rust".to_string(), json!({"edition": "2021"}));

    let config = ResolvedConfig {
        mode: "standard".to_string(),
        presets,
        tools: vec![],
        rules: vec![],
    };

    let context = RuntimeContext::from_resolved(&config);
    assert!(context.has_runtime());
    assert!(!context.has_capabilities());
    assert_eq!(context.runtime.len(), 2);
    assert!(context.capabilities.is_empty());

    // Test with only tool/config presets (no runtime)
    let mut presets2 = HashMap::new();
    presets2.insert("tool:linter".to_string(), json!({"enabled": true}));
    presets2.insert("config:editor".to_string(), json!({"theme": "dark"}));

    let config2 = ResolvedConfig {
        mode: "standard".to_string(),
        presets: presets2,
        tools: vec![],
        rules: vec![],
    };

    let context2 = RuntimeContext::from_resolved(&config2);
    assert!(!context2.has_runtime());
    assert!(context2.has_capabilities());
    assert!(context2.runtime.is_empty());
    assert_eq!(context2.capabilities.len(), 2);

    // Verify capabilities are sorted
    assert_eq!(context2.capabilities[0], "config:editor");
    assert_eq!(context2.capabilities[1], "tool:linter");
}
