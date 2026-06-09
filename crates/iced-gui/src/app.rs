use std::path::PathBuf;
use std::sync::Arc;

use iced::{Element, Task, Theme};

use simpler_notes_core::vault::{Vault, VaultConfig};

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
    FileSelected(PathBuf),
    TabSelected(usize),
    TabClosed(usize),
}

pub struct App {
    pub vault: Option<Arc<Vault>>,
    pub project_panel_visible: bool,
    pub open_tabs: Vec<OpenTab>,
    pub active_tab: Option<usize>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                vault: None,
                project_panel_visible: true,
                open_tabs: Vec::new(),
                active_tab: None,
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

    pub fn _vault_root(&self) -> Option<PathBuf> {
        self.vault.as_ref().map(|v| v.config.path.clone())
    }

    pub fn _md_files(&self) -> Vec<PathBuf> {
        self.vault
            .as_ref()
            .map(|v| v.list_md_files())
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
                Task::none()
            }
            Message::ToggleProjectPanel => {
                self.project_panel_visible = !self.project_panel_visible;
                Task::none()
            }
            Message::FileSelected(path) => {
                let already_open = self.open_tabs.iter().position(|t| t.path == path);
                match already_open {
                    Some(idx) => {
                        self.active_tab = Some(idx);
                    }
                    None => {
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
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        crate::workspace::view(self)
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
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
