//! MCP Protocol message types
//!
//! JSON-RPC 2.0 message structures for MCP communication.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 Request
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Initialize request params
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    pub protocol_version: String,
    #[serde(default)]
    pub capabilities: ClientCapabilities,
    pub client_info: ClientInfo,
}

#[derive(Debug, Deserialize, Default)]
pub struct ClientCapabilities {}

#[derive(Debug, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// Initialize response result
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    pub server_info: ServerInfo,
}

#[derive(Debug, Serialize)]
pub struct ServerCapabilities {
    pub tools: Option<ToolsCapability>,
    pub resources: Option<ResourcesCapability>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsCapability {
    pub list_changed: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcesCapability {
    pub subscribe: Option<bool>,
    pub list_changed: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

/// Tool call params
#[derive(Debug, Deserialize)]
pub struct ToolCallParams {
    pub name: String,
    #[serde(default)]
    pub arguments: Value,
}

/// Resource read params
#[derive(Debug, Deserialize)]
pub struct ReadResourceParams {
    pub uri: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_request_deserialize() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0"}
            }
        }"#;
        let request: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.method, "initialize");
    }

    #[test]
    fn test_response_serialize() {
        let response =
            JsonRpcResponse::success(Some(Value::Number(1.into())), serde_json::json!({"ok": true}));
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("result"));
        assert!(!json.contains("error"));
    }

    #[test]
    fn test_error_response_serialize() {
        let response = JsonRpcResponse::error(
            Some(Value::Number(1.into())),
            -32600,
            "Invalid Request".to_string(),
        );
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("error"));
        assert!(json.contains("-32600"));
    }

    #[test]
    fn test_jsonrpc_request_with_string_id() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": "abc-123",
            "method": "tools/list",
            "params": {}
        }"#;
        let request: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.id, Some(Value::String("abc-123".to_string())));
        assert_eq!(request.method, "tools/list");
    }

    #[test]
    fn test_jsonrpc_request_with_null_id() {
        // In JSON-RPC 2.0, a null id is treated the same as a missing id (notification)
        // serde_json deserializes `"id": null` as None for Option<Value>
        let json = r#"{
            "jsonrpc": "2.0",
            "id": null,
            "method": "notifications/initialized"
        }"#;
        let request: JsonRpcRequest = serde_json::from_str(json).unwrap();
        // Note: serde treats `null` as None for Option types
        assert!(request.id.is_none());
        assert_eq!(request.method, "notifications/initialized");
    }

    #[test]
    fn test_jsonrpc_request_without_id_is_notification() {
        let json = r#"{
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }"#;
        let request: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert!(request.id.is_none());
        assert_eq!(request.method, "notifications/initialized");
    }

    #[test]
    fn test_jsonrpc_request_without_params() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        }"#;
        let request: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.params, Value::Null);
    }

    #[test]
    fn test_initialize_params_deserialize() {
        let json = r#"{
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "Claude Desktop", "version": "1.0.0"}
        }"#;
        let params: InitializeParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.protocol_version, "2024-11-05");
        assert_eq!(params.client_info.name, "Claude Desktop");
        assert_eq!(params.client_info.version, "1.0.0");
    }

    #[test]
    fn test_initialize_result_serialize() {
        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
                resources: Some(ResourcesCapability {
                    subscribe: Some(false),
                    list_changed: Some(false),
                }),
            },
            server_info: ServerInfo {
                name: "repo-mcp".to_string(),
                version: "0.1.0".to_string(),
            },
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("protocolVersion"));
        assert!(json.contains("2024-11-05"));
        assert!(json.contains("repo-mcp"));
        assert!(json.contains("serverInfo"));
    }

    #[test]
    fn test_tool_call_params_deserialize() {
        let json = r#"{
            "name": "repo_init",
            "arguments": {"name": "my-project", "mode": "standard"}
        }"#;
        let params: ToolCallParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.name, "repo_init");
        assert_eq!(params.arguments["name"], "my-project");
        assert_eq!(params.arguments["mode"], "standard");
    }

    #[test]
    fn test_tool_call_params_without_arguments() {
        let json = r#"{"name": "repo_check"}"#;
        let params: ToolCallParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.name, "repo_check");
        assert_eq!(params.arguments, Value::Null);
    }

    #[test]
    fn test_read_resource_params_deserialize() {
        let json = r#"{"uri": "repo://config"}"#;
        let params: ReadResourceParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.uri, "repo://config");
    }

    #[test]
    fn test_response_success_format() {
        let response = JsonRpcResponse::success(
            Some(Value::Number(42.into())),
            serde_json::json!({"status": "ok"}),
        );
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(Value::Number(42.into())));
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_response_error_format() {
        let response = JsonRpcResponse::error(
            Some(Value::Number(1.into())),
            -32601,
            "Method not found".to_string(),
        );
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, Some(Value::Number(1.into())));
        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let err = response.error.unwrap();
        assert_eq!(err.code, -32601);
        assert_eq!(err.message, "Method not found");
    }

    #[test]
    fn test_json_rpc_error_codes() {
        // Standard JSON-RPC error codes
        let parse_error = JsonRpcResponse::error(None, -32700, "Parse error".to_string());
        assert_eq!(parse_error.error.as_ref().unwrap().code, -32700);

        let invalid_request = JsonRpcResponse::error(None, -32600, "Invalid Request".to_string());
        assert_eq!(invalid_request.error.as_ref().unwrap().code, -32600);

        let method_not_found = JsonRpcResponse::error(None, -32601, "Method not found".to_string());
        assert_eq!(method_not_found.error.as_ref().unwrap().code, -32601);

        let invalid_params = JsonRpcResponse::error(None, -32602, "Invalid params".to_string());
        assert_eq!(invalid_params.error.as_ref().unwrap().code, -32602);

        let internal_error = JsonRpcResponse::error(None, -32603, "Internal error".to_string());
        assert_eq!(internal_error.error.as_ref().unwrap().code, -32603);
    }

    #[test]
    fn test_response_serializes_without_null_fields() {
        let response = JsonRpcResponse::success(Some(Value::Number(1.into())), Value::Null);
        let json = serde_json::to_string(&response).unwrap();

        // Should contain result (even if null) but not error
        assert!(json.contains("result"));
        assert!(!json.contains("error"));
    }

    #[test]
    fn test_error_response_serializes_without_result() {
        let response = JsonRpcResponse::error(
            Some(Value::Number(1.into())),
            -32600,
            "Invalid".to_string(),
        );
        let json = serde_json::to_string(&response).unwrap();

        // Should contain error but not result
        assert!(json.contains("error"));
        assert!(!json.contains("result"));
    }

    #[test]
    fn test_server_capabilities_with_null_options() {
        let caps = ServerCapabilities {
            tools: None,
            resources: None,
        };
        let json = serde_json::to_string(&caps).unwrap();
        // Null values should still serialize (they're not skipped)
        assert!(json.contains("tools"));
        assert!(json.contains("resources"));
    }
}
