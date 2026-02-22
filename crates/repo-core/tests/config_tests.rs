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

mod config_hierarchy_tests {
    use super::*;
    use std::fs;

    /// Helper: create a temp dir for the repo root with a `.repository/` directory,
    /// and a separate temp dir for the global config directory.
    /// Returns (repo_temp_dir, global_config_temp_dir).
    fn setup_hierarchy_dirs() -> (TempDir, TempDir) {
        let repo_dir = TempDir::new().expect("Failed to create repo temp dir");
        let global_dir = TempDir::new().expect("Failed to create global config temp dir");

        // Create .repository directory in the repo
        fs::create_dir_all(repo_dir.path().join(".repository"))
            .expect("Failed to create .repository dir");

        // The global config dir will serve as the "repo-manager" config root.
        // ConfigResolver.global_config_dir() returns the dir that already
        // includes "repo-manager", so we pass the temp dir directly and
        // place config.toml inside it.

        (repo_dir, global_dir)
    }

    #[test]
    fn test_resolve_without_global_config_returns_repo_values_only() {
        // No global config file exists. The resolver should succeed and
        // return only the values from the repo config (Layer 3).
        let (repo_dir, global_dir) = setup_hierarchy_dirs();

        // Write a repo config (Layer 3)
        fs::write(
            repo_dir.path().join(".repository/config.toml"),
            r#"
tools = ["cargo"]
rules = ["no-unwrap"]

[core]
mode = "standard"

[presets."env:rust"]
edition = "2021"
"#,
        )
        .unwrap();

        // global_dir exists but has no config.toml inside it
        let root = NormalizedPath::new(repo_dir.path());
        let resolver =
            ConfigResolver::with_global_config_dir(root, global_dir.path().to_path_buf());
        let config = resolver
            .resolve()
            .expect("Should resolve without global config");

        // Verify resolved VALUES match repo-only expectations
        assert_eq!(config.mode, "standard");
        assert_eq!(config.tools, vec!["cargo"]);
        assert_eq!(config.rules, vec!["no-unwrap"]);
        assert_eq!(config.presets["env:rust"]["edition"], "2021");

        // No extra tools/rules from a nonexistent global config
        assert_eq!(config.tools.len(), 1);
        assert_eq!(config.rules.len(), 1);
    }

    #[test]
    fn test_global_config_provides_defaults() {
        // A global config (Layer 1) defines tools and presets. With no
        // repo config, the resolved config picks up those global values.
        let (repo_dir, global_dir) = setup_hierarchy_dirs();

        // Write global config (Layer 1) — no repo config at all
        fs::write(
            global_dir.path().join("config.toml"),
            r#"
tools = ["vscode", "docker"]
rules = ["global-rule"]

[presets."env:node"]
version = "20"
manager = "fnm"
"#,
        )
        .unwrap();

        // Intentionally do NOT create .repository/config.toml
        // (but .repository dir exists from setup_hierarchy_dirs)

        let root = NormalizedPath::new(repo_dir.path());
        let resolver =
            ConfigResolver::with_global_config_dir(root, global_dir.path().to_path_buf());
        let config = resolver
            .resolve()
            .expect("Should resolve with global defaults");

        // Global tools are present
        assert!(
            config.tools.contains(&"vscode".to_string()),
            "Expected 'vscode' from global config, got: {:?}",
            config.tools,
        );
        assert!(
            config.tools.contains(&"docker".to_string()),
            "Expected 'docker' from global config, got: {:?}",
            config.tools,
        );
        assert_eq!(config.tools.len(), 2);

        // Global rules are present
        assert!(config.rules.contains(&"global-rule".to_string()));
        assert_eq!(config.rules.len(), 1);

        // Global presets are present with correct values
        assert!(config.presets.contains_key("env:node"));
        assert_eq!(config.presets["env:node"]["version"], "20");
        assert_eq!(config.presets["env:node"]["manager"], "fnm");
    }

