use repo_git::{NamingStrategy, naming::branch_to_directory};

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
