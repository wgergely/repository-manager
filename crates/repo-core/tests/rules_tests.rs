//! Tests for the Rule Registry system

use repo_core::rules::{Rule, RuleRegistry};
use tempfile::TempDir;

#[test]
fn test_registry_add_rule_generates_uuid() {
    let temp = TempDir::new().unwrap();
    let registry_path = temp.path().join("registry.toml");

    let mut registry = RuleRegistry::new(registry_path);
    let rule = registry
        .add_rule(
            "python-style",
            "Use snake_case for variables",
            vec!["python".to_string()],
        )
        .unwrap();

    assert!(!rule.uuid.is_nil());
    assert_eq!(rule.id, "python-style");
    assert_eq!(rule.content, "Use snake_case for variables");
    assert!(rule.tags.contains(&"python".to_string()));
}

#[test]
fn test_registry_persists_to_toml() {
    let temp = TempDir::new().unwrap();
    let registry_path = temp.path().join("registry.toml");

    // Create and add rule
    {
        let mut registry = RuleRegistry::new(registry_path.clone());
        registry
            .add_rule("test-rule", "Test content", vec![])
            .unwrap();
    }

    // Load from file
    let loaded = RuleRegistry::load(registry_path).unwrap();
    assert_eq!(loaded.all_rules().len(), 1);
    assert_eq!(loaded.all_rules()[0].id, "test-rule");
}

#[test]
fn test_registry_get_rule_by_uuid() {
    let temp = TempDir::new().unwrap();
    let registry_path = temp.path().join("registry.toml");

    let mut registry = RuleRegistry::new(registry_path);
    let rule = registry.add_rule("my-rule", "Content", vec![]).unwrap();
    let uuid = rule.uuid;

    let found = registry.get_rule(uuid);
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, "my-rule");
}

#[test]
fn test_registry_get_rule_by_id() {
    let temp = TempDir::new().unwrap();
    let registry_path = temp.path().join("registry.toml");

    let mut registry = RuleRegistry::new(registry_path);
    registry.add_rule("unique-id", "Content", vec![]).unwrap();

    let found = registry.get_rule_by_id("unique-id");
    assert!(found.is_some());
    assert_eq!(found.unwrap().content, "Content");

    let not_found = registry.get_rule_by_id("nonexistent");
    assert!(not_found.is_none());
}

#[test]
fn test_registry_remove_rule() {
    let temp = TempDir::new().unwrap();
    let registry_path = temp.path().join("registry.toml");

    let mut registry = RuleRegistry::new(registry_path);
    let rule = registry.add_rule("to-remove", "Content", vec![]).unwrap();
    let uuid = rule.uuid;

    assert_eq!(registry.all_rules().len(), 1);

    let removed = registry.remove_rule(uuid);
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().id, "to-remove");
    assert_eq!(registry.all_rules().len(), 0);
}

#[test]
fn test_registry_update_rule() {
    let temp = TempDir::new().unwrap();
    let registry_path = temp.path().join("registry.toml");

    let mut registry = RuleRegistry::new(registry_path);
    let rule = registry
        .add_rule("updatable", "Original content", vec![])
        .unwrap();
    let uuid = rule.uuid;
    let original_hash = rule.content_hash.clone();

    registry.update_rule(uuid, "Updated content").unwrap();

    let updated = registry.get_rule(uuid).unwrap();
    assert_eq!(updated.content, "Updated content");
    assert_ne!(updated.content_hash, original_hash);
}

#[test]
fn test_rule_content_hash() {
    let rule = Rule::new("test", "Same content", vec![]);
    let rule2 = Rule::new("test2", "Same content", vec![]);

    // Same content should produce same hash
    assert_eq!(rule.content_hash, rule2.content_hash);

    let rule3 = Rule::new("test3", "Different content", vec![]);
    assert_ne!(rule.content_hash, rule3.content_hash);
}

#[test]
fn test_rule_drift_detection() {
    let rule = Rule::new("test", "Original content", vec![]);

    assert!(!rule.has_drifted("Original content"));
    assert!(rule.has_drifted("Modified content"));
}

#[test]
fn test_registry_rules_by_tag() {
    let temp = TempDir::new().unwrap();
    let registry_path = temp.path().join("registry.toml");

    let mut registry = RuleRegistry::new(registry_path);
    registry
        .add_rule("py1", "Python rule 1", vec!["python".to_string()])
        .unwrap();
    registry
        .add_rule(
            "py2",
            "Python rule 2",
            vec!["python".to_string(), "style".to_string()],
        )
        .unwrap();
    registry
        .add_rule("js1", "JS rule", vec!["javascript".to_string()])
        .unwrap();

    let python_rules = registry.rules_by_tag("python");
    assert_eq!(python_rules.len(), 2);

    let style_rules = registry.rules_by_tag("style");
    assert_eq!(style_rules.len(), 1);

    let js_rules = registry.rules_by_tag("javascript");
    assert_eq!(js_rules.len(), 1);
}

#[test]
fn test_registry_duplicate_id_allowed() {
    // Unlike UUID, id doesn't have to be unique (though it's recommended)
    let temp = TempDir::new().unwrap();
    let registry_path = temp.path().join("registry.toml");

    let mut registry = RuleRegistry::new(registry_path);
    let uuid1 = registry
        .add_rule("same-id", "Content 1", vec![])
        .unwrap()
        .uuid;
    let uuid2 = registry
        .add_rule("same-id", "Content 2", vec![])
        .unwrap()
        .uuid;

    // Both should exist with different UUIDs
    assert_ne!(uuid1, uuid2);
    assert_eq!(registry.all_rules().len(), 2);
}
