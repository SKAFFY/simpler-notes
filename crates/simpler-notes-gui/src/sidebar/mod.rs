use gpui::*;

use crate::app_state::AppState;
use crate::sidebar::file_tree::FileTree;

pub mod file_tree;

pub struct Sidebar {
    state: View<AppState>,
}

impl Sidebar {
    pub fn new(state: View<AppState>) -> Self {
        Self { state }
    }
}

impl Render for Sidebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let search_query = state.search_query.clone();

        let state_handle = self.state.clone().downgrade();

        div()
            .w(px(250.))
            .h_full()
            .flex()
            .flex_col()
            .bg(rgb(0x252526))
            .border_r_1()
            .border_color(rgb(0x3c3c3c))
            .child(
                div()
                    .p(8.)
                    .child(
                        TextInput::new("sidebar-search", search_query)
                            .placeholder("Search...")
                            .on_input(move |text, cx| {
                                if let Some(state) = state_handle.upgrade() {
                                    state.update(cx, |s, cx| s.set_search_query(&text, cx));
                                }
                            }),
                    ),
            )
            .child(FileTree)
    }
}