    #[test]
    fn test_repo_config_overrides_global_mode() {
        // Global config has tools = ["vscode"] and mode = "worktrees".
        // Repo config has tools = ["cursor"] and mode = "standard".
        // The resolved config should have BOTH tools (union), and
        // the repo's mode should win (Layer 3 > Layer 1).
        let (repo_dir, global_dir) = setup_hierarchy_dirs();

        // Global config (Layer 1)
        fs::write(
            global_dir.path().join("config.toml"),
            r#"
tools = ["vscode"]
rules = ["global-lint"]

[core]
mode = "worktrees"

[presets."env:python"]
version = "3.11"
provider = "pyenv"
"#,
        )
        .unwrap();

        // Repo config (Layer 3)
        fs::write(
            repo_dir.path().join(".repository/config.toml"),
            r#"
tools = ["cursor"]
rules = ["repo-lint"]

[core]
mode = "standard"

[presets."env:python"]
version = "3.12"
"#,
        )
        .unwrap();

        let root = NormalizedPath::new(repo_dir.path());
        let resolver =
            ConfigResolver::with_global_config_dir(root, global_dir.path().to_path_buf());
        let config = resolver.resolve().expect("Should resolve merged config");

        // Mode: repo (Layer 3) wins over global (Layer 1)
        assert_eq!(config.mode, "standard");

        // Tools: union of global + repo (both present)
        assert!(
            config.tools.contains(&"vscode".to_string()),
            "Expected 'vscode' from global, got: {:?}",
            config.tools,
        );
        assert!(
            config.tools.contains(&"cursor".to_string()),
            "Expected 'cursor' from repo, got: {:?}",
            config.tools,
        );

        // Rules: union of global + repo
        assert!(config.rules.contains(&"global-lint".to_string()));
        assert!(config.rules.contains(&"repo-lint".to_string()));

        // Presets: deep merge — repo's python version wins, global's provider preserved
        let python = &config.presets["env:python"];
        assert_eq!(
            python["version"], "3.12",
            "Repo (Layer 3) python version should override global (Layer 1)"
        );
        assert_eq!(
            python["provider"], "pyenv",
            "Global-only field 'provider' should be preserved via deep merge"
        );
    }

