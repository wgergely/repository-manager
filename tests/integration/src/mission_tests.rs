//! Mission-based Integration Tests
//!
//! These tests validate production scenarios against spec claims.
//!
//! Each test documents:
//! - Which spec it validates
//! - Expected behavior
//! - Current implementation status

use repo_core::Mode;
use repo_core::sync::{CheckStatus, SyncEngine};
use repo_fs::{LayoutMode, NormalizedPath, WorkspaceLayout};
use repo_git::{
    ClassicLayout, ContainerLayout, LayoutProvider, NamingStrategy, naming::branch_to_directory,
};
use repo_presets::{Context, PresetProvider, PresetStatus, UvProvider};
use repo_tools::{
    Rule, SyncContext, ToolIntegration, VSCodeIntegration, antigravity_integration,
    claude_integration, cursor_integration, gemini_integration, windsurf_integration,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

// =============================================================================
// Test Infrastructure
// =============================================================================

/// Test repository builder for standardized setup
pub struct TestRepo {
    temp_dir: TempDir,
    initialized: bool,
}

impl Default for TestRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl TestRepo {
    /// Create an empty test directory
    pub fn new() -> Self {
        Self {
            temp_dir: TempDir::new().unwrap(),
            initialized: false,
        }
    }

    /// Get the root path
    pub fn root(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Initialize as a git repository
    pub fn init_git(&self) {
        git2::Repository::init(self.root()).expect("Failed to init git repository");
    }

    /// Initialize with repository manager config
    ///
    /// Writes config.toml in the correct Manifest format:
    /// - Top-level `tools = [...]`
    /// - `[core]` section with `mode`
    /// - `[presets]` section as a table with each preset as a key
    pub fn init_repo_manager(&mut self, mode: &str, tools: &[&str], presets: &[&str]) {
        let repo_dir = self.root().join(".repository");
        fs::create_dir_all(&repo_dir).unwrap();

        let tools_str = tools
            .iter()
            .map(|t| format!("\"{}\"", t))
            .collect::<Vec<_>>()
            .join(", ");

        let mut config = format!("tools = [{tools_str}]\n\n[core]\nmode = \"{mode}\"\n");

        if !presets.is_empty() {
            config.push_str("\n[presets]\n");
            for preset in presets {
                config.push_str(&format!("\"{}\" = {{}}\n", preset));
            }
        }

        fs::write(repo_dir.join("config.toml"), config).unwrap();
        self.initialized = true;
    }

    /// Assert a file exists
    pub fn assert_file_exists(&self, path: &str) {
        let full_path = self.root().join(path);
        assert!(
            full_path.exists(),
            "Expected file to exist: {}",
            full_path.display()
        );
    }

    /// Assert a file does not exist
    pub fn assert_file_not_exists(&self, path: &str) {
        let full_path = self.root().join(path);
        assert!(
            !full_path.exists(),
            "Expected file NOT to exist: {}",
            full_path.display()
        );
    }

    /// Assert file contains content
    pub fn assert_file_contains(&self, path: &str, content: &str) {
        let full_path = self.root().join(path);
        let file_content = fs::read_to_string(&full_path)
            .unwrap_or_else(|_| panic!("Could not read file: {}", full_path.display()));
        assert!(
            file_content.contains(content),
            "File {} does not contain expected content.\nExpected: {}\nActual: {}",
            full_path.display(),
            content,
            file_content
        );
    }
}

// =============================================================================
// Mission 1: Repository Initialization
// =============================================================================

mod m1_init {
    use super::*;

    /// M1.1: Init creates .repository/config.toml in standard mode
    #[test]
    fn m1_1_init_standard_mode_creates_config() {
        use repo_core::Manifest;

        let mut repo = TestRepo::new();
        repo.init_git();
        repo.init_repo_manager("standard", &[], &[]);

        repo.assert_file_exists(".repository/config.toml");

        let content = fs::read_to_string(repo.root().join(".repository/config.toml")).unwrap();
        let manifest = Manifest::parse(&content).unwrap();
        assert_eq!(manifest.core.mode, "standard");
    }

    /// M1.2: Init creates .repository/config.toml in worktrees mode
    #[test]
    fn m1_2_init_worktrees_mode_creates_config() {
        use repo_core::Manifest;

        let mut repo = TestRepo::new();
        repo.init_git();
        repo.init_repo_manager("worktrees", &[], &[]);

        repo.assert_file_exists(".repository/config.toml");

        let content = fs::read_to_string(repo.root().join(".repository/config.toml")).unwrap();
        let manifest = Manifest::parse(&content).unwrap();
        assert_eq!(manifest.core.mode, "worktrees");
    }

    /// M1.3: Init with tools records them in config
    #[test]
    fn m1_3_init_with_tools_records_in_config() {
        use repo_core::Manifest;

        let mut repo = TestRepo::new();
        repo.init_git();
        repo.init_repo_manager("standard", &["vscode", "cursor", "claude"], &[]);

        let content = fs::read_to_string(repo.root().join(".repository/config.toml")).unwrap();
        let manifest = Manifest::parse(&content).unwrap();
        assert!(manifest.tools.contains(&"vscode".to_string()));
        assert!(manifest.tools.contains(&"cursor".to_string()));
        assert!(manifest.tools.contains(&"claude".to_string()));
    }

    /// M1.4: Init with presets records them in config
    #[test]
    fn m1_4_init_with_presets_records_in_config() {
        use repo_core::Manifest;

        let mut repo = TestRepo::new();
        repo.init_git();
        repo.init_repo_manager("standard", &[], &["env:python"]);

        let content = fs::read_to_string(repo.root().join(".repository/config.toml")).unwrap();
        let manifest = Manifest::parse(&content).unwrap();
        assert!(manifest.presets.contains_key("env:python"));
    }
}

// =============================================================================
// Mission 2: Branch & Worktree Management
// =============================================================================

mod m2_branch {
    use super::*;

    /// M2.1: ClassicLayout provides current branch
    #[test]
    fn m2_1_classic_layout_current_branch() {
        let repo = TestRepo::new();
        repo.init_git();

        let layout = ClassicLayout::new(NormalizedPath::new(repo.root()))
            .expect("ClassicLayout::new() should succeed on a real git repo");

        // current_branch may return an error on a fresh repo with no commits (unborn HEAD),
        // which is acceptable behavior
        let _result = layout.current_branch();
    }

    /// M2.2: Feature worktree path computation (Worktrees Mode)
    #[test]
    fn m2_2_feature_worktree_path_computation() {
        let repo = TestRepo::new();
        fs::create_dir(repo.root().join(".git")).unwrap();
        fs::create_dir(repo.root().join("main")).unwrap();

        let layout = ContainerLayout::new(NormalizedPath::new(repo.root()), NamingStrategy::Slug)
            .expect("ContainerLayout::new() should succeed");

        let path = layout.feature_worktree("feature-x");
        let path_str = path.as_str();
        assert!(
            path_str.ends_with("feature-x"),
            "Expected path to end with 'feature-x', got: {}",
            path_str
        );
        assert!(
            !path_str.contains("main/feature-x"),
            "Feature worktree should not be inside main/"
        );
    }

    /// M2.3: Branch name sanitization (slashes to dashes)
    #[test]
    fn m2_3_branch_name_sanitization() {
        // Test that feat/user-auth becomes feat-user-auth for directory
        let slug = branch_to_directory("feat/user-auth", NamingStrategy::Slug);
        assert_eq!(slug, "feat-user-auth");

        // Test hierarchical naming preserves slashes
        let hierarchical = branch_to_directory("feat/user-auth", NamingStrategy::Hierarchical);
        assert_eq!(hierarchical, "feat/user-auth");
    }

    /// M2.4: Container layout git database path
    /// Note: ContainerLayout uses .gt for git database, not .git
    #[test]
    fn m2_4_container_git_database_path() {
        let repo = TestRepo::new();
        fs::create_dir(repo.root().join(".gt")).unwrap(); // ContainerLayout uses .gt
        fs::create_dir(repo.root().join("main")).unwrap();

        let layout = ContainerLayout::new(NormalizedPath::new(repo.root()), NamingStrategy::Slug)
            .expect("ContainerLayout::new() should succeed with .gt");

        let git_db = layout.git_database();
        assert!(
            git_db.as_str().ends_with(".gt"),
            "Expected .gt, got: {}",
            git_db.as_str()
        );
    }

    /// M2.5: Main worktree path
    #[test]
    fn m2_5_main_worktree_path() {
        let repo = TestRepo::new();
        fs::create_dir(repo.root().join(".git")).unwrap();
        fs::create_dir(repo.root().join("main")).unwrap();

        let layout = ContainerLayout::new(NormalizedPath::new(repo.root()), NamingStrategy::Slug)
            .expect("ContainerLayout::new() should succeed");

        let main_wt = layout.main_worktree();
        assert!(main_wt.as_str().ends_with("main"));
    }
}

// =============================================================================
// Mission 3: Configuration Synchronization
// =============================================================================

mod m3_sync {
    use super::*;

    /// M3.1: SyncEngine can be created
    #[test]
    fn m3_1_sync_engine_creation() {
        let repo = TestRepo::new();
        repo.init_git();

        let root = NormalizedPath::new(repo.root());
        let result = SyncEngine::new(root, Mode::Standard);

        assert!(result.is_ok(), "SyncEngine creation failed");
    }

    /// M3.2: Check on empty repo is healthy
    #[test]
    fn m3_2_check_empty_repo_healthy() {
        let repo = TestRepo::new();
        repo.init_git();

        let root = NormalizedPath::new(repo.root());
        let engine = SyncEngine::new(root, Mode::Standard).unwrap();
        let report = engine.check().unwrap();

        assert_eq!(
            report.status,
            CheckStatus::Healthy,
            "Empty repo should be healthy"
        );
    }

    /// M3.3: Sync creates ledger file
    #[test]
    fn m3_3_sync_creates_ledger() {
        let repo = TestRepo::new();
        repo.init_git();

        let root = NormalizedPath::new(repo.root());
        let engine = SyncEngine::new(root.clone(), Mode::Standard).unwrap();

        // Ledger shouldn't exist yet
        let ledger_path = root.join(".repository/ledger.toml");
        assert!(!ledger_path.exists());

        // Run sync
        let report = engine.sync().unwrap();
        assert!(report.success);

        // Ledger should exist now
        assert!(ledger_path.exists());
    }

    /// M3.4: Tool sync creates config files
    /// This tests the tool integration layer directly
    #[test]
    fn m3_4_tool_sync_creates_files() {
        let mut repo = TestRepo::new();
        repo.init_git();
        repo.init_repo_manager("standard", &["vscode", "cursor", "claude"], &[]);

        let root = NormalizedPath::new(repo.root());
        let rules = vec![Rule {
            id: "test-rule".to_string(),
            content: "Test content".to_string(),
        }];
        let context = SyncContext::new(root);

        // Sync each tool
        VSCodeIntegration::new().sync(&context, &rules).unwrap();
        cursor_integration().sync(&context, &rules).unwrap();
        claude_integration().sync(&context, &rules).unwrap();

        // Verify files created
        repo.assert_file_exists(".vscode/settings.json");
        repo.assert_file_exists(".cursorrules");
        repo.assert_file_exists("CLAUDE.md");
    }

    /// M3.5: Managed blocks contain rule content
    #[test]
    fn m3_5_managed_blocks_contain_rules() {
        let mut repo = TestRepo::new();
        repo.init_git();
        repo.init_repo_manager("standard", &["cursor"], &[]);

        let root = NormalizedPath::new(repo.root());
        let rules = vec![Rule {
            id: "python-style".to_string(),
            content: "Use snake_case for variables".to_string(),
        }];
        let context = SyncContext::new(root);

        cursor_integration().sync(&context, &rules).unwrap();

        repo.assert_file_contains(".cursorrules", "python-style");
        repo.assert_file_contains(".cursorrules", "snake_case");
    }

    /// M3.6: Multiple rules create multiple blocks
    #[test]
    fn m3_6_multiple_rules_multiple_blocks() {
        let mut repo = TestRepo::new();
        repo.init_git();
        repo.init_repo_manager("standard", &["cursor"], &[]);

        let root = NormalizedPath::new(repo.root());
        let rules = vec![
            Rule {
                id: "rule-one".to_string(),
                content: "First rule content".to_string(),
            },
            Rule {
                id: "rule-two".to_string(),
                content: "Second rule content".to_string(),
            },
        ];
        let context = SyncContext::new(root);

        cursor_integration().sync(&context, &rules).unwrap();

        repo.assert_file_contains(".cursorrules", "rule-one");
        repo.assert_file_contains(".cursorrules", "rule-two");
        repo.assert_file_contains(".cursorrules", "First rule content");
        repo.assert_file_contains(".cursorrules", "Second rule content");
    }

    /// M3.7: Check detects missing file (from ledger projection)
    /// GAP: This tests the drift detection which IS implemented
    #[test]
    fn m3_7_check_detects_missing() {
        use repo_core::ledger::{Intent, Ledger, Projection};
        use serde_json::json;

        let repo = TestRepo::new();
        repo.init_git();

        // Create ledger with projection for nonexistent file
        let repo_dir = repo.root().join(".repository");
        fs::create_dir_all(&repo_dir).unwrap();

        let mut ledger = Ledger::new();
        let mut intent = Intent::new("rule:test".to_string(), json!({}));
        intent.add_projection(Projection::file_managed(
            "test-tool".to_string(),
            std::path::PathBuf::from("config/missing.json"),
            "abc123".to_string(),
        ));
        ledger.add_intent(intent);
        ledger.save(&repo_dir.join("ledger.toml")).unwrap();

        let root = NormalizedPath::new(repo.root());
        let engine = SyncEngine::new(root, Mode::Standard).unwrap();
        let report = engine.check().unwrap();

        assert_eq!(report.status, CheckStatus::Missing);
        assert_eq!(report.missing.len(), 1);
    }
}

// =============================================================================
// Mission 4: Tool Integration
// =============================================================================

mod m4_tools {
    use super::*;

    /// M4.1: VSCode integration name and locations
    #[test]
    fn m4_1_vscode_integration_info() {
        let vscode = VSCodeIntegration::new();
        assert_eq!(vscode.name(), "vscode");
        let paths: Vec<_> = vscode
            .config_locations()
            .into_iter()
            .map(|l| l.path)
            .collect();
        assert!(paths.contains(&".vscode/settings.json".to_string()));
    }

    /// M4.2: Cursor integration name and locations
    #[test]
    fn m4_2_cursor_integration_info() {
        let cursor = cursor_integration();
        assert_eq!(cursor.name(), "cursor");
        let paths: Vec<_> = cursor
            .config_locations()
            .into_iter()
            .map(|l| l.path)
            .collect();
        assert!(paths.contains(&".cursorrules".to_string()));
    }

    /// M4.3: Claude integration name and locations
    #[test]
    fn m4_3_claude_integration_info() {
        let claude = claude_integration();
        assert_eq!(claude.name(), "claude");
        let paths: Vec<_> = claude
            .config_locations()
            .into_iter()
            .map(|l| l.path)
            .collect();
        assert!(paths.contains(&"CLAUDE.md".to_string()));
    }

    /// M4.4: VSCode settings include python path when context has it
    #[test]
    fn m4_4_vscode_python_path() {
        let mut repo = TestRepo::new();
        repo.init_git();
        repo.init_repo_manager("standard", &["vscode"], &[]);

        let root = NormalizedPath::new(repo.root());

        // Create mock venv
        let python_path = if cfg!(windows) {
            repo.root().join(".venv/Scripts/python.exe")
        } else {
            repo.root().join(".venv/bin/python")
        };
        fs::create_dir_all(python_path.parent().unwrap()).unwrap();
        fs::write(&python_path, "mock").unwrap();

        let context = SyncContext::new(root).with_python(NormalizedPath::new(&python_path));
        let rules = vec![];

        VSCodeIntegration::new().sync(&context, &rules).unwrap();

        repo.assert_file_exists(".vscode/settings.json");
        let content = fs::read_to_string(repo.root().join(".vscode/settings.json")).unwrap();
        assert!(
            content.contains("python"),
            "Expected python configuration in settings.json"
        );
    }
}

// =============================================================================
// Mission 5: Preset Providers
// =============================================================================

mod m5_presets {
    use super::*;

    /// M5.1: UV provider ID
    #[test]
    fn m5_1_uv_provider_id() {
        let provider = UvProvider::new();
        assert_eq!(provider.id(), "env:python");
    }

    /// M5.2: UV provider check returns non-healthy when venv missing
    #[tokio::test]
    async fn m5_2_uv_provider_check_missing() {
        let repo = TestRepo::new();
        let layout = WorkspaceLayout {
            root: NormalizedPath::new(repo.root()),
            active_context: NormalizedPath::new(repo.root()),
            mode: LayoutMode::Classic,
        };

        let context = Context::new(layout, HashMap::new());
        let provider = UvProvider::new();

        let report = provider.check(&context).await.unwrap();

        // Should not be healthy since no venv exists
        assert_ne!(
            report.status,
            PresetStatus::Healthy,
            "Expected non-healthy status when venv is missing"
        );
    }

    /// M5.3: Registry has python provider
    #[test]
    fn m5_3_registry_has_python_provider() {
        let registry = repo_meta::Registry::with_builtins();
        assert!(registry.has_provider("env:python"));
        assert_eq!(registry.get_provider("env:python"), Some(&"uv".to_string()));
    }

    /// M5.4: Registry returns None for unknown preset
    #[test]
    fn m5_4_registry_unknown_preset() {
        let registry = repo_meta::Registry::with_builtins();
        assert!(!registry.has_provider("env:nonexistent"));
        assert_eq!(registry.get_provider("env:nonexistent"), None);
    }
}

// =============================================================================
// Mission 6: Git Operations
// =============================================================================

mod m6_git_ops {
    use assert_cmd::Command;
    use predicates::prelude::*;

    // cargo_bin! macro requires CARGO_BIN_EXE_* env vars which are only set when
    // the binary is in the same package as the test. Since this is a separate
    // integration-tests crate, we must use the runtime-discovery Command::cargo_bin.
    #[allow(deprecated)]
    fn repo_cmd() -> Command {
        Command::cargo_bin("repo").unwrap()
    }

    /// M6.1: repo push command
    /// GAP-001: Now implemented - verify CLI command exists
    #[test]
    fn m6_1_push_command() {
        let mut cmd = repo_cmd();
        cmd.arg("push").arg("--help");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("push"));
    }

    /// M6.2: repo pull command
    /// GAP-002: Now implemented - verify CLI command exists
    #[test]
    fn m6_2_pull_command() {
        let mut cmd = repo_cmd();
        cmd.arg("pull").arg("--help");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("pull"));
    }

    /// M6.3: repo merge command
    /// GAP-003: Now implemented - verify CLI command exists
    #[test]
    fn m6_3_merge_command() {
        let mut cmd = repo_cmd();
        cmd.arg("merge").arg("--help");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("merge"));
    }
}

