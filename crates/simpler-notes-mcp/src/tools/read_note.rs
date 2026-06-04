use std::sync::Arc;
use std::path::PathBuf;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::tool::{GenericTool, ToolHandler, ToolResult, InputSchema, ParamDef};

pub(crate) fn handler(vault: &Vault, params: Option<Value>) -> ToolResult {
    let path = params
        .and_then(|p| p.get("path").and_then(|q| q.as_str().map(|s| s.to_string())))
        .ok_or((-32602, "Missing required parameter: path".to_string()))?;

    let content = vault.read_note(&PathBuf::from(&path))
        .map_err(|e| (-1, e))?;
    Ok(json!({"content": content}))
}

pub(crate) fn tool(vault: Arc<Vault>) -> GenericTool {
    GenericTool {
        vault,
        handler: handler as ToolHandler,
        name: "read_note",
        description: "Read the content of a single note",
        input: InputSchema::new(&[
            ParamDef { name: "path", description: "Relative path to the note", required: true },
        ]),
    }
}
