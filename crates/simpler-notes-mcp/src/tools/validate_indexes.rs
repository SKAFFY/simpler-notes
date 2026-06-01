use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct ValidateIndexesTool {
    vault: Arc<Vault>,
}

impl ValidateIndexesTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        ValidateIndexesTool { vault }
    }
}

impl Tool for ValidateIndexesTool {
    fn call(&self, _params: Option<Value>) -> Result<Value, (i32, String)> {
        let report = self.vault.validate_indexes();
        Ok(json!({
            "total_notes": report.total_notes,
            "total_tags": report.total_tags,
            "total_dates": report.total_dates,
        }))
    }
}
