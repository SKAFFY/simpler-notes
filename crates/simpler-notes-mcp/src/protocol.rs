use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize)]
pub struct JsonRpcRequest {
    #[serde(default)]
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
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
    pub fn success(id: Option<Value>, result: Value) -> Self {
        JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, code: i32, message: String) -> Self {
        JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(JsonRpcErrorValue {
                code,
                message,
                data: None,
            }),
        }
    }
}

/// MCP tool description for tools/list response
#[derive(Serialize)]
pub struct ToolDescription {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// MCP content item for tools/call response
#[derive(Serialize)]
pub struct ContentItem {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_success_response() {
        let resp = JsonRpcResponse::success(Some(json!(1)), json!("ok"));
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], 1);
        assert_eq!(json["result"], "ok");
        assert!(json.get("error").is_none());
    }

    #[test]
    fn test_success_response_no_id() {
        let resp = JsonRpcResponse::success(None, json!("ok"));
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["jsonrpc"], "2.0");
        assert!(json.get("id").is_none());
        assert_eq!(json["result"], "ok");
    }

    #[test]
    fn test_error_response() {
        let resp = JsonRpcResponse::error(Some(json!(42)), -32601, "Method not found".into());
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["id"], 42);
        assert!(json.get("result").is_none());
        assert_eq!(json["error"]["code"], -32601);
        assert_eq!(json["error"]["message"], "Method not found");
    }

    #[test]
    fn test_request_deserialize() {
        let input = r#"{"id":1,"method":"list_notes","params":{"path":"notes"}}"#;
        let req: JsonRpcRequest = serde_json::from_str(input).unwrap();
        assert_eq!(req.id, Some(json!(1)));
        assert_eq!(req.method, "list_notes");
        assert_eq!(req.params.unwrap()["path"], "notes");
    }

    #[test]
    fn test_request_no_params() {
        let input = r#"{"id":2,"method":"reindex"}"#;
        let req: JsonRpcRequest = serde_json::from_str(input).unwrap();
        assert_eq!(req.id, Some(json!(2)));
        assert_eq!(req.method, "reindex");
        assert!(req.params.is_none());
    }

    #[test]
    fn test_notification_has_no_id() {
        let input = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        let req: JsonRpcRequest = serde_json::from_str(input).unwrap();
        assert!(req.id.is_none());
        assert_eq!(req.method, "notifications/initialized");
    }

    #[test]
    fn test_request_string_id() {
        let input = r#"{"id":"custom-1","method":"ping"}"#;
        let req: JsonRpcRequest = serde_json::from_str(input).unwrap();
        assert_eq!(req.id, Some(json!("custom-1")));
    }
}
