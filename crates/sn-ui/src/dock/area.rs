use gpui::*;

pub struct DockArea {
    id: SharedString,
}

impl DockArea {
    pub fn new(id: impl Into<SharedString>, _cx: &mut App) -> Self {
        Self { id: id.into() }
    }
}

impl Render for DockArea {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .size_full()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .size_full()
                    .child(format!("DockArea: {}", self.id)),
            )
    }
}
