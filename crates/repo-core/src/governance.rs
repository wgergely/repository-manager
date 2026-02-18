//! Config governance: lint, diff, export/import
//!
//! Provides rule validation, drift detection against synced state,
//! and AGENTS.md export/import capabilities.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::Manifest;
use crate::error::Result;
use crate::ledger::{Ledger, ProjectionKind};

/// Severity level for lint warnings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WarnLevel {
    /// Informational notice
    Info,
    /// Potential problem
    Warning,
    /// Configuration error
    Error,
}

impl std::fmt::Display for WarnLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// A lint warning about the configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintWarning {
    /// Severity
    pub level: WarnLevel,
    /// Human-readable description
    pub message: String,
    /// Tool this relates to, if applicable
    pub tool: Option<String>,
}

/// Type of configuration drift
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DriftType {
    /// File content has changed since last sync
    Modified,
    /// Expected file is missing from disk
    Missing,
    /// File exists on disk but tool was removed from config
    Extra,
}

impl std::fmt::Display for DriftType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Modified => write!(f, "modified"),
            Self::Missing => write!(f, "missing"),
            Self::Extra => write!(f, "extra"),
        }
    }
}

/// A single config drift between expected and actual state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDrift {
    /// Tool this drift belongs to
    pub tool: String,
    /// File path affected
    pub config_path: PathBuf,
    /// Type of drift
    pub drift_type: DriftType,
    /// Human-readable details
    pub details: String,
}

/// Lint the manifest for consistency issues
///
/// Checks for:
/// - Duplicate tools
/// - Empty tool list
/// - Empty rules list (informational)
/// - Tools that don't have known configurations
pub fn lint_rules(manifest: &Manifest, available_tools: &[String]) -> Vec<LintWarning> {
    let mut warnings = Vec::new();

    // Check for empty tools
    if manifest.tools.is_empty() {
        warnings.push(LintWarning {
            level: WarnLevel::Info,
            message: "No tools configured. Use 'repo add-tool' to add tools.".to_string(),
            tool: None,
        });
    }

    // Check for duplicate tools
    let mut seen = HashSet::new();
    for tool in &manifest.tools {
        if !seen.insert(tool.as_str()) {
            warnings.push(LintWarning {
                level: WarnLevel::Warning,
                message: format!("Duplicate tool '{}' in config.", tool),
                tool: Some(tool.clone()),
            });
        }
    }

    // Check for unknown tools
    if !available_tools.is_empty() {
        for tool in &manifest.tools {
            if !available_tools.iter().any(|t| t == tool) {
                warnings.push(LintWarning {
                    level: WarnLevel::Warning,
                    message: format!(
                        "Tool '{}' is not a recognized tool. It may not sync correctly.",
                        tool
                    ),
                    tool: Some(tool.clone()),
                });
            }
        }
    }

    // Check for empty rules
    if manifest.rules.is_empty() && !manifest.tools.is_empty() {
        warnings.push(LintWarning {
            level: WarnLevel::Info,
            message: "No rules configured. Use 'repo add-rule' to add rules.".to_string(),
            tool: None,
        });
    }

    warnings
}

