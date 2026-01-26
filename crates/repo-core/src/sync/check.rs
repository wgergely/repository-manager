//! Check types for SyncEngine validation
//!
//! Provides types for reporting the synchronization status between
//! the ledger and the filesystem.

use serde::{Deserialize, Serialize};

/// Status of the synchronization check
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckStatus {
    /// Everything is in sync
    Healthy,
    /// Some projections are missing from the filesystem
    Missing,
    /// Some projections have drifted from expected values
    Drifted,
    /// The ledger is corrupted or unreadable
    Broken,
}

/// An item that has drifted or is missing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftItem {
    /// The intent ID this drift belongs to
    pub intent_id: String,
    /// The tool that owns this projection
    pub tool: String,
    /// The file path affected
    pub file: String,
    /// Human-readable description of the drift
    pub description: String,
}

/// Report from a synchronization check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckReport {
    /// Overall status of the check
    pub status: CheckStatus,
    /// Items that have drifted from expected values
    pub drifted: Vec<DriftItem>,
    /// Items that are missing from the filesystem
    pub missing: Vec<DriftItem>,
    /// Additional messages about the check
    pub messages: Vec<String>,
}

impl CheckReport {
    /// Create a healthy check report with no issues
    pub fn healthy() -> Self {
        Self {
            status: CheckStatus::Healthy,
            drifted: Vec::new(),
            missing: Vec::new(),
            messages: Vec::new(),
        }
    }

    /// Create a check report with missing items
    pub fn with_missing(missing: Vec<DriftItem>) -> Self {
        Self {
            status: CheckStatus::Missing,
            drifted: Vec::new(),
            missing,
            messages: Vec::new(),
        }
    }

    /// Create a check report with drifted items
    pub fn with_drifted(drifted: Vec<DriftItem>) -> Self {
        Self {
            status: CheckStatus::Drifted,
            drifted,
            missing: Vec::new(),
            messages: Vec::new(),
        }
    }

    /// Create a check report indicating the ledger is broken
    pub fn broken(message: String) -> Self {
        Self {
            status: CheckStatus::Broken,
            drifted: Vec::new(),
            missing: Vec::new(),
            messages: vec![message],
        }
    }

    /// Merge two check reports, combining their issues
    ///
    /// The resulting status is the "worst" of the two:
    /// Broken > Drifted > Missing > Healthy
    pub fn merge(mut self, other: CheckReport) -> Self {
        self.drifted.extend(other.drifted);
        self.missing.extend(other.missing);
        self.messages.extend(other.messages);

        // Determine the worst status
        self.status = match (self.status, other.status) {
            (CheckStatus::Broken, _) | (_, CheckStatus::Broken) => CheckStatus::Broken,
            (CheckStatus::Drifted, _) | (_, CheckStatus::Drifted) => CheckStatus::Drifted,
            (CheckStatus::Missing, _) | (_, CheckStatus::Missing) => CheckStatus::Missing,
            (CheckStatus::Healthy, CheckStatus::Healthy) => CheckStatus::Healthy,
        };

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healthy_report() {
        let report = CheckReport::healthy();
        assert_eq!(report.status, CheckStatus::Healthy);
        assert!(report.drifted.is_empty());
        assert!(report.missing.is_empty());
        assert!(report.messages.is_empty());
    }

    #[test]
    fn test_with_missing_report() {
        let item = DriftItem {
            intent_id: "test".to_string(),
            tool: "vscode".to_string(),
            file: "settings.json".to_string(),
            description: "File not found".to_string(),
        };
        let report = CheckReport::with_missing(vec![item]);
        assert_eq!(report.status, CheckStatus::Missing);
        assert_eq!(report.missing.len(), 1);
    }

    #[test]
    fn test_with_drifted_report() {
        let item = DriftItem {
            intent_id: "test".to_string(),
            tool: "vscode".to_string(),
            file: "settings.json".to_string(),
            description: "Checksum mismatch".to_string(),
        };
        let report = CheckReport::with_drifted(vec![item]);
        assert_eq!(report.status, CheckStatus::Drifted);
        assert_eq!(report.drifted.len(), 1);
    }

    #[test]
    fn test_merge_reports() {
        let missing_item = DriftItem {
            intent_id: "test1".to_string(),
            tool: "vscode".to_string(),
            file: "a.json".to_string(),
            description: "Missing".to_string(),
        };
        let drifted_item = DriftItem {
            intent_id: "test2".to_string(),
            tool: "cursor".to_string(),
            file: "b.mdc".to_string(),
            description: "Drifted".to_string(),
        };

        let report1 = CheckReport::with_missing(vec![missing_item]);
        let report2 = CheckReport::with_drifted(vec![drifted_item]);

        let merged = report1.merge(report2);

        // Drifted is "worse" than Missing
        assert_eq!(merged.status, CheckStatus::Drifted);
        assert_eq!(merged.missing.len(), 1);
        assert_eq!(merged.drifted.len(), 1);
    }
}
