use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct WriteNoteTool {
    #[allow(dead_code)]
    vault: Arc<Vault>,
}

impl WriteNoteTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        WriteNoteTool { vault }
    }
}

impl Tool for WriteNoteTool {
    fn call(&self, _params: Option<Value>) -> Result<Value, (i32, String)> {
        Ok(json!({}))
    }
}
