use std::collections::HashMap;
use std::sync::Arc;
use serde_json::json;

use crate::protocol::{ContentItem, JsonRpcRequest, JsonRpcResponse, ToolDescription};
use crate::tool::{GenericTool, Tool, ToolInfo};

pub struct Dispatcher {
    tools: HashMap<String, (Arc<dyn Tool>, ToolInfo)>,
    #[allow(dead_code)]
    initialized: bool,
}

impl Dispatcher {
    pub fn new() -> Self {
        Dispatcher {
            tools: HashMap::new(),
            initialized: false,
        }
    }

    #[allow(dead_code)]
    pub fn register(&mut self, name: &str, tool: Arc<dyn Tool>, info: ToolInfo) {
        self.tools.insert(name.to_string(), (tool, info));
    }

    pub fn register_generic(&mut self, tool: GenericTool) {
        let name = tool.name.to_string();
        let info = ToolInfo {
            description: tool.description.to_string(),
            input_schema: tool.input.to_json_schema(),
        };
        self.tools.insert(name, (Arc::new(tool), info));
    }

    pub fn handle_jsonrpc(&mut self, request: JsonRpcRequest) -> Option<JsonRpcResponse> {
        let method = &request.method;
        let id = request.id;

        if id.is_none() {
            if method == "notifications/initialized" {
                self.initialized = true;
            }
            return None;
        }

        let id = id.unwrap();

        match method.as_str() {
            "initialize" => {
                let protocol_version = request.params
                    .as_ref()
                    .and_then(|p| p.get("protocolVersion"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                let capabilities = json!({
                    "protocolVersion": protocol_version,
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "simpler-notes-mcp",
                        "version": "0.1.0"
                    }
                });

                Some(JsonRpcResponse::success(Some(id), capabilities))
            }

            "tools/list" => {
                let tools: Vec<ToolDescription> = self.tools.iter().map(|(name, (_, info))| {
                    ToolDescription {
                        name: name.clone(),
                        description: info.description.clone(),
                        input_schema: info.input_schema.clone(),
                    }
                }).collect();

                Some(JsonRpcResponse::success(Some(id), json!({ "tools": tools })))
            }

            "tools/call" => {
                let tool_name = request.params
                    .as_ref()
                    .and_then(|p| p.get("name"))
                    .and_then(|v| v.as_str());

                let arguments = request.params
                    .as_ref()
                    .and_then(|p| p.get("arguments"))
                    .cloned();

                match tool_name {
                    Some(name) => {
                        match self.tools.get(name) {
                            Some((tool, _)) => {
                                match tool.call(arguments) {
                                    Ok(result) => {
                                        let content = ContentItem {
                                            content_type: "text".to_string(),
                                            text: serde_json::to_string(&result).unwrap_or_default(),
                                        };
                                        Some(JsonRpcResponse::success(Some(id), json!({
                                            "content": [content],
                                            "isError": false,
                                        })))
                                    }
                                    Err((_code, msg)) => {
                                        let content = ContentItem {
                                            content_type: "text".to_string(),
                                            text: msg.clone(),
                                        };
                                        Some(JsonRpcResponse::success(Some(id), json!({
                                            "content": [content],
                                            "isError": true,
                                        })))
                                    }
                                }
                            }
                            None => {
                                let content = ContentItem {
                                    content_type: "text".to_string(),
                                    text: format!("Tool not found: {}", name),
                                };
                                Some(JsonRpcResponse::success(Some(id), json!({
                                    "content": [content],
                                    "isError": true,
                                })))
                            }
                        }
                    }
                    None => {
                        Some(JsonRpcResponse::error(Some(id), -32602, "Missing parameter: name".to_string()))
                    }
                }
            }

            // Legacy direct method dispatch (for backwards compatibility)
            _ => {
                match self.tools.get(method) {
                    Some((tool, _)) => {
                        match tool.call(request.params) {
                            Ok(result) => Some(JsonRpcResponse::success(Some(id), result)),
                            Err((code, msg)) => Some(JsonRpcResponse::error(Some(id), code, msg)),
                        }
                    }
                    None => Some(JsonRpcResponse::error(Some(id), -32601, format!("Method not found: {}", method))),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use crate::tool::ToolResult;

    struct MockTool {
        expected_params: Option<Value>,
        result: ToolResult,
    }

    impl Tool for MockTool {
        fn call(&self, params: Option<Value>) -> ToolResult {
            if let Some(expected) = &self.expected_params {
                assert_eq!(params.as_ref(), Some(expected), "unexpected params");
            }
            self.result.clone()
        }
    }

    fn make_dispatcher() -> Dispatcher {
        let mut d = Dispatcher::new();
        d.register("ping", Arc::new(MockTool {
            expected_params: None,
            result: Ok(json!("pong")),
        }), ToolInfo {
            description: "Ping the server".into(),
            input_schema: json!({"type": "object", "properties": {}}),
        });
        d
    }

    fn req(id: u64, method: &str, params: Option<Value>) -> JsonRpcRequest {
        JsonRpcRequest {
            id: Some(json!(id)),
            method: method.to_string(),
            params,
        }
    }

    fn notification(method: &str) -> JsonRpcRequest {
        JsonRpcRequest {
            id: None,
            method: method.to_string(),
            params: None,
        }
    }

    #[test]
    fn test_new_dispatcher_empty() {
        let d = Dispatcher::new();
        assert!(d.tools.is_empty());
    }

    #[test]
    fn test_register_and_dispatch_legacy() {
        let mut d = Dispatcher::new();
        d.register("ping", Arc::new(MockTool {
            expected_params: None,
            result: Ok(json!("pong")),
        }), ToolInfo {
            description: "".into(),
            input_schema: json!({}),
        });
        let resp = d.handle_jsonrpc(req(1, "ping", None)).unwrap();
        assert_eq!(resp.result, Some(json!("pong")));
    }

    #[test]
    fn test_register_overwrites() {
        let mut d = Dispatcher::new();
        d.register("x", Arc::new(MockTool {
            expected_params: None,
            result: Ok(json!("first")),
        }), ToolInfo { description: "".into(), input_schema: json!({}) });
        d.register("x", Arc::new(MockTool {
            expected_params: None,
            result: Ok(json!("second")),
        }), ToolInfo { description: "".into(), input_schema: json!({}) });
        let resp = d.handle_jsonrpc(req(1, "x", None)).unwrap();
        assert_eq!(resp.result, Some(json!("second")));
    }

    #[test]
    fn test_method_not_found() {
        let mut d = make_dispatcher();
        let resp = d.handle_jsonrpc(req(1, "nonexistent", None)).unwrap();
        let err = resp.error.unwrap();
        assert_eq!(err.code, -32601);
    }

    #[test]
    fn test_initialize() {
        let mut d = Dispatcher::new();
        let resp = d.handle_jsonrpc(JsonRpcRequest {
            id: Some(json!(1)),
            method: "initialize".into(),
            params: Some(json!({"protocolVersion": "2024-11-05"})),
        }).unwrap();
        let result = resp.result.unwrap();
        assert_eq!(result["protocolVersion"], "2024-11-05");
        assert!(result["capabilities"]["tools"].is_object());
        assert_eq!(result["serverInfo"]["name"], "simpler-notes-mcp");
    }

    #[test]
    fn test_initialize_default_version() {
        let mut d = Dispatcher::new();
        let resp = d.handle_jsonrpc(JsonRpcRequest {
            id: Some(json!(1)),
            method: "initialize".into(),
            params: None,
        }).unwrap();
        let result = resp.result.unwrap();
        assert_eq!(result["protocolVersion"], "unknown");
    }

    #[test]
    fn test_tools_list() {
        let mut d = make_dispatcher();
        let resp = d.handle_jsonrpc(req(1, "tools/list", None)).unwrap();
        let result = resp.result.unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "ping");
        assert_eq!(tools[0]["description"], "Ping the server");
        assert!(tools[0].get("input_schema").is_some());
    }

    #[test]
    fn test_tools_call() {
        let mut d = make_dispatcher();
        let resp = d.handle_jsonrpc(req(1, "tools/call", Some(json!({
            "name": "ping",
            "arguments": null,
        })))).unwrap();
        let result = resp.result.unwrap();
        assert_eq!(result["isError"], false);
        let content = result["content"].as_array().unwrap();
        assert_eq!(content[0]["type"], "text");
    }

    #[test]
    fn test_tools_call_missing_name() {
        let mut d = make_dispatcher();
        let resp = d.handle_jsonrpc(req(1, "tools/call", Some(json!({})))).unwrap();
        let err = resp.error.unwrap();
        assert_eq!(err.code, -32602);
    }

    #[test]
    fn test_tools_call_not_found() {
        let mut d = make_dispatcher();
        let resp = d.handle_jsonrpc(req(1, "tools/call", Some(json!({
            "name": "ghost",
        })))).unwrap();
        let result = resp.result.unwrap();
        assert_eq!(result["isError"], true);
    }

    #[test]
    fn test_tools_call_tool_error() {
        let mut d = Dispatcher::new();
        d.register("failing", Arc::new(MockTool {
            expected_params: None,
            result: Err((-1, "something broke".into())),
        }), ToolInfo { description: "".into(), input_schema: json!({}) });
        let resp = d.handle_jsonrpc(req(1, "tools/call", Some(json!({
            "name": "failing",
        })))).unwrap();
        let result = resp.result.unwrap();
        assert_eq!(result["isError"], true);
    }

    #[test]
    fn test_notification_returns_none() {
        let mut d = Dispatcher::new();
        let resp = d.handle_jsonrpc(notification("notifications/initialized"));
        assert!(resp.is_none());
    }
}
