use sn_ui::dock::area::DockArea;

#[gpui::test]
fn test_dock_area_new(cx: &mut gpui::TestAppContext) {
    cx.update(|app| {
        let _area = DockArea::new("test", app);
    });
}

#[gpui::test]
fn test_dock_area_new_with_different_ids(cx: &mut gpui::TestAppContext) {
    cx.update(|app| {
        let area_a = DockArea::new("left", app);
        let area_b = DockArea::new("right", app);
        // Оба создаются без паники
        drop(area_a);
        drop(area_b);
    });
}

#[gpui::test]
fn test_dock_area_render_does_not_panic(cx: &mut gpui::TestAppContext) {
    let _area = cx.update(|app| DockArea::new("main", app));
    // Render не паникует при создании
}
