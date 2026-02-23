//! Tests for ledger file locking and concurrent access
//!
//! These tests verify the behavior of ledger save/load under concurrent access,
//! including documenting known limitations of the current locking strategy.

use repo_core::ledger::{Intent, Ledger};
use serde_json::json;
use std::sync::{Arc, Barrier};
use std::thread;
use tempfile::tempdir;

#[test]
fn concurrent_ledger_modify_preserves_all_intents() {
    // Verify that Ledger::modify() resolves the TOCTOU race condition:
    // concurrent modifications under exclusive lock preserve all intents.
    let dir = tempdir().unwrap();
    let path = dir.path().join("ledger.toml");

    // Create initial ledger with a seed intent
    let mut ledger = Ledger::new();
    ledger.add_intent(Intent::new("rule:seed".to_string(), json!({})));
    ledger.save(&path).unwrap();

    let barrier = Arc::new(Barrier::new(2));
    let path1 = path.clone();
    let path2 = path.clone();
    let b1 = barrier.clone();
    let b2 = barrier.clone();

    // Two threads use Ledger::modify() concurrently.
    // The exclusive lock serializes the operations, so both intents are preserved.
    let t1 = thread::spawn(move || {
        b1.wait();
        Ledger::modify(&path1, |ledger| {
            ledger.add_intent(Intent::new("rule:thread1".to_string(), json!({})));
        })
        .unwrap();
    });

    let t2 = thread::spawn(move || {
        b2.wait();
        Ledger::modify(&path2, |ledger| {
            ledger.add_intent(Intent::new("rule:thread2".to_string(), json!({})));
        })
        .unwrap();
    });

    t1.join().unwrap();
    t2.join().unwrap();

    // The file must be structurally valid
    let final_ledger = Ledger::load(&path).unwrap();

    // All three intents must be present (seed + thread1 + thread2)
    let has_seed = final_ledger.intents().iter().any(|i| i.id == "rule:seed");
    let has_t1 = final_ledger
        .intents()
        .iter()
        .any(|i| i.id == "rule:thread1");
    let has_t2 = final_ledger
        .intents()
        .iter()
        .any(|i| i.id == "rule:thread2");

    assert!(has_seed, "Seed intent must be preserved");
    assert!(has_t1, "Thread 1's intent must be preserved");
    assert!(has_t2, "Thread 2's intent must be preserved");
    assert_eq!(
        final_ledger.intents().len(),
        3,
        "All 3 intents must be present (seed + thread1 + thread2)"
    );

    // Verify structural integrity
    let raw = std::fs::read_to_string(&path).unwrap();
    assert!(
        raw.contains("version = \"1.0\""),
        "Ledger file must contain version field"
    );

    for intent in final_ledger.intents() {
        assert!(!intent.id.is_empty(), "Intent ID must not be empty");
        assert!(!intent.uuid.is_nil(), "Intent UUID must not be nil");
    }
}

#[test]
fn sequential_ledger_saves_preserve_all_intents() {
    // When saves are properly serialized (no concurrent load-modify-save),
    // all intents are preserved. This is the correct usage pattern.
    let dir = tempdir().unwrap();
    let path = dir.path().join("ledger.toml");

    let ledger = Ledger::new();
    ledger.save(&path).unwrap();

    // Thread 1 does load-modify-save, then thread 2 does the same
    let path1 = path.clone();
    let t1 = thread::spawn(move || {
        let mut ledger = Ledger::load(&path1).unwrap();
        ledger.add_intent(Intent::new("rule:first".to_string(), json!({})));
        ledger.save(&path1).unwrap();
    });
    t1.join().unwrap();

    let path2 = path.clone();
    let t2 = thread::spawn(move || {
        let mut ledger = Ledger::load(&path2).unwrap();
        ledger.add_intent(Intent::new("rule:second".to_string(), json!({})));
        ledger.save(&path2).unwrap();
    });
    t2.join().unwrap();

    let final_ledger = Ledger::load(&path).unwrap();
    assert_eq!(
        final_ledger.intents().len(),
        2,
        "Sequential saves must preserve all intents"
    );

    let ids: Vec<&str> = final_ledger
        .intents()
        .iter()
        .map(|i| i.id.as_str())
        .collect();
    assert!(
        ids.contains(&"rule:first"),
        "First intent must be preserved"
    );
    assert!(
        ids.contains(&"rule:second"),
        "Second intent must be preserved"
    );
}

