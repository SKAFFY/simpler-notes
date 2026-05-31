use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct SearchNotesTool {
    #[allow(dead_code)]
    #[allow(dead_code)]
    vault: Arc<Vault>,
}

impl SearchNotesTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        SearchNotesTool { vault }
    }
}

impl Tool for SearchNotesTool {
    fn call(&self, _params: Option<Value>) -> Result<Value, (i32, String)> {
        Ok(json!([]))
    }
}
