use iced::widget::{button, center, column, container, row, text, text_editor};
use iced::{keyboard, Center, Element, Fill};

use crate::app::{App, Message};

pub fn view(app: &App) -> Element<'_, Message> {
    let content: Element<'_, Message> = if let Some(path) = app.active_tab_path() {
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
                row![tab_btn, close_btn].spacing(2).into()
            })
            .collect();

        let editor: Element<'_, Message> = app
            .editors
            .get(&path)
            .map(|content| {
                text_editor(content)
                    .height(Fill)
                    .on_action(Message::EditorAction)
                    .key_binding(|key_press| {
                        match key_press.key.as_ref() {
                            keyboard::Key::Character("s") if key_press.modifiers.command() => {
                                Some(text_editor::Binding::Custom(Message::Save))
                            }
                            _ => text_editor::Binding::from_key_press(key_press),
                        }
                    })
                    .into()
            })
            .unwrap_or_else(|| text("Error loading editor").into());

        column![
            row(tab_bar).spacing(2).padding([2, 4]),
            container(editor).height(Fill),
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
