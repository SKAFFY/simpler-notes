use gpui::*;

pub struct FileTree;

impl Render for FileTree {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_1()
            .p(8.)
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x888888))
                    .child("No vault opened"),
            )
    }
}
