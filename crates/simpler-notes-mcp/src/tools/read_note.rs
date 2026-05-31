use std::sync::Arc;
use std::path::PathBuf;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct ReadNoteTool {
    vault: Arc<Vault>,
}

impl ReadNoteTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        ReadNoteTool { vault }
    }
}

impl Tool for ReadNoteTool {
    fn call(&self, params: Option<Value>) -> Result<Value, (i32, String)> {
        let path = params
            .and_then(|p| p.get("path").and_then(|q| q.as_str().map(|s| s.to_string())))
            .ok_or((-32602, "Missing required parameter: path".to_string()))?;

        let content = self.vault.read_note(&PathBuf::from(&path))
            .map_err(|e| (-1, e))?;
        Ok(json!({"content": content}))
    }
}