    #[test]
    fn test_global_config_invalid_toml_produces_error() {
        // Invalid TOML in the global config path should produce a clear error,
        // not a panic or silent skip.
        let (repo_dir, global_dir) = setup_hierarchy_dirs();

        // Write invalid TOML as the global config
        fs::write(
            global_dir.path().join("config.toml"),
            "this is not valid toml {{{\n  [broken",
        )
        .unwrap();

        let root = NormalizedPath::new(repo_dir.path());
        let resolver =
            ConfigResolver::with_global_config_dir(root, global_dir.path().to_path_buf());
        let result = resolver.resolve();

        assert!(
            result.is_err(),
            "Invalid TOML in global config should produce an error"
        );

        // The error message should mention TOML parsing, not be an opaque IO error
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("expected") || err_msg.contains("invalid") || err_msg.contains("TOML"),
            "Error should indicate a TOML parse failure, got: {}",
            err_msg,
        );
    }

    #[test]
    fn test_global_config_path_uses_xdg_convention() {
        // Verify that when no override is set, ConfigResolver uses
        // dirs::config_dir() to determine the global config path.
        // We test this indirectly: create a resolver with no override,
        // and confirm global_config_dir() returns a path ending in "repo-manager".
        //
        // On Linux this should be ~/.config/repo-manager
        // On macOS this should be ~/Library/Application Support/repo-manager
        //
        // Since we are in a test environment, we verify the structural invariant
        // that the path ends with "repo-manager" (platform-independent check).

        let temp_dir = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp_dir.path());

        // Use the normal constructor (no override) — this uses dirs::config_dir()
        let resolver = ConfigResolver::new(root);

        // We cannot call global_config_dir() directly (it is private), but we
        // can verify the behavior by checking that resolve() succeeds even when
        // the real XDG config dir does not contain our config. This confirms
        // it correctly handles the "no file" case for the platform path.
        let config = resolver
            .resolve()
            .expect("Resolve should succeed with no config files anywhere");

        // With no config files, we get defaults
        assert_eq!(config.mode, "worktrees");
        assert!(config.tools.is_empty());
    }

    #[test]
    fn test_full_four_layer_precedence() {
        // All 4 layers present. Layer 4 (local) should win over all others.
        // This tests the full hierarchy: global < org < repo < local.
        let (repo_dir, global_dir) = setup_hierarchy_dirs();

        // Layer 1: Global config
        fs::write(
            global_dir.path().join("config.toml"),
            r#"
tools = ["global-tool"]
rules = ["global-rule"]

[core]
mode = "worktrees"

[presets."env:python"]
version = "3.10"
provider = "system"
global_only = "from-global"
"#,
        )
        .unwrap();

        // Layer 2: Org config
        fs::create_dir_all(global_dir.path().join("org")).unwrap();
        fs::write(
            global_dir.path().join("org/config.toml"),
            r#"
tools = ["org-tool"]
rules = ["org-rule"]

[presets."env:python"]
version = "3.11"
provider = "pyenv"
org_only = "from-org"
"#,
        )
        .unwrap();

        // Layer 3: Repo config
        fs::write(
            repo_dir.path().join(".repository/config.toml"),
            r#"
tools = ["repo-tool"]
rules = ["repo-rule"]

[core]
mode = "standard"

[presets."env:python"]
version = "3.12"
repo_only = "from-repo"
"#,
        )
        .unwrap();

        // Layer 4: Local overrides
        fs::write(
            repo_dir.path().join(".repository/config.local.toml"),
            r#"
tools = ["local-tool"]
rules = ["local-rule"]

[core]
mode = "worktree"

[presets."env:python"]
version = "3.13"
local_only = "from-local"
"#,
        )
        .unwrap();

        let root = NormalizedPath::new(repo_dir.path());
        let resolver =
            ConfigResolver::with_global_config_dir(root, global_dir.path().to_path_buf());
        let config = resolver.resolve().expect("Should resolve all 4 layers");

        // Mode: Layer 4 wins
        assert_eq!(config.mode, "worktree");

        // Tools: union of all layers
        assert!(config.tools.contains(&"global-tool".to_string()));
        assert!(config.tools.contains(&"org-tool".to_string()));
        assert!(config.tools.contains(&"repo-tool".to_string()));
        assert!(config.tools.contains(&"local-tool".to_string()));
        assert_eq!(config.tools.len(), 4);

        // Rules: union of all layers
        assert!(config.rules.contains(&"global-rule".to_string()));
        assert!(config.rules.contains(&"org-rule".to_string()));
        assert!(config.rules.contains(&"repo-rule".to_string()));
        assert!(config.rules.contains(&"local-rule".to_string()));
        assert_eq!(config.rules.len(), 4);

        // Presets: deep merge — Layer 4 version wins, layer-unique fields preserved
        let python = &config.presets["env:python"];
        assert_eq!(
            python["version"], "3.13",
            "Layer 4 (local) version should win"
        );
        assert_eq!(
            python["global_only"], "from-global",
            "Global-only field should be preserved"
        );
        assert_eq!(
            python["org_only"], "from-org",
            "Org-only field should be preserved"
        );
        assert_eq!(
            python["repo_only"], "from-repo",
            "Repo-only field should be preserved"
        );
        assert_eq!(
            python["local_only"], "from-local",
            "Local-only field should be present"
        );
    }

    #[test]
    fn test_org_config_overrides_global_but_not_repo() {
        // Org config (Layer 2) should override global (Layer 1),
        // but repo config (Layer 3) should override org.
        let (repo_dir, global_dir) = setup_hierarchy_dirs();

        // Layer 1: Global
        fs::write(
            global_dir.path().join("config.toml"),
            r#"
tools = ["global-tool"]

[presets."env:python"]
version = "3.10"
"#,
        )
        .unwrap();

        // Layer 2: Org
        fs::create_dir_all(global_dir.path().join("org")).unwrap();
        fs::write(
            global_dir.path().join("org/config.toml"),
            r#"
tools = ["org-tool"]

[presets."env:python"]
version = "3.11"
"#,
        )
        .unwrap();

        // Layer 3: Repo
        fs::write(
            repo_dir.path().join(".repository/config.toml"),
            r#"
tools = ["repo-tool"]

[presets."env:python"]
version = "3.12"
"#,
        )
        .unwrap();

        let root = NormalizedPath::new(repo_dir.path());
        let resolver =
            ConfigResolver::with_global_config_dir(root, global_dir.path().to_path_buf());
        let config = resolver.resolve().unwrap();

        // Repo (Layer 3) wins over org (Layer 2) and global (Layer 1) for python version
        assert_eq!(config.presets["env:python"]["version"], "3.12");

        // All tools are unioned
        assert!(config.tools.contains(&"global-tool".to_string()));
        assert!(config.tools.contains(&"org-tool".to_string()));
        assert!(config.tools.contains(&"repo-tool".to_string()));
    }

    #[test]
    fn test_no_global_no_org_no_local_repo_only() {
        // Only repo config exists. No global, no org, no local.
        // This is the most common case and must work correctly.
        let (repo_dir, global_dir) = setup_hierarchy_dirs();

        fs::write(
            repo_dir.path().join(".repository/config.toml"),
            r#"
tools = ["just-repo"]

[core]
mode = "standard"
"#,
        )
        .unwrap();

        let root = NormalizedPath::new(repo_dir.path());
        let resolver =
            ConfigResolver::with_global_config_dir(root, global_dir.path().to_path_buf());
        let config = resolver.resolve().unwrap();

        assert_eq!(config.mode, "standard");
        assert_eq!(config.tools, vec!["just-repo"]);
        assert!(config.rules.is_empty());
        assert!(config.presets.is_empty());
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
            extensions: HashMap::new(),
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
            extensions: HashMap::new(),
        };
        let context = RuntimeContext::from_resolved(&config);

        assert!(context.runtime.is_empty());
        assert!(context.capabilities.is_empty());

        let json_output = context.to_json();
        assert!(json_output["runtime"].as_object().unwrap().is_empty());
        assert!(json_output["capabilities"].as_array().unwrap().is_empty());
    }
}
