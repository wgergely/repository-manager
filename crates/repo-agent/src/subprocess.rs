//! Subprocess execution for vaultspec CLI commands
//!
//! This module wraps vaultspec CLI invocations as subprocesses,
//! capturing structured JSON output and translating it into
//! repo-agent types.

use std::path::Path;
use std::process::Command;

use crate::error::{AgentError, Result};
use crate::types::AgentInfo;

/// Execute a vaultspec CLI command and return raw stdout
///
/// Invokes `python -m vaultspec <args>` in the given working directory.
/// Returns the stdout as a string on success.
pub fn run_vaultspec(
    python_path: &Path,
    vaultspec_path: &Path,
    working_dir: &Path,
    args: &[&str],
) -> Result<String> {
    let mut cmd = Command::new(python_path);
    cmd.current_dir(working_dir)
        .env("VAULTSPEC_HOME", vaultspec_path)
        .arg("-m")
        .arg("vaultspec")
        .args(args);

    let output = cmd.output().map_err(AgentError::Io)?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let code = output.status.code().unwrap_or(-1);
        Err(AgentError::CommandFailed { code, stderr })
    }
}

/// List agents by reading agent definition files from the vaultspec directory
///
/// Parses YAML/TOML agent definition files from `.vaultspec/agents/`
/// and returns structured `AgentInfo` for each.
pub fn list_agents(vaultspec_path: &Path) -> Result<Vec<AgentInfo>> {
    let agents_dir = vaultspec_path.join("agents");
    if !agents_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut agents = Vec::new();

    let entries = std::fs::read_dir(&agents_dir).map_err(AgentError::Io)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !matches!(ext, "yaml" | "yml" | "toml" | "json") {
            continue;
        }

        // Extract agent name from filename
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Try to parse basic fields from the file
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let (tier, provider) = parse_agent_metadata(&content, ext);

        agents.push(AgentInfo {
            name,
            tier,
            provider,
            available: true,
        });
    }

    agents.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(agents)
}

/// Parse basic agent metadata from file content
///
/// Extracts tier and provider fields from YAML/TOML agent definitions.
/// Returns defaults if parsing fails.
fn parse_agent_metadata(content: &str, ext: &str) -> (String, String) {
    let default_tier = "worker".to_string();
    let default_provider = "default".to_string();

    match ext {
        "yaml" | "yml" => {
            // Simple line-based parsing to avoid adding serde_yaml dependency
            let tier = extract_yaml_field(content, "tier").unwrap_or(default_tier);
            let provider = extract_yaml_field(content, "provider").unwrap_or(default_provider);
            (tier, provider)
        }
        "toml" => {
            // Simple line-based parsing
            let tier = extract_toml_field(content, "tier").unwrap_or(default_tier);
            let provider = extract_toml_field(content, "provider").unwrap_or(default_provider);
            (tier, provider)
        }
        "json" => {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(content) {
                let tier = value
                    .get("tier")
                    .and_then(|v| v.as_str())
                    .unwrap_or("worker")
                    .to_string();
                let provider = value
                    .get("provider")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default")
                    .to_string();
                (tier, provider)
            } else {
                (default_tier, default_provider)
            }
        }
        _ => (default_tier, default_provider),
    }
}

