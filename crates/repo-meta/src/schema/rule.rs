//! Rule definition schema - loaded from .repository/rules/*.toml
//!
//! Rules define coding guidelines, style rules, and best practices
//! that can be synced to various tools.
//!
//! # Example TOML
//!
//! ```toml
//! [meta]
//! id = "python-snake-case"
//! severity = "mandatory"
//! tags = ["python", "style"]
//!
//! [content]
//! instruction = "Use snake_case for all Python variables and function names."
//!
//! [examples]
//! positive = ["my_variable = 1", "def calculate_total():"]
//! negative = ["myVariable = 1", "def calculateTotal():"]
//!
//! [targets]
//! files = ["**/*.py"]
//! ```

use serde::{Deserialize, Serialize};

/// Complete rule definition loaded from TOML
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuleDefinition {
    /// Rule metadata
    pub meta: RuleMeta,
    /// Rule content (the actual instruction)
    pub content: RuleContent,
    /// Optional examples (positive and negative)
    #[serde(default)]
    pub examples: Option<RuleExamples>,
    /// Optional targeting information
    #[serde(default)]
    pub targets: Option<RuleTargets>,
}

/// Rule metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuleMeta {
    /// Unique rule identifier (e.g., "python-snake-case")
    pub id: String,
    /// How strictly the rule should be enforced
    #[serde(default)]
    pub severity: Severity,
    /// Tags for categorization and filtering
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Rule severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Suggestion that can be optionally followed
    #[default]
    Suggestion,
    /// Mandatory rule that must be followed
    Mandatory,
}

/// The actual rule content/instruction
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuleContent {
    /// The instruction text that describes the rule
    pub instruction: String,
}

/// Examples demonstrating the rule
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RuleExamples {
    /// Examples that follow the rule correctly
    #[serde(default)]
    pub positive: Vec<String>,
    /// Examples that violate the rule
    #[serde(default)]
    pub negative: Vec<String>,
}

/// File targeting for the rule
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RuleTargets {
    /// Glob patterns for files this rule applies to
    #[serde(default, rename = "files")]
    pub file_patterns: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_default() {
        let severity = Severity::default();
        assert_eq!(severity, Severity::Suggestion);
    }

    #[test]
    fn test_parse_severity_from_toml() {
        // Test parsing severity within a TOML structure
        let toml_mandatory = r#"
[meta]
id = "test"
severity = "mandatory"

[content]
instruction = "test"
"#;
        let def: RuleDefinition = toml::from_str(toml_mandatory).unwrap();
        assert_eq!(def.meta.severity, Severity::Mandatory);

        let toml_suggestion = r#"
[meta]
id = "test"
severity = "suggestion"

[content]
instruction = "test"
"#;
        let def_sug: RuleDefinition = toml::from_str(toml_suggestion).unwrap();
        assert_eq!(def_sug.meta.severity, Severity::Suggestion);
    }

    #[test]
    fn test_examples_default() {
        let examples = RuleExamples::default();
        assert!(examples.positive.is_empty());
        assert!(examples.negative.is_empty());
    }

    #[test]
    fn test_targets_default() {
        let targets = RuleTargets::default();
        assert!(targets.file_patterns.is_empty());
    }
}
