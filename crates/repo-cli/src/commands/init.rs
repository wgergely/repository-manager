//! Init command implementation
//!
//! Initializes a new repository with Repository Manager configuration.

use std::path::{Path, PathBuf};
use std::process::Command;

use colored::Colorize;

use crate::error::{CliError, Result};

/// Configuration for init command
pub struct InitConfig {
    pub name: String,
    pub mode: String,
    pub tools: Vec<String>,
    pub presets: Vec<String>,
    pub extensions: Vec<String>,
    pub remote: Option<String>,
}

/// Run the init command
///
/// Initializes a repository with the specified mode, tools, and presets.
/// If name is not ".", creates a new folder with the sanitized name.
pub fn run_init(cwd: &Path, config: InitConfig) -> Result<PathBuf> {
    // Normalize mode early so all downstream usage (printing, config writing) is canonical
    let normalized_mode = normalize_mode(&config.mode)?;

    // Determine target path
    let target_path = if config.name == "." {
        cwd.to_path_buf()
    } else {
        let sanitized = sanitize_project_name(&config.name);
        let path = cwd.join(&sanitized);

        // Create the folder
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
            println!(
                "{} Created project folder: {}",
                "=>".blue().bold(),
                sanitized.cyan()
            );
        }
        path
    };

    println!(
        "{} Initializing repository in {} mode...",
        "=>".blue().bold(),
        normalized_mode.cyan()
    );

    if !config.tools.is_empty() {
        println!("   Tools: {}", config.tools.join(", ").yellow());
    }
    if !config.presets.is_empty() {
        println!("   Presets: {}", config.presets.join(", ").yellow());
    }
    if !config.extensions.is_empty() {
        println!("   Extensions: {}", config.extensions.join(", ").yellow());
    }

    init_repository(
        &target_path,
        &normalized_mode,
        &config.tools,
        &config.presets,
        &config.extensions,
    )?;

    // Add remote if specified
    if let Some(remote_url) = &config.remote {
        add_git_remote(&target_path, remote_url)?;
        println!("   Remote: {}", remote_url.yellow());
    }

    println!("{} Repository initialized!", "OK".green().bold());

    // Post-init guidance
    println!();
    println!(
        "{} Done. Run {} to generate tool configuration files.",
        "=>".green().bold(),
        "repo sync".cyan()
    );
    if config.tools.is_empty() {
        println!(
            "   No tools selected yet — run {} to add one first.",
            "repo add-tool <name>".cyan()
        );
        println!("   Run {} to see available tools", "repo list-tools".cyan());
    }

    Ok(target_path)
}

/// Normalize a mode string to its canonical form.
///
/// Accepts aliases like "worktree" and returns the canonical form "worktrees".
/// Returns an error for unrecognized mode strings.
fn normalize_mode(mode: &str) -> Result<String> {
    match mode {
        "standard" => Ok("standard".to_string()),
        "worktree" | "worktrees" => Ok("worktrees".to_string()),
        _ => Err(CliError::user(format!(
            "Invalid mode '{}'. Must be 'standard' or 'worktrees'.",
            mode
        ))),
    }
}

/// Sanitize a project name to a valid folder name
///
/// - Converts to lowercase
/// - Replaces spaces and underscores with hyphens
/// - Removes special characters
/// - Collapses multiple hyphens
pub fn sanitize_project_name(name: &str) -> String {
    let mut result = String::with_capacity(name.len());
    let mut last_was_hyphen = false;

    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            result.push(c.to_ascii_lowercase());
            last_was_hyphen = false;
        } else if (c == ' ' || c == '_' || c == '-') && !last_was_hyphen && !result.is_empty() {
            result.push('-');
            last_was_hyphen = true;
        }
        // Other characters are dropped
    }

    // Remove trailing hyphen
    if result.ends_with('-') {
        result.pop();
    }

    // Fallback if empty
    if result.is_empty() {
        result = "project".to_string();
    }

    result
}

