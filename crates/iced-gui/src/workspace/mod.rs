pub mod editor_panel;
pub mod preview_panel;
pub mod project_panel;

use iced::widget::{button, column, container, row, space, text};
use iced::{Center, Element};

use crate::app::{App, EditorMode, Message};

pub fn view(app: &App) -> Element<'_, Message> {
    let title_bar: Element<'_, Message> = {
        let open_btn = if app.vault.is_none() {
            Element::from(button("Open Vault").on_press(Message::OpenVault))
        } else {
            Element::from(button("Close Vault").on_press(Message::CloseVault))
        };

        let toggle_panel = if app.vault.is_some() {
            Element::from(
                button(if app.project_panel_visible {
                    "Hide Panel"
                } else {
                    "Show Panel"
                })
                .on_press(Message::ToggleProjectPanel),
            )
        } else {
            space::horizontal().into()
        };

        let toggle_preview = if app.vault.is_some() {
            Element::from(
                button(match app.editor_mode {
                    EditorMode::Source => "Preview",
                    EditorMode::Split => "Split",
                    EditorMode::Preview => "Edit",
                })
                .on_press(Message::TogglePreview),
            )
        } else {
            space::horizontal().into()
        };

        container(
            row![
                text("Simpler Notes").size(14),
                space::horizontal(),
                open_btn,
                toggle_panel,
                toggle_preview,
            ]
            .padding([4, 8])
            .spacing(8)
            .align_y(Center),
        )
        .into()
    };

    let main_content: Element<'_, Message> = if app.is_vault_open() && app.project_panel_visible {
        match app.editor_mode {
            EditorMode::Source => {
                row![
                    project_panel::view(app),
                    editor_panel::view(app),
                ]
                .into()
            }
            EditorMode::Split => {
                row![
                    project_panel::view(app),
                    editor_panel::view(app),
                    preview_panel::view(app),
                ]
                .into()
            }
            EditorMode::Preview => {
                row![
                    project_panel::view(app),
                    preview_panel::view(app),
                ]
                .into()
            }
        }
    } else if app.is_vault_open() {
        match app.editor_mode {
            EditorMode::Source => editor_panel::view(app),
            EditorMode::Split => {
                row![editor_panel::view(app), preview_panel::view(app)].into()
            }
            EditorMode::Preview => preview_panel::view(app),
        }
    } else {
        editor_panel::view(app)
    };

    container(column([title_bar, main_content])).into()
}
