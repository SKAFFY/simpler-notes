use gpui::*;

pub struct SourceEditor;

impl Render for SourceEditor {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_1()
            .p(8.)
            .text_color(rgb(0x888888))
            .child("Open a file to start editing")
    }
}
