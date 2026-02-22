//! Tests for git primitive handlers and initialization validation.
//!
//! These tests verify:
//! - `initialize()` rejects directories without `.repository/`
//! - `initialize()` rejects directories with `.repository/` but no `config.toml`
//! - `initialize()` succeeds with valid `.repository/config.toml`
//! - `git_push`, `git_pull`, `git_merge` handlers are implemented (not NotImplemented)
//! - Git handlers parse arguments correctly and return structured results
//! - Git handlers return meaningful errors for invalid operations

use repo_mcp::{RepoMcpServer, handle_tool_call};
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a temp dir with valid .repository/config.toml for server initialization.
fn create_valid_repo_structure(temp: &TempDir) {
    fs::create_dir_all(temp.path().join(".repository")).unwrap();
    fs::write(
        temp.path().join(".repository/config.toml"),
        "tools = []\n\n[core]\nmode = \"standard\"\n",
    )
    .unwrap();
}

/// Create a temp dir with valid .repository/config.toml AND .git for tool operations.
fn create_full_test_repo(temp: &TempDir) {
    create_valid_repo_structure(temp);
    // Initialize a real git repo for git operations
    git2::Repository::init(temp.path()).unwrap();
}

/// Create a git repo with an initial commit so HEAD points to a branch.
fn create_git_repo_with_commit(temp: &TempDir) -> git2::Repository {
    create_valid_repo_structure(temp);
    let repo = git2::Repository::init(temp.path()).unwrap();
    {
        let sig = repo.signature().unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .unwrap();
    }
    repo
}

// ==========================================================================
// Initialize Validation Tests
// ==========================================================================

#[tokio::test]
async fn test_mcp_initialize_rejects_non_repository() {
    // Temp dir without .repository/ directory
    let temp = TempDir::new().unwrap();
    let mut server = RepoMcpServer::new(PathBuf::from(temp.path()));

    let result = server.initialize().await;

    assert!(
        result.is_err(),
        "initialize() must fail when .repository/ does not exist"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains(".repository"),
        "Error message should mention .repository, got: {}",
        err_msg
    );
    assert!(
        !server.is_initialized(),
        "Server must not be marked as initialized after failure"
    );
}

#[tokio::test]
async fn test_mcp_initialize_rejects_missing_config() {
    // Temp dir with .repository/ but no config.toml
    let temp = TempDir::new().unwrap();
    fs::create_dir_all(temp.path().join(".repository")).unwrap();

    let mut server = RepoMcpServer::new(PathBuf::from(temp.path()));

    let result = server.initialize().await;

    assert!(
        result.is_err(),
        "initialize() must fail when config.toml is missing"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("config.toml"),
        "Error message should mention config.toml, got: {}",
        err_msg
    );
    assert!(
        !server.is_initialized(),
        "Server must not be marked as initialized after failure"
    );
}

#[tokio::test]
async fn test_mcp_initialize_succeeds_valid_repo() {
    let temp = TempDir::new().unwrap();
    create_valid_repo_structure(&temp);

    let mut server = RepoMcpServer::new(PathBuf::from(temp.path()));

    let result = server.initialize().await;

    assert!(
        result.is_ok(),
        "initialize() should succeed with valid .repository/config.toml, got: {:?}",
        result.err()
    );
    assert!(
        server.is_initialized(),
        "Server should be marked as initialized after success"
    );
    assert!(
        !server.tools().is_empty(),
        "Tools should be loaded after initialization"
    );
    assert!(
        !server.resources().is_empty(),
        "Resources should be loaded after initialization"
    );
}

// ==========================================================================
// Git Handler Existence Tests (no longer NotImplemented)
// ==========================================================================

#[tokio::test]
async fn test_mcp_git_push_handler_exists() {
    let temp = TempDir::new().unwrap();
    create_full_test_repo(&temp);

    let result = handle_tool_call(temp.path(), "git_push", json!({})).await;

    // The handler should NOT return NotImplemented.
    // It may return a different error (e.g., no remote configured), but
    // NotImplemented means the handler is a stub.
    match &result {
        Err(e) => {
            let err_str = format!("{}", e);
            assert!(
                !err_str.contains("not implemented"),
                "git_push should be implemented, but got NotImplemented error: {}",
                err_str
            );
        }
        Ok(val) => {
            // If it somehow succeeds (unlikely without a remote), that's fine
            assert!(
                val.get("success").is_some(),
                "Success response should have 'success' field"
            );
        }
    }
}

#[tokio::test]
async fn test_mcp_git_pull_handler_exists() {
    let temp = TempDir::new().unwrap();
    create_full_test_repo(&temp);

    let result = handle_tool_call(temp.path(), "git_pull", json!({})).await;

    match &result {
        Err(e) => {
            let err_str = format!("{}", e);
            assert!(
                !err_str.contains("not implemented"),
                "git_pull should be implemented, but got NotImplemented error: {}",
                err_str
            );
        }
        Ok(val) => {
            assert!(
                val.get("success").is_some(),
                "Success response should have 'success' field"
            );
        }
    }
}

