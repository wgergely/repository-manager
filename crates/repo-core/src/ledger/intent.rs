//! Intent type for tracking configuration rules
//!
//! An intent represents a configuration rule that should be applied to one
//! or more tools. Each intent has projections that track how the rule is
//! rendered in each tool's configuration format.

use super::projection::Projection;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use uuid::Uuid;

/// An intent representing a configuration rule instance
///
/// Intents are the core unit of configuration in repository-manager.
/// Each intent references a rule (by ID) and tracks how that rule
/// is projected into various tool configurations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    /// The rule identifier (e.g., "rule:python/style/snake-case")
    pub id: String,
    /// Unique instance identifier for this intent
    pub uuid: Uuid,
    /// When this intent was created
    pub timestamp: DateTime<Utc>,
    /// Rule arguments/configuration
    pub args: Value,
    /// Projections of this intent into tool configurations
    projections: Vec<Projection>,
}

impl Intent {
    /// Create a new intent with a generated UUID and current timestamp
    ///
    /// # Arguments
    ///
    /// * `id` - The rule identifier
    /// * `args` - Rule arguments as a JSON value
    pub fn new(id: String, args: Value) -> Self {
        Self {
            id,
            uuid: Uuid::new_v4(),
            timestamp: Utc::now(),
            args,
            projections: Vec::new(),
        }
    }

    /// Create an intent with a specific UUID (useful for testing/recreation)
    ///
    /// # Arguments
    ///
    /// * `id` - The rule identifier
    /// * `uuid` - Specific UUID to use
    /// * `args` - Rule arguments as a JSON value
    pub fn with_uuid(id: String, uuid: Uuid, args: Value) -> Self {
        Self {
            id,
            uuid,
            timestamp: Utc::now(),
            args,
            projections: Vec::new(),
        }
    }

    /// Get all projections for this intent
    pub fn projections(&self) -> &[Projection] {
        &self.projections
    }

    /// Add a projection to this intent
    pub fn add_projection(&mut self, projection: Projection) {
        self.projections.push(projection);
    }

    /// Remove a projection by tool and file path
    ///
    /// Returns the removed projection if found, None otherwise.
    pub fn remove_projection(&mut self, tool: &str, file: &Path) -> Option<Projection> {
        let pos = self
            .projections
            .iter()
            .position(|p| p.tool == tool && p.file == file)?;
        Some(self.projections.remove(pos))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn intent_new_generates_uuid_and_timestamp() {
        let intent = Intent::new("rule:test".to_string(), json!({}));

        assert!(!intent.uuid.is_nil());
        // Timestamp should be recent (within last minute)
        let now = Utc::now();
        let diff = now.signed_duration_since(intent.timestamp);
        assert!(diff.num_seconds() < 60);
    }

    #[test]
    fn intent_with_uuid_uses_provided_uuid() {
        let fixed_uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let intent = Intent::with_uuid("rule:test".to_string(), fixed_uuid, json!({}));

        assert_eq!(intent.uuid, fixed_uuid);
    }

    #[test]
    fn intent_serializes_to_toml() {
        let intent = Intent::new("rule:python/style".to_string(), json!({"level": "strict"}));

        let serialized = toml::to_string(&intent).unwrap();
        assert!(serialized.contains("rule:python/style"));
    }
}