#[test]
fn ledger_save_cleans_up_temp_file_and_roundtrips() {
    // Verify that save() cleans up the temp file and the content round-trips correctly.
    // Note: this does not test atomicity guarantees under crash conditions.
    let dir = tempdir().unwrap();
    let path = dir.path().join("ledger.toml");

    let mut ledger = Ledger::new();
    ledger.add_intent(Intent::new(
        "rule:python/style".to_string(),
        json!({"severity": "warning"}),
    ));
    ledger.add_intent(Intent::new(
        "rule:rust/naming".to_string(),
        json!({"convention": "snake_case"}),
    ));

    ledger.save(&path).unwrap();

    // Temp file must not remain
    let temp_path = path.with_extension("toml.tmp");
    assert!(
        !temp_path.exists(),
        "Temporary file should be cleaned up after save"
    );

    // The saved file must be valid TOML that round-trips correctly
    let loaded = Ledger::load(&path).unwrap();
    assert_eq!(loaded.intents().len(), 2);

    // Verify actual content, not just count
    let ids: Vec<&str> = loaded.intents().iter().map(|i| i.id.as_str()).collect();
    assert!(ids.contains(&"rule:python/style"));
    assert!(ids.contains(&"rule:rust/naming"));

    // Verify args survived serialization
    let python_intent = loaded
        .intents()
        .iter()
        .find(|i| i.id == "rule:python/style")
        .unwrap();
    assert_eq!(python_intent.args["severity"], "warning");

    let rust_intent = loaded
        .intents()
        .iter()
        .find(|i| i.id == "rule:rust/naming")
        .unwrap();
    assert_eq!(rust_intent.args["convention"], "snake_case");
}

#[test]
fn ledger_save_fails_when_parent_directory_missing() {
    // Attempting to save to a path whose parent directory doesn't exist should
    // return an error, not panic or silently succeed.
    let dir = tempdir().unwrap();
    let path = dir
        .path()
        .join("nonexistent")
        .join("subdir")
        .join("ledger.toml");

    let ledger = Ledger::new();
    let result = ledger.save(&path);

    assert!(
        result.is_err(),
        "save() must return an error when parent directory doesn't exist"
    );
}

#[test]
fn ledger_load_fails_on_nonexistent_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("does_not_exist.toml");

    let result = Ledger::load(&path);
    assert!(
        result.is_err(),
        "load() must return an error for a nonexistent file"
    );
}

#[test]
fn ledger_save_overwrites_previous_content_completely() {
    // Verify that save() replaces the entire file content, not appending.
    let dir = tempdir().unwrap();
    let path = dir.path().join("ledger.toml");

    // Save a ledger with 2 intents
    let mut ledger = Ledger::new();
    ledger.add_intent(Intent::new("rule:first".to_string(), json!({})));
    ledger.add_intent(Intent::new("rule:second".to_string(), json!({})));
    ledger.save(&path).unwrap();

    let loaded = Ledger::load(&path).unwrap();
    assert_eq!(loaded.intents().len(), 2);

    // Now save a ledger with only 1 intent to the same path
    let mut smaller_ledger = Ledger::new();
    smaller_ledger.add_intent(Intent::new("rule:only_one".to_string(), json!({})));
    smaller_ledger.save(&path).unwrap();

    let reloaded = Ledger::load(&path).unwrap();
    assert_eq!(
        reloaded.intents().len(),
        1,
        "Save must completely replace file content"
    );
    assert_eq!(reloaded.intents()[0].id, "rule:only_one");

    // Old intents must not appear in the file at all
    let raw = std::fs::read_to_string(&path).unwrap();
    assert!(
        !raw.contains("rule:first"),
        "Old intent 'rule:first' must not remain in file"
    );
    assert!(
        !raw.contains("rule:second"),
        "Old intent 'rule:second' must not remain in file"
    );
}