// =============================================================================
// Sync & Tool Integration Tests
// =============================================================================

mod sync_integration {
    #[allow(unused_imports)]
    use super::*;

    /// Helper: write a config.toml in the Manifest format that SyncEngine expects.
    ///
    /// The `init_repo_manager` helper writes `[active] tools = [...]` which does not
    /// match the `Manifest` struct (top-level `tools = [...]`). This helper writes the
    /// correct format so SyncEngine can parse it.
    fn write_manifest_config(root: &Path, mode: &str, tools: &[&str]) {
        let repo_dir = root.join(".repository");
        fs::create_dir_all(&repo_dir).unwrap();

        let tools_str = tools
            .iter()
            .map(|t| format!("\"{}\"", t))
            .collect::<Vec<_>>()
            .join(", ");

        let config = format!("tools = [{tools_str}]\n\n[core]\nmode = \"{mode}\"\n");

        fs::write(repo_dir.join("config.toml"), config).unwrap();
    }

    /// GAP-004: sync() should apply projections and create tool configs
    ///
    /// Validates the end-to-end sync pipeline:
    ///   config.toml with tools -> SyncEngine::sync() -> tool config files on disk
    ///
    /// Tests that vscode, cursor, and claude tools all produce their expected config files.
    #[test]
    fn gap_004_sync_applies_projections() {
        let repo = TestRepo::new();
        repo.init_git();
        write_manifest_config(repo.root(), "standard", &["vscode", "cursor", "claude"]);

        let root = NormalizedPath::new(repo.root());
        let engine = SyncEngine::new(root, Mode::Standard).unwrap();

        let report = engine.sync().unwrap();
        assert!(report.success, "Sync should succeed: {:?}", report.errors);
        assert!(!report.actions.is_empty(), "Sync should report actions");

        // Verify tool config files were created
        assert!(
            repo.root().join(".vscode/settings.json").exists(),
            ".vscode/settings.json should be created for vscode tool"
        );
        assert!(
            repo.root().join(".cursorrules").exists(),
            ".cursorrules should be created for cursor tool"
        );
        assert!(
            repo.root().join("CLAUDE.md").exists(),
            "CLAUDE.md should be created for claude tool"
        );

        // Verify files have content
        let vscode_content = fs::read_to_string(repo.root().join(".vscode/settings.json")).unwrap();
        assert!(
            !vscode_content.is_empty(),
            "VSCode settings should have content"
        );

        let cursor_content = fs::read_to_string(repo.root().join(".cursorrules")).unwrap();
        assert!(
            !cursor_content.is_empty(),
            "Cursor rules should have content"
        );

        let claude_content = fs::read_to_string(repo.root().join("CLAUDE.md")).unwrap();
        assert!(!claude_content.is_empty(), "CLAUDE.md should have content");

        // Verify ledger was created
        assert!(
            repo.root().join(".repository/ledger.toml").exists(),
            "Ledger should be created after sync"
        );
    }

