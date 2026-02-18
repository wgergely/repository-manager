use repo_git::{NamingStrategy, naming::branch_to_directory};

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
    // Should not panic, and result should be the full slug
    assert_eq!(result.len(), 300);
}

#[test]
fn test_hierarchical_special_characters() {
    let result = branch_to_directory("feat:bug#123", NamingStrategy::Hierarchical);
    // Special chars should be replaced with dashes
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
    // Emoji characters are not alphanumeric in Rust, so they should be stripped/replaced
    let result = branch_to_directory("fix/bug-🐛", NamingStrategy::Slug);
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
