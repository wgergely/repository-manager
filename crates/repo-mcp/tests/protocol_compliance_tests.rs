//! MCP Protocol Compliance Integration Tests
//!
//! Tests that the MCP server correctly implements JSON-RPC 2.0 and
//! MCP protocol requirements, including ID preservation, error codes,
//! required field validation, and end-to-end tool execution.

use repo_mcp::RepoMcpServer;
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a temp dir with a valid repository structure needed for server initialization.
fn setup_repo_structure(temp: &TempDir) {
    fs::create_dir_all(temp.path().join(".repository")).unwrap();
    fs::write(
        temp.path().join(".repository/config.toml"),
        "tools = []\n\n[core]\nmode = \"standard\"\n",
    )
    .unwrap();
}

/// Create an initialized server with a temp directory as root.
async fn setup_server(temp: &TempDir) -> RepoMcpServer {
    setup_repo_structure(temp);
    let mut server = RepoMcpServer::new(PathBuf::from(temp.path()));
    server.initialize().await.unwrap();
    server
}

/// Create a temp dir with a valid repository structure for tool tests.
fn create_test_repo(temp: &TempDir) {
    fs::create_dir_all(temp.path().join(".git")).unwrap();
    fs::create_dir_all(temp.path().join(".repository/rules")).unwrap();
    fs::write(
        temp.path().join(".repository/config.toml"),
        "tools = []\n\n[core]\nmode = \"standard\"\n",
    )
    .unwrap();
}

// ==========================================================================
// JSON-RPC 2.0 ID Preservation
// ==========================================================================

#[tokio::test]
async fn test_numeric_id_preserved_in_response() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;

    let request = r#"{"jsonrpc":"2.0","id":42,"method":"initialize","params":{}}"#;
    let response: Value =
        serde_json::from_str(&server.handle_message(request).await.unwrap()).unwrap();

    assert_eq!(response["id"], 42, "Numeric ID must be echoed back exactly");
    assert_eq!(response["jsonrpc"], "2.0");
}

#[tokio::test]
async fn test_string_id_preserved_in_response() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;

    let request = r#"{"jsonrpc":"2.0","id":"req-abc-123","method":"initialize","params":{}}"#;
    let response: Value =
        serde_json::from_str(&server.handle_message(request).await.unwrap()).unwrap();

    assert_eq!(
        response["id"], "req-abc-123",
        "String ID must be echoed back exactly"
    );
}

#[tokio::test]
async fn test_id_preserved_in_error_response() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;

    let request = r#"{"jsonrpc":"2.0","id":"err-test","method":"nonexistent/method","params":{}}"#;
    let response: Value =
        serde_json::from_str(&server.handle_message(request).await.unwrap()).unwrap();

    assert_eq!(
        response["id"], "err-test",
        "ID must be preserved even in error responses"
    );
    assert!(
        response.get("error").is_some(),
        "Should be an error response"
    );
}

#[tokio::test]
async fn test_large_numeric_id_preserved() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;

    // Use a large numeric ID to test no truncation
    let request = r#"{"jsonrpc":"2.0","id":999999999,"method":"tools/list","params":{}}"#;
    let response: Value =
        serde_json::from_str(&server.handle_message(request).await.unwrap()).unwrap();

    assert_eq!(response["id"], 999999999);
}

// ==========================================================================
// Error Code Correctness (JSON-RPC 2.0 / MCP spec)
// ==========================================================================

#[tokio::test]
async fn test_method_not_found_returns_32601() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;

    let request = r#"{"jsonrpc":"2.0","id":1,"method":"completely/unknown","params":{}}"#;
    let response: Value =
        serde_json::from_str(&server.handle_message(request).await.unwrap()).unwrap();

    assert_eq!(
        response["error"]["code"], -32601,
        "Unknown method must return -32601 (Method not found)"
    );
    let msg = response["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("completely/unknown"),
        "Error message should include the unknown method name, got: {}",
        msg
    );
}

#[tokio::test]
async fn test_invalid_json_returns_parse_error() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;

    // Malformed JSON - handle_message returns Err which maps to serde_json::Error
    let result = server.handle_message(r#"{"not valid json"#).await;
    assert!(
        result.is_err(),
        "Malformed JSON should cause handle_message to return Err"
    );
}

#[tokio::test]
async fn test_missing_method_field_is_parse_error() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;

    // Valid JSON but missing required "method" field
    let result = server
        .handle_message(r#"{"jsonrpc":"2.0","id":1,"params":{}}"#)
        .await;
    assert!(
        result.is_err(),
        "Missing 'method' field should fail deserialization"
    );
}

