use gpui::*;

use crate::app_state::AppState;
use crate::editor::EditorContainer;
use crate::menu::MenuBar;
use crate::sidebar::Sidebar;

pub struct Workspace {
    state: View<AppState>,
}

impl Workspace {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            state: cx.new(|_| AppState::new()),
        }
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.clone();
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
                    .child(Sidebar::new(state.clone()))
                    .child(EditorContainer::new(state)),
            )
    }
}
