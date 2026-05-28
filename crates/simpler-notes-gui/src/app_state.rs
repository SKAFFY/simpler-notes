use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui::Context;
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

impl AppState {
    pub fn new() -> Self {
        Self {
            vault: None,
            vault_path: None,
            open_tabs: Vec::new(),
            active_tab: None,
            editor_mode: EditorMode::Source,
            sidebar_visible: true,
            search_query: String::new(),
        }
    }

    pub fn set_editor_mode(&mut self, mode: EditorMode, _cx: &mut Context<Self>) {
        self.editor_mode = mode;
        _cx.notify();
    }

    pub fn set_search_query(&mut self, query: &str, _cx: &mut Context<Self>) {
        self.search_query = query.to_string();
        _cx.notify();
    }

    pub fn open_vault(&mut self, path: &Path, cx: &mut Context<Self>) {
        match Vault::open(path) {
            Ok(vault) => {
                self.vault = Some(Arc::new(vault));
                self.vault_path = Some(path.to_owned());
                cx.notify();
            }
            Err(e) => {
                eprintln!("Failed to open vault: {}", e);
            }
        }
    }

    pub fn open_file(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let title = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let already_open = self.open_tabs.iter().position(|t| t.path == path);
        match already_open {
            Some(idx) => {
                self.active_tab = Some(idx);
                self.editor_mode = EditorMode::Source;
            }
            None => {
                let content = std::fs::read_to_string(&path).unwrap_or_default();
                self.open_tabs.push(OpenTab {
                    path: path.clone(),
                    title,
                    content_dirty: false,
                    source_content: content,
                });
                self.active_tab = Some(self.open_tabs.len() - 1);
                self.editor_mode = EditorMode::Source;
            }
        }
        cx.notify();
    }

    pub fn close_tab(&mut self, idx: usize, _cx: &mut Context<Self>) {
        if idx < self.open_tabs.len() {
            self.open_tabs.remove(idx);
            match self.active_tab {
                Some(active) if active == idx => {
                    self.active_tab = if self.open_tabs.is_empty() {
                        None
                    } else {
                        Some(active.min(self.open_tabs.len() - 1))
                    };
                }
                Some(active) if active > idx => {
                    self.active_tab = Some(active - 1);
                }
                _ => {}
            }
        }
        _cx.notify();
    }
}
