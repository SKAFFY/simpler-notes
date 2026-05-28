use gpui::*;

use crate::editor::EditorContainer;
use crate::menu::MenuBar;
use crate::sidebar::Sidebar;

pub struct Workspace;

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x1e1e1e))
            .child(MenuBar)
            .child(
                h_flex()
                    .flex_1()
                    .size_full()
                    .child(Sidebar)
                    .child(EditorContainer),
            )
    }
}
