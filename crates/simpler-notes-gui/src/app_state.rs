use std::path::PathBuf;

use gpui::Context;
use simpler_notes_core::vault::{Vault, VaultConfig};

#[derive(PartialEq, Clone, Copy)]
pub enum EditorMode {
    Source,
    Split,
    Preview,
}

#[derive(PartialEq, Clone, Copy)]
pub enum LowerPanelTab {
    Search,
    Timeline,
    Graph,
    Diagnostics,
}

#[derive(Clone)]
pub struct OpenTab {
    pub path: PathBuf,
    pub title: String,
}

pub struct AppState {
    pub vault: Option<Box<Vault>>,
    pub open_tabs: Vec<OpenTab>,
    pub active_tab: Option<usize>,
    pub editor_mode: EditorMode,
    pub project_panel_visible: bool,
    pub project_panel_width: f32,
    pub search_query: String,
    pub lower_panel_visible: bool,
    pub lower_panel_active_tab: LowerPanelTab,
    pub lower_panel_height: f32,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            vault: None,
            open_tabs: Vec::new(),
            active_tab: None,
            editor_mode: EditorMode::Source,
            project_panel_visible: true,
            project_panel_width: 250.0,
            search_query: String::new(),
            lower_panel_visible: false,
            lower_panel_active_tab: LowerPanelTab::Search,
            lower_panel_height: 200.0,
        }
    }

    pub fn open_vault(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let config = VaultConfig {
            path,
            ..Default::default()
        };
        match Vault::open(config) {
            Ok(vault) => {
                self.vault = Some(Box::new(vault));
                self.open_tabs.clear();
                self.active_tab = None;
                cx.notify();
            }
            Err(e) => {
                eprintln!("Failed to open vault: {}", e);
            }
        }
    }

    pub fn open_file(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let already_open = self.open_tabs.iter().position(|t| t.path == path);
        match already_open {
            Some(idx) => {
                self.active_tab = Some(idx);
            }
            None => {
                let title = path
                    .file_stem()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                if let Some(vault) = &self.vault {
                    let content = std::fs::read_to_string(&path).unwrap_or_default();
                    vault.buffer.write().open(&path, content);
                }

                self.open_tabs.push(OpenTab {
                    path: path.clone(),
                    title,
                });
                self.active_tab = Some(self.open_tabs.len() - 1);
            }
        }
        self.editor_mode = EditorMode::Source;
        cx.notify();
    }

    pub fn active_tab_path(&self) -> Option<PathBuf> {
        self.active_tab
            .and_then(|idx| self.open_tabs.get(idx))
            .map(|t| t.path.clone())
    }

    pub fn select_tab(&mut self, idx: usize, cx: &mut Context<Self>) {
        if idx < self.open_tabs.len() {
            self.active_tab = Some(idx);
            cx.notify();
        }
    }

    pub fn close_tab(&mut self, idx: usize, cx: &mut Context<Self>) {
        if idx < self.open_tabs.len() {
            let tab = self.open_tabs.remove(idx);

            if let Some(vault) = &self.vault {
                vault.buffer.write().close(&tab.path);
            }

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

    pub fn close_vault(&mut self, cx: &mut Context<Self>) {
        self.vault = None;
        self.open_tabs.clear();
        self.active_tab = None;
        cx.notify();
    }

    pub fn cycle_editor_mode(&mut self, target: EditorMode, cx: &mut Context<Self>) {
        self.editor_mode = if self.editor_mode == target {
            EditorMode::Source
        } else {
            target
        };
        cx.notify();
    }

    pub fn toggle_project_panel(&mut self, cx: &mut Context<Self>) {
        self.project_panel_visible = !self.project_panel_visible;
        cx.notify();
    }

    pub fn toggle_lower_panel(&mut self, cx: &mut Context<Self>) {
        self.lower_panel_visible = !self.lower_panel_visible;
        cx.notify();
    }

    pub fn set_lower_panel_tab(&mut self, tab: LowerPanelTab, cx: &mut Context<Self>) {
        if self.lower_panel_active_tab == tab && self.lower_panel_visible {
            self.lower_panel_visible = false;
        } else {
            self.lower_panel_active_tab = tab;
            self.lower_panel_visible = true;
        }
        cx.notify();
    }
}