#[tokio::test]
async fn test_invalid_params_for_tools_call_returns_error() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;

    // tools/call requires params with "name" field; send garbage params
    let request = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":"not-an-object"}"#;
    let result = server.handle_message(request).await;

    // Should fail because params can't be deserialized into ToolCallParams
    assert!(
        result.is_err(),
        "tools/call with non-object params should fail"
    );
}

#[tokio::test]
async fn test_invalid_params_for_resources_read_returns_error() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;

    // resources/read requires params with "uri" field
    let request =
        r#"{"jsonrpc":"2.0","id":1,"method":"resources/read","params":{"wrong_field":"value"}}"#;
    let result = server.handle_message(request).await;

    assert!(
        result.is_err(),
        "resources/read without 'uri' param should fail deserialization"
    );
}

#[tokio::test]
async fn test_unknown_resource_uri_returns_32602() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;

    let request = r#"{"jsonrpc":"2.0","id":1,"method":"resources/read","params":{"uri":"repo://nonexistent"}}"#;
    let response: Value =
        serde_json::from_str(&server.handle_message(request).await.unwrap()).unwrap();

    assert_eq!(
        response["error"]["code"], -32602,
        "Unknown resource URI should return -32602"
    );
    let msg = response["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("nonexistent"),
        "Error message should mention the bad URI, got: {}",
        msg
    );
}

#[tokio::test]
async fn test_malformed_uri_scheme_returns_error() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;

    // URI with wrong scheme
    let request = r#"{"jsonrpc":"2.0","id":1,"method":"resources/read","params":{"uri":"http://example.com/config"}}"#;
    let response: Value =
        serde_json::from_str(&server.handle_message(request).await.unwrap()).unwrap();

    assert!(
        response.get("error").is_some(),
        "Non-repo:// URI should return an error"
    );
    assert_eq!(
        response["error"]["code"], -32602,
        "Invalid URI should return -32602"
    );
}

// ==========================================================================
// Protocol Version Negotiation
// ==========================================================================

#[tokio::test]
async fn test_initialize_returns_protocol_version() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;

    let request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#;
    let response: Value =
        serde_json::from_str(&server.handle_message(request).await.unwrap()).unwrap();

    let protocol_version = response["result"]["protocolVersion"].as_str().unwrap();
    assert_eq!(
        protocol_version, "2024-11-05",
        "Server must respond with its supported protocol version"
    );
}

#[tokio::test]
async fn test_initialize_returns_server_info() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;

    let request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    let response: Value =
        serde_json::from_str(&server.handle_message(request).await.unwrap()).unwrap();

    let server_info = &response["result"]["serverInfo"];
    assert_eq!(
        server_info["name"].as_str().unwrap(),
        "repo-mcp",
        "Server name must be 'repo-mcp'"
    );
    assert!(
        server_info["version"].as_str().is_some(),
        "Server must report a version string"
    );
    // Version should look like a semver
    let version = server_info["version"].as_str().unwrap();
    assert!(
        version.contains('.'),
        "Version should be semver-like, got: {}",
        version
    );
}

#[tokio::test]
async fn test_initialize_returns_capabilities() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;

    let request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    let response: Value =
        serde_json::from_str(&server.handle_message(request).await.unwrap()).unwrap();

    let capabilities = &response["result"]["capabilities"];

    // Must declare tools capability
    assert!(
        capabilities.get("tools").is_some(),
        "Server must declare tools capability"
    );
    // Must declare resources capability
    assert!(
        capabilities.get("resources").is_some(),
        "Server must declare resources capability"
    );
}

// ==========================================================================
// Notification Handling
// ==========================================================================

#[tokio::test]
async fn test_initialized_notification_returns_empty() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;

    let request = r#"{"jsonrpc":"2.0","method":"initialized"}"#;
    let response = server.handle_message(request).await.unwrap();

    assert!(
        response.is_empty(),
        "Notifications must return empty string, got: {}",
        response
    );
}

#[tokio::test]
async fn test_notifications_initialized_returns_empty() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;

    let request = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    let response = server.handle_message(request).await.unwrap();

    assert!(
        response.is_empty(),
        "notifications/initialized must return empty string"
    );
}

// ==========================================================================
// Response Structure Validation
// ==========================================================================

