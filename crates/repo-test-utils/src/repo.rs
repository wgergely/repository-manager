//! [`TestRepo`] builder for repository-manager test scenarios.
//!
//! Extracted from `tests/integration/src/mission_tests.rs` to enable reuse
//! across all crates in the workspace.

use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// A temporary repository directory with helper methods for test setup and
/// assertion.
///
/// # Example
///
/// ```rust,no_run
/// use repo_test_utils::repo::TestRepo;
///
/// let mut repo = TestRepo::new();
/// repo.init_git();
/// repo.init_repo_manager("standard", &["cursor", "claude"], &[]);
/// repo.assert_file_exists(".repository/config.toml");
/// ```
pub struct TestRepo {
    temp_dir: TempDir,
    /// Whether `init_repo_manager` has been called.
    pub initialized: bool,
}

impl Default for TestRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl TestRepo {
    /// Create an empty temporary directory.
    pub fn new() -> Self {
        Self {
            temp_dir: TempDir::new().unwrap(),
            initialized: false,
        }
    }

    /// Return the root path of the temporary directory.
    pub fn root(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Initialise the directory as a real git repository using `git2`.
    ///
    /// Realism level: REAL — valid git state, empty history.
    pub fn init_git(&self) {
        git2::Repository::init(self.root())
            .expect("TestRepo::init_git: failed to init git repository");
    }

    /// Write a valid `.repository/config.toml` in the correct `Manifest` format:
    ///
    /// - Top-level `tools = [...]`
    /// - `[core]` section with `mode`
    /// - `[presets]` section as a table (one key per preset)
    pub fn init_repo_manager(&mut self, mode: &str, tools: &[&str], presets: &[&str]) {
        let repo_dir = self.root().join(".repository");
        fs::create_dir_all(&repo_dir).unwrap();

        let tools_str = tools
            .iter()
            .map(|t| format!("\"{}\"", t))
            .collect::<Vec<_>>()
            .join(", ");

        let mut config = format!("tools = [{tools_str}]\n\n[core]\nmode = \"{mode}\"\n");

        if !presets.is_empty() {
            config.push_str("\n[presets]\n");
            for preset in presets {
                config.push_str(&format!("\"{}\" = {{}}\n", preset));
            }
        }

        fs::write(repo_dir.join("config.toml"), config).unwrap();
        self.initialized = true;
    }

    /// Assert that `path` (relative to the repo root) exists.
    ///
    /// # Panics
    /// Panics with a descriptive message if the path does not exist.
    pub fn assert_file_exists(&self, path: &str) {
        let full_path = self.root().join(path);
        assert!(
            full_path.exists(),
            "Expected file to exist: {}",
            full_path.display()
        );
    }

    /// Assert that `path` (relative to the repo root) does **not** exist.
    ///
    /// # Panics
    /// Panics with a descriptive message if the path exists.
    pub fn assert_file_not_exists(&self, path: &str) {
        let full_path = self.root().join(path);
        assert!(
            !full_path.exists(),
            "Expected file NOT to exist: {}",
            full_path.display()
        );
    }

    /// Assert that the file at `path` (relative to root) contains `content`.
    ///
    /// # Panics
    /// Panics if the file cannot be read or does not contain `content`.
    pub fn assert_file_contains(&self, path: &str, content: &str) {
        let full_path = self.root().join(path);
        let file_content = fs::read_to_string(&full_path)
            .unwrap_or_else(|_| panic!("Could not read file: {}", full_path.display()));
        assert!(
            file_content.contains(content),
            "File {} does not contain expected content.\nExpected: {}\nActual: {}",
            full_path.display(),
            content,
            file_content
        );
    }
}