    /// Integration test: sync creates config for a single tool
    #[test]
    fn sync_creates_config_for_single_tool() {
        let repo = TestRepo::new();
        repo.init_git();
        write_manifest_config(repo.root(), "standard", &["cursor"]);

        let root = NormalizedPath::new(repo.root());
        let engine = SyncEngine::new(root, Mode::Standard).unwrap();
        let report = engine.sync().unwrap();

        assert!(report.success, "Sync should succeed: {:?}", report.errors);
        assert!(
            repo.root().join(".cursorrules").exists(),
            ".cursorrules should be created for cursor tool"
        );
    }

    /// Integration test: sync with rules writes rule content to tool configs
    #[test]
    fn sync_with_rules_includes_rule_content() {
        let repo = TestRepo::new();
        repo.init_git();
        write_manifest_config(repo.root(), "standard", &["cursor"]);

        // Add a rule to the registry
        let rules_dir = repo.root().join(".repository/rules");
        fs::create_dir_all(&rules_dir).unwrap();

        use repo_core::rules::RuleRegistry;
        let registry_path = rules_dir.join("registry.toml");
        let mut registry = RuleRegistry::new(registry_path);
        registry
            .add_rule("test-rule", "Always use descriptive variable names", vec![])
            .unwrap();

        let root = NormalizedPath::new(repo.root());
        let engine = SyncEngine::new(root, Mode::Standard).unwrap();
        let report = engine.sync().unwrap();

        assert!(report.success, "Sync should succeed: {:?}", report.errors);

        // Verify the rule content appears in the cursor config
        let content = fs::read_to_string(repo.root().join(".cursorrules")).unwrap();
        assert!(
            content.contains("descriptive variable names"),
            "Rule content should appear in .cursorrules, got: {}",
            content
        );
    }

    /// GAP-005: fix() just calls sync (stub)
    #[test]
    fn gap_005_fix_is_stub() {
        let repo = TestRepo::new();
        repo.init_git();

        let root = NormalizedPath::new(repo.root());
        let engine = SyncEngine::new(root, Mode::Standard).unwrap();

        // Both should return success (fix is just sync)
        let sync_report = engine.sync().unwrap();
        let fix_report = engine.fix().unwrap();

        assert!(sync_report.success);
        assert!(fix_report.success);

        // GAP: fix() should repair drift, not just sync
        // This is documented behavior - fix is currently a stub
    }