#[tokio::test]
async fn test_mcp_git_merge_handler_exists() {
    let temp = TempDir::new().unwrap();
    create_full_test_repo(&temp);

    let result = handle_tool_call(
        temp.path(),
        "git_merge",
        json!({"source": "nonexistent-branch"}),
    )
    .await;

    match &result {
        Err(e) => {
            let err_str = format!("{}", e);
            assert!(
                !err_str.contains("not implemented"),
                "git_merge should be implemented, but got NotImplemented error: {}",
                err_str
            );
        }
        Ok(val) => {
            assert!(
                val.get("success").is_some(),
                "Success response should have 'success' field"
            );
        }
    }
}

// ==========================================================================
// Git Handler Structured Result Tests
// ==========================================================================

#[tokio::test]
async fn test_mcp_git_push_returns_structured_error_on_no_remote() {
    let temp = TempDir::new().unwrap();
    create_git_repo_with_commit(&temp);

    // Push with no remote configured -- should return a meaningful error
    let result = handle_tool_call(temp.path(), "git_push", json!({})).await;

    assert!(
        result.is_err(),
        "git_push should fail when no remote is configured"
    );
    let err_str = format!("{}", result.unwrap_err());
    // Should mention "remote" in the error
    assert!(
        err_str.to_lowercase().contains("remote"),
        "Error should mention remote, got: {}",
        err_str
    );
}

#[tokio::test]
async fn test_mcp_git_push_parses_arguments() {
    let temp = TempDir::new().unwrap();
    create_git_repo_with_commit(&temp);

    // Provide explicit remote and branch arguments
    let result = handle_tool_call(
        temp.path(),
        "git_push",
        json!({"remote": "upstream", "branch": "feature-x"}),
    )
    .await;

    // Should fail because "upstream" remote doesn't exist, but the error
    // should mention "upstream" proving the argument was parsed
    assert!(result.is_err());
    let err_str = format!("{}", result.unwrap_err());
    assert!(
        err_str.contains("upstream"),
        "Error should reference the provided remote name 'upstream', got: {}",
        err_str
    );
}

#[tokio::test]
async fn test_mcp_git_pull_returns_structured_error_on_no_remote() {
    let temp = TempDir::new().unwrap();
    create_git_repo_with_commit(&temp);

    let result = handle_tool_call(temp.path(), "git_pull", json!({})).await;

    assert!(
        result.is_err(),
        "git_pull should fail when no remote is configured"
    );
    let err_str = format!("{}", result.unwrap_err());
    assert!(
        err_str.to_lowercase().contains("remote"),
        "Error should mention remote, got: {}",
        err_str
    );
}

#[tokio::test]
async fn test_mcp_git_merge_returns_error_on_missing_branch() {
    let temp = TempDir::new().unwrap();
    create_git_repo_with_commit(&temp);

    let result = handle_tool_call(
        temp.path(),
        "git_merge",
        json!({"source": "nonexistent-branch"}),
    )
    .await;

    assert!(
        result.is_err(),
        "git_merge should fail when source branch doesn't exist"
    );
    let err_str = format!("{}", result.unwrap_err());
    assert!(
        err_str.contains("nonexistent-branch"),
        "Error should mention the missing branch name, got: {}",
        err_str
    );
}

#[tokio::test]
async fn test_mcp_git_merge_requires_source_argument() {
    let temp = TempDir::new().unwrap();
    create_full_test_repo(&temp);

    // Call git_merge without the required "source" argument
    let result = handle_tool_call(temp.path(), "git_merge", json!({})).await;

    assert!(
        result.is_err(),
        "git_merge should fail when 'source' argument is missing"
    );
    let err_str = format!("{}", result.unwrap_err());
    assert!(
        err_str.to_lowercase().contains("source")
            || err_str.to_lowercase().contains("argument")
            || err_str.to_lowercase().contains("missing"),
        "Error should mention missing source argument, got: {}",
        err_str
    );
}

// ==========================================================================
// End-to-End MCP Protocol Tests for Git Handlers
// ==========================================================================

#[tokio::test]
async fn test_mcp_git_push_via_protocol_returns_is_error_not_not_implemented() {
    let temp = TempDir::new().unwrap();
    // Use a repo with a commit so current_branch resolution succeeds
    create_git_repo_with_commit(&temp);

    let mut server = RepoMcpServer::new(PathBuf::from(temp.path()));
    server.initialize().await.unwrap();

    let request = serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "git_push",
            "arguments": {}
        }
    }))
    .unwrap();

    let response: Value =
        serde_json::from_str(&server.handle_message(&request).await.unwrap()).unwrap();

    // The response should be a successful JSON-RPC response with tool error content
    let result = &response["result"];
    assert_eq!(
        result["is_error"], true,
        "git_push without remote should return is_error=true"
    );
    let text = result["content"][0]["text"].as_str().unwrap();
    // The error text should NOT contain "not implemented"
    assert!(
        !text.contains("not implemented"),
        "git_push error text should not say 'not implemented', got: {}",
        text
    );
    // It should contain a git-related error (remote not found, etc.)
    assert!(
        text.to_lowercase().contains("remote") || text.to_lowercase().contains("git"),
        "git_push error should mention a git-related issue, got: {}",
        text
    );
}

