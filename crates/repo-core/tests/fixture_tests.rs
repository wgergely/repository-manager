//! Golden-file tests using test-fixtures/
//!
//! These tests wire the test-fixtures directory into actual integration tests,
//! verifying that syncing rules to tools produces the expected output files.

use pretty_assertions::assert_eq;
use repo_core::ledger::Ledger;
use repo_core::sync::ToolSyncer;
use repo_fs::NormalizedPath;
use repo_tools::Rule;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

/// Normalize line endings to LF for cross-platform comparison.
fn normalize_line_endings(s: &str) -> String {
    s.replace("\r\n", "\n")
}

/// Path to the test-fixtures directory (relative to the workspace root).
fn fixtures_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // crates/repo-core -> ../../test-fixtures
    manifest_dir.join("../../test-fixtures")
}

/// Read the coding-standards rule from the config-test fixture.
fn load_coding_standards_rule() -> Rule {
    let rule_path = fixtures_dir()
        .join("repos/config-test/.repository/rules/coding-standards.md");
    let content = fs::read_to_string(&rule_path)
        .unwrap_or_else(|e| panic!("Failed to read fixture rule at {}: {}", rule_path.display(), e));
    Rule {
        id: "coding-standards".to_string(),
        content,
    }
}

/// Read the expected output for a tool from test-fixtures/expected/.
fn load_expected_output(tool_dir: &str, filename: &str) -> String {
    let path = fixtures_dir().join("expected").join(tool_dir).join(filename);
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read expected output at {}: {}", path.display(), e))
}

// ==========================================================================
// Fixture Validity Tests
// ==========================================================================

#[test]
fn test_config_test_fixture_has_valid_manifest() {
    let config_path = fixtures_dir().join("repos/config-test/.repository/config.toml");
    let content = fs::read_to_string(&config_path).unwrap();

    // Should parse as a valid Manifest
    let manifest = repo_core::Manifest::parse(&content).unwrap();
    assert_eq!(manifest.core.mode, "standard");
    assert!(
        !manifest.tools.is_empty(),
        "config-test fixture should declare at least one tool"
    );
    assert!(
        manifest.tools.contains(&"cursor".to_string()),
        "config-test fixture should include cursor tool, got: {:?}",
        manifest.tools
    );
    assert!(
        manifest.tools.contains(&"claude".to_string()),
        "config-test fixture should include claude tool, got: {:?}",
        manifest.tools
    );
}

#[test]
fn test_coding_standards_rule_is_non_empty() {
    let rule = load_coding_standards_rule();
    assert!(
        !rule.content.is_empty(),
        "Coding standards rule should have content"
    );
    assert!(
        rule.content.contains("rustfmt"),
        "Coding standards should mention rustfmt"
    );
    assert!(
        rule.content.contains("clippy"),
        "Coding standards should mention clippy"
    );
}

// ==========================================================================
// Golden-File Tests: Sync Rule to Tools and Compare Output
// ==========================================================================

#[test]
fn test_golden_file_cursor_output_matches_expected() {
    let temp = tempdir().unwrap();
    let root = NormalizedPath::new(temp.path());

    let syncer = ToolSyncer::new(root.clone(), false);
    let mut ledger = Ledger::new();
    let rule = load_coding_standards_rule();

    syncer
        .sync_tool_with_rules("cursor", &[rule], &mut ledger)
        .unwrap();

    // Read the generated .cursorrules file
    let generated_path = temp.path().join(".cursorrules");
    assert!(
        generated_path.exists(),
        ".cursorrules should be created after sync"
    );
    let generated = fs::read_to_string(&generated_path).unwrap();

    // Load the expected output
    let expected = load_expected_output("cursor", ".cursorrules");

    // Compare: both should contain the same managed block with coding-standards content
    assert_eq!(
        normalize_line_endings(&generated).trim(),
        normalize_line_endings(&expected).trim(),
        "Generated .cursorrules should match the expected golden file"
    );
}

#[test]
fn test_golden_file_claude_output_matches_expected() {
    let temp = tempdir().unwrap();
    let root = NormalizedPath::new(temp.path());

    let syncer = ToolSyncer::new(root.clone(), false);
    let mut ledger = Ledger::new();
    let rule = load_coding_standards_rule();

    syncer
        .sync_tool_with_rules("claude", &[rule], &mut ledger)
        .unwrap();

    // Read the generated CLAUDE.md file
    let generated_path = temp.path().join("CLAUDE.md");
    assert!(
        generated_path.exists(),
        "CLAUDE.md should be created after sync"
    );
    let generated = fs::read_to_string(&generated_path).unwrap();

    // Load the expected output
    let expected = load_expected_output("claude", "CLAUDE.md");

    assert_eq!(
        normalize_line_endings(&generated).trim(),
        normalize_line_endings(&expected).trim(),
        "Generated CLAUDE.md should match the expected golden file"
    );
}

// ==========================================================================
// Fixture-Based Multi-Tool Sync Test
// ==========================================================================

