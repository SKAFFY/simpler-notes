use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::tool::{GenericTool, ToolHandler, ToolResult, InputSchema, ParamDef};

pub(crate) fn handler(vault: &Vault, params: Option<Value>) -> ToolResult {
    let p = params.ok_or((-32602, "Missing parameters".to_string()))?;
    let target = p.get("target")
        .and_then(|v| v.as_str())
        .ok_or((-32602, "Missing required parameter: target".to_string()))?;
    match vault.resolve_link(target) {
        Ok(path) => Ok(json!({"path": path.to_string_lossy()})),
        Err(e) => Err((-32000, e)),
    }
}

pub(crate) fn tool(vault: Arc<Vault>) -> GenericTool {
    GenericTool {
        vault,
        handler: handler as ToolHandler,
        name: "resolve_link",
        description: "Resolve a [[wiki-link]] target to an actual file path",
        input: InputSchema::new(&[
            ParamDef { name: "target", description: "Link target (e.g. note name without extension)", required: true },
        ]),
    }
}
