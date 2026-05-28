use gpui::*;

use crate::editor::preview::PreviewRenderer;
use crate::editor::source::SourceEditor;

pub mod preview;
pub mod source;

pub struct EditorContainer;

impl Render for EditorContainer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .flex_col()
            .bg(rgb(0x1e1e1e))
            .child(
                h_flex()
                    .h(px(35.))
                    .bg(rgb(0x252526))
                    .border_b_1()
                    .border_color(rgb(0x3c3c3c))
                    .px(4.)
                    .gap(2.)
                    .child(
                        div()
                            .px(12.)
                            .py(6.)
                            .bg(rgb(0x2d2d2d))
                            .text_sm()
                            .text_color(rgb(0xcccccc))
                            .cursor_pointer()
                            .child("Source"),
                    )
                    .child(
                        div()
                            .px(12.)
                            .py(6.)
                            .text_sm()
                            .text_color(rgb(0x888888))
                            .hover(|style| style.bg(rgb(0x2a2d2a)))
                            .cursor_pointer()
                            .child("Preview"),
                    ),
            )
            .child(SourceEditor)
    }
}
