use std::sync::Arc;
use std::path::PathBuf;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::tool::{GenericTool, ToolHandler, ToolResult, InputSchema, ParamDef};

pub(crate) fn handler(vault: &Vault, params: Option<Value>) -> ToolResult {
    let p = params.ok_or((-32602, "Missing parameters".to_string()))?;
    let path = p.get("path")
        .and_then(|v| v.as_str())
        .ok_or((-32602, "Missing required parameter: path".to_string()))?;
    let content = p.get("content")
        .and_then(|v| v.as_str())
        .ok_or((-32602, "Missing required parameter: content".to_string()))?;

    vault.write_note(&PathBuf::from(path), content)
        .map_err(|e| (-1, e))?;
    Ok(json!({"ok": true}))
}

pub(crate) fn tool(vault: Arc<Vault>) -> GenericTool {
    GenericTool {
        vault,
        handler: handler as ToolHandler,
        name: "write_note",
        description: "Create or overwrite a note with the given content",
        input: InputSchema::new(&[
            ParamDef { name: "path", description: "Relative path to the note", required: true },
            ParamDef { name: "content", description: "Markdown content to write", required: true },
        ]),
    }
}
