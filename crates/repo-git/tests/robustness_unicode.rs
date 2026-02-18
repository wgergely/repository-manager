use git2::{Repository, Signature};
use repo_fs::NormalizedPath;
use repo_git::{ContainerLayout, LayoutProvider, NamingStrategy};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_unicode_and_emoji_support() {
    // Setup
    let dir = tempdir().unwrap();
    let root = NormalizedPath::new(dir.path());
    let git_dir = root.join(".gt");

    // Initialize bare repo
    let repo = Repository::init_bare(git_dir.to_native()).unwrap();
    let signature = Signature::now("Test User", "test@example.com").unwrap();

    // Initial commit
    let tree_id = {
        let mut index = repo.index().unwrap();
        index.write_tree().unwrap()
    };
    let tree = repo.find_tree(tree_id).unwrap();
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Initial commit",
        &tree,
        &[],
    )
    .unwrap();

    let layout = ContainerLayout::new(root.clone(), NamingStrategy::Slug).unwrap();

    // Create 'main' worktree manually
    let main_path = root.join("main");
    fs::create_dir_all(main_path.to_native()).unwrap();

    // 1. Japanese Feature Branch
    // "feature/user-auth" in Japanese: "feature/ユーザー認証"
    let jp_name = "feature/ユーザー認証";
    let jp_path = layout
        .create_feature(jp_name, None)
        .expect("Should create japanese branch");

    assert!(jp_path.exists(), "Japanese path should exist on disk");
    // Verify naming strategy (Slug) converts slashes but keeps unicode if safe?
    // NamingStrategy::Slug implementation:
    // slugify("feature/ユーザー認証") -> "feature-ユーザー認証" likely (alphanumeric includes unicode letters)
    // Let's verify expectations dynamically

    // 2. Emoji Feature Branch
    // "fix/bug-🐛"
    let emoji_name = "fix/bug-🐛";
    let emoji_path = layout
        .create_feature(emoji_name, None)
        .expect("Should create emoji branch");

    assert!(emoji_path.exists(), "Emoji path should exist on disk");

    // 3. List and Verify
    let worktrees = layout.list_worktrees().expect("Should list worktrees");

    // NamingStrategy::Slug converts "feature/..." to "feature-..."
    // We expect the Unicode characters to be preserved.
    let expected_jp_slug = "feature-ユーザー認証";
    let found_jp = worktrees
        .iter()
        .find(|wt| wt.name == expected_jp_slug || wt.branch == expected_jp_slug);
    assert!(
        found_jp.is_some(),
        "Should find Japanese worktree (as slug: {})",
        expected_jp_slug
    );

    let expected_emoji_slug = "fix-bug"; // Emoji is not alphanumeric, so it gets stripped/sanitized
    let found_emoji = worktrees
        .iter()
        .find(|wt| wt.name == expected_emoji_slug || wt.branch == expected_emoji_slug);
    assert!(
        found_emoji.is_some(),
        "Should find Emoji worktree (sanitized as: {})",
        expected_emoji_slug
    );

    // 4. Remove
    layout
        .remove_feature(jp_name)
        .expect("Should remove jp branch");
    assert!(!jp_path.exists(), "Path should be gone");

    layout
        .remove_feature(emoji_name)
        .expect("Should remove emoji branch");
    assert!(!emoji_path.exists(), "Path should be gone");
}
