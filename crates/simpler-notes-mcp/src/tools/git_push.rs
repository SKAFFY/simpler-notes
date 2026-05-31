use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use simpler_notes_core::git::GitBackend;
use crate::dispatcher::Tool;

pub struct GitPushTool {
    vault: Arc<Vault>,
}

impl GitPushTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        GitPushTool { vault }
    }
}

impl Tool for GitPushTool {
    fn call(&self, _params: Option<Value>) -> Result<Value, (i32, String)> {
        let git = GitBackend::open(&self.vault.config.path)
            .map_err(|e| (-1, e))?;

        git.stage_all().map_err(|e| (-1, e))?;

        if git.is_dirty().map_err(|e| (-1, e))? {
            git.commit("sync: manual push").map_err(|e| (-1, e))?;
        }

        git.pull().map_err(|e| (-1, e))?;

        git.push().map_err(|e| (-1, e))?;

        Ok(json!({"ok": true}))
    }
}
