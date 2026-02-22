use repo_fs::{LayoutMode, NormalizedPath, WorkspaceLayout};
use repo_presets::context::Context;
use std::collections::HashMap;
use tempfile::TempDir;

pub fn create_test_context(temp: &TempDir) -> Context {
    let layout = WorkspaceLayout {
        root: NormalizedPath::new(temp.path()),
        active_context: NormalizedPath::new(temp.path()),
        mode: LayoutMode::Classic,
    };
    Context::new(layout, HashMap::new())
}
