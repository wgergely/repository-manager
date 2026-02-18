//! Ledger system for intent and projection tracking
//!
//! The ledger is the central registry of all active intents and their
//! projections. It provides persistence via TOML serialization and
//! query methods for finding intents and projections.

mod intent;
mod projection;

pub use intent::Intent;
pub use projection::{Projection, ProjectionKind};

use crate::Result;
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::path::Path;
use uuid::Uuid;

/// The ledger tracks all active intents and their projections
///
/// The ledger is persisted as a TOML file and provides the source of truth
/// for what configuration should be present in the repository.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Ledger {
    /// Ledger format version for forward compatibility
    version: String,
    /// All active intents
    intents: Vec<Intent>,
}

impl Ledger {
    /// Create a new empty ledger
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            intents: Vec::new(),
        }
    }

    /// Load a ledger from a TOML file with shared lock
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the ledger TOML file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read, locked, or parsed.
    pub fn load(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        file.lock_shared()?;

        // Read through the locked file handle to avoid TOCTOU race
        let mut content = String::new();
        use std::io::Read;
        (&file).read_to_string(&mut content)?;
        let ledger: Ledger = toml::from_str(&content)?;

        // Lock released when file is dropped
        Ok(ledger)
    }

    /// Save the ledger to a TOML file atomically with exclusive lock
    ///
    /// Uses write-to-temp-then-rename pattern with file locking to prevent
    /// corruption and race conditions.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to save the ledger TOML file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written or locked.
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;

        // Create or open the target file for locking
        let lock_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;

        // Acquire exclusive lock (blocks if another process holds lock)
        lock_file.lock_exclusive()?;

        // Write to temporary file first
        let temp_path = path.with_extension("toml.tmp");
        fs::write(&temp_path, &content)?;

        // Atomically rename to target
        fs::rename(&temp_path, path)?;

        // Lock released when lock_file is dropped
        Ok(())
    }

    /// Get all intents in the ledger
    pub fn intents(&self) -> &[Intent] {
        &self.intents
    }

    /// Add an intent to the ledger
    pub fn add_intent(&mut self, intent: Intent) {
        self.intents.push(intent);
    }

    /// Remove an intent by UUID
    ///
    /// Returns the removed intent if found, None otherwise.
    pub fn remove_intent(&mut self, uuid: Uuid) -> Option<Intent> {
        let pos = self.intents.iter().position(|i| i.uuid == uuid)?;
        Some(self.intents.remove(pos))
    }

    /// Get an intent by UUID
    pub fn get_intent(&self, uuid: Uuid) -> Option<&Intent> {
        self.intents.iter().find(|i| i.uuid == uuid)
    }

    /// Get a mutable reference to an intent by UUID
    pub fn get_intent_mut(&mut self, uuid: Uuid) -> Option<&mut Intent> {
        self.intents.iter_mut().find(|i| i.uuid == uuid)
    }

    /// Find all intents with a specific rule ID
    pub fn find_by_rule(&self, rule_id: &str) -> Vec<&Intent> {
        self.intents.iter().filter(|i| i.id == rule_id).collect()
    }

    /// Find all projections that target a specific file
    ///
    /// Returns tuples of (intent, projection) for all projections
    /// that write to the specified file.
    pub fn projections_for_file(&self, file: &Path) -> Vec<(&Intent, &Projection)> {
        let mut results = Vec::new();
        for intent in &self.intents {
            for projection in intent.projections() {
                if projection.file == file {
                    results.push((intent, projection));
                }
            }
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ledger_new_has_correct_version() {
        let ledger = Ledger::new();
        assert_eq!(ledger.version, "1.0");
    }

    #[test]
    fn ledger_save_is_atomic() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("ledger.toml");

        let mut ledger = Ledger::new();
        ledger.add_intent(Intent::new(
            "rule:test".to_string(),
            json!({"key": "value"}),
        ));

        // Save ledger
        ledger.save(&path).unwrap();

        // Verify no temp file left behind
        let temp_path = path.with_extension("toml.tmp");
        assert!(!temp_path.exists(), "Temporary file should be cleaned up");

        // Verify the saved content round-trips correctly with full content checks
        let loaded = Ledger::load(&path).unwrap();
        assert_eq!(loaded.intents().len(), 1);
        assert_eq!(loaded.intents()[0].id, "rule:test");
        assert_eq!(loaded.intents()[0].args["key"], "value");

        // Verify the raw file contains expected TOML structure
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(raw.contains("version = \"1.0\""));
        assert!(raw.contains("rule:test"));
    }

    #[test]
    fn ledger_serializes_with_version() {
        let ledger = Ledger::new();
        let serialized = toml::to_string(&ledger).unwrap();
        assert!(serialized.contains("version = \"1.0\""));
    }

    #[test]
    fn ledger_round_trips_through_toml() {
        let mut ledger = Ledger::new();
        ledger.add_intent(Intent::new(
            "rule:test".to_string(),
            json!({"key": "value"}),
        ));

        let serialized = toml::to_string(&ledger).unwrap();
        let deserialized: Ledger = toml::from_str(&serialized).unwrap();

        assert_eq!(deserialized.intents.len(), 1);
        assert_eq!(deserialized.intents[0].id, "rule:test");
    }
}
