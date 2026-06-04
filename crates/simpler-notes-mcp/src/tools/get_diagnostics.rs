use std::sync::Arc;
use std::path::PathBuf;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::tool::{GenericTool, ToolHandler, ToolResult, InputSchema, ParamDef};

pub(crate) fn handler(vault: &Vault, params: Option<Value>) -> ToolResult {
    let single_path = params
        .and_then(|p| p.get("path").and_then(|v| v.as_str().map(PathBuf::from)));

    if let Some(path) = single_path {
        let diags = vault.get_diagnostics(&path);
        return Ok(json!({"diagnostics": diags}));
    }

    let all = vault.all_diagnostics();
    let files: Vec<Value> = all.into_iter().map(|(path, diags)| {
        json!({"path": path.to_string_lossy(), "diagnostics": diags})
    }).collect();
    Ok(json!({"files": files}))
}

pub(crate) fn tool(vault: Arc<Vault>) -> GenericTool {
    GenericTool {
        vault,
        handler: handler as ToolHandler,
        name: "get_diagnostics",
        description: "Get diagnostics (broken links, parse errors) for all notes or a specific note",
        input: InputSchema::new(&[
            ParamDef { name: "path", description: "Path to a specific note (optional, returns all diagnostics if omitted)", required: false },
        ]),
    }
}