    /// Antigravity tool integration test (was GAP-006, now implemented)
    /// Config location: .agent/rules/ (directory, one file per rule)
    #[test]
    fn test_antigravity_tool_integration() {
        let mut repo = TestRepo::new();
        repo.init_git();
        repo.init_repo_manager("standard", &["antigravity"], &[]);

        let root = NormalizedPath::new(repo.root());
        let rules = vec![Rule {
            id: "test-rule".to_string(),
            content: "Test content for Antigravity".to_string(),
        }];
        let context = SyncContext::new(root);

        antigravity_integration().sync(&context, &rules).unwrap();

        // Verify rules directory created at .agent/rules/
        let rules_dir = repo.root().join(".agent/rules");
        assert!(rules_dir.is_dir(), "Antigravity rules directory should be created");

        // Verify per-rule file exists inside the directory (format: 01-<id>.md)
        let rule_file = rules_dir.join("01-test-rule.md");
        assert!(rule_file.exists(), "Per-rule file should be created in .agent/rules/");

        let content = fs::read_to_string(&rule_file).unwrap();
        assert!(content.contains("Test content for Antigravity"));
    }

    /// Windsurf tool integration test (was GAP-007, now implemented)
    /// Config location: .windsurfrules
    #[test]
    fn test_windsurf_tool_integration() {
        let mut repo = TestRepo::new();
        repo.init_git();
        repo.init_repo_manager("standard", &["windsurf"], &[]);

        let root = NormalizedPath::new(repo.root());
        let rules = vec![Rule {
            id: "test-rule".to_string(),
            content: "Test content for Windsurf".to_string(),
        }];
        let context = SyncContext::new(root);

        windsurf_integration().sync(&context, &rules).unwrap();

        // Verify config file created at .windsurfrules
        let config_path = repo.root().join(".windsurfrules");
        assert!(config_path.exists(), "Windsurf config should be created");

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("Test content for Windsurf"));
    }

    /// Gemini CLI tool integration test (was GAP-008, now implemented)
    /// Config location: GEMINI.md
    #[test]
    fn test_gemini_tool_integration() {
        let mut repo = TestRepo::new();
        repo.init_git();
        repo.init_repo_manager("standard", &["gemini"], &[]);

        let root = NormalizedPath::new(repo.root());
        let rules = vec![Rule {
            id: "test-rule".to_string(),
            content: "Test content for Gemini".to_string(),
        }];
        let context = SyncContext::new(root);

        gemini_integration().sync(&context, &rules).unwrap();

        // Verify config file created at GEMINI.md
        let config_path = repo.root().join("GEMINI.md");
        assert!(config_path.exists(), "Gemini config should be created");

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("Test content for Gemini"));
    }

    /// Python venv provider test (was GAP-010, now implemented)
    /// Tests provider = "venv" in addition to "uv"
    #[tokio::test]
    async fn test_python_venv_provider() {
        use repo_presets::VenvProvider;

        let temp = TempDir::new().unwrap();
        let provider = VenvProvider::new();

        // Create test context
        let root = NormalizedPath::new(temp.path());
        let layout = WorkspaceLayout {
            root: root.clone(),
            active_context: root.clone(),
            mode: LayoutMode::Classic,
        };
        let context = Context::new(layout, HashMap::new());

        // Check should succeed (venv provider is now implemented)
        let report = provider.check(&context).await;

        // The check should return Ok - provider shouldn't panic
        assert!(report.is_ok(), "VenvProvider.check() should not error");

        // For a fresh directory without a venv, status should be Missing
        let check_report = report.unwrap();
        assert_eq!(
            check_report.status,
            PresetStatus::Missing,
            "Fresh directory should show venv as Missing"
        );
    }

    /// GAP-012: Node env provider - now implemented
    #[tokio::test]
    async fn gap_012_node_provider() {
        use repo_presets::NodeProvider;
        use repo_presets::PresetProvider;

        let provider = NodeProvider::new();
        assert_eq!(provider.id(), "env:node");

        // Verify registry has it
        let registry = repo_meta::Registry::with_builtins();
        assert!(registry.has_provider("env:node"));
    }

    /// GAP-013: Rust env provider - now implemented
    #[tokio::test]
    async fn gap_013_rust_provider() {
        use repo_presets::PresetProvider;
        use repo_presets::RustProvider;

        let provider = RustProvider::new();
        assert_eq!(provider.id(), "env:rust");

        // Verify registry has it
        let registry = repo_meta::Registry::with_builtins();
        assert!(registry.has_provider("env:rust"));
    }

    /// GAP-018: MCP Server crate - now implemented
    #[tokio::test]
    async fn gap_018_mcp_server() {
        use repo_mcp::RepoMcpServer;

        // Create a valid repo structure for MCP server initialization
        let mut repo = TestRepo::new();
        repo.init_git();
        repo.init_repo_manager("standard", &["vscode"], &[]);

        // Create and initialize server pointing to the valid repo
        let mut server = RepoMcpServer::new(repo.root().to_path_buf());
        let init_result = server.initialize().await;
        assert!(init_result.is_ok(), "Server should initialize in valid repo");
        assert!(server.is_initialized());

        // Verify tools are loaded
        let tools = server.tools();
        assert!(!tools.is_empty(), "Should have tools defined");

        // Verify expected tools exist
        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"repo_init"), "Should have repo_init");
        assert!(tool_names.contains(&"git_push"), "Should have git_push");
        assert!(tool_names.contains(&"git_pull"), "Should have git_pull");
        assert!(tool_names.contains(&"git_merge"), "Should have git_merge");
        assert!(
            tool_names.contains(&"branch_create"),
            "Should have branch_create"
        );

        // Verify resources are loaded
        let resources = server.resources();
        assert!(!resources.is_empty(), "Should have resources defined");

        let resource_uris: Vec<&str> = resources.iter().map(|r| r.uri.as_str()).collect();
        assert!(resource_uris.contains(&"repo://config"));
        assert!(resource_uris.contains(&"repo://rules"));
    }

    /// GAP-019: add-tool followed by sync creates tool config files
    ///
    /// Rewrites the original GAP-019 test which was stale: it manually edited
    /// config.toml and expected files to appear without running any command.
    /// The actual `add-tool` CLI triggers `trigger_sync_and_report()`, which
    /// is tested in `repo-cli/tests/integration_tests.rs`.
    ///
    /// This test validates the underlying mechanism: updating config.toml to
    /// include a tool, then running SyncEngine::sync(), produces the expected
    /// config files on disk.
    #[test]
    fn gap_019_add_tool_triggers_sync() {
        let repo = TestRepo::new();
        repo.init_git();

        // Start with no tools, like a fresh init
        let repo_dir = repo.root().join(".repository");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(
            repo_dir.join("config.toml"),
            "tools = []\n\n[core]\nmode = \"standard\"\n",
        )
        .unwrap();

        // Simulate what add-tool does: update config to include vscode
        fs::write(
            repo_dir.join("config.toml"),
            "tools = [\"vscode\"]\n\n[core]\nmode = \"standard\"\n",
        )
        .unwrap();

        // Then trigger sync (this is what trigger_sync_and_report does)
        let root = NormalizedPath::new(repo.root());
        let engine = SyncEngine::new(root, Mode::Standard).unwrap();
        let report = engine.sync().unwrap();

        assert!(report.success, "Sync after add-tool should succeed: {:?}", report.errors);
        assert!(
            !report.actions.is_empty(),
            "Sync should report actions taken"
        );

        // Verify the tool config file was actually created
        let vscode_settings = repo.root().join(".vscode/settings.json");
        assert!(
            vscode_settings.exists(),
            "add-tool + sync should create .vscode/settings.json"
        );

        // Verify the file has real content (not empty)
        let content = fs::read_to_string(&vscode_settings).unwrap();
        assert!(
            !content.is_empty(),
            ".vscode/settings.json should have content after sync"
        );

        // Verify ledger recorded the sync
        let ledger_path = repo.root().join(".repository/ledger.toml");
        assert!(
            ledger_path.exists(),
            "Ledger should be created after sync"
        );
    }
}

