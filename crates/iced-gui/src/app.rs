use std::path::PathBuf;
use std::sync::Arc;

use iced::widget::{button, center, column, row, space, text};
use iced::{Center, Element, Task, Theme};

use simpler_notes_core::vault::{Vault, VaultConfig};

#[derive(Debug, Clone)]
pub enum Message {
    OpenVault,
    VaultOpened(Result<PathBuf, String>),
    CloseVault,
}

pub struct App {
    vault: Option<Arc<Vault>>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (Self { vault: None }, Task::none())
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
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let header: Element<'_, Message> = row![
            text("Simpler Notes").size(20),
            space::horizontal(),
            button("Open Vault").on_press(Message::OpenVault),
        ]
        .padding(10)
        .spacing(10)
        .align_y(Center)
        .into();

        let body: Element<'_, Message> = if self.vault.is_some() {
            center(
                column![
                    text("Vault is open").size(16),
                    text("Select a file from the project panel to start editing.")
                        .size(14),
                ]
                .spacing(10)
                .align_x(Center),
            )
            .into()
        } else {
            center(
                column![
                    text("Welcome to Simpler Notes!").size(24),
                    text("Open a folder with markdown notes to get started.")
                        .size(14),
                    button("Open Vault")
                        .on_press(Message::OpenVault)
                        .padding(10),
                ]
                .spacing(20)
                .align_x(Center),
            )
            .into()
        };

        column![header, body].into()
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
