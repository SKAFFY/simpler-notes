use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;

pub type ToolResult = Result<Value, (i32, String)>;

pub type ToolHandler = fn(&Vault, Option<Value>) -> ToolResult;

pub trait Tool: Send + Sync {
    fn call(&self, params: Option<Value>) -> ToolResult;
}

pub struct ParamDef {
    pub name: &'static str,
    pub description: &'static str,
    pub required: bool,
}

pub struct InputSchema {
    pub params: &'static [ParamDef],
}

impl InputSchema {
    pub const fn new(params: &'static [ParamDef]) -> Self {
        InputSchema { params }
    }

    pub fn to_json_schema(&self) -> Value {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for p in self.params {
            let mut prop = serde_json::Map::new();
            prop.insert("type".into(), Value::String("string".into()));
            prop.insert("description".into(), Value::String(p.description.into()));
            properties.insert(p.name.into(), Value::Object(prop));

            if p.required {
                required.push(Value::String(p.name.into()));
            }
        }

        json!({
            "type": "object",
            "properties": properties,
            "required": required,
        })
    }
}

pub struct ToolInfo {
    pub description: String,
    pub input_schema: Value,
}

pub struct GenericTool {
    pub vault: Arc<Vault>,
    pub handler: ToolHandler,
    pub name: &'static str,
    pub description: &'static str,
    pub input: InputSchema,
}

impl Tool for GenericTool {
    fn call(&self, params: Option<Value>) -> ToolResult {
        (self.handler)(&self.vault, params)
    }
}
