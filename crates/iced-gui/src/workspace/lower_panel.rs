use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Color, Element, Fill};

use crate::app::{App, LowerPanelTab, Message};

pub fn view(app: &App) -> Element<'_, Message> {
    let tab_bar = row![
        button(text("Search").size(13))
            .on_press(Message::SelectLowerTab(LowerPanelTab::Search))
            .style(if app.lower_panel_active_tab == LowerPanelTab::Search {
                button::primary
            } else {
                button::text
            }),
        button(text("Diagnostics").size(13))
            .on_press(Message::SelectLowerTab(LowerPanelTab::Diagnostics))
            .style(if app.lower_panel_active_tab == LowerPanelTab::Diagnostics {
                button::primary
            } else {
                button::text
            }),
    ]
    .spacing(4)
    .padding([2, 4]);

    let body: Element<'_, Message> = match app.lower_panel_active_tab {
        LowerPanelTab::Search => search_panel(app),
        LowerPanelTab::Diagnostics => diagnostics_panel(app),
    };

    column![tab_bar, container(body).height(Fill).width(Fill)].into()
}

fn search_panel(app: &App) -> Element<'_, Message> {
    let input = text_input("Search...", &app.search_query)
        .on_input(Message::SearchQueryChanged)
        .on_submit(Message::Search)
        .width(Fill);

    let results: Element<'_, Message> = if app.search_query.is_empty() || app.vault.is_none() {
        text("Enter a query to search").size(13).into()
    } else if let Some(ref error) = app.search_error {
        text(format!("Search error: {}", error)).size(13).into()
    } else if app.search_results.is_empty() {
        text("No results found").size(13).into()
    } else {
        let items: Vec<Element<'_, Message>> = app
            .search_results
            .iter()
            .map(|r| {
                let path = r.path.clone();
                button(
                    container(
                        column![
                            text(&r.title).size(13).color(Color::from_rgb(0.4, 0.6, 1.0)),
                            text(r.path.display().to_string()).size(11),
                        ]
                        .spacing(2),
                    )
                    .padding([2, 4]),
                )
                .on_press(Message::SearchResultClicked(path))
                .into()
            })
            .collect();
        scrollable(iced::widget::Column::with_children(items).spacing(2)).into()
    };

    column![input, container(results).height(Fill).width(Fill)].spacing(4).padding(4).into()
}

fn diagnostics_panel(app: &App) -> Element<'_, Message> {
    if let Some(ref vault) = app.vault {
        let all_diags = vault.all_diagnostics();
        if all_diags.is_empty() {
            return container(text("No issues found").size(13))
                .padding(8)
                .into();
        }

        let items: Vec<Element<'_, Message>> = all_diags
            .into_iter()
            .flat_map(|(path, diags)| {
                diags.into_iter().map(move |d| {
                    container(
                        column![
                            text(path.display().to_string()).size(11).color(Color::from_rgb(0.4, 0.6, 1.0)),
                            text(format!("{} (offset {})", d.message, d.span.offset)).size(13).color(Color::from_rgb(1.0, 0.3, 0.3)),
                        ]
                        .spacing(2),
                    )
                    .padding([2, 4])
                    .into()
                })
            })
            .collect();

        scrollable(iced::widget::Column::with_children(items).spacing(2)).into()
    } else {
        container(text("No vault open").size(13)).padding(8).into()
    }
}
