use std::sync::Arc;
use std::path::PathBuf;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct WriteNoteTool {
    vault: Arc<Vault>,
}

impl WriteNoteTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        WriteNoteTool { vault }
    }
}

impl Tool for WriteNoteTool {
    fn call(&self, params: Option<Value>) -> Result<Value, (i32, String)> {
        let p = params.ok_or((-32602, "Missing parameters".to_string()))?;
        let path = p.get("path")
            .and_then(|v| v.as_str())
            .ok_or((-32602, "Missing required parameter: path".to_string()))?;
        let content = p.get("content")
            .and_then(|v| v.as_str())
            .ok_or((-32602, "Missing required parameter: content".to_string()))?;

        self.vault.write_note(&PathBuf::from(path), content)
            .map_err(|e| (-1, e))?;
        Ok(json!({"ok": true}))
    }
}