#[tokio::test]
async fn test_mcp_git_pull_via_protocol_returns_is_error_not_not_implemented() {
    let temp = TempDir::new().unwrap();
    create_git_repo_with_commit(&temp);

    let mut server = RepoMcpServer::new(PathBuf::from(temp.path()));
    server.initialize().await.unwrap();

    let request = serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "git_pull",
            "arguments": {}
        }
    }))
    .unwrap();

    let response: Value =
        serde_json::from_str(&server.handle_message(&request).await.unwrap()).unwrap();

    let result = &response["result"];
    assert_eq!(
        result["is_error"], true,
        "git_pull without remote should return is_error=true"
    );
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(
        !text.contains("not implemented"),
        "git_pull error text should not say 'not implemented', got: {}",
        text
    );
}

#[tokio::test]
async fn test_mcp_git_merge_via_protocol_returns_is_error_not_not_implemented() {
    let temp = TempDir::new().unwrap();
    create_git_repo_with_commit(&temp);

    let mut server = RepoMcpServer::new(PathBuf::from(temp.path()));
    server.initialize().await.unwrap();

    let request = serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "git_merge",
            "arguments": { "source": "nonexistent-branch" }
        }
    }))
    .unwrap();

    let response: Value =
        serde_json::from_str(&server.handle_message(&request).await.unwrap()).unwrap();

    let result = &response["result"];
    assert_eq!(
        result["is_error"], true,
        "git_merge with nonexistent branch should return is_error=true"
    );
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(
        !text.contains("not implemented"),
        "git_merge error text should not say 'not implemented', got: {}",
        text
    );
    assert!(
        text.contains("nonexistent-branch"),
        "git_merge error should mention the missing branch, got: {}",
        text
    );
}

// ==========================================================================
// Git Merge End-to-End Success Test
// ==========================================================================

#[tokio::test]
async fn test_mcp_git_merge_succeeds_with_valid_branch() {
    let temp = TempDir::new().unwrap();
    let repo = create_git_repo_with_commit(&temp);

    // Create a feature branch with a new commit
    let head_commit = repo.head().unwrap().peel_to_commit().unwrap();
    repo.branch("feature", &head_commit, false).unwrap();

    // Checkout feature branch and add a commit
    let feature_ref = repo
        .find_branch("feature", git2::BranchType::Local)
        .unwrap();
    repo.set_head(feature_ref.get().name().unwrap()).unwrap();
    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
        .unwrap();

    // Create a file and commit on feature branch
    fs::write(temp.path().join("feature-file.txt"), "feature content").unwrap();
    let mut index = repo.index().unwrap();
    index
        .add_path(std::path::Path::new("feature-file.txt"))
        .unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = repo.signature().unwrap();
    let parent = repo.head().unwrap().peel_to_commit().unwrap();
    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "Feature commit",
        &tree,
        &[&parent],
    )
    .unwrap();

    // Switch back to the default branch (main or master)
    let default_branch = if repo
        .find_branch("main", git2::BranchType::Local)
        .is_ok()
    {
        "main"
    } else {
        "master"
    };

    let default_ref = format!("refs/heads/{}", default_branch);
    repo.set_head(&default_ref).unwrap();
    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
        .unwrap();

    // Now merge feature into main/master
    let result = handle_tool_call(
        temp.path(),
        "git_merge",
        json!({"source": "feature"}),
    )
    .await;

    assert!(
        result.is_ok(),
        "git_merge should succeed when merging a valid branch, got: {:?}",
        result.err()
    );
    let value = result.unwrap();
    assert_eq!(
        value["success"], true,
        "Merge result should report success=true"
    );
    assert_eq!(
        value["source"], "feature",
        "Merge result should report the source branch"
    );

    // Verify the merge actually happened - the feature file should be in the working tree
    assert!(
        temp.path().join("feature-file.txt").exists(),
        "feature-file.txt should exist after merge"
    );
}

// ==========================================================================
// Initialize Rejection via Protocol
// ==========================================================================

#[tokio::test]
async fn test_mcp_initialize_non_repo_via_run_fails() {
    // Verify that calling run() on a non-repository fails during initialization
    let temp = TempDir::new().unwrap();
    let mut server = RepoMcpServer::new(PathBuf::from(temp.path()));

    let result = server.initialize().await;
    assert!(
        result.is_err(),
        "Server should fail to initialize outside a repository"
    );
    assert!(
        !server.is_initialized(),
        "Server must not be initialized after failure"
    );
    assert!(
        server.tools().is_empty(),
        "No tools should be loaded when initialization fails"
    );
}
