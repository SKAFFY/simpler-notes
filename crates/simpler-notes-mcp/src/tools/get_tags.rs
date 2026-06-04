use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::tool::{GenericTool, ToolHandler, ToolResult, InputSchema};

pub(crate) fn handler(vault: &Vault, _params: Option<Value>) -> ToolResult {
    let tags = vault.get_all_tags();
    let items: Vec<Value> = tags.into_iter().map(|tag| {
        let count = vault.index.tags.get(&tag).len();
        json!({"tag": tag, "count": count})
    }).collect();
    Ok(json!(items))
}

pub(crate) fn tool(vault: Arc<Vault>) -> GenericTool {
    GenericTool {
        vault,
        handler: handler as ToolHandler,
        name: "get_tags",
        description: "Get all tags across the vault with their occurrence counts",
        input: InputSchema::new(&[]),
    }
}
