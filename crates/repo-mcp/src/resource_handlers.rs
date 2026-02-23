//! MCP Resource Handlers
//!
//! Read-only access to repository state.
//!
//! Note: Handler functions use `async fn` for consistency with the MCP server's
//! tokio runtime, even though the current implementations perform synchronous I/O.
//! This allows for future migration to async file operations without API changes.

use std::path::Path;

use tracing::warn;

use crate::resources::ResourceContent;
use crate::{Error, Result};

/// Read a resource by URI
///
/// # Arguments
///
/// * `root` - The repository root path
/// * `uri` - The resource URI (e.g., "repo://config")
///
/// # Returns
///
/// The resource content including URI, mime type, and text content.
///
/// # Errors
///
/// Returns `Error::UnknownResource` if the URI is not recognized.
pub async fn read_resource(root: &Path, uri: &str) -> Result<ResourceContent> {
    match uri {
        "repo://config" => read_config(root).await,
        "repo://state" => read_state(root).await,
        "repo://rules" => read_rules(root).await,
        _ => Err(Error::UnknownResource(uri.to_string())),
    }
}

/// Maximum file size for resource reads (10 MB)
const MAX_RESOURCE_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Read a file with a size limit to prevent OOM from maliciously large files.
fn read_file_bounded(path: &std::path::Path) -> std::result::Result<String, std::io::Error> {
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > MAX_RESOURCE_FILE_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "File too large ({} bytes, limit is {} bytes)",
                metadata.len(),
                MAX_RESOURCE_FILE_SIZE
            ),
        ));
    }
    std::fs::read_to_string(path)
}

/// Read repository configuration from .repository/config.toml
async fn read_config(root: &Path) -> Result<ResourceContent> {
    let config_path = root.join(".repository/config.toml");
    let text = match read_file_bounded(&config_path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            "# No configuration found\ntools = []\n\n[core]\nmode = \"worktrees\"\n".to_string()
        }
        Err(e) => {
            warn!("Failed to read config at {}: {}", config_path.display(), e);
            format!("# Error reading configuration: {}\n", e)
        }
    };

    Ok(ResourceContent {
        uri: "repo://config".to_string(),
        mime_type: "application/toml".to_string(),
        text,
    })
}

/// Read repository state from .repository/ledger.toml
async fn read_state(root: &Path) -> Result<ResourceContent> {
    let ledger_path = root.join(".repository/ledger.toml");
    let text = match read_file_bounded(&ledger_path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            "# No ledger found - run 'repo sync' to create\n".to_string()
        }
        Err(e) => {
            warn!("Failed to read ledger at {}: {}", ledger_path.display(), e);
            format!("# Error reading ledger: {}\n", e)
        }
    };

    Ok(ResourceContent {
        uri: "repo://state".to_string(),
        mime_type: "application/toml".to_string(),
        text,
    })
}

/// Maximum number of rule files to read
const MAX_RULE_FILES: usize = 500;

