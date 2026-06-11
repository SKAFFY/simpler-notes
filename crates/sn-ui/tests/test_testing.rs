#![recursion_limit = "512"]

use gpui::EntityId;
use sn_ui::testing::MockPanel;
use sn_ui::PanelView;

#[gpui::test]
fn test_mock_panel(cx: &mut gpui::TestAppContext) {
    cx.update(|app| {
        let id = EntityId::from(42u64);
        let panel = MockPanel::new(id, "TestMock");
        assert_eq!(panel.panel_name(app), "TestMock");
        assert!(panel.closable(app));
    });
}
