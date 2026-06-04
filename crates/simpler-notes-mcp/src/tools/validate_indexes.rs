use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::tool::{GenericTool, ToolHandler, ToolResult, InputSchema};

pub(crate) fn handler(vault: &Vault, _params: Option<Value>) -> ToolResult {
    let report = vault.validate_indexes();
    Ok(json!({
        "total_notes": report.total_notes,
        "total_tags": report.total_tags,
        "total_dates": report.total_dates,
    }))
}

pub(crate) fn tool(vault: Arc<Vault>) -> GenericTool {
    GenericTool {
        vault,
        handler: handler as ToolHandler,
        name: "validate_indexes",
        description: "Validate the integrity of all indexed data (notes, tags, dates)",
        input: InputSchema::new(&[]),
    }
}
