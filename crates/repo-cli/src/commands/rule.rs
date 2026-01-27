//! Rule management command implementations
//!
//! Provides add/remove/list operations for repository rules stored in .repository/rules/.

use std::fs;
use std::path::Path;

use colored::Colorize;

use crate::error::{CliError, Result};

/// Validate a rule ID to prevent path traversal and invalid filenames
fn validate_rule_id(id: &str) -> Result<()> {
    if id.is_empty() {
        return Err(CliError::user("Rule ID cannot be empty"));
    }
    if id.len() > 64 {
        return Err(CliError::user("Rule ID cannot exceed 64 characters"));
    }
    if id.contains('/') || id.contains('\\') || id.contains("..") {
        return Err(CliError::user("Rule ID cannot contain path separators or '..'"));
    }
    // Only allow alphanumeric, hyphens, and underscores
    if !id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(CliError::user("Rule ID can only contain alphanumeric characters, hyphens, and underscores"));
    }
    Ok(())
}

/// Path to rules directory within a repository
const RULES_DIR: &str = ".repository/rules";

/// Run the add-rule command
///
/// Adds a rule to the repository's rules directory as a markdown file.
pub fn run_add_rule(path: &Path, id: &str, instruction: &str, tags: Vec<String>) -> Result<()> {
    // Validate rule ID to prevent path traversal
    validate_rule_id(id)?;

    println!("{} Adding rule: {}", "=>".blue().bold(), id.cyan());

    let rules_dir = path.join(RULES_DIR);
    fs::create_dir_all(&rules_dir)?;

    let rule_path = rules_dir.join(format!("{}.md", id));

    // Generate rule content
    let mut content = String::new();
    if !tags.is_empty() {
        content.push_str(&format!("tags: {}\n\n", tags.join(", ")));
    }
    content.push_str(instruction);

    fs::write(&rule_path, &content)?;

    println!("{} Rule '{}' added.", "OK".green().bold(), id);
    Ok(())
}

/// Run the remove-rule command
///
/// Removes a rule from the repository's rules directory.
pub fn run_remove_rule(path: &Path, id: &str) -> Result<()> {
    // Validate rule ID to prevent path traversal
    validate_rule_id(id)?;

    println!("{} Removing rule: {}", "=>".blue().bold(), id.cyan());

    let rule_path = path.join(RULES_DIR).join(format!("{}.md", id));

    if !rule_path.exists() {
        println!("{} Rule '{}' not found.", "WARN".yellow().bold(), id);
        return Ok(());
    }

    fs::remove_file(&rule_path)?;
    println!("{} Rule '{}' removed.", "OK".green().bold(), id);
    Ok(())
}

/// Run the list-rules command
///
/// Lists all active rules in the repository's rules directory.
pub fn run_list_rules(path: &Path) -> Result<()> {
    let rules_dir = path.join(RULES_DIR);

    if !rules_dir.exists() {
        println!("No rules defined.");
        return Ok(());
    }

    println!("{} Active rules:", "=>".blue().bold());

    let mut found = false;
    for entry in fs::read_dir(&rules_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "md") {
            let id = path.file_stem().unwrap().to_string_lossy();
            println!("   {} {}", "-".cyan(), id);
            found = true;
        }
    }

    if !found {
        println!("   (none)");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Helper to create a test repository structure
    fn create_test_repo(dir: &Path) {
        let repo_dir = dir.join(".repository");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(
            repo_dir.join("config.toml"),
            "[core]\nmode = \"standard\"\n",
        )
        .unwrap();
    }

    #[test]
    fn test_add_rule() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        create_test_repo(path);

        let result = run_add_rule(path, "python-style", "Use snake_case for variables.", vec![]);
        assert!(result.is_ok());

        // Verify rule file was created
        let rule_path = path.join(".repository/rules/python-style.md");
        assert!(rule_path.exists());

        let content = fs::read_to_string(&rule_path).unwrap();
        assert!(content.contains("Use snake_case for variables."));
    }

    #[test]
    fn test_add_rule_with_tags() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        create_test_repo(path);

        let result = run_add_rule(
            path,
            "naming-conventions",
            "Follow consistent naming.",
            vec!["style".to_string(), "python".to_string()],
        );
        assert!(result.is_ok());

        // Verify rule file was created with tags
        let rule_path = path.join(".repository/rules/naming-conventions.md");
        assert!(rule_path.exists());

        let content = fs::read_to_string(&rule_path).unwrap();
        assert!(content.contains("tags: style, python"));
        assert!(content.contains("Follow consistent naming."));
    }

    #[test]
    fn test_remove_rule() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        create_test_repo(path);

        // First add a rule
        run_add_rule(path, "test-rule", "Test instruction.", vec![]).unwrap();

        // Verify it exists
        let rule_path = path.join(".repository/rules/test-rule.md");
        assert!(rule_path.exists());

        // Remove the rule
        let result = run_remove_rule(path, "test-rule");
        assert!(result.is_ok());

        // Verify it was removed
        assert!(!rule_path.exists());
    }

    #[test]
    fn test_remove_nonexistent_rule() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        create_test_repo(path);

        // Remove a rule that doesn't exist - should succeed with warning
        let result = run_remove_rule(path, "nonexistent");
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_rules_empty() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        create_test_repo(path);

        // List rules when none exist
        let result = run_list_rules(path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_rules_with_rules() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        create_test_repo(path);

        // Add some rules
        run_add_rule(path, "rule-one", "First rule.", vec![]).unwrap();
        run_add_rule(path, "rule-two", "Second rule.", vec![]).unwrap();

        // List rules
        let result = run_list_rules(path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_rule_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        // Don't create the repository structure

        // Add a rule - should create the rules directory
        let result = run_add_rule(path, "new-rule", "A new rule.", vec![]);
        assert!(result.is_ok());

        // Verify directory and file were created
        let rules_dir = path.join(".repository/rules");
        assert!(rules_dir.exists());
        assert!(rules_dir.join("new-rule.md").exists());
    }

    #[test]
    fn test_add_rule_overwrites_existing() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        create_test_repo(path);

        // Add a rule
        run_add_rule(path, "my-rule", "Original content.", vec![]).unwrap();

        // Overwrite the rule
        let result = run_add_rule(path, "my-rule", "Updated content.", vec![]);
        assert!(result.is_ok());

        // Verify content was overwritten
        let rule_path = path.join(".repository/rules/my-rule.md");
        let content = fs::read_to_string(&rule_path).unwrap();
        assert!(content.contains("Updated content."));
        assert!(!content.contains("Original content."));
    }

    #[test]
    fn test_rule_id_validation_empty() {
        let temp_dir = TempDir::new().unwrap();
        let result = run_add_rule(temp_dir.path(), "", "content", vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_rule_id_validation_path_traversal() {
        let temp_dir = TempDir::new().unwrap();
        let result = run_add_rule(temp_dir.path(), "../../../etc/passwd", "content", vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_rule_id_validation_special_chars() {
        let temp_dir = TempDir::new().unwrap();
        let result = run_add_rule(temp_dir.path(), "rule with spaces", "content", vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_rule_id_validation_valid() {
        // Valid IDs should work
        assert!(validate_rule_id("valid-rule").is_ok());
        assert!(validate_rule_id("valid_rule").is_ok());
        assert!(validate_rule_id("ValidRule123").is_ok());
    }
}
