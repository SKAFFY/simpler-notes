use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct GitPullTool {
    #[allow(dead_code)]
    vault: Arc<Vault>,
}

impl GitPullTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        GitPullTool { vault }
    }
}

impl Tool for GitPullTool {
    fn call(&self, _params: Option<Value>) -> Result<Value, (i32, String)> {
        Ok(json!({}))
    }
}
