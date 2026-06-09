pub mod editor_panel;
pub mod lower_panel;
pub mod preview_panel;
pub mod project_panel;

use iced::widget::{button, column, container, row, space, text};
use iced::{Center, Color, Element, Fill, Length};

use crate::app::{App, EditorMode, Message};

fn vsplit<'a>() -> Element<'a, crate::app::Message> {
    container(space::vertical())
        .width(1)
        .height(Fill)
        .style(|_: &iced::Theme| {
            container::Style::default()
                .background(Color::from_rgb(0.3, 0.3, 0.3))
        })
        .into()
}

fn hsplit<'a>() -> Element<'a, crate::app::Message> {
    container(space::horizontal())
        .height(1)
        .width(Fill)
        .style(|_: &iced::Theme| {
            container::Style::default()
                .background(Color::from_rgb(0.3, 0.3, 0.3))
        })
        .into()
}

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

        let toggle_lower = if app.vault.is_some() {
            Element::from(
                button(if app.lower_panel_visible { "Hide" } else { "Search" })
                    .on_press(Message::ToggleLowerPanel),
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
                toggle_lower,
            ]
            .padding([4, 8])
            .spacing(8)
            .align_y(Center),
        )
        .width(Fill)
        .into()
    };

    let editor_width = if app.project_panel_visible {
        Length::FillPortion(4)
    } else {
        Fill
    };
    let project_width = Length::FillPortion(2);

    let main_content: Element<'_, Message> = if app.is_vault_open() && app.project_panel_visible {
        let panels: Element<'_, Message> = match app.editor_mode {
            EditorMode::Source => {
                row![
                    container(project_panel::view(app)).width(project_width),
                    vsplit(),
                    container(editor_panel::view(app)).width(editor_width),
                ]
                .into()
            }
            EditorMode::Split => {
                let preview_width = Length::FillPortion(3);
                row![
                    container(project_panel::view(app)).width(project_width),
                    vsplit(),
                    container(editor_panel::view(app)).width(editor_width),
                    vsplit(),
                    container(preview_panel::view(app)).width(preview_width),
                ]
                .into()
            }
            EditorMode::Preview => {
                row![
                    container(project_panel::view(app)).width(project_width),
                    vsplit(),
                    container(preview_panel::view(app)).width(editor_width),
                ]
                .into()
            }
        };
        container(panels).height(Fill).into()
    } else if app.is_vault_open() {
        let panels = match app.editor_mode {
            EditorMode::Source => editor_panel::view(app),
            EditorMode::Split => {
                row![
                    container(editor_panel::view(app)).width(Length::FillPortion(4)),
                    vsplit(),
                    container(preview_panel::view(app)).width(Length::FillPortion(3)),
                ]
                .into()
            }
            EditorMode::Preview => preview_panel::view(app),
        };
        container(panels).height(Fill).into()
    } else {
        editor_panel::view(app)
    };

    let mut children: Vec<Element<'_, Message>> = vec![title_bar, main_content];

    if app.lower_panel_visible && app.vault.is_some() {
        let lower = container(
            column![
                hsplit(),
                lower_panel::view(app),
            ]
        )
        .height(Length::FillPortion(3))
        .width(Fill)
        .into();
        children.push(lower);
    }

    container(column(children)).height(Fill).into()
}