/// Initialize a repository with the given configuration
///
/// This function:
/// - Creates the `.repository` directory
/// - Creates `config.toml` with the specified mode, tools, and presets
/// - Initializes git if `.git` doesn't exist
/// - For worktrees mode, creates the `main/` directory
pub fn init_repository(
    path: &Path,
    mode: &str,
    tools: &[String],
    presets: &[String],
    extensions: &[String],
) -> Result<()> {
    // Validate and normalize mode to canonical form
    let canonical_mode = normalize_mode(mode)?;

    // Create .repository directory
    let repo_dir = path.join(".repository");
    std::fs::create_dir_all(&repo_dir)?;

    // Generate and write config.toml
    let config_content = generate_config(&canonical_mode, tools, presets, extensions);
    let config_path = repo_dir.join("config.toml");
    std::fs::write(&config_path, config_content)?;

    // Initialize git if .git doesn't exist
    let git_dir = path.join(".git");
    if !git_dir.exists() {
        init_git(path)?;
    }

    // For worktree mode, create main/ directory
    if canonical_mode == "worktrees" {
        let main_dir = path.join("main");
        if !main_dir.exists() {
            std::fs::create_dir_all(&main_dir)?;
        }
    }

    Ok(())
}

/// Generate the config.toml content
///
/// Generates config in the Manifest format (top-level tools and presets arrays):
/// ```toml
/// tools = ["cursor", "claude"]
///
/// [core]
/// mode = "standard"
///
/// [presets."env:python"]
/// version = "3.12"
/// ```
/// Escape a string for safe inclusion in a TOML quoted value.
///
/// Prevents injection of newlines, quotes, or backslashes that could break
/// the TOML structure or inject arbitrary configuration sections.
fn escape_toml_value(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            c if c.is_control() => {} // Strip other control characters
            c => escaped.push(c),
        }
    }
    escaped
}

pub fn generate_config(
    mode: &str,
    tools: &[String],
    presets: &[String],
    extensions: &[String],
) -> String {
    use repo_extensions::ExtensionRegistry;

    let mut config = String::new();

    // tools array at top level (before [core] section)
    let tools_arr: Vec<String> = tools
        .iter()
        .map(|t| format!("\"{}\"", escape_toml_value(t)))
        .collect();
    config.push_str(&format!("tools = [{}]\n", tools_arr.join(", ")));

    // [core] section
    config.push('\n');
    config.push_str("[core]\n");
    config.push_str(&format!("mode = \"{}\"\n", escape_toml_value(mode)));

    // [presets] section with each preset as a table
    if !presets.is_empty() {
        for preset in presets {
            config.push('\n');
            config.push_str(&format!("[presets.\"{}\"]\n", escape_toml_value(preset)));
        }
    }

    // [extensions] section
    if !extensions.is_empty() {
        let registry = ExtensionRegistry::with_known();
        for ext in extensions {
            config.push('\n');
            let escaped_ext = escape_toml_value(ext);
            if let Some(entry) = registry.get(ext) {
                // Known extension: use registry source
                config.push_str(&format!("[extensions.\"{}\"]\n", escaped_ext));
                config.push_str(&format!(
                    "source = \"{}\"\n",
                    escape_toml_value(&entry.source)
                ));
                config.push_str("ref = \"main\"\n");
            } else {
                // Custom extension: treat the value as a source URL
                config.push_str(&format!("[extensions.\"{}\"]\n", escaped_ext));
                config.push_str(&format!("source = \"{}\"\n", escaped_ext));
                config.push_str("ref = \"main\"\n");
            }
        }
    }

    config
}

/// Initialize git in the given directory
fn init_git(path: &Path) -> Result<()> {
    let output = Command::new("git").arg("init").current_dir(path).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CliError::user(format!(
            "Failed to initialize git: {}",
            stderr
        )));
    }

    Ok(())
}

