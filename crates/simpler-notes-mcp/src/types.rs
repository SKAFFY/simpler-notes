use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcErrorValue>,
}

#[derive(Serialize)]
pub struct JsonRpcErrorValue {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    pub fn success(id: u64, result: Value) -> Self {
        JsonRpcResponse { jsonrpc: "2.0".into(), id, result: Some(result), error: None }
    }

    pub fn error(id: u64, code: i32, message: String) -> Self {
        JsonRpcResponse { jsonrpc: "2.0".into(), id, result: None, error: Some(JsonRpcErrorValue { code, message, data: None }) }
    }
}
