use repo_fs::{ConfigStore, NormalizedPath};
use serde::{Deserialize, Serialize};
use std::fs;
use tempfile::TempDir;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestConfig {
    name: String,
    count: i32,
}

#[test]
fn test_load_toml() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("config.toml");
    fs::write(&file_path, r#"name = "test"
count = 42"#).unwrap();

    let store = ConfigStore::new();
    let path = NormalizedPath::new(&file_path);
    let config: TestConfig = store.load(&path).unwrap();

    assert_eq!(config.name, "test");
    assert_eq!(config.count, 42);
}

#[test]
fn test_load_json() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("config.json");
    fs::write(&file_path, r#"{"name": "test", "count": 42}"#).unwrap();

    let store = ConfigStore::new();
    let path = NormalizedPath::new(&file_path);
    let config: TestConfig = store.load(&path).unwrap();

    assert_eq!(config.name, "test");
    assert_eq!(config.count, 42);
}

#[test]
fn test_load_yaml() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("config.yaml");
    fs::write(&file_path, "name: test\ncount: 42").unwrap();

    let store = ConfigStore::new();
    let path = NormalizedPath::new(&file_path);
    let config: TestConfig = store.load(&path).unwrap();

    assert_eq!(config.name, "test");
    assert_eq!(config.count, 42);
}

#[test]
fn test_save_toml() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("config.toml");
    let path = NormalizedPath::new(&file_path);

    let config = TestConfig { name: "test".into(), count: 42 };
    let store = ConfigStore::new();
    store.save(&path, &config).unwrap();

    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("name = \"test\""));
    assert!(content.contains("count = 42"));
}

#[test]
fn test_save_json() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("config.json");
    let path = NormalizedPath::new(&file_path);

    let config = TestConfig { name: "test".into(), count: 42 };
    let store = ConfigStore::new();
    store.save(&path, &config).unwrap();

    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("\"name\""));
    assert!(content.contains("\"test\""));
}

#[test]
fn test_unsupported_format() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("config.xyz");
    fs::write(&file_path, "data").unwrap();

    let store = ConfigStore::new();
    let path = NormalizedPath::new(&file_path);
    let result: repo_fs::Result<TestConfig> = store.load(&path);

    assert!(result.is_err());
}

#[test]
fn test_roundtrip_toml() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("config.toml");
    let path = NormalizedPath::new(&file_path);

    let original = TestConfig { name: "roundtrip".into(), count: 123 };
    let store = ConfigStore::new();

    store.save(&path, &original).unwrap();
    let loaded: TestConfig = store.load(&path).unwrap();

    assert_eq!(original, loaded);
}