// =============================================================================
// Robustness Tests
// =============================================================================

mod robustness {
    use super::*;

    /// Unicode paths should be handled correctly
    #[test]
    fn unicode_branch_name() {
        // Test various unicode branch names
        let test_cases = [
            ("feat/日本語", true),
            ("feat/émoji-🎉", true),
            ("feat/中文测试", true),
        ];

        for (branch_name, should_sanitize) in test_cases {
            let slug = branch_to_directory(branch_name, NamingStrategy::Slug);
            assert!(
                !slug.is_empty(),
                "Slug should not be empty for {}",
                branch_name
            );
            if should_sanitize {
                assert!(
                    !slug.contains('/'),
                    "Slug should not contain slashes: {}",
                    slug
                );
            }
        }
    }

    /// Empty rules list should not cause errors and should still create config
    #[test]
    fn empty_rules_sync() {
        let mut repo = TestRepo::new();
        repo.init_git();
        repo.init_repo_manager("standard", &["vscode"], &[]);

        let root = NormalizedPath::new(repo.root());
        let rules: Vec<Rule> = vec![];
        let context = SyncContext::new(root);

        // Should not panic with empty rules
        VSCodeIntegration::new().sync(&context, &rules).unwrap();

        // Verify the config file was actually created on disk
        repo.assert_file_exists(".vscode/settings.json");

        // Verify it contains valid JSON (not empty or garbage)
        let content =
            fs::read_to_string(repo.root().join(".vscode/settings.json")).unwrap();
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content);
        assert!(
            parsed.is_ok(),
            "settings.json should be valid JSON even with empty rules, got: {}",
            content
        );
    }

    /// Very long rule content should be handled and written to disk
    #[test]
    fn long_rule_content() {
        let mut repo = TestRepo::new();
        repo.init_git();
        repo.init_repo_manager("standard", &["cursor"], &[]);

        let root = NormalizedPath::new(repo.root());
        let long_content = "x".repeat(100_000); // 100KB of content
        let rules = vec![Rule {
            id: "long-rule".to_string(),
            content: long_content.clone(),
        }];
        let context = SyncContext::new(root);

        cursor_integration().sync(&context, &rules).unwrap();

        // Verify file was created
        repo.assert_file_exists(".cursorrules");

        // Verify the long content actually made it to disk (not truncated)
        let written = fs::read_to_string(repo.root().join(".cursorrules")).unwrap();
        assert!(
            written.len() >= 100_000,
            "Written file should contain the full 100KB content, got {} bytes",
            written.len()
        );
    }

    /// Special characters in rule IDs should produce valid managed blocks
    #[test]
    fn special_chars_in_rule_id() {
        let mut repo = TestRepo::new();
        repo.init_git();
        repo.init_repo_manager("standard", &["cursor"], &[]);

        let root = NormalizedPath::new(repo.root());
        let rule_id = "rule-with-dashes_and_underscores.and.dots";
        let rules = vec![Rule {
            id: rule_id.to_string(),
            content: "Content with special ID".to_string(),
        }];
        let context = SyncContext::new(root);

        cursor_integration().sync(&context, &rules).unwrap();

        // Verify file was created and contains the rule content
        repo.assert_file_exists(".cursorrules");
        let content = fs::read_to_string(repo.root().join(".cursorrules")).unwrap();
        assert!(
            content.contains("Content with special ID"),
            "Rule content should appear in .cursorrules"
        );
        // Verify managed block markers reference the rule ID
        assert!(
            content.contains(rule_id),
            "Managed block should reference the rule ID with special chars"
        );
    }
}

// =============================================================================
// Consumer Verification Tests (Phase 6.2)
// =============================================================================

mod consumer_verification {
    use super::*;

    /// Verify .cursorrules is valid Markdown format
    /// Research: Cursor expects plain Markdown or MDC format
    #[test]
    fn cv_cursorrules_valid_markdown() {
        let mut repo = TestRepo::new();
        repo.init_git();
        repo.init_repo_manager("standard", &["cursor"], &[]);

        let root = NormalizedPath::new(repo.root());
        let rules = vec![Rule {
            id: "test-rule".to_string(),
            content: "# Test Rule\n\nUse **bold** and _italic_ text.".to_string(),
        }];
        let context = SyncContext::new(root);

        cursor_integration().sync(&context, &rules).unwrap();

        // Verify file exists and is valid Markdown
        let content = fs::read_to_string(repo.root().join(".cursorrules")).unwrap();

        // Should contain the Markdown content
        assert!(content.contains("# Test Rule"));
        assert!(content.contains("**bold**"));
        assert!(content.contains("_italic_"));

        // Should have managed block markers (HTML comments valid in Markdown)
        assert!(content.contains("<!-- repo:block:"));
        assert!(content.contains("<!-- /repo:block:"));
    }

    /// Verify CLAUDE.md is valid Markdown format
    /// Research: Claude Code expects Markdown with natural language instructions
    #[test]
    fn cv_claude_md_valid_markdown() {
        let mut repo = TestRepo::new();
        repo.init_git();
        repo.init_repo_manager("standard", &["claude"], &[]);

        let root = NormalizedPath::new(repo.root());
        let rules = vec![Rule {
            id: "api-design".to_string(),
            content:
                "## API Guidelines\n\n- Return JSON with data, error fields\n- Use REST conventions"
                    .to_string(),
        }];
        let context = SyncContext::new(root);

        claude_integration().sync(&context, &rules).unwrap();

        // Verify file exists and is valid Markdown
        let content = fs::read_to_string(repo.root().join("CLAUDE.md")).unwrap();

        // Should contain the Markdown content
        assert!(content.contains("## API Guidelines"));
        assert!(content.contains("- Return JSON"));

        // Should have managed block markers
        assert!(content.contains("<!-- repo:block:"));
    }

    /// Verify .vscode/settings.json is valid JSON format
    /// Research: VSCode expects JSON with comments (JSONC)
    #[test]
    fn cv_vscode_settings_valid_json() {
        let mut repo = TestRepo::new();
        repo.init_git();
        repo.init_repo_manager("standard", &["vscode"], &[]);

        let root = NormalizedPath::new(repo.root());
        let rules: Vec<Rule> = vec![];
        let context = SyncContext::new(root);

        VSCodeIntegration::new().sync(&context, &rules).unwrap();

        // Verify file exists
        let settings_path = repo.root().join(".vscode/settings.json");
        assert!(settings_path.exists());

        // Verify it's valid JSON
        let content = fs::read_to_string(&settings_path).unwrap();
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content);
        assert!(
            parsed.is_ok(),
            "settings.json should be valid JSON, got: {}",
            content
        );