/// Add a git remote to the repository
fn add_git_remote(path: &Path, remote_url: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["remote", "add", "origin", remote_url])
        .current_dir(path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Ignore error if remote already exists
        if !stderr.contains("already exists") {
            return Err(CliError::user(format!("Failed to add remote: {}", stderr)));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_sanitize_project_name_basic() {
        assert_eq!(sanitize_project_name("my-project"), "my-project");
        assert_eq!(sanitize_project_name("MyProject"), "myproject");
        assert_eq!(sanitize_project_name("my_project"), "my-project");
        assert_eq!(sanitize_project_name("my project"), "my-project");
    }

    #[test]
    fn test_sanitize_project_name_special_chars() {
        assert_eq!(sanitize_project_name("My Project Name!"), "my-project-name");
        assert_eq!(sanitize_project_name("project@2024"), "project2024");
        assert_eq!(sanitize_project_name("hello--world"), "hello-world");
        assert_eq!(sanitize_project_name("  leading  "), "leading");
    }

    #[test]
    fn test_sanitize_project_name_edge_cases() {
        assert_eq!(sanitize_project_name("!!!"), "project");
        assert_eq!(sanitize_project_name(""), "project");
        assert_eq!(sanitize_project_name("a"), "a");
    }

    #[test]
    fn test_run_init_creates_project_folder() {
        let temp_dir = TempDir::new().unwrap();

        let config = InitConfig {
            name: "my-project".to_string(),
            mode: "standard".to_string(),
            tools: vec![],
            presets: vec![],
            extensions: vec![],
            remote: None,
        };

        let result = run_init(temp_dir.path(), config);
        assert!(result.is_ok());

        let project_path = result.unwrap();
        assert!(project_path.exists());
        assert!(project_path.join(".repository").exists());
        assert_eq!(project_path.file_name().unwrap(), "my-project");
    }

    #[test]
    fn test_run_init_sanitizes_name() {
        let temp_dir = TempDir::new().unwrap();

        let config = InitConfig {
            name: "My Project Name!".to_string(),
            mode: "standard".to_string(),
            tools: vec![],
            presets: vec![],
            extensions: vec![],
            remote: None,
        };

        let result = run_init(temp_dir.path(), config);
        assert!(result.is_ok());

        // Check sanitized folder exists
        let sanitized_path = temp_dir.path().join("my-project-name");
        assert!(sanitized_path.exists());
        assert!(sanitized_path.join(".repository").exists());
    }

    #[test]
    fn test_run_init_current_dir() {
        let temp_dir = TempDir::new().unwrap();

        let config = InitConfig {
            name: ".".to_string(),
            mode: "standard".to_string(),
            tools: vec![],
            presets: vec![],
            extensions: vec![],
            remote: None,
        };

        let result = run_init(temp_dir.path(), config);
        assert!(result.is_ok());

        // Should init in current directory
        let project_path = result.unwrap();
        assert_eq!(project_path, temp_dir.path());
        assert!(temp_dir.path().join(".repository").exists());
    }

    #[test]
    fn test_init_creates_repository_structure() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        let result = init_repository(path, "standard", &[], &[], &[]);
        assert!(result.is_ok());

        // Verify .repository directory exists
        let repo_dir = path.join(".repository");
        assert!(repo_dir.exists(), ".repository directory should exist");
        assert!(repo_dir.is_dir(), ".repository should be a directory");

        // Verify config.toml exists
        let config_path = repo_dir.join("config.toml");
        assert!(config_path.exists(), "config.toml should exist");

        // Verify config content
        let config_content = std::fs::read_to_string(&config_path).unwrap();
        assert!(config_content.contains("[core]"));
        assert!(config_content.contains("mode = \"standard\""));
    }

    #[test]
    fn test_init_with_tools() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        let tools = vec!["eslint".to_string(), "prettier".to_string()];
        let result = init_repository(path, "standard", &tools, &[], &[]);
        assert!(result.is_ok());

        // Verify tools in config using top-level array format
        let config_path = path.join(".repository").join("config.toml");
        let config_content = std::fs::read_to_string(&config_path).unwrap();
        assert!(config_content.contains("tools = ["));
        assert!(config_content.contains("\"eslint\""));
        assert!(config_content.contains("\"prettier\""));
    }

    #[test]
    fn test_init_with_presets() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        let presets = vec!["typescript".to_string(), "react".to_string()];
        let result = init_repository(path, "standard", &[], &presets, &[]);
        assert!(result.is_ok());

        // Verify presets in config using [presets.X] section format
        let config_path = path.join(".repository").join("config.toml");
        let config_content = std::fs::read_to_string(&config_path).unwrap();
        assert!(config_content.contains("[presets.\"typescript\"]"));
        assert!(config_content.contains("[presets.\"react\"]"));
    }

    #[test]
    fn test_generate_config() {
        // Test basic config (always has tools array)
        let config = generate_config("standard", &[], &[], &[]);
        assert!(config.contains("tools = []"));
        assert!(config.contains("[core]\nmode = \"standard\"\n"));

        // Test with tools
        let tools = vec!["eslint".to_string()];
        let config = generate_config("standard", &tools, &[], &[]);
        assert!(config.contains("[core]\nmode = \"standard\"\n"));
        assert!(config.contains("tools = [\"eslint\"]"));

        // Test with presets
        let presets = vec!["typescript".to_string()];
        let config = generate_config("standard", &[], &presets, &[]);
        assert!(config.contains("[core]\nmode = \"standard\"\n"));
        assert!(config.contains("[presets.\"typescript\"]"));

        // Test with both tools and presets
        let tools = vec!["eslint".to_string(), "prettier".to_string()];
        let presets = vec!["typescript".to_string()];
        let config = generate_config("worktrees", &tools, &presets, &[]);
        assert!(config.contains("[core]\nmode = \"worktrees\"\n"));
        assert!(config.contains("\"eslint\""));
        assert!(config.contains("\"prettier\""));
        assert!(config.contains("[presets.\"typescript\"]"));

        // Test with extensions (known)
        let extensions = vec!["vaultspec".to_string()];
        let config = generate_config("standard", &[], &[], &extensions);
        assert!(config.contains("[extensions.\"vaultspec\"]"));
        assert!(config.contains("source = \"https://github.com/vaultspec/vaultspec.git\""));
        assert!(config.contains("ref = \"main\""));

        // Test with custom extension URL
        let extensions = vec!["https://github.com/custom/ext.git".to_string()];
        let config = generate_config("standard", &[], &[], &extensions);
        assert!(config.contains("source = \"https://github.com/custom/ext.git\""));
        assert!(config.contains("ref = \"main\""));
    }

    #[test]
    fn test_init_worktree_mode_creates_main_directory() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        let result = init_repository(path, "worktree", &[], &[], &[]);
        assert!(result.is_ok());

        // Verify main/ directory exists for worktree mode
        let main_dir = path.join("main");
        assert!(
            main_dir.exists(),
            "main/ directory should exist for worktree mode"
        );
        assert!(main_dir.is_dir(), "main should be a directory");
    }

    #[test]
    fn test_init_standard_mode_does_not_create_main_directory() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        let result = init_repository(path, "standard", &[], &[], &[]);
        assert!(result.is_ok());

        // Verify main/ directory does NOT exist for standard mode
        let main_dir = path.join("main");
        assert!(
            !main_dir.exists(),
            "main/ directory should NOT exist for standard mode"
        );
    }

    #[test]
    fn test_init_invalid_mode() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        let result = init_repository(path, "invalid", &[], &[], &[]);
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_msg = format!("{}", err);
        assert!(err_msg.contains("Invalid mode"));
    }

    #[test]
    fn test_init_initializes_git() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Ensure .git doesn't exist
        assert!(!path.join(".git").exists());

        let result = init_repository(path, "standard", &[], &[], &[]);
        assert!(result.is_ok());

        // Verify .git was created
        let git_dir = path.join(".git");
        assert!(git_dir.exists(), ".git directory should exist after init");
    }

    #[test]
    fn test_init_does_not_reinitialize_git() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Pre-create .git directory to simulate existing repo
        std::fs::create_dir(path.join(".git")).unwrap();
        std::fs::write(path.join(".git").join("marker"), "test").unwrap();

        let result = init_repository(path, "standard", &[], &[], &[]);
        assert!(result.is_ok());

        // Verify marker file still exists (git was not reinitialized)
        let marker = path.join(".git").join("marker");
        assert!(
            marker.exists(),
            "marker file should still exist - git should not be reinitialized"
        );
    }

    #[test]
    fn test_run_init_with_tools_returns_correct_path() {
        let temp_dir = TempDir::new().unwrap();

        let config = InitConfig {
            name: "tooled-project".to_string(),
            mode: "standard".to_string(),
            tools: vec!["cursor".to_string(), "claude".to_string()],
            presets: vec![],
            extensions: vec![],
            remote: None,
        };

        let result = run_init(temp_dir.path(), config);
        assert!(result.is_ok());

        let project_path = result.unwrap();
        assert_eq!(project_path.file_name().unwrap(), "tooled-project");
        assert!(project_path.join(".repository").exists());
        assert!(project_path.join(".repository/config.toml").exists());
    }

    #[test]
    fn test_run_init_without_tools_returns_correct_path() {
        let temp_dir = TempDir::new().unwrap();

        let config = InitConfig {
            name: "empty-project".to_string(),
            mode: "standard".to_string(),
            tools: vec![],
            presets: vec![],
            extensions: vec![],
            remote: None,
        };

        let result = run_init(temp_dir.path(), config);
        assert!(result.is_ok());

        let project_path = result.unwrap();
        assert_eq!(project_path.file_name().unwrap(), "empty-project");
        assert!(project_path.join(".repository").exists());
    }
}
