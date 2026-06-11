use sn_ui::dock::{DockPlacement, Axis, dock_placement_to_axis};
use sn_ui::layout_state::{DockItemState, DockItemVariant};

#[test]
fn test_dock_placement_to_axis() {
    assert_eq!(dock_placement_to_axis(DockPlacement::Left), Axis::Horizontal);
    assert_eq!(dock_placement_to_axis(DockPlacement::Right), Axis::Horizontal);
    assert_eq!(dock_placement_to_axis(DockPlacement::Top), Axis::Vertical);
    assert_eq!(dock_placement_to_axis(DockPlacement::Bottom), Axis::Vertical);
    assert_eq!(dock_placement_to_axis(DockPlacement::Center), Axis::Horizontal);
}

#[test]
fn test_dock_item_variant_name() {
    let variant = DockItemVariant::Panel { name: "notes".into() };
    assert_eq!(variant.name(), Some("notes"));

    let split = DockItemVariant::Split { axis: "horizontal".into() };
    assert_eq!(split.name(), None);

    let tabs = DockItemVariant::Tabs;
    assert_eq!(tabs.name(), None);
}

#[test]
fn test_dock_item_state_serialize() {
    let state = DockItemState {
        variant: DockItemVariant::Panel { name: "test".into() },
        children: vec![],
        sizes: vec![],
        active_index: None,
    };
    let json = serde_json::to_string(&state).unwrap();
    assert!(json.contains("panel"));
    assert!(json.contains("test"));
}

#[test]
fn test_dock_item_state_deserialize() {
    let json = r#"{"variant":{"panel":{"name":"notes"}},"children":[],"sizes":[],"active_index":null}"#;
    let state: DockItemState = serde_json::from_str(json).unwrap();
    assert_eq!(state.variant.name(), Some("notes"));
}
