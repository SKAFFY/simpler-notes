use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct GetTagsTool {
    #[allow(dead_code)]
    vault: Arc<Vault>,
}

impl GetTagsTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        GetTagsTool { vault }
    }
}

impl Tool for GetTagsTool {
    fn call(&self, _params: Option<Value>) -> Result<Value, (i32, String)> {
        Ok(json!([]))
    }
}
