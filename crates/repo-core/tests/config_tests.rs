//! Tests for configuration resolution

use repo_core::config::{ConfigResolver, Manifest, ResolvedConfig, RuntimeContext};
use repo_fs::NormalizedPath;
use serde_json::json;
use std::collections::HashMap;
use tempfile::TempDir;

mod manifest_tests {
    use super::*;

    #[test]
    fn test_manifest_parse_basic() {
        // Note: top-level keys like tools/rules must come before [section] headers
        let toml_content = r#"
tools = ["cargo", "rustfmt"]
rules = ["no-unsafe", "no-unwrap"]

[core]
mode = "worktree"
"#;

        let manifest = Manifest::parse(toml_content).expect("Should parse valid TOML");

        assert_eq!(manifest.core.mode, "worktree");
        assert_eq!(manifest.tools, vec!["cargo", "rustfmt"]);
        assert_eq!(manifest.rules, vec!["no-unsafe", "no-unwrap"]);
    }

    #[test]
    fn test_manifest_parse_with_presets() {
        let toml_content = r#"
tools = ["python", "ruff"]

[core]
mode = "standard"

[presets."env:python"]
provider = "uv"
version = "3.12"

[presets."tool:linter"]
enabled = true
config = "strict"
"#;

        let manifest = Manifest::parse(toml_content).expect("Should parse valid TOML");

        assert_eq!(manifest.core.mode, "standard");
        assert!(manifest.presets.contains_key("env:python"));
        assert!(manifest.presets.contains_key("tool:linter"));

        let python_preset = &manifest.presets["env:python"];
        assert_eq!(python_preset["provider"], "uv");
        assert_eq!(python_preset["version"], "3.12");

        let linter_preset = &manifest.presets["tool:linter"];
        assert_eq!(linter_preset["enabled"], true);
        assert_eq!(linter_preset["config"], "strict");
    }

    #[test]
    fn test_manifest_parse_defaults() {
        let toml_content = "";
        let manifest = Manifest::parse(toml_content).expect("Should parse empty TOML");

        assert_eq!(manifest.core.mode, "worktrees"); // default per spec
        assert!(manifest.presets.is_empty());
        assert!(manifest.tools.is_empty());
        assert!(manifest.rules.is_empty());
    }

    #[test]
    fn test_manifest_empty() {
        let manifest = Manifest::empty();

        assert_eq!(manifest.core.mode, "worktrees");
        assert!(manifest.presets.is_empty());
        assert!(manifest.tools.is_empty());
        assert!(manifest.rules.is_empty());
    }

    #[test]
    fn test_manifest_merge_basic() {
        let mut base = Manifest::parse(
            r#"
tools = ["cargo"]
rules = ["base-rule"]

[core]
mode = "standard"
"#,
        )
        .unwrap();

        let overlay = Manifest::parse(
            r#"
tools = ["rustfmt", "clippy"]
rules = ["overlay-rule"]

[core]
mode = "worktree"
"#,
        )
        .unwrap();

        base.merge(&overlay);

        // Overlay takes precedence for scalar values
        assert_eq!(base.core.mode, "worktree");

        // Tools and rules are extended (overlay appended, no duplicates)
        assert!(base.tools.contains(&"cargo".to_string()));
        assert!(base.tools.contains(&"rustfmt".to_string()));
        assert!(base.tools.contains(&"clippy".to_string()));

        assert!(base.rules.contains(&"base-rule".to_string()));
        assert!(base.rules.contains(&"overlay-rule".to_string()));
    }

    #[test]
    fn test_manifest_merge_presets_deep() {
        let mut base = Manifest::parse(
            r#"
[presets."env:python"]
provider = "pyenv"
version = "3.11"
extra = "base-only"
"#,
        )
        .unwrap();

        let overlay = Manifest::parse(
            r#"
[presets."env:python"]
provider = "uv"
version = "3.12"

[presets."tool:linter"]
enabled = true
"#,
        )
        .unwrap();

        base.merge(&overlay);

        // Deep merge: overlay values override, but base-only values preserved
        let python = &base.presets["env:python"];
        assert_eq!(python["provider"], "uv"); // overridden
        assert_eq!(python["version"], "3.12"); // overridden
        assert_eq!(python["extra"], "base-only"); // preserved from base

        // New preset from overlay is added
        assert!(base.presets.contains_key("tool:linter"));
        assert_eq!(base.presets["tool:linter"]["enabled"], true);
    }