/// Read aggregated rules from .repository/rules/*.md
async fn read_rules(root: &Path) -> Result<ResourceContent> {
    let rules_dir = root.join(".repository/rules");
    let mut content = String::from("# Active Rules\n\n");

    if rules_dir.exists() {
        let mut entries: Vec<_> = std::fs::read_dir(&rules_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
            .collect();

        entries.sort_by_key(|e| e.file_name());

        if entries.len() > MAX_RULE_FILES {
            warn!(
                "Rules directory contains {} files, limiting to {}",
                entries.len(),
                MAX_RULE_FILES
            );
            entries.truncate(MAX_RULE_FILES);
        }

        let mut total_size: u64 = 0;

        for entry in entries {
            let rule_name = entry
                .path()
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            match read_file_bounded(&entry.path()) {
                Ok(rule_content) => {
                    total_size += rule_content.len() as u64;
                    if total_size > MAX_RESOURCE_FILE_SIZE {
                        content.push_str("\n_... truncated (total size limit reached)_\n");
                        break;
                    }
                    content.push_str(&format!("## {}\n\n", rule_name));
                    content.push_str(&rule_content);
                    content.push_str("\n\n---\n\n");
                }
                Err(e) => {
                    warn!("Failed to read rule file {}: {}", entry.path().display(), e);
                }
            }
        }
    }

    if content == "# Active Rules\n\n" {
        content.push_str("_No rules defined._\n");
    }

    Ok(ResourceContent {
        uri: "repo://rules".to_string(),
        mime_type: "text/markdown".to_string(),
        text: content,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_read_config_resource() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(".repository")).unwrap();
        fs::write(
            temp.path().join(".repository/config.toml"),
            "tools = [\"cursor\"]\n",
        )
        .unwrap();

        let result = read_resource(temp.path(), "repo://config").await.unwrap();
        assert_eq!(result.uri, "repo://config");
        assert_eq!(result.mime_type, "application/toml");
        assert!(result.text.contains("cursor"));
    }

    #[tokio::test]
    async fn test_read_config_resource_missing() {
        let temp = TempDir::new().unwrap();

        let result = read_resource(temp.path(), "repo://config").await.unwrap();
        assert_eq!(result.uri, "repo://config");
        assert_eq!(result.mime_type, "application/toml");
        assert!(result.text.contains("No configuration found"));
    }

    #[tokio::test]
    async fn test_read_state_resource() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(".repository")).unwrap();
        fs::write(
            temp.path().join(".repository/ledger.toml"),
            "[branches]\nmain = { protected = true }\n",
        )
        .unwrap();

        let result = read_resource(temp.path(), "repo://state").await.unwrap();
        assert_eq!(result.uri, "repo://state");
        assert_eq!(result.mime_type, "application/toml");
        assert!(result.text.contains("branches"));
    }

    #[tokio::test]
    async fn test_read_state_resource_missing() {
        let temp = TempDir::new().unwrap();

        let result = read_resource(temp.path(), "repo://state").await.unwrap();
        assert_eq!(result.uri, "repo://state");
        assert_eq!(result.mime_type, "application/toml");
        assert!(result.text.contains("No ledger"));
    }

    #[tokio::test]
    async fn test_read_rules_resource() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(".repository/rules")).unwrap();
        fs::write(
            temp.path().join(".repository/rules/test-rule.md"),
            "# Test Rule\n\nThis is a test rule.",
        )
        .unwrap();

        let result = read_resource(temp.path(), "repo://rules").await.unwrap();
        assert_eq!(result.uri, "repo://rules");
        assert_eq!(result.mime_type, "text/markdown");
        assert!(result.text.contains("test-rule"));
        assert!(result.text.contains("This is a test rule"));
    }

    #[tokio::test]
    async fn test_read_rules_resource_multiple() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(".repository/rules")).unwrap();
        fs::write(
            temp.path().join(".repository/rules/a-rule.md"),
            "First rule content.",
        )
        .unwrap();
        fs::write(
            temp.path().join(".repository/rules/b-rule.md"),
            "Second rule content.",
        )
        .unwrap();

        let result = read_resource(temp.path(), "repo://rules").await.unwrap();
        assert!(result.text.contains("a-rule"));
        assert!(result.text.contains("b-rule"));
        assert!(result.text.contains("First rule content"));
        assert!(result.text.contains("Second rule content"));
        // Verify sorted order (a-rule should come before b-rule)
        let a_pos = result.text.find("a-rule").unwrap();
        let b_pos = result.text.find("b-rule").unwrap();
        assert!(a_pos < b_pos);
    }

    #[tokio::test]
    async fn test_read_rules_resource_empty() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(".repository/rules")).unwrap();

        let result = read_resource(temp.path(), "repo://rules").await.unwrap();
        assert!(result.text.contains("No rules defined"));
    }

    #[tokio::test]
    async fn test_read_rules_resource_no_directory() {
        let temp = TempDir::new().unwrap();

        let result = read_resource(temp.path(), "repo://rules").await.unwrap();
        assert!(result.text.contains("No rules defined"));
    }

    #[tokio::test]
    async fn test_unknown_resource() {
        let temp = TempDir::new().unwrap();
        let result = read_resource(temp.path(), "repo://unknown").await;
        assert!(result.is_err());
        match result {
            Err(Error::UnknownResource(uri)) => assert_eq!(uri, "repo://unknown"),
            _ => panic!("Expected UnknownResource error"),
        }
    }
}
