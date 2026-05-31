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
