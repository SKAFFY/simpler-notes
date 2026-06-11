use gpui::IntoElement;
use sn_ui::panel::PanelView;

struct MockPanel {
    name: &'static str,
    id: gpui::EntityId,
}

impl PanelView for MockPanel {
    fn panel_id(&self, _cx: &gpui::App) -> gpui::EntityId {
        self.id
    }

    fn panel_name(&self, _cx: &gpui::App) -> &'static str {
        self.name
    }

    fn tab_name(&self, _cx: &gpui::App) -> Option<gpui::SharedString> {
        Some(self.name.into())
    }

    fn title(
        &self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::App,
    ) -> gpui::AnyElement {
        gpui::div().into_any_element()
    }

    fn render(
        &self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::App,
    ) -> gpui::AnyElement {
        gpui::div().into_any_element()
    }
}

#[gpui::test]
fn test_panel_view_basic(cx: &mut gpui::TestAppContext) {
    cx.update(|app| {
        let id = gpui::EntityId::from(42u64);
        let panel = MockPanel { name: "Test", id };
        assert_eq!(panel.panel_name(app), "Test");
        assert!(panel.closable(app));
        assert!(panel.dump(app).is_none());
    });
}

#[gpui::test]
fn test_panel_registry(cx: &mut gpui::TestAppContext) {
    use sn_ui::panel::registry::PanelRegistry;
    cx.update(|app| {
        let mut registry = PanelRegistry::new();
        registry.register(
            "MockPanel",
            Box::new(|_| {
                Box::new(MockPanel {
                    name: "Mock",
                    id: gpui::EntityId::from(1u64),
                })
            }),
        );
        assert!(registry.contains("MockPanel"));
        assert!(!registry.contains("NonExistent"));

        let built = registry.build("MockPanel", app);
        assert!(built.is_some());
        assert_eq!(built.unwrap().panel_name(app), "Mock");

        let not_built = registry.build("NonExistent", app);
        assert!(not_built.is_none());
    });
}
