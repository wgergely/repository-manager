//! ToolIntegration trait for syncing to external tools

use crate::error::Result;
use repo_fs::NormalizedPath;

/// Rule to be synced to tools
#[derive(Debug, Clone)]
pub struct Rule {
    pub id: String,
    pub content: String,
}

/// Context for tool sync operations
#[derive(Debug, Clone)]
pub struct SyncContext {
    pub root: NormalizedPath,
    pub python_path: Option<NormalizedPath>,
}

impl SyncContext {
    pub fn new(root: NormalizedPath) -> Self {
        Self {
            root,
            python_path: None,
        }
    }

    pub fn with_python(mut self, path: NormalizedPath) -> Self {
        self.python_path = Some(path);
        self
    }
}

/// Trait for tool integrations
pub trait ToolIntegration {
    fn name(&self) -> &str;
    fn config_paths(&self) -> Vec<&str>;
    fn sync(&self, context: &SyncContext, rules: &[Rule]) -> Result<()>;
}
