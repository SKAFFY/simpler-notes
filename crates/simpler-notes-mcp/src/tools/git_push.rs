use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct GitPushTool {
    #[allow(dead_code)]
    vault: Arc<Vault>,
}

impl GitPushTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        GitPushTool { vault }
    }
}

impl Tool for GitPushTool {
    fn call(&self, _params: Option<Value>) -> Result<Value, (i32, String)> {
        Ok(json!({}))
    }
}
