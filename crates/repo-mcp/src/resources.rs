//! MCP Resource implementations
//!
//! Resources provide read-only access to repository state.

use serde::{Deserialize, Serialize};

/// Resource definition for MCP protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDefinition {
    pub uri: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
}

/// Result from reading a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    pub uri: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub text: String,
}

/// Get all available resource definitions
pub fn get_resource_definitions() -> Vec<ResourceDefinition> {
    vec![
        ResourceDefinition {
            uri: "repo://config".to_string(),
            name: "Repository Configuration".to_string(),
            description: "Repository configuration from .repository/config.toml".to_string(),
            mime_type: "application/toml".to_string(),
        },
        ResourceDefinition {
            uri: "repo://state".to_string(),
            name: "Repository State".to_string(),
            description: "Computed state from .repository/ledger.toml".to_string(),
            mime_type: "application/toml".to_string(),
        },
        ResourceDefinition {
            uri: "repo://rules".to_string(),
            name: "Active Rules".to_string(),
            description: "Aggregated view of all active rules".to_string(),
            mime_type: "text/markdown".to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_resource_definitions() {
        let resources = get_resource_definitions();
        assert_eq!(resources.len(), 3);

        let uris: Vec<&str> = resources.iter().map(|r| r.uri.as_str()).collect();
        assert!(uris.contains(&"repo://config"));
        assert!(uris.contains(&"repo://state"));
        assert!(uris.contains(&"repo://rules"));
    }

    #[test]
    fn test_resource_definitions_serialize() {
        let resources = get_resource_definitions();
        let json = serde_json::to_string(&resources).unwrap();
        assert!(json.contains("repo://config"));
        assert!(json.contains("mimeType"));
    }
}