#[tokio::test]
async fn test_success_response_has_result_not_error() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;

    let request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    let response: Value =
        serde_json::from_str(&server.handle_message(request).await.unwrap()).unwrap();

    assert!(
        response.get("result").is_some(),
        "Success response must have 'result' field"
    );
    assert!(
        response.get("error").is_none(),
        "Success response must NOT have 'error' field"
    );
    assert_eq!(response["jsonrpc"], "2.0");
}

#[tokio::test]
async fn test_error_response_has_error_not_result() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;

    let request = r#"{"jsonrpc":"2.0","id":1,"method":"no/such/method","params":{}}"#;
    let response: Value =
        serde_json::from_str(&server.handle_message(request).await.unwrap()).unwrap();

    assert!(
        response.get("error").is_some(),
        "Error response must have 'error' field"
    );
    assert!(
        response.get("result").is_none(),
        "Error response must NOT have 'result' field"
    );
    assert!(
        response["error"]["code"].is_i64(),
        "Error code must be an integer"
    );
    assert!(
        response["error"]["message"].is_string(),
        "Error message must be a string"
    );
}

// ==========================================================================
// Tools List Verification
// ==========================================================================

#[tokio::test]
async fn test_tools_list_returns_all_defined_tools() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;

    let request = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#;
    let response: Value =
        serde_json::from_str(&server.handle_message(request).await.unwrap()).unwrap();

    let tools = response["result"]["tools"].as_array().unwrap();
    assert!(
        tools.len() >= 15,
        "Should list a substantial number of tools, got {}",
        tools.len()
    );

    // Verify each tool has required MCP fields
    for tool in tools {
        assert!(
            tool["name"].is_string(),
            "Each tool must have a 'name' string"
        );
        assert!(
            tool["description"].is_string(),
            "Each tool must have a 'description' string"
        );
        assert!(
            tool["inputSchema"].is_object(),
            "Each tool must have an 'inputSchema' object"
        );
    }

    // Verify specific tools are present
    let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(tool_names.contains(&"repo_init"));
    assert!(tool_names.contains(&"repo_check"));
    assert!(tool_names.contains(&"branch_create"));
    assert!(tool_names.contains(&"rule_add"));
}

// ==========================================================================
// Resources List Verification
// ==========================================================================

#[tokio::test]
async fn test_resources_list_returns_all_defined_resources() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;

    let request = r#"{"jsonrpc":"2.0","id":1,"method":"resources/list","params":{}}"#;
    let response: Value =
        serde_json::from_str(&server.handle_message(request).await.unwrap()).unwrap();

    let resources = response["result"]["resources"].as_array().unwrap();
    assert_eq!(resources.len(), 3, "Should list all 3 defined resources");

    // Verify each resource has required MCP fields
    for resource in resources {
        assert!(
            resource["uri"].is_string(),
            "Each resource must have a 'uri' string"
        );
        assert!(
            resource["name"].is_string(),
            "Each resource must have a 'name' string"
        );
        assert!(
            resource["mimeType"].is_string(),
            "Each resource must have a 'mimeType' string"
        );
    }

    let uris: Vec<&str> = resources
        .iter()
        .map(|r| r["uri"].as_str().unwrap())
        .collect();
    assert!(uris.contains(&"repo://config"));
    assert!(uris.contains(&"repo://state"));
    assert!(uris.contains(&"repo://rules"));
}

// ==========================================================================
// Tool Invocation End-to-End
// ==========================================================================

#[tokio::test]
async fn test_tool_call_repo_init_creates_repo() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;
    // Remove .repository so repo_init can create it fresh
    fs::remove_dir_all(temp.path().join(".repository")).unwrap();
    // Create minimal .git so mode detection works
    fs::create_dir_all(temp.path().join(".git")).unwrap();

    let request = serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "repo_init",
            "arguments": {
                "name": "test-project",
                "mode": "standard",
                "tools": ["claude"]
            }
        }
    }))
    .unwrap();

    let response: Value =
        serde_json::from_str(&server.handle_message(&request).await.unwrap()).unwrap();

    // Should be a success response (not a JSON-RPC error)
    assert!(
        response.get("result").is_some(),
        "Tool call should return a result, not an error"
    );
    assert!(
        response.get("error").is_none(),
        "Successful tool call should not have error field"
    );

    // The result should contain tool content (text type)
    let result = &response["result"];
    let content = result["content"].as_array().unwrap();
    assert!(!content.is_empty(), "Tool result must have content");
    assert_eq!(content[0]["type"], "text");

    // Parse the inner text to verify the tool actually ran
    let inner_text = content[0]["text"].as_str().unwrap();
    let inner: Value = serde_json::from_str(inner_text).unwrap();
    assert_eq!(
        inner["success"], true,
        "repo_init should report success=true"
    );

    // Verify the file was actually created on disk
    assert!(
        temp.path().join(".repository/config.toml").exists(),
        "repo_init should create .repository/config.toml on disk"
    );
    let config_content = fs::read_to_string(temp.path().join(".repository/config.toml")).unwrap();
    assert!(
        config_content.contains("claude"),
        "Config should contain the 'claude' tool we requested"
    );
}

