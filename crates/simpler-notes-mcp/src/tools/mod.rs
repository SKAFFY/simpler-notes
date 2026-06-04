use std::sync::Arc;
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Dispatcher;

pub mod search_notes;
pub mod read_note;
pub mod write_note;
pub mod list_notes;
pub mod get_tags;
pub mod get_dates;
pub mod get_backlinks;
pub mod get_outgoing_links;
pub mod resolve_link;
#[cfg(feature = "git")]
pub mod git_push;
#[cfg(feature = "git")]
pub mod git_pull;
pub mod validate_indexes;
pub mod reindex;
pub mod get_diagnostics;

pub fn register_all(dispatcher: &mut Dispatcher, vault: Arc<Vault>) {
    dispatcher.register_generic(search_notes::tool(vault.clone()));
    dispatcher.register_generic(read_note::tool(vault.clone()));
    dispatcher.register_generic(write_note::tool(vault.clone()));
    dispatcher.register_generic(list_notes::tool(vault.clone()));
    dispatcher.register_generic(get_tags::tool(vault.clone()));
    dispatcher.register_generic(get_dates::tool(vault.clone()));
    dispatcher.register_generic(get_backlinks::tool(vault.clone()));
    dispatcher.register_generic(get_outgoing_links::tool(vault.clone()));
    dispatcher.register_generic(resolve_link::tool(vault.clone()));
    #[cfg(feature = "git")]
    dispatcher.register_generic(git_push::tool(vault.clone()));
    #[cfg(feature = "git")]
    dispatcher.register_generic(git_pull::tool(vault.clone()));
    dispatcher.register_generic(validate_indexes::tool(vault.clone()));
    dispatcher.register_generic(reindex::tool(vault.clone()));
    dispatcher.register_generic(get_diagnostics::tool(vault));
}