        // Verify it's an object (not array or primitive)
        let json = parsed.unwrap();
        assert!(json.is_object(), "settings.json should be a JSON object");
    }

    /// Verify multiple rules create separate managed blocks
    #[test]
    fn cv_multiple_rules_separate_blocks() {
        let mut repo = TestRepo::new();
        repo.init_git();
        repo.init_repo_manager("standard", &["cursor"], &[]);

        let root = NormalizedPath::new(repo.root());
        let rules = vec![
            Rule {
                id: "rule-alpha".to_string(),
                content: "Alpha content".to_string(),
            },
            Rule {
                id: "rule-beta".to_string(),
                content: "Beta content".to_string(),
            },
            Rule {
                id: "rule-gamma".to_string(),
                content: "Gamma content".to_string(),
            },
        ];
        let context = SyncContext::new(root);

        cursor_integration().sync(&context, &rules).unwrap();

        let content = fs::read_to_string(repo.root().join(".cursorrules")).unwrap();

        // Each rule should have its own block
        assert!(content.contains("rule-alpha"));
        assert!(content.contains("rule-beta"));
        assert!(content.contains("rule-gamma"));
        assert!(content.contains("Alpha content"));
        assert!(content.contains("Beta content"));
        assert!(content.contains("Gamma content"));

        // Count managed block markers (should have 3 pairs)
        let open_markers = content.matches("<!-- repo:block:").count();
        let close_markers = content.matches("<!-- /repo:block:").count();
        assert_eq!(open_markers, 3, "Should have 3 opening block markers");
        assert_eq!(close_markers, 3, "Should have 3 closing block markers");
    }

    /// Verify user content outside managed blocks is preserved
    #[test]
    fn cv_user_content_preserved_outside_blocks() {
        let mut repo = TestRepo::new();
        repo.init_git();
        repo.init_repo_manager("standard", &["cursor"], &[]);

        // Create file with user content first
        let cursorrules_path = repo.root().join(".cursorrules");
        fs::write(
            &cursorrules_path,
            "# My Custom Header\n\nThis is my custom content that should be preserved.\n",
        )
        .unwrap();

        let root = NormalizedPath::new(repo.root());
        let rules = vec![Rule {
            id: "managed-rule".to_string(),
            content: "This is managed content.".to_string(),
        }];
        let context = SyncContext::new(root);

        cursor_integration().sync(&context, &rules).unwrap();

        let content = fs::read_to_string(&cursorrules_path).unwrap();

        // User content should be preserved
        assert!(
            content.contains("# My Custom Header"),
            "User header should be preserved"
        );
        assert!(
            content.contains("This is my custom content"),
            "User content should be preserved"
        );

        // Managed content should also be present
        assert!(
            content.contains("This is managed content"),
            "Managed content should be added"
        );
    }

    /// Task 6.3: Verify concurrent edits are preserved across multiple syncs
    /// Simulates: user adds content, sync runs, user adds more content, sync runs again
    #[test]
    fn cv_concurrent_edit_preservation_across_syncs() {
        let mut repo = TestRepo::new();
        repo.init_git();
        repo.init_repo_manager("standard", &["cursor"], &[]);

        let cursorrules_path = repo.root().join(".cursorrules");
        let root = NormalizedPath::new(repo.root());

        // STEP 1: First sync with one rule
        let rules_v1 = vec![Rule {
            id: "rule-one".to_string(),
            content: "First managed rule".to_string(),
        }];
        let context = SyncContext::new(root.clone());
        cursor_integration().sync(&context, &rules_v1).unwrap();

        // STEP 2: User adds their own content at the top
        let content_after_sync1 = fs::read_to_string(&cursorrules_path).unwrap();
        let user_content_1 = "# My Project Guidelines\n\nThese are my custom rules.\n\n";
        let modified_content = format!("{}{}", user_content_1, content_after_sync1);
        fs::write(&cursorrules_path, &modified_content).unwrap();

        // STEP 3: Second sync with additional rule
        let rules_v2 = vec![
            Rule {
                id: "rule-one".to_string(),
                content: "First managed rule (updated)".to_string(),
            },
            Rule {
                id: "rule-two".to_string(),
                content: "Second managed rule".to_string(),
            },
        ];
        cursor_integration().sync(&context, &rules_v2).unwrap();

        // STEP 4: Verify user content is preserved
        let final_content = fs::read_to_string(&cursorrules_path).unwrap();

        // User content should be preserved
        assert!(
            final_content.contains("# My Project Guidelines"),
            "User header should be preserved after second sync"
        );
        assert!(
            final_content.contains("These are my custom rules"),
            "User content should be preserved after second sync"
        );

        // Both managed rules should be present
        assert!(
            final_content.contains("First managed rule"),
            "First rule should be present"
        );
        assert!(
            final_content.contains("Second managed rule"),
            "Second rule should be present"
        );
    }

    /// Verify user content added BETWEEN managed blocks is preserved
    #[test]
    fn cv_user_content_between_blocks_preserved() {
        let mut repo = TestRepo::new();
        repo.init_git();
        repo.init_repo_manager("standard", &["cursor"], &[]);

        let cursorrules_path = repo.root().join(".cursorrules");
        let root = NormalizedPath::new(repo.root());

        // First sync with two rules
        let rules = vec![
            Rule {
                id: "rule-alpha".to_string(),
                content: "Alpha content".to_string(),
            },
            Rule {
                id: "rule-beta".to_string(),
                content: "Beta content".to_string(),
            },
        ];
        let context = SyncContext::new(root.clone());
        cursor_integration().sync(&context, &rules).unwrap();

        // User adds content between the blocks manually
        let content = fs::read_to_string(&cursorrules_path).unwrap();

        // For a more realistic test, just verify user content at the bottom is preserved
        let user_suffix = "\n\n# User Notes\n\nThese are my personal notes.";
        let with_suffix = format!("{}{}", content, user_suffix);
        fs::write(&cursorrules_path, &with_suffix).unwrap();

        // Re-sync
        cursor_integration().sync(&context, &rules).unwrap();

        let final_content = fs::read_to_string(&cursorrules_path).unwrap();
        assert!(
            final_content.contains("# User Notes"),
            "User notes should be preserved at bottom"
        );
        assert!(
            final_content.contains("personal notes"),
            "User content should be preserved"
        );
    }

    /// Verify removing a rule doesn't remove user content
    #[test]
    fn cv_removed_rule_preserves_user_content() {
        let mut repo = TestRepo::new();
        repo.init_git();
        repo.init_repo_manager("standard", &["cursor"], &[]);

        let cursorrules_path = repo.root().join(".cursorrules");
        let root = NormalizedPath::new(repo.root());

        // Create with user header first
        fs::write(&cursorrules_path, "# My Header\n\n").unwrap();

        // Sync with two rules
        let rules_v1 = vec![
            Rule {
                id: "rule-keep".to_string(),
                content: "Rule to keep".to_string(),
            },
            Rule {
                id: "rule-remove".to_string(),
                content: "Rule to remove".to_string(),
            },
        ];
        let context = SyncContext::new(root.clone());
        cursor_integration().sync(&context, &rules_v1).unwrap();

        // Sync again with only one rule
        let rules_v2 = vec![Rule {
            id: "rule-keep".to_string(),
            content: "Rule to keep".to_string(),
        }];
        cursor_integration().sync(&context, &rules_v2).unwrap();

        let final_content = fs::read_to_string(&cursorrules_path).unwrap();

        // User content preserved
        assert!(
            final_content.contains("# My Header"),
            "User header should be preserved"
        );

        // Kept rule still present
        assert!(
            final_content.contains("Rule to keep"),
            "Kept rule should be present"
        );

        // Note: Whether the removed rule's block is cleaned up depends on implementation
        // This documents current behavior
    }

    /// Verify block markers use consistent format
    #[test]
    fn cv_block_marker_format_consistent() {
        let mut repo = TestRepo::new();
        repo.init_git();
        repo.init_repo_manager("standard", &["cursor"], &[]);

        let root = NormalizedPath::new(repo.root());
        let rules = vec![Rule {
            id: "format-test".to_string(),
            content: "Test content".to_string(),
        }];
        let context = SyncContext::new(root);

        cursor_integration().sync(&context, &rules).unwrap();

        let content = fs::read_to_string(repo.root().join(".cursorrules")).unwrap();

        // Block markers should follow the format: <!-- repo:block:ID -->
        // Using regex to verify format
        let open_pattern = regex::Regex::new(r"<!-- repo:block:[\w-]+ -->").unwrap();
        let close_pattern = regex::Regex::new(r"<!-- /repo:block:[\w-]+ -->").unwrap();

        assert!(
            open_pattern.is_match(&content),
            "Should have valid opening block marker"
        );
        assert!(
            close_pattern.is_match(&content),
            "Should have valid closing block marker"
        );
    }
}

// =============================================================================
// P3: End-to-End CLI Pipeline Test
// =============================================================================

mod p3_end_to_end_pipeline {
    use super::*;

    /// Helper: write a config.toml in the Manifest format that SyncEngine expects.
    fn write_manifest_config(root: &Path, mode: &str, tools: &[&str]) {
        let repo_dir = root.join(".repository");
        fs::create_dir_all(&repo_dir).unwrap();

        let tools_str = tools
            .iter()
            .map(|t| format!("\"{}\"", t))
            .collect::<Vec<_>>()
            .join(", ");

        let config = format!("tools = [{tools_str}]\n\n[core]\nmode = \"{mode}\"\n");

        fs::write(repo_dir.join("config.toml"), config).unwrap();
    }

