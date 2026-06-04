use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use simpler_notes_core::git::GitBackend;
use crate::tool::{GenericTool, ToolHandler, ToolResult, InputSchema};

pub(crate) fn handler(vault: &Vault, _params: Option<Value>) -> ToolResult {
    let git = GitBackend::open(&vault.config.path)
        .map_err(|e| (-1, e))?;

    git.stage_all().map_err(|e| (-1, e))?;

    if git.is_dirty().map_err(|e| (-1, e))? {
        git.commit("sync: manual push").map_err(|e| (-1, e))?;
    }

    git.pull().map_err(|e| (-1, e))?;

    git.push().map_err(|e| (-1, e))?;

    Ok(json!({"ok": true}))
}

pub(crate) fn tool(vault: Arc<Vault>) -> GenericTool {
    GenericTool {
        vault,
        handler: handler as ToolHandler,
        name: "git_push",
        description: "Stage all changes, commit, pull, and push to the remote git repository",
        input: InputSchema::new(&[]),
    }
}
