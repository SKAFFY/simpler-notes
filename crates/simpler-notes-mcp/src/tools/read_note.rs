use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct ReadNoteTool {
    #[allow(dead_code)]
    vault: Arc<Vault>,
}

impl ReadNoteTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        ReadNoteTool { vault }
    }
}

impl Tool for ReadNoteTool {
    fn call(&self, _params: Option<Value>) -> Result<Value, (i32, String)> {
        Ok(json!({}))
    }
}