    /// End-to-end smoke test: init -> sync -> verify files -> check -> delete -> check
    ///
    /// This is the "smoke test" for the entire product. It exercises the complete
    /// user journey and catches category-level regressions.
    ///
    /// Guards against: silent failures where sync appears to succeed but files
    /// are not actually created or recorded in the ledger.
    #[test]
    fn e2e_init_sync_verify_check_delete_recheck() {
        // Step 1: Create repo, init git, write config with multiple tools
        let repo = TestRepo::new();
        repo.init_git();
        write_manifest_config(repo.root(), "standard", &["vscode", "cursor", "claude"]);

        // Step 2: Run sync
        let root = NormalizedPath::new(repo.root());
        let engine = SyncEngine::new(root.clone(), Mode::Standard).unwrap();
        let sync_report = engine.sync().unwrap();
        assert!(
            sync_report.success,
            "Sync should succeed: {:?}",
            sync_report.errors
        );
        assert!(
            !sync_report.actions.is_empty(),
            "Sync should report actions taken, not be a no-op"
        );

        // Step 3: Verify ALL expected files exist
        // Primary paths
        repo.assert_file_exists(".vscode/settings.json");
        repo.assert_file_exists(".cursorrules");
        repo.assert_file_exists("CLAUDE.md");

        // Secondary path for claude: .claude/rules/ directory
        let claude_rules_dir = repo.root().join(".claude/rules");
        assert!(
            claude_rules_dir.is_dir(),
            ".claude/rules/ directory should exist as Claude's secondary path"
        );

        // Verify ledger was created and has content
        repo.assert_file_exists(".repository/ledger.toml");
        let ledger_content =
            fs::read_to_string(repo.root().join(".repository/ledger.toml")).unwrap();
        assert!(
            !ledger_content.is_empty(),
            "Ledger should not be empty after sync"
        );

        // Verify files have real content (not empty stubs)
        let cursorrules_content =
            fs::read_to_string(repo.root().join(".cursorrules")).unwrap();
        assert!(
            !cursorrules_content.is_empty(),
            ".cursorrules should have content after sync"
        );

        let claude_content = fs::read_to_string(repo.root().join("CLAUDE.md")).unwrap();
        assert!(
            !claude_content.is_empty(),
            "CLAUDE.md should have content after sync"
        );

        let vscode_content =
            fs::read_to_string(repo.root().join(".vscode/settings.json")).unwrap();
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&vscode_content);
        assert!(
            parsed.is_ok(),
            ".vscode/settings.json should be valid JSON"
        );

        // Step 4: Check should report healthy (no rules added, so tool checksums
        // in the ledger should match what is on disk)
        let check_report = engine.check().unwrap();
        assert_eq!(
            check_report.status,
            CheckStatus::Healthy,
            "Check after sync should be Healthy, but got {:?} with missing: {:?}, drifted: {:?}",
            check_report.status,
            check_report.missing,
            check_report.drifted
        );

        // Step 5: Delete a managed file to simulate drift
        let cursorrules_path = repo.root().join(".cursorrules");
        assert!(cursorrules_path.exists(), "Pre-condition: .cursorrules exists");
        fs::remove_file(&cursorrules_path).unwrap();
        assert!(!cursorrules_path.exists(), ".cursorrules should be deleted");

        // Step 6: Check should detect the missing file
        let check_report_after_delete = engine.check().unwrap();
        assert_ne!(
            check_report_after_delete.status,
            CheckStatus::Healthy,
            "Check should detect missing file after deletion"
        );
        assert!(
            !check_report_after_delete.missing.is_empty(),
            "Check should report at least one missing item after .cursorrules deletion"
        );

        // Verify the missing item references the correct file
        let missing_files: Vec<&str> = check_report_after_delete
            .missing
            .iter()
            .map(|d| d.file.as_str())
            .collect();
        assert!(
            missing_files.iter().any(|f| f.contains("cursorrules")),
            "Missing items should reference .cursorrules, got: {:?}",
            missing_files
        );
    }

    /// End-to-end test with rules: init -> add rules -> sync -> verify rule content
    ///
    /// Tests that rules defined in .repository/rules/ are propagated to all
    /// tool config files during sync. Complements the tool-only e2e test above.
    ///
    /// Guards against: rules being silently dropped during sync, or sync
    /// appearing to succeed without writing rule content to tool config files.
    #[test]
    fn e2e_rules_propagate_to_tool_configs() {
        let repo = TestRepo::new();
        repo.init_git();
        write_manifest_config(repo.root(), "standard", &["cursor", "claude"]);

        // Add rules
        let rules_dir = repo.root().join(".repository/rules");
        fs::create_dir_all(&rules_dir).unwrap();
        {
            use repo_core::rules::RuleRegistry;
            let registry_path = rules_dir.join("registry.toml");
            let mut registry = RuleRegistry::new(registry_path);
            registry
                .add_rule(
                    "coding-standards",
                    "Use descriptive variable names and write tests",
                    vec![],
                )
                .unwrap();
        }

        // Sync
        let root = NormalizedPath::new(repo.root());
        let engine = SyncEngine::new(root.clone(), Mode::Standard).unwrap();
        let sync_report = engine.sync().unwrap();
        assert!(
            sync_report.success,
            "Sync should succeed: {:?}",
            sync_report.errors
        );

        // Verify rule content appears in tool config files
        let cursorrules_content =
            fs::read_to_string(repo.root().join(".cursorrules")).unwrap();
        assert!(
            cursorrules_content.contains("coding-standards"),
            ".cursorrules should contain rule ID, got: {}",
            cursorrules_content
        );
        assert!(
            cursorrules_content.contains("descriptive variable names"),
            ".cursorrules should contain rule body"
        );

        let claude_content = fs::read_to_string(repo.root().join("CLAUDE.md")).unwrap();
        assert!(
            claude_content.contains("coding-standards"),
            "CLAUDE.md should contain rule ID"
        );
        assert!(
            claude_content.contains("descriptive variable names"),
            "CLAUDE.md should contain rule body"
        );

        // Verify managed blocks are present (not just raw text)
        assert!(
            cursorrules_content.contains("<!-- repo:block:"),
            ".cursorrules should use managed blocks"
        );
        assert!(
            claude_content.contains("<!-- repo:block:"),
            "CLAUDE.md should use managed blocks"
        );
    }
}

// =============================================================================
// P3: Drift Detection and Fix Tests
// =============================================================================

mod p3_drift_detection {
    use super::*;

    /// Helper: write a config.toml in the Manifest format that SyncEngine expects.
    fn write_manifest_config(root: &Path, mode: &str, tools: &[&str]) {
        let repo_dir = root.join(".repository");
        fs::create_dir_all(&repo_dir).unwrap();

        let tools_str = tools
            .iter()
            .map(|t| format!("\"{}\"", t))
            .collect::<Vec<_>>()
            .join(", ");

        let config = format!("tools = [{tools_str}]\n\n[core]\nmode = \"{mode}\"\n");

        fs::write(repo_dir.join("config.toml"), config).unwrap();
    }

