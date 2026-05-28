use gpui::*;

use crate::app_state::AppState;

pub struct SourceEditor {
    state: View<AppState>,
}

impl SourceEditor {
    pub fn new(state: View<AppState>) -> Self {
        Self { state }
    }
}

impl Render for SourceEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        match state.active_tab {
            Some(idx) => {
                if let Some(tab) = state.open_tabs.get(idx) {
                    div()
                        .flex_1()
                        .p(8.)
                        .text_color(rgb(0xd4d4d4))
                        .child(tab.source_content.clone())
                } else {
                    div()
                        .flex_1()
                        .p(8.)
                        .text_color(rgb(0x888888))
                        .child("Open a file to start editing")
                }
            }
            None => div()
                .flex_1()
                .p(8.)
                .text_color(rgb(0x888888))
                .child("Open a file to start editing"),
        }
    }
}
