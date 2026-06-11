use sn_ui::dock::tab::TabPanel;

#[gpui::test]
fn test_tab_panel_new(cx: &mut gpui::TestAppContext) {
    cx.update(|app| {
        let tp = TabPanel::new(app);
        assert_eq!(tp.active_index(), 0);
    });
}

#[gpui::test]
fn test_tab_panel_set_active(cx: &mut gpui::TestAppContext) {
    cx.update(|app| {
        let mut tp = TabPanel::new(app);
        assert_eq!(tp.active_index(), 0);
        tp.set_active_index(1);
        assert_eq!(tp.active_index(), 1);
        tp.set_active_index(5);
        assert_eq!(tp.active_index(), 5);
    });
}

#[gpui::test]
fn test_tab_panel_render(cx: &mut gpui::TestAppContext) {
    let _tp = cx.update(TabPanel::new);
    cx.run_until_parked();
}
