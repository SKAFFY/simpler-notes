use iced::widget::{center, column, container, text};
use iced::{Element, Fill};

use crate::app::{App, Message};

pub fn view(app: &App) -> Element<'_, Message> {
    let content: Element<'_, Message> = if app.is_vault_open() {
        center(
            column![
                text("Vault is open").size(16),
                text("Select a file from the project panel to start editing.")
                    .size(14),
            ]
            .spacing(10)
            .align_x(iced::Center),
        )
        .into()
    } else {
        center(
            column![
                text("Welcome to Simpler Notes!").size(24),
                text("Open a folder with markdown notes to get started.")
                    .size(14),
            ]
            .spacing(20)
            .align_x(iced::Center),
        )
        .into()
    };

    container(content).height(Fill).width(Fill).into()
}
