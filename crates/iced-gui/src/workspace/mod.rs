pub mod editor_panel;
pub mod project_panel;

use iced::widget::{button, column, container, row, space, text};
use iced::{Center, Element};

use crate::app::{App, Message};

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

        container(
            row![
                text("Simpler Notes").size(14),
                space::horizontal(),
                open_btn,
                toggle_panel,
            ]
            .padding([4, 8])
            .spacing(8)
            .align_y(Center),
        )
        .into()
    };

    let main_content: Element<'_, Message> = if app.is_vault_open() && app.project_panel_visible {
        row![
            project_panel::view(app),
            editor_panel::view(app),
        ]
        .into()
    } else {
        editor_panel::view(app)
    };

    container(column([title_bar, main_content])).into()
}
