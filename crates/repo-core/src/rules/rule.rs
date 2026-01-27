//! Rule type for the central rule registry
//!
//! A Rule represents a configuration instruction that can be projected
//! to multiple tool config files. The Rule UUID becomes the block marker.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// A rule in the registry
///
/// Rules are the atomic unit of configuration. Each rule has a unique UUID
/// that is used as the managed block marker in tool config files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Unique identifier - used as block marker in tool configs
    pub uuid: Uuid,
    /// Human-readable identifier (e.g., "python-style")
    pub id: String,
    /// The rule content (Markdown)
    pub content: String,
    /// When the rule was created
    pub created: DateTime<Utc>,
    /// When the rule was last updated
    pub updated: DateTime<Utc>,
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
    /// SHA-256 hash of content for drift detection
    pub content_hash: String,
}

impl Rule {
    /// Create a new rule with generated UUID and computed hash
    pub fn new(id: impl Into<String>, content: impl Into<String>, tags: Vec<String>) -> Self {
        let content = content.into();
        let content_hash = Self::compute_hash_for(&content);
        let now = Utc::now();

        Self {
            uuid: Uuid::new_v4(),
            id: id.into(),
            content,
            created: now,
            updated: now,
            tags,
            content_hash,
        }
    }

    /// Create a rule with a specific UUID (for testing or migration)
    pub fn with_uuid(
        uuid: Uuid,
        id: impl Into<String>,
        content: impl Into<String>,
        tags: Vec<String>,
    ) -> Self {
        let content = content.into();
        let content_hash = Self::compute_hash_for(&content);
        let now = Utc::now();

        Self {
            uuid,
            id: id.into(),
            content,
            created: now,
            updated: now,
            tags,
            content_hash,
        }
    }

    /// Compute SHA-256 hash for content
    fn compute_hash_for(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        format!("sha256:{:x}", result)
    }

    /// Update the content and recompute hash
    pub fn update_content(&mut self, new_content: impl Into<String>) {
        self.content = new_content.into();
        self.content_hash = Self::compute_hash_for(&self.content);
        self.updated = Utc::now();
    }

    /// Check if given content has drifted from this rule
    pub fn has_drifted(&self, current_content: &str) -> bool {
        let current_hash = Self::compute_hash_for(current_content);
        self.content_hash != current_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_generates_uuid() {
        let rule = Rule::new("test", "content", vec![]);
        assert!(!rule.uuid.is_nil());
    }

    #[test]
    fn test_rule_computes_hash() {
        let rule = Rule::new("test", "content", vec![]);
        assert!(rule.content_hash.starts_with("sha256:"));
    }

    #[test]
    fn test_same_content_same_hash() {
        let rule1 = Rule::new("r1", "same content", vec![]);
        let rule2 = Rule::new("r2", "same content", vec![]);
        assert_eq!(rule1.content_hash, rule2.content_hash);
    }

    #[test]
    fn test_different_content_different_hash() {
        let rule1 = Rule::new("r1", "content a", vec![]);
        let rule2 = Rule::new("r2", "content b", vec![]);
        assert_ne!(rule1.content_hash, rule2.content_hash);
    }

    #[test]
    fn test_update_content_changes_hash() {
        let mut rule = Rule::new("test", "original", vec![]);
        let original_hash = rule.content_hash.clone();

        rule.update_content("modified");

        assert_ne!(rule.content_hash, original_hash);
        assert_eq!(rule.content, "modified");
    }

    #[test]
    fn test_drift_detection() {
        let rule = Rule::new("test", "original content", vec![]);

        assert!(!rule.has_drifted("original content"));
        assert!(rule.has_drifted("drifted content"));
    }
}
