use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct ListNotesTool {
    vault: Arc<Vault>,
}

impl ListNotesTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        ListNotesTool { vault }
    }
}

impl Tool for ListNotesTool {
    fn call(&self, params: Option<Value>) -> Result<Value, (i32, String)> {
        let subdir = params
            .and_then(|p| p.get("path").and_then(|v| v.as_str().map(|s| s.to_string())));

        let base = self.vault.config.path.clone();
        let search_path = match &subdir {
            Some(s) => base.join(s),
            None => base.clone(),
        };

        let mut items = Vec::new();
        if search_path.is_dir() {
            for entry in std::fs::read_dir(&search_path).map_err(|e| (-1, e.to_string()))? {
                let entry = entry.map_err(|e| (-1, e.to_string()))?;
                let name = entry.file_name().to_string_lossy().to_string();
                let file_type = if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    "directory"
                } else {
                    "file"
                };
                items.push(json!({"name": name, "type": file_type}));
            }
        }
        Ok(json!(items))
    }
}
