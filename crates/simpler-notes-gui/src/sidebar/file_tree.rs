use std::path::PathBuf;

use gpui::*;

use crate::app_state::AppState;

fn list_md_files(path: &PathBuf) -> Vec<PathBuf> {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
        .map(|e| e.path().to_owned())
        .collect()
}

pub struct FileTree {
    state: View<AppState>,
}

impl FileTree {
    pub fn new(state: View<AppState>) -> Self {
        Self { state }
    }
}

impl Render for FileTree {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let vault_path = state.vault_path.clone();
        let search_query = state.search_query.clone();
        let state_handle = self.state.clone();

        let files: Vec<PathBuf> = match vault_path {
            Some(ref path) => {
                let mut files = list_md_files(path);
                files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
                if search_query.is_empty() {
                    files
                } else {
                    let q = search_query.to_lowercase();
                    files.into_iter().filter(|f| {
                        f.file_name()
                            .and_then(|n| n.to_str())
                            .map(|n| n.to_lowercase().contains(&q))
                            .unwrap_or(false)
                    }).collect()
                }
            }
            None => Vec::new(),
        };

        if vault_path.is_none() {
            return div()
                .flex_1()
                .p(8.)
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x888888))
                        .child("No vault opened"),
                )
                .into_any_element();
        }

        if files.is_empty() {
            return div()
                .flex_1()
                .p(8.)
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x888888))
                        .child("No markdown files found"),
                )
                .into_any_element();
        }

        div()
            .flex_1()
            .overflow_y_scroll()
            .p(8.)
            .children(files.into_iter().map(move |path| {
                let handle = state_handle.clone();
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                div()
                    .px(8.)
                    .py(4.)
                    .text_sm()
                    .text_color(rgb(0xcccccc))
                    .hover(|style| style.bg(rgb(0x2a2d2e)))
                    .cursor_pointer()
                    .on_click(move |_, cx| {
                        handle.update(cx, |s, cx| s.open_file(path.clone(), cx));
                    })
                    .child(name)
            }))
    }
}
