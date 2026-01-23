//! Integration tests for provider registry

use repo_meta::Registry;

#[test]
fn test_register_and_get() {
    let mut registry = Registry::new();
    registry.register("env:python", "uv");
    registry.register("env:node", "nvm");

    assert_eq!(registry.get_provider("env:python"), Some(&"uv".to_string()));
    assert_eq!(registry.get_provider("env:node"), Some(&"nvm".to_string()));
}

#[test]
fn test_unknown_preset_returns_none() {
    let registry = Registry::new();

    assert_eq!(registry.get_provider("unknown:preset"), None);
    assert!(!registry.has_provider("unknown:preset"));
}

#[test]
fn test_list_presets() {
    let mut registry = Registry::new();
    registry.register("env:python", "uv");
    registry.register("env:node", "nvm");
    registry.register("tool:formatter", "prettier");

    let presets = registry.list_presets();

    // Should be sorted alphabetically
    assert_eq!(presets.len(), 3);
    assert_eq!(presets[0], "env:node");
    assert_eq!(presets[1], "env:python");
    assert_eq!(presets[2], "tool:formatter");
}

#[test]
fn test_has_provider() {
    let mut registry = Registry::new();
    registry.register("env:python", "uv");

    assert!(registry.has_provider("env:python"));
    assert!(!registry.has_provider("env:node"));
}

#[test]
fn test_with_builtins() {
    let registry = Registry::with_builtins();

    // env:python should be pre-registered with uv
    assert!(registry.has_provider("env:python"));
    assert_eq!(registry.get_provider("env:python"), Some(&"uv".to_string()));
}

#[test]
fn test_register_overwrites_existing() {
    let mut registry = Registry::new();
    registry.register("env:python", "pyenv");
    registry.register("env:python", "uv");

    // Should have the new value
    assert_eq!(registry.get_provider("env:python"), Some(&"uv".to_string()));
    // Should still only have one entry
    assert_eq!(registry.len(), 1);
}

#[test]
fn test_empty_registry() {
    let registry = Registry::new();

    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
    assert!(registry.list_presets().is_empty());
}

#[test]
fn test_string_types_for_register() {
    let mut registry = Registry::new();

    // Test with &str
    registry.register("preset1", "provider1");

    // Test with String
    registry.register(String::from("preset2"), String::from("provider2"));

    assert_eq!(registry.get_provider("preset1"), Some(&"provider1".to_string()));
    assert_eq!(registry.get_provider("preset2"), Some(&"provider2".to_string()));
}
