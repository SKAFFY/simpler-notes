use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::tool::{GenericTool, ToolHandler, ToolResult, InputSchema, ParamDef};

pub(crate) fn handler(vault: &Vault, params: Option<Value>) -> ToolResult {
    let subdir = params
        .and_then(|p| p.get("path").and_then(|v| v.as_str().map(|s| s.to_string())));

    let base = vault.config.path.clone();
    let search_path = match &subdir {
        Some(s) => base.join(s),
        None => base.clone(),
    };

    let mut items = Vec::new();
    if search_path.is_dir() {
        for entry in std::fs::read_dir(&search_path).map_err(|e| (-1, e.to_string()))? {
            let entry = entry.map_err(|e| (-1, e.to_string()))?;
            let name = entry.file_name().to_string_lossy().to_string();
            let file_type = if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                "directory"
            } else {
                "file"
            };
            items.push(json!({"name": name, "type": file_type}));
        }
    }
    Ok(json!(items))
}

pub(crate) fn tool(vault: Arc<Vault>) -> GenericTool {
    GenericTool {
        vault,
        handler: handler as ToolHandler,
        name: "list_notes",
        description: "List files and directories within a vault path",
        input: InputSchema::new(&[
            ParamDef { name: "path", description: "Path relative to vault root (optional, defaults to vault root)", required: false },
        ]),
    }
}
