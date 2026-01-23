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
}
