use std::path::PathBuf;
use std::sync::Arc;

use simpler_notes_core::vault::Vault;

#[derive(PartialEq, Clone, Copy)]
pub enum EditorMode {
    Source,
    Split,
    Preview,
}

pub struct OpenTab {
    pub path: PathBuf,
    pub title: String,
    pub content_dirty: bool,
    pub source_content: String,
}

pub struct AppState {
    pub vault: Option<Arc<Vault>>,
    pub vault_path: Option<PathBuf>,
    pub open_tabs: Vec<OpenTab>,
    pub active_tab: Option<usize>,
    pub editor_mode: EditorMode,
    pub sidebar_visible: bool,
    pub search_query: String,
}
