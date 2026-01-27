//! Execution context for preset providers

use repo_fs::{NormalizedPath, WorkspaceLayout};
use std::collections::HashMap;

/// Context passed to providers for check/apply operations
#[derive(Debug, Clone)]
pub struct Context {
    pub layout: WorkspaceLayout,
    pub root: NormalizedPath,
    pub config: HashMap<String, toml::Value>,
    /// Optional tag for venv naming (e.g., "main-win-py311")
    pub venv_tag: Option<String>,
}

impl Context {
    pub fn new(layout: WorkspaceLayout, config: HashMap<String, toml::Value>) -> Self {
        let root = layout.root.clone();
        Self {
            layout,
            root,
            config,
            venv_tag: None,
        }
    }

    /// Create a context with a venv tag
    pub fn with_venv_tag(mut self, tag: impl Into<String>) -> Self {
        self.venv_tag = Some(tag.into());
        self
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

    /// Get the venv path, optionally tagged
    ///
    /// Returns:
    /// - `.venv` if no tag is set
    /// - `.venv-{tag}` if a tag is set
    pub fn venv_path(&self) -> NormalizedPath {
        match &self.venv_tag {
            Some(tag) => self.root.join(&format!(".venv-{}", tag)),
            None => self.root.join(".venv"),
        }
    }

    /// Get the venv path with an explicit tag
    ///
    /// Returns `.venv-{tag}` for the given tag.
    pub fn tagged_venv_path(&self, tag: &str) -> NormalizedPath {
        self.root.join(&format!(".venv-{}", tag))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_fs::LayoutMode;
    use tempfile::TempDir;

    fn make_test_context(temp: &TempDir, tag: Option<&str>) -> Context {
        let root = NormalizedPath::new(temp.path());
        let layout = WorkspaceLayout {
            root: root.clone(),
            active_context: root.clone(),
            mode: LayoutMode::Classic,
        };

        let mut ctx = Context::new(layout, HashMap::new());
        if let Some(t) = tag {
            ctx = ctx.with_venv_tag(t);
        }
        ctx
    }

    #[test]
    fn test_venv_path_untagged() {
        let temp = TempDir::new().unwrap();
        let ctx = make_test_context(&temp, None);
        let path = ctx.venv_path();
        assert!(path.as_str().ends_with(".venv"));
    }

    #[test]
    fn test_venv_path_tagged() {
        let temp = TempDir::new().unwrap();
        let ctx = make_test_context(&temp, Some("main-win-py312"));
        let path = ctx.venv_path();
        assert!(path.as_str().ends_with(".venv-main-win-py312"));
    }

    #[test]
    fn test_tagged_venv_path_explicit() {
        let temp = TempDir::new().unwrap();
        let ctx = make_test_context(&temp, None);
        let path = ctx.tagged_venv_path("feature-linux-py311");
        assert!(path.as_str().ends_with(".venv-feature-linux-py311"));
    }

    #[test]
    fn test_with_venv_tag() {
        let temp = TempDir::new().unwrap();
        let ctx = make_test_context(&temp, None);
        assert!(ctx.venv_tag.is_none());

        let ctx = ctx.with_venv_tag("test-tag");
        assert_eq!(ctx.venv_tag, Some("test-tag".to_string()));
    }
}
