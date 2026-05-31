use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct GetDatesTool {
    #[allow(dead_code)]
    vault: Arc<Vault>,
}

impl GetDatesTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        GetDatesTool { vault }
    }
}

impl Tool for GetDatesTool {
    fn call(&self, _params: Option<Value>) -> Result<Value, (i32, String)> {
        Ok(json!([]))
    }
}
