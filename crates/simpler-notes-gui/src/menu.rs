use gpui::*;

use crate::app_state::AppState;

pub struct MenuBar {
    state: View<AppState>,
}

impl MenuBar {
    pub fn new(state: View<AppState>) -> Self {
        Self { state }
    }
}

impl Render for MenuBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state_handle = self.state.clone();

        div()
            .h(TITLE_BAR_HEIGHT)
            .flex()
            .bg(rgb(0x2d2d2d))
            .px(4.)
            .gap(4.)
            .child(
                div()
                    .px(8.)
                    .py(2.)
                    .text_sm()
                    .text_color(rgb(0xcccccc))
                    .hover(|style| style.bg(rgb(0x3c3c3c)))
                    .cursor_pointer()
                    .on_click(move |_, cx| {
                        let handle = state_handle.clone();
                        cx.spawn(async move |cx| {
                            let file = rfd::AsyncFileDialog::new()
                                .pick_folder();
                            if let Some(path) = file {
                                handle.update(cx, |s, cx| {
                                    s.open_vault(&path, cx);
                                });
                            }
                        })
                        .detach();
                    })
                    .child("Файл"),
            )
            .child(
                div()
                    .px(8.)
                    .py(2.)
                    .text_sm()
                    .text_color(rgb(0xcccccc))
                    .hover(|style| style.bg(rgb(0x3c3c3c)))
                    .cursor_pointer()
                    .child("Правка"),
            )
            .child(
                div()
                    .px(8.)
                    .py(2.)
                    .text_sm()
                    .text_color(rgb(0xcccccc))
                    .hover(|style| style.bg(rgb(0x3c3c3c)))
                    .cursor_pointer()
                    .child("Вид"),
            )
            .child(
                div()
                    .px(8.)
                    .py(2.)
                    .text_sm()
                    .text_color(rgb(0xcccccc))
                    .hover(|style| style.bg(rgb(0x3c3c3c)))
                    .cursor_pointer()
                    .child("Помощь"),
            )
    }
}
