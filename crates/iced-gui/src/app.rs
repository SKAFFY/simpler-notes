use std::path::PathBuf;
use std::sync::Arc;

use iced::{Element, Task, Theme};

use simpler_notes_core::vault::{Vault, VaultConfig};

#[derive(Debug, Clone)]
pub enum Message {
    OpenVault,
    VaultOpened(Result<PathBuf, String>),
    CloseVault,
    ToggleProjectPanel,
}

pub struct App {
    pub vault: Option<Arc<Vault>>,
    pub project_panel_visible: bool,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                vault: None,
                project_panel_visible: true,
            },
            Task::none(),
        )
    }

    pub fn is_vault_open(&self) -> bool {
        self.vault.is_some()
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
                Task::none()
            }
            Message::ToggleProjectPanel => {
                self.project_panel_visible = !self.project_panel_visible;
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
