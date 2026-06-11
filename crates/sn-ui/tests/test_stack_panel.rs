use gpui::px;
use sn_ui::dock::stack::StackPanel;
use sn_ui::dock::Axis;

#[gpui::test]
fn test_stack_panel_new(_cx: &mut gpui::TestAppContext) {
    let sp = StackPanel::new(Axis::Horizontal);
    assert_eq!(sp.axis(), Axis::Horizontal);
}

#[gpui::test]
fn test_stack_panel_min_size_default(_cx: &mut gpui::TestAppContext) {
    let sp = StackPanel::new(Axis::Vertical);
    assert_eq!(sp.min_size(), px(100.));
}

#[gpui::test]
fn test_stack_panel_axis_default(_cx: &mut gpui::TestAppContext) {
    let sp = StackPanel::new(Axis::Horizontal);
    assert_eq!(sp.axis(), Axis::Horizontal);
}
