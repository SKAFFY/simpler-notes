use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::tool::{GenericTool, ToolHandler, ToolResult, InputSchema};

pub(crate) fn handler(vault: &Vault, _params: Option<Value>) -> ToolResult {
    vault.index.clear();
    let report = vault.reindex_all()
        .map_err(|e| (-1, e))?;
    Ok(json!({
        "ok": true,
        "total_notes": report.total_notes,
        "total_tags": report.total_tags,
        "total_dates": report.total_dates,
    }))
}

pub(crate) fn tool(vault: Arc<Vault>) -> GenericTool {
    GenericTool {
        vault,
        handler: handler as ToolHandler,
        name: "reindex",
        description: "Clear all indexes and rebuild them from scratch",
        input: InputSchema::new(&[]),
    }
}