/// Extract a simple key: value field from YAML content
fn extract_yaml_field(content: &str, key: &str) -> Option<String> {
    let prefix = format!("{}:", key);
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&prefix) {
            let value = trimmed[prefix.len()..].trim();
            // Strip quotes if present
            let value = value.trim_matches('"').trim_matches('\'');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Extract a simple key = "value" field from TOML content
fn extract_toml_field(content: &str, key: &str) -> Option<String> {
    let prefix = format!("{} =", key);
    let prefix_nospace = format!("{}=", key);
    for line in content.lines() {
        let trimmed = line.trim();
        let rest = if trimmed.starts_with(&prefix) {
            Some(trimmed[prefix.len()..].trim())
        } else if trimmed.starts_with(&prefix_nospace) {
            Some(trimmed[prefix_nospace.len()..].trim())
        } else {
            None
        };
        if let Some(value) = rest {
            let value = value.trim_matches('"').trim_matches('\'');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_list_agents_empty_dir() {
        let temp = TempDir::new().unwrap();
        let vaultspec_dir = temp.path().join(".vaultspec");
        std::fs::create_dir_all(&vaultspec_dir).unwrap();

        let agents = list_agents(&vaultspec_dir).unwrap();
        assert!(agents.is_empty());
    }

    #[test]
    fn test_list_agents_with_yaml() {
        let temp = TempDir::new().unwrap();
        let agents_dir = temp.path().join(".vaultspec/agents");
        std::fs::create_dir_all(&agents_dir).unwrap();

        std::fs::write(
            agents_dir.join("researcher.yaml"),
            "name: researcher\ntier: specialist\nprovider: claude\n",
        )
        .unwrap();

        let agents = list_agents(&temp.path().join(".vaultspec")).unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "researcher");
        assert_eq!(agents[0].tier, "specialist");
        assert_eq!(agents[0].provider, "claude");
    }

    #[test]
    fn test_list_agents_with_toml() {
        let temp = TempDir::new().unwrap();
        let agents_dir = temp.path().join(".vaultspec/agents");
        std::fs::create_dir_all(&agents_dir).unwrap();

        std::fs::write(
            agents_dir.join("coder.toml"),
            "name = \"coder\"\ntier = \"worker\"\nprovider = \"gemini\"\n",
        )
        .unwrap();

        let agents = list_agents(&temp.path().join(".vaultspec")).unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "coder");
        assert_eq!(agents[0].tier, "worker");
        assert_eq!(agents[0].provider, "gemini");
    }

    #[test]
    fn test_list_agents_with_json() {
        let temp = TempDir::new().unwrap();
        let agents_dir = temp.path().join(".vaultspec/agents");
        std::fs::create_dir_all(&agents_dir).unwrap();

        std::fs::write(
            agents_dir.join("executor.json"),
            r#"{"name": "executor", "tier": "orchestrator", "provider": "claude"}"#,
        )
        .unwrap();

        let agents = list_agents(&temp.path().join(".vaultspec")).unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "executor");
        assert_eq!(agents[0].tier, "orchestrator");
        assert_eq!(agents[0].provider, "claude");
    }

    #[test]
    fn test_list_agents_skips_non_agent_files() {
        let temp = TempDir::new().unwrap();
        let agents_dir = temp.path().join(".vaultspec/agents");
        std::fs::create_dir_all(&agents_dir).unwrap();

        std::fs::write(agents_dir.join("readme.md"), "not an agent").unwrap();
        std::fs::write(agents_dir.join("agent.yaml"), "tier: worker").unwrap();

        let agents = list_agents(&temp.path().join(".vaultspec")).unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "agent");
    }

    #[test]
    fn test_list_agents_sorted() {
        let temp = TempDir::new().unwrap();
        let agents_dir = temp.path().join(".vaultspec/agents");
        std::fs::create_dir_all(&agents_dir).unwrap();

        std::fs::write(agents_dir.join("zulu.yaml"), "tier: worker").unwrap();
        std::fs::write(agents_dir.join("alpha.yaml"), "tier: worker").unwrap();
        std::fs::write(agents_dir.join("mike.yaml"), "tier: worker").unwrap();

        let agents = list_agents(&temp.path().join(".vaultspec")).unwrap();
        assert_eq!(agents[0].name, "alpha");
        assert_eq!(agents[1].name, "mike");
        assert_eq!(agents[2].name, "zulu");
    }

    #[test]
    fn test_extract_yaml_field() {
        let content = "name: researcher\ntier: specialist\nprovider: claude\n";
        assert_eq!(
            extract_yaml_field(content, "tier"),
            Some("specialist".into())
        );
        assert_eq!(
            extract_yaml_field(content, "provider"),
            Some("claude".into())
        );
        assert_eq!(extract_yaml_field(content, "missing"), None);
    }

    #[test]
    fn test_extract_yaml_field_quoted() {
        let content = "tier: \"orchestrator\"\nprovider: 'gemini'\n";
        assert_eq!(
            extract_yaml_field(content, "tier"),
            Some("orchestrator".into())
        );
        assert_eq!(
            extract_yaml_field(content, "provider"),
            Some("gemini".into())
        );
    }

    #[test]
    fn test_extract_toml_field() {
        let content = "name = \"coder\"\ntier = \"worker\"\n";
        assert_eq!(extract_toml_field(content, "tier"), Some("worker".into()));
        assert_eq!(extract_toml_field(content, "name"), Some("coder".into()));
        assert_eq!(extract_toml_field(content, "missing"), None);
    }

    #[test]
    fn test_parse_agent_metadata_yaml() {
        let content = "tier: specialist\nprovider: claude\n";
        let (tier, provider) = parse_agent_metadata(content, "yaml");
        assert_eq!(tier, "specialist");
        assert_eq!(provider, "claude");
    }

    #[test]
    fn test_parse_agent_metadata_defaults() {
        let (tier, provider) = parse_agent_metadata("", "yaml");
        assert_eq!(tier, "worker");
        assert_eq!(provider, "default");
    }
}