/// Compare current config file state against the last-synced state in the ledger
///
/// For each tool in the config, checks if its generated config files:
/// - Still exist on disk
/// - Have the same content hash as recorded in the ledger
pub fn diff_configs(root: &Path, manifest: &Manifest) -> Result<Vec<ConfigDrift>> {
    let mut drifts = Vec::new();
    let ledger_path = root.join(".repository").join("ledger.toml");

    // If no ledger exists, everything is "missing" (never synced)
    if !ledger_path.exists() {
        for tool in &manifest.tools {
            drifts.push(ConfigDrift {
                tool: tool.clone(),
                config_path: PathBuf::from(format!("<{} config>", tool)),
                drift_type: DriftType::Missing,
                details: "Never synced (no ledger found).".to_string(),
            });
        }
        return Ok(drifts);
    }

    let ledger: Ledger = match Ledger::load(&ledger_path) {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!("Ledger is corrupt or unreadable: {}", e);
            // Report corruption explicitly rather than silently treating as all-drifted
            for tool in &manifest.tools {
                drifts.push(ConfigDrift {
                    tool: tool.clone(),
                    config_path: PathBuf::from("ledger.toml"),
                    drift_type: DriftType::Modified,
                    details: format!("Ledger is corrupt or unreadable: {}", e),
                });
            }
            return Ok(drifts);
        }
    };

    // Check each intent/projection in the ledger
    let configured_tools: HashSet<&str> = manifest.tools.iter().map(|s| s.as_str()).collect();

    for intent in ledger.intents() {
        for proj in intent.projections() {
            let tool = &proj.tool;
            let file_path = root.join(&proj.file);

            if !configured_tools.contains(tool.as_str()) {
                // Tool was removed from config but projections remain
                if file_path.exists() {
                    drifts.push(ConfigDrift {
                        tool: tool.clone(),
                        config_path: proj.file.clone(),
                        drift_type: DriftType::Extra,
                        details: format!(
                            "Tool '{}' removed from config but file still exists.",
                            tool
                        ),
                    });
                }
                continue;
            }

            if !file_path.exists() {
                drifts.push(ConfigDrift {
                    tool: tool.clone(),
                    config_path: proj.file.clone(),
                    drift_type: DriftType::Missing,
                    details: "Config file missing from disk.".to_string(),
                });
                continue;
            }

            // Check content hash based on projection kind
            let expected_checksum = match &proj.kind {
                ProjectionKind::TextBlock { checksum, .. } => Some(checksum),
                ProjectionKind::FileManaged { checksum } => Some(checksum),
                ProjectionKind::JsonKey { .. } => None,
            };

            if let Some(expected) = expected_checksum {
                let actual_content = std::fs::read_to_string(&file_path)?;
                let actual_checksum = crate::compute_checksum(&actual_content);
                if *expected != actual_checksum {
                    drifts.push(ConfigDrift {
                        tool: tool.clone(),
                        config_path: proj.file.clone(),
                        drift_type: DriftType::Modified,
                        details: "File content differs from last sync.".to_string(),
                    });
                }
            }
        }
    }

    Ok(drifts)
}

/// Export rules to AGENTS.md format
///
/// Generates a markdown document listing all rules with their content.
pub fn export_agents_md(root: &Path) -> Result<String> {
    let rules_dir = root.join(".repository").join("rules");
    let mut output = String::new();

    output.push_str("# AGENTS.md\n\n");
    output.push_str("<!-- Generated by repo rules export -->\n\n");

    if !rules_dir.is_dir() {
        output.push_str("No rules defined.\n");
        return Ok(output);
    }

    let mut entries: Vec<_> = std::fs::read_dir(&rules_dir)?
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    if entries.is_empty() {
        output.push_str("No rules defined.\n");
        return Ok(output);
    }

    for entry in entries {
        let path = entry.path();
        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let content = std::fs::read_to_string(&path)?;

        output.push_str(&format!("## {}\n\n", id));
        output.push_str(&content);
        if !content.ends_with('\n') {
            output.push('\n');
        }
        output.push('\n');
    }

    Ok(output)
}

