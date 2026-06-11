use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use iced::widget::text_editor;
use iced::{Element, Task, Theme};

use simpler_notes_core::vault::{Vault, VaultConfig, VaultSearchResult};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditorMode {
    Source,
    Split,
    Preview,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LowerPanelTab {
    Search,
    Diagnostics,
}

#[derive(Debug, Clone)]
pub struct OpenTab {
    pub path: PathBuf,
    pub title: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenVault,
    VaultOpened(Result<PathBuf, String>),
    CloseVault,
    ToggleProjectPanel,
    TogglePreview,
    ToggleLowerPanel,
    SelectLowerTab(LowerPanelTab),
    FileSelected(PathBuf),
    TabSelected(usize),
    TabClosed(usize),
    EditorAction(text_editor::Action),
    SearchQueryChanged(String),
    Search,
    SearchResultsUpdated(Result<Vec<VaultSearchResult>, String>),
    CloseCurrentTab,
    Save,
    ToggleDir(PathBuf),
    LinkClicked(String),
    SearchResultClicked(PathBuf),
}

pub struct App {
    pub vault: Option<Arc<Vault>>,
    pub project_panel_visible: bool,
    pub editor_mode: EditorMode,
    pub open_tabs: Vec<OpenTab>,
    pub active_tab: Option<usize>,
    pub editors: HashMap<PathBuf, text_editor::Content>,
    pub search_query: String,
    pub search_results: Vec<VaultSearchResult>,
    pub search_error: Option<String>,
    pub lower_panel_visible: bool,
    pub lower_panel_active_tab: LowerPanelTab,
    pub expanded_dirs: HashSet<PathBuf>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                vault: None,
                project_panel_visible: true,
                editor_mode: EditorMode::Source,
                open_tabs: Vec::new(),
                active_tab: None,
                editors: HashMap::new(),
                search_query: String::new(),
                search_results: Vec::new(),
                search_error: None,
                lower_panel_visible: false,
                lower_panel_active_tab: LowerPanelTab::Search,
                expanded_dirs: HashSet::new(),
            },
            Task::none(),
        )
    }

    pub fn is_vault_open(&self) -> bool {
        self.vault.is_some()
    }

    pub fn active_tab_path(&self) -> Option<PathBuf> {
        self.active_tab
            .and_then(|idx| self.open_tabs.get(idx))
            .map(|t| t.path.clone())
    }

    pub fn active_editor(&self) -> Option<&text_editor::Content> {
        self.active_tab_path()
            .and_then(|path| self.editors.get(&path))
    }

    pub fn active_editor_text(&self) -> String {
        self.active_editor()
            .map(|e| e.text())
            .unwrap_or_default()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenVault => Task::perform(open_vault_dialog(), Message::VaultOpened),
            Message::VaultOpened(result) => {
                match result {
                    Ok(path) => {
                        let config = VaultConfig {
                            path,
                            ..Default::default()
                        };
                        match Vault::open(config) {
                            Ok(vault) => {
                                self.vault = Some(Arc::new(vault));
                            }
                            Err(e) => {
                                eprintln!("Failed to open vault: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Vault dialog error: {}", e);
                    }
                }
                Task::none()
            }
            Message::CloseVault => {
                self.vault = None;
                self.open_tabs.clear();
                self.active_tab = None;
                self.editors.clear();
                self.search_query.clear();
                self.lower_panel_visible = false;
                Task::none()
            }
            Message::ToggleProjectPanel => {
                self.project_panel_visible = !self.project_panel_visible;
                Task::none()
            }
            Message::TogglePreview => {
                self.editor_mode = match self.editor_mode {
                    EditorMode::Source => EditorMode::Split,
                    EditorMode::Split => EditorMode::Preview,
                    EditorMode::Preview => EditorMode::Source,
                };
                Task::none()
            }
            Message::ToggleLowerPanel => {
                self.lower_panel_visible = !self.lower_panel_visible;
                Task::none()
            }
            Message::SelectLowerTab(tab) => {
                self.lower_panel_active_tab = tab;
                self.lower_panel_visible = true;
                Task::none()
            }
            Message::FileSelected(path) => {
                let already_open = self.open_tabs.iter().position(|t| t.path == path);
                match already_open {
                    Some(idx) => {
                        self.active_tab = Some(idx);
                    }
                    None => {
                        if !self.editors.contains_key(&path) {
                            let content = std::fs::read_to_string(&path).unwrap_or_default();
                            self.editors
                                .insert(path.clone(), text_editor::Content::with_text(&content));
                        }
                        let title = path
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default();
                        self.open_tabs.push(OpenTab { path, title });
                        self.active_tab = Some(self.open_tabs.len() - 1);
                    }
                }
                Task::none()
            }
            Message::TabSelected(idx) => {
                if idx < self.open_tabs.len() {
                    self.active_tab = Some(idx);
                }
                Task::none()
            }
            Message::TabClosed(idx) => {
                if idx < self.open_tabs.len() {
                    let tab = &self.open_tabs[idx];
                    self.editors.remove(&tab.path);
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
                Task::none()
            }
            Message::EditorAction(action) => {
                if let Some(path) = self.active_tab_path() {
                    if let Some(editor) = self.editors.get_mut(&path) {
                        editor.perform(action);
                    }
                }
                Task::none()
            }
            Message::SearchQueryChanged(query) => {
                self.search_query = query;
                Task::none()
            }
            Message::Search => {
                if let Some(ref vault) = self.vault {
                    let query = self.search_query.clone();
                    let v = vault.clone();
                    Task::perform(
                        async move { v.search(&query) },
                        Message::SearchResultsUpdated,
                    )
                } else {
                    Task::none()
                }
            }
            Message::SearchResultsUpdated(result) => {
                match result {
                    Ok(results) => {
                        self.search_results = results;
                        self.search_error = None;
                    }
                    Err(e) => {
                        self.search_error = Some(e);
                    }
                }
                Task::none()
            }
            Message::Save => {
                if let (Some(path), Some(editor)) = (self.active_tab_path(), self.active_editor()) {
                    let text = editor.text();
                    if let Some(ref vault) = self.vault {
                        let _ = vault.write_note(&path, &text);
                    }
                }
                Task::none()
            }
            Message::CloseCurrentTab => {
                if let Some(active) = self.active_tab {
                    self.editors.remove(&self.open_tabs[active].path);
                    self.open_tabs.remove(active);
                    self.active_tab = if self.open_tabs.is_empty() {
                        None
                    } else {
                        Some(active.min(self.open_tabs.len() - 1))
                    };
                }
                Task::none()
            }
            Message::ToggleDir(path) => {
                if self.expanded_dirs.contains(&path) {
                    self.expanded_dirs.remove(&path);
                } else {
                    self.expanded_dirs.insert(path);
                }
                Task::none()
            }
            Message::LinkClicked(target) => {
                if let Some(ref vault) = self.vault {
                    let target_path = vault.config.path.join(&target);
                    if target_path.exists() {
                        return self.update(Message::FileSelected(target_path));
                    }
                }
                Task::none()
            }
            Message::SearchResultClicked(path) => {
                if let Some(ref vault) = self.vault {
                    let abs_path = if path.is_absolute() {
                        path
                    } else {
                        vault.config.path.join(&path)
                    };
                    self.update(Message::FileSelected(abs_path))
                } else {
                    Task::none()
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        crate::workspace::view(self)
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        use iced::keyboard;
        keyboard::listen().filter_map(|event| {
            match event {
                keyboard::Event::KeyPressed { key, modifiers, .. } => {
                    let msg = match key.as_ref() {
                        keyboard::Key::Character("o") if modifiers.command() => Some(Message::OpenVault),
                        keyboard::Key::Character("w") if modifiers.command() => Some(Message::CloseCurrentTab),
                        keyboard::Key::Character("b") if modifiers.command() => Some(Message::ToggleProjectPanel),
                        keyboard::Key::Character("p") if modifiers.command() => Some(Message::TogglePreview),
                        keyboard::Key::Character("f") if modifiers.command() => Some(Message::ToggleLowerPanel),
                        _ => None,
                    };
                    msg
                }
                _ => None,
            }
        })
    }
}

async fn open_vault_dialog() -> Result<PathBuf, String> {
    match rfd::AsyncFileDialog::new()
        .set_title("Select vault folder")
        .pick_folder()
        .await
    {
        Some(path) => Ok(path.path().to_path_buf()),
        None => Err("Dialog closed".to_string()),
    }
}