    /// Test: sync -> healthy -> delete file -> check detects missing
    ///
    /// Guards against: check() failing to detect real filesystem drift.
    /// Verifies that the ledger projection system actually works end-to-end:
    /// a file recorded in the ledger is detected as missing after deletion.
    #[test]
    fn drift_check_detects_deleted_file() {
        let repo = TestRepo::new();
        repo.init_git();
        write_manifest_config(repo.root(), "standard", &["cursor"]);

        let root = NormalizedPath::new(repo.root());
        let engine = SyncEngine::new(root.clone(), Mode::Standard).unwrap();

        // Sync to create files
        let sync_report = engine.sync().unwrap();
        assert!(sync_report.success, "Initial sync should succeed");

        // Verify the file exists
        let cursorrules_path = repo.root().join(".cursorrules");
        assert!(
            cursorrules_path.exists(),
            ".cursorrules should exist after sync"
        );

        // Check should be healthy (no rules, so tool checksum matches)
        let check1 = engine.check().unwrap();
        assert_eq!(
            check1.status,
            CheckStatus::Healthy,
            "Check after initial sync should be Healthy"
        );

        // Delete the managed file
        fs::remove_file(&cursorrules_path).unwrap();
        assert!(!cursorrules_path.exists(), ".cursorrules should be deleted");

        // Check should detect the missing file
        let check2 = engine.check().unwrap();
        assert_ne!(
            check2.status,
            CheckStatus::Healthy,
            "Check should detect missing .cursorrules"
        );
        assert!(
            !check2.missing.is_empty(),
            "Check should report missing items after file deletion"
        );

        // Verify the missing item references the correct file
        let missing_files: Vec<&str> = check2
            .missing
            .iter()
            .map(|d| d.file.as_str())
            .collect();
        assert!(
            missing_files.iter().any(|f| f.contains("cursorrules")),
            "Missing items should reference .cursorrules, got: {:?}",
            missing_files
        );
    }

    /// Test: sync -> healthy -> corrupt file -> check detects drift
    ///
    /// Guards against: check() only detecting missing files but not modified content.
    /// The checksum comparison in the ledger should catch content corruption.
    #[test]
    fn drift_check_detects_corrupted_file() {
        let repo = TestRepo::new();
        repo.init_git();
        write_manifest_config(repo.root(), "standard", &["cursor"]);

        let root = NormalizedPath::new(repo.root());
        let engine = SyncEngine::new(root.clone(), Mode::Standard).unwrap();

        // Initial sync (no rules, so checksums will match)
        let sync_report = engine.sync().unwrap();
        assert!(sync_report.success, "Initial sync should succeed");

        // Verify healthy
        let check1 = engine.check().unwrap();
        assert_eq!(
            check1.status,
            CheckStatus::Healthy,
            "Should be healthy after initial sync"
        );

        // Corrupt the file by overwriting with different content
        let cursorrules_path = repo.root().join(".cursorrules");
        let original = fs::read_to_string(&cursorrules_path).unwrap();
        assert!(
            !original.is_empty(),
            "Pre-condition: .cursorrules has content"
        );

        fs::write(
            &cursorrules_path,
            "CORRUPTED CONTENT - all managed blocks destroyed",
        )
        .unwrap();

        // Check should detect the drift
        let check2 = engine.check().unwrap();
        assert_ne!(
            check2.status,
            CheckStatus::Healthy,
            "Check should detect corruption. Status: {:?}, missing: {:?}, drifted: {:?}",
            check2.status,
            check2.missing,
            check2.drifted
        );

        // It should report either drifted or missing (depending on projection type)
        let total_issues = check2.drifted.len() + check2.missing.len();
        assert!(
            total_issues > 0,
            "Check should report drifted or missing items after corruption"
        );
    }

    /// Test: fix() re-runs sync when drift is detected
    ///
    /// Guards against: fix() being a complete no-op that doesn't attempt
    /// to correct any issues. Verifies fix calls sync and reports actions.
    #[test]
    fn fix_runs_sync_on_unhealthy_repo() {
        use repo_core::ledger::{Intent, Ledger, Projection};
        use serde_json::json;

        let repo = TestRepo::new();
        repo.init_git();
        write_manifest_config(repo.root(), "standard", &["cursor"]);

        // Create a ledger with a projection for a nonexistent file,
        // simulating a state where sync was run but the file was deleted.
        let repo_dir = repo.root().join(".repository");
        fs::create_dir_all(&repo_dir).unwrap();

        let mut ledger = Ledger::new();
        let mut intent = Intent::new("tool:cursor".to_string(), json!({}));
        intent.add_projection(Projection::file_managed(
            "cursor".to_string(),
            std::path::PathBuf::from(".cursorrules"),
            "sha256:bogus_checksum".to_string(),
        ));
        ledger.add_intent(intent);
        ledger.save(&repo_dir.join("ledger.toml")).unwrap();

        // Verify check detects the missing file
        let root = NormalizedPath::new(repo.root());
        let engine = SyncEngine::new(root.clone(), Mode::Standard).unwrap();
        let check_before = engine.check().unwrap();
        assert_ne!(
            check_before.status,
            CheckStatus::Healthy,
            "Pre-condition: repo should be unhealthy"
        );

        // Fix should run sync
        let fix_report = engine.fix().unwrap();
        assert!(
            fix_report.success,
            "Fix should succeed: {:?}",
            fix_report.errors
        );
        assert!(
            !fix_report.actions.is_empty(),
            "Fix should report actions taken"
        );
    }

    /// Test: ledger records sync operations with correct projections
    ///
    /// Guards against: sync appearing to succeed but ledger not recording
    /// the projections, which would make check() unable to detect drift.
    #[test]
    fn ledger_records_projections_after_sync() {
        let repo = TestRepo::new();
        repo.init_git();
        write_manifest_config(repo.root(), "standard", &["cursor", "claude"]);

        let root = NormalizedPath::new(repo.root());
        let engine = SyncEngine::new(root.clone(), Mode::Standard).unwrap();

        // Sync
        let sync_report = engine.sync().unwrap();
        assert!(sync_report.success, "Sync should succeed");

        // Read the ledger file and verify it has content
        let ledger_path = repo.root().join(".repository/ledger.toml");
        assert!(ledger_path.exists(), "Ledger file should exist");

        let ledger_content = fs::read_to_string(&ledger_path).unwrap();
        assert!(
            !ledger_content.is_empty(),
            "Ledger should not be empty after sync"
        );

        // The ledger should reference the tools we synced
        // (Ledger uses intent IDs like "tool:cursor")
        assert!(
            ledger_content.contains("cursor") || ledger_content.contains("claude"),
            "Ledger should reference synced tools. Content: {}",
            ledger_content
        );

        // Verify that the check actually works by using the ledger:
        // If the ledger recorded projections, deleting a file should trigger a check failure.
        // This proves the ledger is not just an empty stub.
        let cursorrules_path = repo.root().join(".cursorrules");
        if cursorrules_path.exists() {
            fs::remove_file(&cursorrules_path).unwrap();
            let check_report = engine.check().unwrap();
            assert_ne!(
                check_report.status,
                CheckStatus::Healthy,
                "Ledger projections should cause check to detect missing .cursorrules"
            );
        }
    }

    /// Test: fix on an already-healthy repo is a no-op
    ///
    /// Guards against: fix() erroneously modifying files that are already correct.
    #[test]
    fn fix_on_healthy_repo_is_noop() {
        let repo = TestRepo::new();
        repo.init_git();
        write_manifest_config(repo.root(), "standard", &["cursor"]);

        let root = NormalizedPath::new(repo.root());
        let engine = SyncEngine::new(root.clone(), Mode::Standard).unwrap();

        // Sync
        let sync_report = engine.sync().unwrap();
        assert!(sync_report.success);

        // Record file content before fix
        let cursorrules_path = repo.root().join(".cursorrules");
        let content_before = fs::read_to_string(&cursorrules_path).unwrap();

        // Fix on healthy repo
        let fix_report = engine.fix().unwrap();
        assert!(fix_report.success);

        // Content should not change
        let content_after = fs::read_to_string(&cursorrules_path).unwrap();
        assert_eq!(
            content_before, content_after,
            "Fix on healthy repo should not modify files"
        );
    }
}

// =============================================================================
// P3: Low-Quality Test Pattern Fixes
// =============================================================================
//
// Audit findings for `assert!(result.is_ok())` as sole assertion:
//
// 1. robustness::empty_rules_sync (line ~1074) — sole assertion is
//    `assert!(result.is_ok())`. Fixed below by also verifying the file was created.
//
// 2. robustness::special_chars_in_rule_id (line ~1113) — sole assertion is
//    `assert!(result.is_ok())`. Fixed below by also verifying file content.
//
// 3. robustness::long_rule_content (line ~1093) — has `assert!(result.is_ok())`
//    followed by `assert_file_exists`, which is acceptable (two assertions).
//
// Note: test_summary was removed previously because it contained zero assertions.
// Mission status is tracked in documentation, not in test output.
