use gpui::*;

use crate::sidebar::file_tree::FileTree;

pub mod file_tree;

pub struct Sidebar;

impl Render for Sidebar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
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
                        div()
                            .bg(rgb(0x3c3c3c))
                            .px(8.)
                            .py(4.)
                            .rounded(4.)
                            .text_sm()
                            .text_color(rgb(0x888888))
                            .child("Search..."),
                    ),
            )
            .child(FileTree)
    }
}
