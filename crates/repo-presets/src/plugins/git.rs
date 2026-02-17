//! Git operations for plugin installation

use crate::error::{Error, Result};
use std::path::Path;

/// Clone a git repository to a destination directory.
///
/// # Arguments
/// * `url` - Git repository URL
/// * `dest` - Destination directory
/// * `tag` - Optional tag/branch to checkout
pub fn clone_repo(url: &str, dest: &Path, tag: Option<&str>) -> Result<()> {
    use git2::build::RepoBuilder;

    // Create parent directories
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| Error::GitClone {
            url: url.to_string(),
            message: format!("Failed to create directory: {}", e),
        })?;
    }

    // Clone the repository
    let mut builder = RepoBuilder::new();

    if let Some(tag_name) = tag {
        // For tags, we clone then checkout
        let repo = builder.clone(url, dest).map_err(|e| Error::GitClone {
            url: url.to_string(),
            message: e.message().to_string(),
        })?;

        // Checkout the specific tag
        let (object, reference) = repo.revparse_ext(tag_name).map_err(|e| Error::GitClone {
            url: url.to_string(),
            message: format!("Tag {} not found: {}", tag_name, e),
        })?;

        repo.checkout_tree(&object, None)
            .map_err(|e| Error::GitClone {
                url: url.to_string(),
                message: format!("Failed to checkout {}: {}", tag_name, e),
            })?;

        // Set HEAD to the tag
        if let Some(ref_name) = reference {
            repo.set_head(ref_name.name().unwrap_or(tag_name))
        } else {
            repo.set_head_detached(object.id())
        }
        .map_err(|e| Error::GitClone {
            url: url.to_string(),
            message: format!("Failed to set HEAD: {}", e),
        })?;
    } else {
        builder.clone(url, dest).map_err(|e| Error::GitClone {
            url: url.to_string(),
            message: e.message().to_string(),
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_clone_requires_valid_url() {
        let temp = TempDir::new().unwrap();
        let result = clone_repo("not-a-valid-url", temp.path(), None);
        assert!(result.is_err());
    }

    // Note: Integration test with real git clone should be in integration tests
    // to avoid network dependency in unit tests
}
