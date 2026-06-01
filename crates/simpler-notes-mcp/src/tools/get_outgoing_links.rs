use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct GetOutgoingLinksTool {
    vault: Arc<Vault>,
}

impl GetOutgoingLinksTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        GetOutgoingLinksTool { vault }
    }
}

impl Tool for GetOutgoingLinksTool {
    fn call(&self, params: Option<Value>) -> Result<Value, (i32, String)> {
        let p = params.ok_or((-32602, "Missing parameters".to_string()))?;
        let path = p.get("path")
            .and_then(|v| v.as_str())
            .ok_or((-32602, "Missing required parameter: path".to_string()))?;
        // Sources are stored as full paths, resolve relative to vault root
        let source = self.vault.config.path.join(path);
        let outgoing = self.vault.get_outgoing_links(&source);
        let items: Vec<Value> = outgoing.into_iter().map(|e| {
            json!({
                "source": e.source.to_string_lossy(),
                "target": e.target.to_string_lossy(),
                "label": e.label,
            })
        }).collect();
        Ok(json!(items))
    }
}
