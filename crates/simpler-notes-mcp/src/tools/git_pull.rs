use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use simpler_notes_core::git::GitBackend;
use crate::dispatcher::Tool;

pub struct GitPullTool {
    vault: Arc<Vault>,
}

impl GitPullTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        GitPullTool { vault }
    }
}

impl Tool for GitPullTool {
    fn call(&self, _params: Option<Value>) -> Result<Value, (i32, String)> {
        let git = GitBackend::open(&self.vault.config.path)
            .map_err(|e| (-1, e))?;

        git.pull().map_err(|e| (-1, e))?;

        Ok(json!({"ok": true}))
    }
}