/// Import rules from AGENTS.md format
///
/// Parses markdown content with `## rule-id` headers and returns
/// a list of (rule_id, content) pairs.
pub fn import_agents_md(content: &str) -> Vec<(String, String)> {
    let mut rules = Vec::new();
    let mut current_id: Option<String> = None;
    let mut current_content = String::new();

    for line in content.lines() {
        if let Some(id) = line.strip_prefix("## ") {
            // Save previous rule if any
            if let Some(prev_id) = current_id.take() {
                let trimmed = current_content.trim().to_string();
                if !trimmed.is_empty() {
                    rules.push((prev_id, trimmed));
                }
            }
            current_id = Some(id.trim().to_string());
            current_content = String::new();
        } else if current_id.is_some() {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    // Save last rule
    if let Some(id) = current_id {
        let trimmed = current_content.trim().to_string();
        if !trimmed.is_empty() {
            rules.push((id, trimmed));
        }
    }

    rules
}

#[cfg(test)]
mod tests {
    use super::*;
    fn make_manifest(tools: &[&str], rules: &[&str]) -> Manifest {
        Manifest {
            tools: tools.iter().map(|s| s.to_string()).collect(),
            rules: rules.iter().map(|s| s.to_string()).collect(),
            ..Manifest::empty()
        }
    }

    #[test]
    fn test_lint_empty_config() {
        let manifest = make_manifest(&[], &[]);
        let warnings = lint_rules(&manifest, &[]);
        assert!(
            warnings
                .iter()
                .any(|w| w.message.contains("No tools configured"))
        );
    }

    #[test]
    fn test_lint_duplicate_tools() {
        let manifest = make_manifest(&["claude", "cursor", "claude"], &[]);
        let warnings = lint_rules(&manifest, &[]);
        assert!(
            warnings
                .iter()
                .any(|w| w.message.contains("Duplicate tool"))
        );
    }

    #[test]
    fn test_lint_unknown_tool() {
        let available = vec!["claude".to_string(), "cursor".to_string()];
        let manifest = make_manifest(&["claude", "nonexistent"], &[]);
        let warnings = lint_rules(&manifest, &available);
        assert!(
            warnings
                .iter()
                .any(|w| w.message.contains("not a recognized tool"))
        );
    }

    #[test]
    fn test_lint_no_rules_info() {
        let manifest = make_manifest(&["claude"], &[]);
        let warnings = lint_rules(&manifest, &[]);
        assert!(
            warnings
                .iter()
                .any(|w| w.message.contains("No rules configured"))
        );
    }

    #[test]
    fn test_lint_clean_config() {
        let available = vec!["claude".to_string(), "cursor".to_string()];
        let manifest = make_manifest(&["claude", "cursor"], &["style-guide"]);
        let warnings = lint_rules(&manifest, &available);
        // Should only have info-level or no warnings
        assert!(warnings.iter().all(|w| w.level != WarnLevel::Error));
    }

    #[test]
    fn test_diff_no_ledger() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path();
        std::fs::create_dir_all(root.join(".repository")).unwrap();

        let manifest = make_manifest(&["claude"], &[]);
        let drifts = diff_configs(root, &manifest).unwrap();
        assert_eq!(drifts.len(), 1);
        assert_eq!(drifts[0].drift_type, DriftType::Missing);
        assert!(drifts[0].details.contains("Never synced"));
    }

    #[test]
    fn test_diff_empty_config() {
        let temp = tempfile::TempDir::new().unwrap();
        let manifest = make_manifest(&[], &[]);
        let drifts = diff_configs(temp.path(), &manifest).unwrap();
        assert!(drifts.is_empty());
    }

    #[test]
    fn test_export_agents_md_empty() {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join(".repository")).unwrap();

        let output = export_agents_md(temp.path()).unwrap();
        assert!(output.contains("# AGENTS.md"));
        assert!(output.contains("No rules defined"));
    }

    #[test]
    fn test_export_agents_md_with_rules() {
        let temp = tempfile::TempDir::new().unwrap();
        let rules_dir = temp.path().join(".repository/rules");
        std::fs::create_dir_all(&rules_dir).unwrap();

        std::fs::write(
            rules_dir.join("code-style.md"),
            "Use consistent formatting.",
        )
        .unwrap();
        std::fs::write(
            rules_dir.join("naming.md"),
            "tags: python\n\nUse snake_case.",
        )
        .unwrap();

        let output = export_agents_md(temp.path()).unwrap();
        assert!(output.contains("## code-style"));
        assert!(output.contains("Use consistent formatting."));
        assert!(output.contains("## naming"));
        assert!(output.contains("Use snake_case."));
    }

    #[test]
    fn test_import_agents_md() {
        let content = "# AGENTS.md\n\n## code-style\n\nUse consistent formatting.\n\n## naming\n\ntags: python\n\nUse snake_case.\n";
        let rules = import_agents_md(content);
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].0, "code-style");
        assert!(rules[0].1.contains("Use consistent formatting."));
        assert_eq!(rules[1].0, "naming");
        assert!(rules[1].1.contains("Use snake_case."));
    }

    #[test]
    fn test_import_agents_md_empty() {
        let rules = import_agents_md("# AGENTS.md\n\nNo rules defined.\n");
        assert!(rules.is_empty());
    }

    #[test]
    fn test_export_import_roundtrip() {
        let temp = tempfile::TempDir::new().unwrap();
        let rules_dir = temp.path().join(".repository/rules");
        std::fs::create_dir_all(&rules_dir).unwrap();

        std::fs::write(rules_dir.join("alpha.md"), "Alpha rule content.").unwrap();
        std::fs::write(rules_dir.join("beta.md"), "Beta rule content.").unwrap();

        let exported = export_agents_md(temp.path()).unwrap();
        let imported = import_agents_md(&exported);

        assert_eq!(imported.len(), 2);
        assert_eq!(imported[0].0, "alpha");
        assert!(imported[0].1.contains("Alpha rule content."));
        assert_eq!(imported[1].0, "beta");
        assert!(imported[1].1.contains("Beta rule content."));
    }

    #[test]
    fn test_warn_level_display() {
        assert_eq!(WarnLevel::Info.to_string(), "info");
        assert_eq!(WarnLevel::Warning.to_string(), "warning");
        assert_eq!(WarnLevel::Error.to_string(), "error");
    }

    #[test]
    fn test_drift_type_display() {
        assert_eq!(DriftType::Modified.to_string(), "modified");
        assert_eq!(DriftType::Missing.to_string(), "missing");
        assert_eq!(DriftType::Extra.to_string(), "extra");
    }
}
