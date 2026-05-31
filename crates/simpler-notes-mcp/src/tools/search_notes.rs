use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct SearchNotesTool {
    vault: Arc<Vault>,
}

impl SearchNotesTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        SearchNotesTool { vault }
    }
}

impl Tool for SearchNotesTool {
    fn call(&self, params: Option<Value>) -> Result<Value, (i32, String)> {
        let query = params
            .and_then(|p| p.get("query").and_then(|q| q.as_str().map(|s| s.to_string())))
            .ok_or((-32602, "Missing required parameter: query".to_string()))?;

        let results = self.vault.search(&query).map_err(|e| (-1, e))?;
        let items: Vec<Value> = results.into_iter()
            .map(|r| json!({"path": r.path.to_string_lossy(), "title": r.title}))
            .collect();
        Ok(json!(items))
    }
}