#[tokio::test]
async fn test_tool_call_unknown_tool_returns_is_error() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;

    let request = serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "completely_fake_tool",
            "arguments": {}
        }
    }))
    .unwrap();

    let response: Value =
        serde_json::from_str(&server.handle_message(&request).await.unwrap()).unwrap();

    // Per MCP spec, tool errors are returned as successful JSON-RPC responses with is_error=true
    let result = &response["result"];
    assert_eq!(
        result["is_error"], true,
        "Unknown tool should return is_error=true in result"
    );
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(
        text.contains("unknown tool"),
        "Error text should mention 'unknown tool', got: {}",
        text
    );
}

#[tokio::test]
async fn test_tool_call_rule_add_then_read_rules_resource() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;
    create_test_repo(&temp);

    // Step 1: Add a rule via tools/call
    let add_request = serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "rule_add",
            "arguments": {
                "id": "no-panics",
                "content": "Never use unwrap() in production code."
            }
        }
    }))
    .unwrap();

    let add_response: Value =
        serde_json::from_str(&server.handle_message(&add_request).await.unwrap()).unwrap();
    let add_text = add_response["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    let add_inner: Value = serde_json::from_str(add_text).unwrap();
    assert_eq!(add_inner["success"], true, "rule_add should succeed");

    // Step 2: Read rules resource and verify the rule appears
    let read_request = serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "resources/read",
        "params": { "uri": "repo://rules" }
    }))
    .unwrap();

    let read_response: Value =
        serde_json::from_str(&server.handle_message(&read_request).await.unwrap()).unwrap();

    let contents = read_response["result"]["contents"].as_array().unwrap();
    assert_eq!(contents.len(), 1);
    assert_eq!(contents[0]["uri"], "repo://rules");

    let rules_text = contents[0]["text"].as_str().unwrap();
    assert!(
        rules_text.contains("no-panics"),
        "Rules resource should contain the added rule ID, got:\n{}",
        rules_text
    );
    assert!(
        rules_text.contains("Never use unwrap()"),
        "Rules resource should contain the rule content, got:\n{}",
        rules_text
    );
}

#[tokio::test]
async fn test_tool_call_not_implemented_returns_is_error() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;
    create_test_repo(&temp);

    // extension_install is explicitly not implemented
    let request = serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "extension_install",
            "arguments": { "source": "https://example.com/ext.git" }
        }
    }))
    .unwrap();

    let response: Value =
        serde_json::from_str(&server.handle_message(&request).await.unwrap()).unwrap();

    // Not-implemented tools return as tool errors, not JSON-RPC errors
    let result = &response["result"];
    assert_eq!(
        result["is_error"], true,
        "Not-implemented tool should return is_error=true"
    );
}

#[tokio::test]
async fn test_resource_read_config_returns_valid_content() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;
    create_test_repo(&temp);

    let request = serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "resources/read",
        "params": { "uri": "repo://config" }
    }))
    .unwrap();

    let response: Value =
        serde_json::from_str(&server.handle_message(&request).await.unwrap()).unwrap();

    let contents = response["result"]["contents"].as_array().unwrap();
    assert_eq!(contents.len(), 1, "Should return exactly one content entry");
    assert_eq!(contents[0]["uri"], "repo://config");
    assert_eq!(contents[0]["mimeType"], "application/toml");

    let text = contents[0]["text"].as_str().unwrap();
    assert!(
        text.contains("[core]"),
        "Config content should contain [core] section, got:\n{}",
        text
    );
    assert!(
        text.contains("mode"),
        "Config content should contain mode setting"
    );
}

// ==========================================================================
// Extension Handlers Return NotImplemented
// ==========================================================================

