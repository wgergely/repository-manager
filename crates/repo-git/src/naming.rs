//! Branch name to directory name mapping strategies

/// Strategy for converting branch names to directory names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NamingStrategy {
    /// Convert slashes to dashes, remove unsafe characters.
    /// `feat/user-auth` -> `feat-user-auth`
    #[default]
    Slug,

    /// Preserve slashes as directory hierarchy.
    /// `feat/user-auth` -> `feat/user-auth`
    Hierarchical,
}

/// Convert a branch name to a directory name using the given strategy.
pub fn branch_to_directory(branch: &str, strategy: NamingStrategy) -> String {
    match strategy {
        NamingStrategy::Slug => slugify(branch),
        NamingStrategy::Hierarchical => sanitize_hierarchical(branch),
    }
}

/// Convert branch name to a flat slug.
fn slugify(branch: &str) -> String {
    let mut result = String::with_capacity(branch.len());
    let mut last_was_dash = true; // Start true to skip leading dashes

    for c in branch.chars() {
        if c.is_alphanumeric() || c == '-' || c == '_' {
            if c == '-' || c == '_' {
                if !last_was_dash {
                    result.push('-');
                    last_was_dash = true;
                }
            } else {
                result.push(c);
                last_was_dash = false;
            }
        } else {
            // Replace other characters (including /) with dash
            if !last_was_dash {
                result.push('-');
                last_was_dash = true;
            }
        }
    }

    // Remove trailing dash
    if result.ends_with('-') {
        result.pop();
    }

    result
}

/// Sanitize for hierarchical naming, keeping slashes but removing unsafe chars.
fn sanitize_hierarchical(branch: &str) -> String {
    let mut result = String::with_capacity(branch.len());

    for c in branch.chars() {
        if c.is_alphanumeric() || c == '-' || c == '_' || c == '/' {
            result.push(c);
        } else {
            result.push('-');
        }
    }

    // Clean up multiple slashes and leading/trailing slashes
    let result = result.trim_matches('/');
    let mut cleaned = String::with_capacity(result.len());
    let mut last_was_slash = false;

    for c in result.chars() {
        if c == '/' {
            if !last_was_slash {
                cleaned.push(c);
                last_was_slash = true;
            }
        } else {
            cleaned.push(c);
            last_was_slash = false;
        }
    }

    cleaned
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify_basic() {
        assert_eq!(slugify("hello-world"), "hello-world");
    }

    #[test]
    fn test_slugify_with_slashes() {
        assert_eq!(slugify("feat/auth"), "feat-auth");
    }

    #[test]
    fn test_slug_empty_string() {
        let result = branch_to_directory("", NamingStrategy::Slug);
        assert_eq!(
            result, "",
            "Empty branch name should produce empty directory name"
        );
    }

    #[test]
    fn test_hierarchical_empty_string() {
        let result = branch_to_directory("", NamingStrategy::Hierarchical);
        assert_eq!(
            result, "",
            "Empty branch name should produce empty directory name"
        );
    }

    #[test]
    fn test_slug_very_long_branch_name() {
        let long_name = "a".repeat(300);
        let result = branch_to_directory(&long_name, NamingStrategy::Slug);
        assert_eq!(result.len(), 300);
    }

    #[test]
    fn test_hierarchical_special_characters() {
        let result = branch_to_directory("feat:bug#123", NamingStrategy::Hierarchical);
        assert_eq!(result, "feat-bug-123");
    }

    #[test]
    fn test_hierarchical_leading_trailing_slashes() {
        let result = branch_to_directory("/feat/auth/", NamingStrategy::Hierarchical);
        assert_eq!(
            result, "feat/auth",
            "Should strip leading and trailing slashes"
        );
    }

    #[test]
    fn test_hierarchical_multiple_consecutive_slashes() {
        let result = branch_to_directory("feat//double//slash", NamingStrategy::Hierarchical);
        assert_eq!(
            result, "feat/double/slash",
            "Should collapse consecutive slashes"
        );
    }

    #[test]
    fn test_slug_emoji_stripped() {
        let result = branch_to_directory("fix/bug-\u{1f41b}", NamingStrategy::Slug);
        assert_eq!(result, "fix-bug", "Emoji should be stripped from slug");
    }

    #[test]
    fn test_slug_simple_branch() {
        let result = branch_to_directory("feature-auth", NamingStrategy::Slug);
        assert_eq!(result, "feature-auth");
    }

    #[test]
    fn test_slug_branch_with_slash() {
        let result = branch_to_directory("feat/user-auth", NamingStrategy::Slug);
        assert_eq!(result, "feat-user-auth");
    }

    #[test]
    fn test_slug_multiple_slashes() {
        let result = branch_to_directory("feat/user/auth/login", NamingStrategy::Slug);
        assert_eq!(result, "feat-user-auth-login");
    }

    #[test]
    fn test_slug_special_characters() {
        let result = branch_to_directory("fix:bug#123", NamingStrategy::Slug);
        assert_eq!(result, "fix-bug-123");
    }

    #[test]
    fn test_hierarchical_simple_branch() {
        let result = branch_to_directory("feature-auth", NamingStrategy::Hierarchical);
        assert_eq!(result, "feature-auth");
    }

    #[test]
    fn test_hierarchical_branch_with_slash() {
        let result = branch_to_directory("feat/user-auth", NamingStrategy::Hierarchical);
        assert_eq!(result, "feat/user-auth");
    }

    #[test]
    fn test_slug_removes_leading_trailing_dashes() {
        let result = branch_to_directory("/feat/", NamingStrategy::Slug);
        assert_eq!(result, "feat");
    }

    #[test]
    fn test_slug_collapses_multiple_dashes() {
        let result = branch_to_directory("feat//double//slash", NamingStrategy::Slug);
        assert_eq!(result, "feat-double-slash");
    }
}
