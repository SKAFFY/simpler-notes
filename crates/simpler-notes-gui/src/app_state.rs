use std::path::PathBuf;

use gpui::Context;

#[derive(PartialEq, Clone, Copy)]
pub enum EditorMode {
    Source,
    Split,
    Preview,
}

pub struct OpenTab {
    pub path: PathBuf,
    pub title: String,
    pub source_content: String,
}

pub struct AppState {
    pub vault_path: Option<PathBuf>,
    pub open_tabs: Vec<OpenTab>,
    pub active_tab: Option<usize>,
    pub editor_mode: EditorMode,
    pub collapsed: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            vault_path: None,
            open_tabs: Vec::new(),
            active_tab: None,
            editor_mode: EditorMode::Source,
            collapsed: false,
        }
    }

    pub fn toggle_collapsed(&mut self, _cx: &mut Context<Self>) {
        self.collapsed = !self.collapsed;
        _cx.notify();
    }

    pub fn open_vault(&mut self, path: &PathBuf, _cx: &mut Context<Self>) {
        self.vault_path = Some(path.clone());
        _cx.notify();
    }

    pub fn list_markdown_files(&self) -> Vec<PathBuf> {
        match &self.vault_path {
            Some(vault_path) => {
                walkdir::WalkDir::new(vault_path)
                    .max_depth(1)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.file_type().is_file()
                            && e.path().extension().map(|e| e == "md").unwrap_or(false)
                    })
                    .map(|e| e.path().to_owned())
                    .collect()
            }
            None => Vec::new(),
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
            }
            None => {
                let content = std::fs::read_to_string(&path).unwrap_or_default();
                self.open_tabs.push(OpenTab {
                    path,
                    title,
                    source_content: content,
                });
                self.active_tab = Some(self.open_tabs.len() - 1);
            }
        }
        self.editor_mode = EditorMode::Source;
        cx.notify();
    }

    pub fn select_tab(&mut self, idx: usize, _cx: &mut Context<Self>) {
        self.active_tab = Some(idx);
        _cx.notify();
    }

    pub fn close_tab(&mut self, idx: usize, cx: &mut Context<Self>) {
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
        cx.notify();
    }
}
