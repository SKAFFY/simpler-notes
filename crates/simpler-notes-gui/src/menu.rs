use gpui::*;

pub struct MenuBar;

impl Render for MenuBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let items = ["Файл", "Правка", "Вид", "Помощь"];
        h_flex()
            .h(TITLE_BAR_HEIGHT)
            .bg(rgb(0x2d2d2d))
            .px(4.)
            .gap(4.)
            .children(items.iter().map(|label| {
                div()
                    .px(8.)
                    .py(2.)
                    .text_sm()
                    .text_color(rgb(0xcccccc))
                    .hover(|style| style.bg(rgb(0x3c3c3c)))
                    .cursor_pointer()
                    .child(label.to_string())
            }))
    }
}
