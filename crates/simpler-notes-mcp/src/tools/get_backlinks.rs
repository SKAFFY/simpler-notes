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

    let target = PathBuf::from(path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());
    let target_buf = PathBuf::from(&target);
    let backlinks = vault.get_backlinks(&target_buf);
    let items: Vec<Value> = backlinks.into_iter().map(|e| {
        json!({
            "source": e.source.to_string_lossy(),
            "target": e.target.to_string_lossy(),
            "label": e.label,
        })
    }).collect();
    Ok(json!(items))
}

pub(crate) fn tool(vault: Arc<Vault>) -> GenericTool {
    GenericTool {
        vault,
        handler: handler as ToolHandler,
        name: "get_backlinks",
        description: "Get all notes that link to a given target note",
        input: InputSchema::new(&[
            ParamDef { name: "path", description: "Path to the target note", required: true },
        ]),
    }
}
