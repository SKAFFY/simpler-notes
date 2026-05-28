use gpui::*;

use crate::app_state::{AppState, EditorMode};
use crate::editor::preview::PreviewRenderer;
use crate::editor::source::SourceEditor;

pub mod preview;
pub mod source;

pub struct EditorContainer {
    state: View<AppState>,
}

impl EditorContainer {
    pub fn new(state: View<AppState>) -> Self {
        Self { state }
    }
}

impl Render for EditorContainer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let mode = state.editor_mode;
        let mode_name = match mode {
            EditorMode::Source => "Source",
            EditorMode::Split => "Split",
            EditorMode::Preview => "Preview",
        };

        let source_handle = self.state.clone();
        let preview_handle = self.state.clone();

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
                            .bg(if mode == EditorMode::Source {
                                rgb(0x2d2d2d)
                            } else {
                                rgb(0x252526)
                            })
                            .text_sm()
                            .text_color(rgb(0xcccccc))
                            .cursor_pointer()
                            .on_click(move |_, cx| {
                                source_handle.update(cx, |s, cx| {
                                    s.set_editor_mode(EditorMode::Source, cx)
                                });
                            })
                            .child("Source"),
                    )
                    .child(
                        div()
                            .px(12.)
                            .py(6.)
                            .bg(if mode == EditorMode::Preview {
                                rgb(0x2d2d2d)
                            } else {
                                rgb(0x252526)
                            })
                            .text_sm()
                            .text_color(rgb(0xcccccc))
                            .cursor_pointer()
                            .on_click(move |_, cx| {
                                preview_handle.update(cx, |s, cx| {
                                    let next = match s.editor_mode {
                                        EditorMode::Source => EditorMode::Split,
                                        EditorMode::Split => EditorMode::Preview,
                                        EditorMode::Preview => EditorMode::Source,
                                    };
                                    s.set_editor_mode(next, cx)
                                });
                            })
                            .child(mode_name),
                    ),
            )
            .child(SourceEditor)
    }
}
