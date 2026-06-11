use sn_ui::app::SnApp;

#[gpui::test]
fn test_sn_app_new(_cx: &mut gpui::TestAppContext) {
    SnApp::new();
}
