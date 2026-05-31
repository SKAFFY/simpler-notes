use std::sync::Arc;
use std::path::PathBuf;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct GetDiagnosticsTool {
    vault: Arc<Vault>,
}

impl GetDiagnosticsTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        GetDiagnosticsTool { vault }
    }
}

impl Tool for GetDiagnosticsTool {
    fn call(&self, params: Option<Value>) -> Result<Value, (i32, String)> {
        let single_path = params
            .and_then(|p| p.get("path").and_then(|v| v.as_str().map(|s| PathBuf::from(s))));

        if let Some(path) = single_path {
            let diags = self.vault.get_diagnostics(&path);
            return Ok(json!({"diagnostics": diags}));
        }

        let all = self.vault.all_diagnostics();
        let files: Vec<Value> = all.into_iter().map(|(path, diags)| {
            json!({"path": path.to_string_lossy(), "diagnostics": diags})
        }).collect();
        Ok(json!({"files": files}))
    }
}