    #[test]
    fn test_manifest_parse_invalid_toml() {
        let result = Manifest::parse("not valid toml {{{");
        assert!(result.is_err(), "Should reject malformed TOML");
    }

    #[test]
    fn test_manifest_parse_wrong_type_for_mode() {
        let result = Manifest::parse("[core]\nmode = 123");
        assert!(result.is_err(), "Should reject wrong type for mode field");
    }

    #[test]
    fn test_manifest_parse_wrong_type_for_tools() {
        let result = Manifest::parse("tools = \"not-an-array\"");
        assert!(result.is_err(), "Should reject wrong type for tools field");
    }

    #[test]
    fn test_manifest_merge_deduplicates_tools() {
        let mut base = Manifest::parse(
            r#"
tools = ["cargo", "rustfmt"]
rules = ["shared-rule"]
"#,
        )
        .unwrap();

        let overlay = Manifest::parse(
            r#"
tools = ["cargo", "clippy"]
rules = ["shared-rule", "new-rule"]
"#,
        )
        .unwrap();

        base.merge(&overlay);

        // "cargo" appears in both - should only appear once after merge
        let cargo_count = base.tools.iter().filter(|t| t.as_str() == "cargo").count();
        assert_eq!(
            cargo_count, 1,
            "Duplicate tool 'cargo' should be deduplicated"
        );
        assert_eq!(base.tools.len(), 3, "Should have cargo, rustfmt, clippy");

        // "shared-rule" appears in both - should only appear once
        let rule_count = base
            .rules
            .iter()
            .filter(|r| r.as_str() == "shared-rule")
            .count();
        assert_eq!(
            rule_count, 1,
            "Duplicate rule 'shared-rule' should be deduplicated"
        );
        assert_eq!(base.rules.len(), 2, "Should have shared-rule, new-rule");
    }
}

mod resolver_tests {
    use super::*;
    use std::fs;

    fn setup_test_repo() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create .repository directory
        let repo_config_dir = temp_dir.path().join(".repository");
        fs::create_dir_all(&repo_config_dir).expect("Failed to create .repository dir");

        temp_dir
    }

    #[test]
    fn test_config_resolver_no_config() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root = NormalizedPath::new(temp_dir.path());

        let resolver = ConfigResolver::new(root);
        let config = resolver.resolve().expect("Should resolve with defaults");

        assert_eq!(config.mode, "worktrees");
        assert!(config.presets.is_empty());
        assert!(config.tools.is_empty());
        assert!(config.rules.is_empty());
    }

    #[test]
    fn test_config_resolver_repo_config() {
        let temp_dir = setup_test_repo();

        // Write repo config - note tools must be before [core] section
        let config_path = temp_dir.path().join(".repository/config.toml");
        fs::write(
            &config_path,
            r#"
tools = ["cargo", "python"]

[core]
mode = "worktree"

[presets."env:python"]
version = "3.12"
"#,
        )
        .expect("Failed to write config");

        let root = NormalizedPath::new(temp_dir.path());
        let resolver = ConfigResolver::new(root);
        let config = resolver.resolve().expect("Should resolve config");

        assert_eq!(config.mode, "worktree");
        assert!(config.presets.contains_key("env:python"));
        assert_eq!(config.tools, vec!["cargo", "python"]);
    }

    #[test]
    fn test_config_resolver_hierarchy_local_overrides_repo() {
        let temp_dir = setup_test_repo();

        // Write repo config
        let config_path = temp_dir.path().join(".repository/config.toml");
        fs::write(
            &config_path,
            r#"
tools = ["cargo"]
rules = ["base-rule"]

[core]
mode = "standard"

[presets."env:python"]
version = "3.11"
provider = "pyenv"
"#,
        )
        .expect("Failed to write config");

        // Write local overrides (git-ignored)
        let local_config_path = temp_dir.path().join(".repository/config.local.toml");
        fs::write(
            &local_config_path,
            r#"
tools = ["rustfmt"]
rules = ["local-rule"]

[core]
mode = "worktree"

[presets."env:python"]
version = "3.12"
"#,
        )
        .expect("Failed to write local config");

        let root = NormalizedPath::new(temp_dir.path());
        let resolver = ConfigResolver::new(root);
        let config = resolver.resolve().expect("Should resolve config");

        // Local overrides repo for scalar values
        assert_eq!(config.mode, "worktree");

        // Presets are deep merged - local version wins, provider preserved
        let python = &config.presets["env:python"];
        assert_eq!(python["version"], "3.12");
        assert_eq!(python["provider"], "pyenv"); // preserved from base

        // Tools and rules are merged (unique values)
        assert!(config.tools.contains(&"cargo".to_string()));
        assert!(config.tools.contains(&"rustfmt".to_string()));
        assert!(config.rules.contains(&"base-rule".to_string()));
        assert!(config.rules.contains(&"local-rule".to_string()));
    }
}

