use gpui::*;

pub mod registry;

pub trait PanelView: Send + Sync {
    fn panel_id(&self, _cx: &App) -> EntityId;
    fn panel_name(&self, _cx: &App) -> &'static str;
    fn tab_name(&self, _cx: &App) -> Option<SharedString>;
    fn title(&self, _window: &mut Window, cx: &mut App) -> AnyElement;
    fn render(&self, _window: &mut Window, cx: &mut App) -> AnyElement;

    fn closable(&self, _cx: &App) -> bool {
        true
    }

    fn set_active(&self, _active: bool, _cx: &mut App) {}

    fn on_added(&self, _tab_panel: EntityId, _cx: &mut App) {}

    fn on_removed(&self, _cx: &mut App) {}

    fn dump(&self, _cx: &App) -> Option<serde_json::Value> {
        None
    }
}
