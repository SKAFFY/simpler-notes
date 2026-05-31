use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct ListNotesTool {
    #[allow(dead_code)]
    vault: Arc<Vault>,
}

impl ListNotesTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        ListNotesTool { vault }
    }
}

impl Tool for ListNotesTool {
    fn call(&self, _params: Option<Value>) -> Result<Value, (i32, String)> {
        Ok(json!([]))
    }
}