#[tokio::test]
async fn test_mcp_extension_install_not_implemented() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;
    create_test_repo(&temp);

    let request = serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "extension_install",
            "arguments": { "source": "https://example.com/ext.git" }
        }
    }))
    .unwrap();

    let response: Value =
        serde_json::from_str(&server.handle_message(&request).await.unwrap()).unwrap();

    // Extension handlers return NotImplemented, which surfaces as is_error=true
    let result = &response["result"];
    assert_eq!(
        result["is_error"], true,
        "extension_install should return is_error=true (not implemented)"
    );
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(
        text.contains("not implemented"),
        "Error text should mention 'not implemented', got: {}",
        text
    );
}

#[tokio::test]
async fn test_mcp_extension_handlers_return_not_implemented() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;
    create_test_repo(&temp);

    // All extension mutation operations should return is_error=true
    let extension_tools = vec![
        ("extension_install", json!({ "source": "test" })),
        ("extension_add", json!({ "name": "test" })),
        ("extension_init", json!({ "name": "test" })),
        ("extension_remove", json!({ "name": "test" })),
    ];

    for (tool_name, args) in extension_tools {
        let request = serde_json::to_string(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": args
            }
        }))
        .unwrap();

        let response: Value =
            serde_json::from_str(&server.handle_message(&request).await.unwrap()).unwrap();

        let result = &response["result"];
        assert_eq!(
            result["is_error"], true,
            "{} should return is_error=true (not implemented)",
            tool_name
        );
    }
}

#[tokio::test]
async fn test_mcp_extension_list_succeeds() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;
    create_test_repo(&temp);

    let request = serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "extension_list",
            "arguments": {}
        }
    }))
    .unwrap();

    let response: Value =
        serde_json::from_str(&server.handle_message(&request).await.unwrap()).unwrap();

    // extension_list IS a valid operation -- it should succeed
    assert!(
        response.get("result").is_some(),
        "extension_list should return a result, not an error"
    );
    let result = &response["result"];
    assert!(
        result.get("is_error").is_none() || result["is_error"] == false,
        "extension_list should NOT return is_error=true"
    );

    // Parse the inner content to verify it returns known extensions
    let content = result["content"][0]["text"].as_str().unwrap();
    let inner: Value = serde_json::from_str(content).unwrap();
    assert!(
        inner.get("known").is_some(),
        "extension_list should return known extensions"
    );
    assert_eq!(
        inner["installed_count"], 0,
        "No extensions should be installed"
    );
}

// ==========================================================================
// Multiple Sequential Requests (statelessness check)
// ==========================================================================

#[tokio::test]
async fn test_sequential_requests_use_correct_ids() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;

    // Send multiple requests and verify each gets its own ID back
    let requests = vec![
        (
            r#"{"jsonrpc":"2.0","id":100,"method":"initialize","params":{}}"#,
            100,
        ),
        (
            r#"{"jsonrpc":"2.0","id":200,"method":"tools/list","params":{}}"#,
            200,
        ),
        (
            r#"{"jsonrpc":"2.0","id":300,"method":"resources/list","params":{}}"#,
            300,
        ),
    ];

    for (request, expected_id) in requests {
        let response: Value =
            serde_json::from_str(&server.handle_message(request).await.unwrap()).unwrap();
        assert_eq!(
            response["id"], expected_id,
            "Request with id={} should get that id back",
            expected_id
        );
    }
}

#[tokio::test]
async fn test_error_after_success_does_not_corrupt_state() {
    let temp = TempDir::new().unwrap();
    let server = setup_server(&temp).await;

    // First: valid request
    let r1 = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#;
    let resp1: Value = serde_json::from_str(&server.handle_message(r1).await.unwrap()).unwrap();
    assert!(resp1.get("result").is_some());

    // Second: invalid method (should error)
    let r2 = r#"{"jsonrpc":"2.0","id":2,"method":"fake/method","params":{}}"#;
    let resp2: Value = serde_json::from_str(&server.handle_message(r2).await.unwrap()).unwrap();
    assert!(resp2.get("error").is_some());

    // Third: valid request again (should still work)
    let r3 = r#"{"jsonrpc":"2.0","id":3,"method":"resources/list","params":{}}"#;
    let resp3: Value = serde_json::from_str(&server.handle_message(r3).await.unwrap()).unwrap();
    assert!(
        resp3.get("result").is_some(),
        "Server should still work after an error response"
    );
    let resources = resp3["result"]["resources"].as_array().unwrap();
    assert_eq!(resources.len(), 3, "Should still list all 3 resources");
}
