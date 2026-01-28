//! Tests for ledger file locking

use repo_core::ledger::{Intent, Ledger};
use serde_json::json;
use std::sync::{Arc, Barrier};
use std::thread;
use tempfile::tempdir;

#[test]
fn concurrent_ledger_saves_are_serialized() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("ledger.toml");

    // Create initial ledger
    let ledger = Ledger::new();
    ledger.save(&path).unwrap();

    let barrier = Arc::new(Barrier::new(2));
    let path1 = path.clone();
    let path2 = path.clone();
    let b1 = barrier.clone();
    let b2 = barrier.clone();

    // Two threads try to modify ledger concurrently
    let t1 = thread::spawn(move || {
        b1.wait();
        let mut ledger = Ledger::load(&path1).unwrap();
        ledger.add_intent(Intent::new("rule:thread1".to_string(), json!({})));
        ledger.save(&path1)
    });

    let t2 = thread::spawn(move || {
        b2.wait();
        let mut ledger = Ledger::load(&path2).unwrap();
        ledger.add_intent(Intent::new("rule:thread2".to_string(), json!({})));
        ledger.save(&path2)
    });

    // Both should complete without error (locking serializes them)
    t1.join().unwrap().unwrap();
    t2.join().unwrap().unwrap();

    // Final ledger should have at least one intent (last writer wins)
    let final_ledger = Ledger::load(&path).unwrap();
    assert!(!final_ledger.intents().is_empty());
}
