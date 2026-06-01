use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct GetTagsTool {
    vault: Arc<Vault>,
}

impl GetTagsTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        GetTagsTool { vault }
    }
}

impl Tool for GetTagsTool {
    fn call(&self, _params: Option<Value>) -> Result<Value, (i32, String)> {
        let tags = self.vault.get_all_tags();
        let items: Vec<Value> = tags.into_iter().map(|tag| {
            let count = self.vault.index.tags.get(&tag).len();
            json!({"tag": tag, "count": count})
        }).collect();
        Ok(json!(items))
    }
}
