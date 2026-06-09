use iced::widget::{button, center, column, container, row, scrollable, text};
use iced::{Center, Element, Fill};

use crate::app::{App, Message};

pub fn view(app: &App) -> Element<'_, Message> {
    let content: Element<'_, Message> = if let Some(path) = app.active_tab_path() {
        let content_str = std::fs::read_to_string(&path).unwrap_or_default();

        let tab_bar: Vec<Element<'_, Message>> = app
            .open_tabs
            .iter()
            .enumerate()
            .map(|(ix, tab)| {
                let _is_active = app.active_tab == Some(ix);
                let tab_btn = button(text(tab.title.as_str()).size(13))
                    .on_press(Message::TabSelected(ix));
                let close_btn = button(text("✕").size(11))
                    .on_press(Message::TabClosed(ix));
                let item: Element<'_, Message> = row![tab_btn, close_btn]
                    .spacing(2)
                    .into();
                item
            })
            .collect();

        column![
            row(tab_bar).spacing(2).padding([2, 4]),
            container(scrollable(text(content_str).size(13))).height(Fill),
        ]
        .into()
    } else if app.is_vault_open() {
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
            ]
            .spacing(20)
            .align_x(Center),
        )
        .into()
    };

    container(content).height(Fill).width(Fill).into()
}