#[test]
fn test_config_test_fixture_syncs_all_declared_tools() {
    let config_path = fixtures_dir().join("repos/config-test/.repository/config.toml");
    let content = fs::read_to_string(&config_path).unwrap();
    let manifest = repo_core::Manifest::parse(&content).unwrap();

    let temp = tempdir().unwrap();
    let root = NormalizedPath::new(temp.path());
    let syncer = ToolSyncer::new(root.clone(), false);
    let mut ledger = Ledger::new();

    let rule = load_coding_standards_rule();

    // Sync each tool declared in the fixture config
    for tool_name in &manifest.tools {
        if syncer.has_tool(tool_name) {
            let actions = syncer
                .sync_tool_with_rules(tool_name, &[rule.clone()], &mut ledger)
                .unwrap();
            assert!(
                !actions.is_empty(),
                "Syncing tool '{}' should produce at least one action",
                tool_name
            );
        }
    }

    // Verify that synced tools created files
    // cursor -> .cursorrules
    let cursorrules = temp.path().join(".cursorrules");
    assert!(cursorrules.exists(), ".cursorrules should exist after cursor sync");
    let cursor_content = fs::read_to_string(&cursorrules).unwrap();
    assert!(
        cursor_content.contains("coding-standards"),
        ".cursorrules should contain coding-standards block marker"
    );

    // claude -> CLAUDE.md
    let claude_md = temp.path().join("CLAUDE.md");
    assert!(claude_md.exists(), "CLAUDE.md should exist after claude sync");
    let claude_content = fs::read_to_string(&claude_md).unwrap();
    assert!(
        claude_content.contains("coding-standards"),
        "CLAUDE.md should contain coding-standards block marker"
    );
}

// ==========================================================================
// Simple-Project Fixture Tests
// ==========================================================================

#[test]
fn test_simple_project_fixture_files_exist() {
    // Verify the simple-project fixture has the expected structure
    let base = fixtures_dir().join("repos/simple-project");

    let expected_files = [
        "Cargo.toml",
        "CLAUDE.md",
        "GEMINI.md",
        ".aider.conf.yml",
        "src/main.rs",
    ];

    for file in &expected_files {
        let path = base.join(file);
        assert!(
            path.exists(),
            "Simple-project fixture should contain {}, but it's missing at {}",
            file,
            path.display()
        );
    }
}

#[test]
fn test_simple_project_claude_md_has_expected_structure() {
    let path = fixtures_dir().join("repos/simple-project/CLAUDE.md");
    let content = fs::read_to_string(&path).unwrap();

    // The simple-project CLAUDE.md is a manually written file (not managed blocks)
    // It should be a markdown document with a heading
    assert!(
        content.starts_with("# "),
        "CLAUDE.md should start with a heading, got: {}",
        content.chars().take(20).collect::<String>()
    );
    assert!(
        content.contains("Rules") || content.contains("rules"),
        "CLAUDE.md should mention rules"
    );
}

#[test]
fn test_simple_project_aider_config_is_valid_yaml() {
    let path = fixtures_dir().join("repos/simple-project/.aider.conf.yml");
    let content = fs::read_to_string(&path).unwrap();

    // Basic YAML validity checks (not a full parser, but structural checks)
    assert!(
        content.contains("model:"),
        "Aider config should contain a model key"
    );
    assert!(
        content.contains("auto-commits:"),
        "Aider config should contain auto-commits key"
    );
}

// ==========================================================================
// Expected Output Structure Tests
// ==========================================================================

#[test]
fn test_expected_outputs_contain_managed_block_markers() {
    // The expected/ directory files should all contain repo:block markers
    let expected_dir = fixtures_dir().join("expected");

    let checks = vec![
        ("claude/CLAUDE.md", "repo:block:coding-standards"),
        ("cursor/.cursorrules", "repo:block:coding-standards"),
        ("aider/.aider.conf.yml", "repo:block:coding-standards"),
    ];

    for (rel_path, marker) in checks {
        let path = expected_dir.join(rel_path);
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
        assert!(
            content.contains(marker),
            "Expected output {} should contain marker '{}', got:\n{}",
            rel_path,
            marker,
            content
        );
    }
}

#[test]
fn test_expected_outputs_have_matching_open_close_markers() {
    let expected_dir = fixtures_dir().join("expected");

    let files = vec![
        "claude/CLAUDE.md",
        "cursor/.cursorrules",
    ];

    for rel_path in files {
        let path = expected_dir.join(rel_path);
        let content = fs::read_to_string(&path).unwrap();

        let open_count = content.matches("<!-- repo:block:").count();
        let close_count = content.matches("<!-- /repo:block:").count();

        assert_eq!(
            open_count, close_count,
            "{}: open markers ({}) should equal close markers ({})",
            rel_path, open_count, close_count
        );
        assert!(
            open_count > 0,
            "{}: should have at least one block marker pair",
            rel_path
        );
    }
}
