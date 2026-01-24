//! Execution context for preset providers

use repo_fs::{NormalizedPath, WorkspaceLayout};
use std::collections::HashMap;

/// Context passed to providers for check/apply operations
#[derive(Debug, Clone)]
pub struct Context {
    pub layout: WorkspaceLayout,
    pub root: NormalizedPath,
    pub config: HashMap<String, toml::Value>,
}

impl Context {
    pub fn new(layout: WorkspaceLayout, config: HashMap<String, toml::Value>) -> Self {
        let root = layout.root.clone();
        Self {
            layout,
            root,
            config,
        }
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.config
            .get(key)
            .and_then(|v| v.as_str().map(String::from))
    }

    pub fn python_version(&self) -> String {
        self.get_string("version")
            .unwrap_or_else(|| "3.12".to_string())
    }

    pub fn provider(&self) -> String {
        self.get_string("provider")
            .unwrap_or_else(|| "uv".to_string())
    }

    pub fn venv_path(&self) -> NormalizedPath {
        self.root.join(".venv")
    }
}
