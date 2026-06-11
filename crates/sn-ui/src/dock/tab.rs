use gpui::*;

pub struct TabPanel {
    #[allow(dead_code)]
    active_ix: usize,
}

impl TabPanel {
    pub fn new(_cx: &mut App) -> Self {
        Self { active_ix: 0 }
    }
}

impl Render for TabPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child("TabPanel")
    }
}
