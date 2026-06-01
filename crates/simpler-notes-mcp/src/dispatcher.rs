use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;

pub type ToolResult = Result<Value, (i32, String)>;

pub trait Tool: Send + Sync {
    fn call(&self, params: Option<Value>) -> ToolResult;
}

pub struct Dispatcher {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Dispatcher { tools: HashMap::new() }
    }

    pub fn register(&mut self, name: &str, tool: Arc<dyn Tool>) {
        self.tools.insert(name.to_string(), tool);
    }

    pub fn dispatch(&self, method: &str, params: Option<Value>) -> ToolResult {
        match self.tools.get(method) {
            Some(tool) => tool.call(params),
            None => Err((-32601, format!("Method not found: {}", method))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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

    #[test]
    fn test_new_dispatcher_empty() {
        let d = Dispatcher::new();
        let err = d.dispatch("any", None).unwrap_err();
        assert_eq!(err.0, -32601);
        assert!(err.1.contains("any"));
    }

    #[test]
    fn test_register_and_dispatch() {
        let mut d = Dispatcher::new();
        d.register("ping", Arc::new(MockTool {
            expected_params: None,
            result: Ok(json!("pong")),
        }));
        let result = d.dispatch("ping", None).unwrap();
        assert_eq!(result, "pong");
    }

    #[test]
    fn test_register_overwrites() {
        let mut d = Dispatcher::new();
        d.register("x", Arc::new(MockTool {
            expected_params: None,
            result: Ok(json!("first")),
        }));
        d.register("x", Arc::new(MockTool {
            expected_params: None,
            result: Ok(json!("second")),
        }));
        let result = d.dispatch("x", None).unwrap();
        assert_eq!(result, "second");
    }

    #[test]
    fn test_passes_params() {
        let mut d = Dispatcher::new();
        d.register("echo", Arc::new(MockTool {
            expected_params: Some(json!({"key": "val"})),
            result: Ok(json!("ok")),
        }));
        let result = d.dispatch("echo", Some(json!({"key": "val"}))).unwrap();
        assert_eq!(result, "ok");
    }
}
