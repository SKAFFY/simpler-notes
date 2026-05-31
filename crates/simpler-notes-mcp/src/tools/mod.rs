use std::sync::Arc;
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Dispatcher;

pub mod search_notes;
pub mod read_note;
pub mod write_note;
pub mod list_notes;
pub mod get_tags;
pub mod get_dates;
pub mod git_push;
pub mod git_pull;
pub mod validate_indexes;
pub mod reindex;
pub mod get_diagnostics;

pub fn register_all(dispatcher: &mut Dispatcher, vault: Arc<Vault>) {
    dispatcher.register("search_notes", Arc::new(search_notes::SearchNotesTool::new(vault.clone())));
    dispatcher.register("read_note", Arc::new(read_note::ReadNoteTool::new(vault.clone())));
    dispatcher.register("write_note", Arc::new(write_note::WriteNoteTool::new(vault.clone())));
    dispatcher.register("list_notes", Arc::new(list_notes::ListNotesTool::new(vault.clone())));
    dispatcher.register("get_tags", Arc::new(get_tags::GetTagsTool::new(vault.clone())));
    dispatcher.register("get_dates", Arc::new(get_dates::GetDatesTool::new(vault.clone())));
    dispatcher.register("git_push", Arc::new(git_push::GitPushTool::new(vault.clone())));
    dispatcher.register("git_pull", Arc::new(git_pull::GitPullTool::new(vault.clone())));
    dispatcher.register("validate_indexes", Arc::new(validate_indexes::ValidateIndexesTool::new(vault.clone())));
    dispatcher.register("reindex", Arc::new(reindex::ReindexTool::new(vault.clone())));
    dispatcher.register("get_diagnostics", Arc::new(get_diagnostics::GetDiagnosticsTool::new(vault)));
}