mod runtime_context_tests {
    use super::*;

    fn create_resolved_config() -> ResolvedConfig {
        let mut presets = HashMap::new();
        presets.insert(
            "env:python".to_string(),
            json!({
                "provider": "uv",
                "version": "3.12"
            }),
        );
        presets.insert(
            "env:node".to_string(),
            json!({
                "version": "20",
                "manager": "fnm"
            }),
        );
        presets.insert(
            "tool:linter".to_string(),
            json!({
                "enabled": true,
                "strict": true
            }),
        );
        presets.insert(
            "config:editor".to_string(),
            json!({
                "theme": "dark"
            }),
        );

        ResolvedConfig {
            mode: "standard".to_string(),
            presets,
            tools: vec!["cargo".to_string(), "python".to_string()],
            rules: vec!["no-unsafe".to_string()],
        }
    }

    #[test]
    fn test_runtime_context_from_resolved() {
        let config = create_resolved_config();
        let context = RuntimeContext::from_resolved(&config);

        // env: presets become runtime info
        assert!(context.runtime.contains_key("python"));
        let python = &context.runtime["python"];
        assert_eq!(python["provider"], "uv");
        assert_eq!(python["version"], "3.12");

        assert!(context.runtime.contains_key("node"));
        let node = &context.runtime["node"];
        assert_eq!(node["version"], "20");
        assert_eq!(node["manager"], "fnm");

        // tool: and config: presets become capabilities
        assert!(context.capabilities.contains(&"tool:linter".to_string()));
        assert!(context.capabilities.contains(&"config:editor".to_string()));
    }

    #[test]
    fn test_runtime_context_to_json() {
        let config = create_resolved_config();
        let context = RuntimeContext::from_resolved(&config);
        let json_output = context.to_json();

        // Verify structure
        assert!(json_output["runtime"].is_object());
        assert!(json_output["capabilities"].is_array());

        // Verify runtime content
        assert_eq!(json_output["runtime"]["python"]["version"], "3.12");
        assert_eq!(json_output["runtime"]["node"]["manager"], "fnm");

        // Verify capabilities array
        let caps = json_output["capabilities"].as_array().unwrap();
        assert!(caps.iter().any(|v| v == "tool:linter"));
        assert!(caps.iter().any(|v| v == "config:editor"));
    }

    #[test]
    fn test_runtime_context_empty() {
        let config = ResolvedConfig {
            mode: "standard".to_string(),
            presets: HashMap::new(),
            tools: vec![],
            rules: vec![],
        };
        let context = RuntimeContext::from_resolved(&config);

        assert!(context.runtime.is_empty());
        assert!(context.capabilities.is_empty());

        let json_output = context.to_json();
        assert!(json_output["runtime"].as_object().unwrap().is_empty());
        assert!(json_output["capabilities"].as_array().unwrap().is_empty());
    }
}
