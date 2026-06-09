use iced::widget::{column, container, scrollable, text};
use iced::{Element, Fill};

use crate::app::{App, Message};

pub fn view(app: &App) -> Element<'_, Message> {
    let content: Element<'_, Message> = if let Some(ref _vault) = app.vault {
        let files = _vault.list_md_files();
        let file_entries: Vec<Element<'_, Message>> = files
            .iter()
            .map(|path| {
                let name = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                let _path_clone = path.clone();
                text(name)
                    .size(13)
                    .into()
            })
            .collect();

        column(file_entries).spacing(2).padding(8).into()
    } else {
        text("Open a vault to see files").size(13).into()
    };

    container(scrollable(content))
        .width(250)
        .height(Fill)
        .into()
}
