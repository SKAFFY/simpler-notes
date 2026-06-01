use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct ResolveLinkTool {
    vault: Arc<Vault>,
}

impl ResolveLinkTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        ResolveLinkTool { vault }
    }
}

impl Tool for ResolveLinkTool {
    fn call(&self, params: Option<Value>) -> Result<Value, (i32, String)> {
        let p = params.ok_or((-32602, "Missing parameters".to_string()))?;
        let target = p.get("target")
            .and_then(|v| v.as_str())
            .ok_or((-32602, "Missing required parameter: target".to_string()))?;
        match self.vault.resolve_link(target) {
            Ok(path) => Ok(json!({"path": path.to_string_lossy()})),
            Err(e) => Err((-32000, e)),
        }
    }
}
