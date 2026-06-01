use std::sync::Arc;
use std::path::PathBuf;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct GetBacklinksTool {
    vault: Arc<Vault>,
}

impl GetBacklinksTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        GetBacklinksTool { vault }
    }
}

impl Tool for GetBacklinksTool {
    fn call(&self, params: Option<Value>) -> Result<Value, (i32, String)> {
        let p = params.ok_or((-32602, "Missing parameters".to_string()))?;
        let path = p.get("path")
            .and_then(|v| v.as_str())
            .ok_or((-32602, "Missing required parameter: path".to_string()))?;
        // Links are stored with target = flattened file_stem (e.g., "gamma")
        let target = PathBuf::from(path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());
        let target_buf = PathBuf::from(&target);
        let backlinks = self.vault.get_backlinks(&target_buf);
        let items: Vec<Value> = backlinks.into_iter().map(|e| {
            json!({
                "source": e.source.to_string_lossy(),
                "target": e.target.to_string_lossy(),
                "label": e.label,
            })
        }).collect();
        Ok(json!(items))
    }
}
