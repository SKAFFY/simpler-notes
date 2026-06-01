use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct ReindexTool {
    vault: Arc<Vault>,
}

impl ReindexTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        ReindexTool { vault }
    }
}

impl Tool for ReindexTool {
    fn call(&self, _params: Option<Value>) -> Result<Value, (i32, String)> {
        self.vault.index.clear();
        let report = self.vault.reindex_all()
            .map_err(|e| (-1, e))?;
        Ok(json!({
            "ok": true,
            "total_notes": report.total_notes,
            "total_tags": report.total_tags,
            "total_dates": report.total_dates,
        }))
    }
}
