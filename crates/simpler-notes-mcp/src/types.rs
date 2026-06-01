use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize)]
pub struct JsonRpcRequest {
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
        JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: u64, code: i32, message: String) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_success_response() {
        let resp = JsonRpcResponse::success(1, json!("ok"));
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], 1);
        assert_eq!(json["result"], "ok");
        assert!(json.get("error").is_none());
    }

    #[test]
    fn test_error_response() {
        let resp = JsonRpcResponse::error(42, -32601, "Method not found".into());
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
        assert_eq!(req.id, 1);
        assert_eq!(req.method, "list_notes");
        assert_eq!(req.params.unwrap()["path"], "notes");
    }

    #[test]
    fn test_request_no_params() {
        let input = r#"{"id":2,"method":"reindex"}"#;
        let req: JsonRpcRequest = serde_json::from_str(input).unwrap();
        assert_eq!(req.id, 2);
        assert_eq!(req.method, "reindex");
        assert!(req.params.is_none());
    }
}
