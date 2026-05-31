use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

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

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, code: i32, message: impl Into<String>) -> Self {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
            }),
        }
    }

    pub fn parse_error(id: Option<Value>, detail: impl Into<String>) -> Self {
        Self::error(id, -32700, format!("Parse error: {}", detail.into()))
    }

    pub fn method_not_found(id: Option<Value>, method: impl Into<String>) -> Self {
        Self::error(id, -32601, format!("Method not found: {}", method.into()))
    }

    pub fn invalid_params(id: Option<Value>, detail: impl Into<String>) -> Self {
        Self::error(id, -32602, format!("Invalid params: {}", detail.into()))
    }

    pub fn internal_error(id: Option<Value>, detail: impl Into<String>) -> Self {
        Self::error(id, -32000, format!("Internal error: {}", detail.into()))
    }
}
