use gpui::*;
use crate::panel::PanelView;

pub struct MockPanel {
    id: EntityId,
    name: &'static str,
}

impl MockPanel {
    pub fn new(id: EntityId, name: &'static str) -> Self {
        Self { id, name }
    }
}

impl PanelView for MockPanel {
    fn panel_id(&self, _cx: &App) -> EntityId {
        self.id
    }

    fn panel_name(&self, _cx: &App) -> &'static str {
        self.name
    }

    fn tab_name(&self, _cx: &App) -> Option<SharedString> {
        Some(self.name.into())
    }

    fn title(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        div().into_any_element()
    }

    fn render(&self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        div().into_any_element()
    }
}
