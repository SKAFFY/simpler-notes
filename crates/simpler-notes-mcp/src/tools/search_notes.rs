use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::tool::{GenericTool, ToolHandler, ToolResult, InputSchema, ParamDef};

pub(crate) fn handler(vault: &Vault, params: Option<Value>) -> ToolResult {
    let query = params
        .and_then(|p| p.get("query").and_then(|q| q.as_str().map(|s| s.to_string())))
        .ok_or((-32602, "Missing required parameter: query".to_string()))?;

    let results = vault.search(&query).map_err(|e| (-1, e))?;
    let items: Vec<Value> = results.into_iter()
        .map(|r| json!({"path": r.path.to_string_lossy(), "title": r.title}))
        .collect();
    Ok(json!(items))
}

pub(crate) fn tool(vault: Arc<Vault>) -> GenericTool {
    GenericTool {
        vault,
        handler: handler as ToolHandler,
        name: "search_notes",
        description: "Search notes in the vault by full-text query",
        input: InputSchema::new(&[
            ParamDef { name: "query", description: "Search query string", required: true },
        ]),
    }
}
