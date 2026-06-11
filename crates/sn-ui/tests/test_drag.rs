use gpui::{Bounds, EntityId, Point};

use sn_ui::dock::drag::{detect_split_zone, DragPayload, SplitZone};
use sn_ui::dock::DockPlacement;

#[test]
fn test_split_zone_left() {
    let bounds = Bounds {
        origin: Point::new(gpui::px(0.), gpui::px(0.)),
        size: gpui::Size {
            width: gpui::px(200.),
            height: gpui::px(100.),
        },
    };
    let pos = Point::new(gpui::px(30.), gpui::px(50.));
    assert_eq!(
        detect_split_zone(&bounds, pos),
        SplitZone::Placement(DockPlacement::Left)
    );
}

#[test]
fn test_split_zone_center() {
    let bounds = Bounds {
        origin: Point::new(gpui::px(0.), gpui::px(0.)),
        size: gpui::Size {
            width: gpui::px(200.),
            height: gpui::px(100.),
        },
    };
    let pos = Point::new(gpui::px(100.), gpui::px(50.));
    assert_eq!(detect_split_zone(&bounds, pos), SplitZone::Merge);
}

#[test]
fn test_split_zone_right() {
    let bounds = Bounds {
        origin: Point::new(gpui::px(0.), gpui::px(0.)),
        size: gpui::Size {
            width: gpui::px(200.),
            height: gpui::px(100.),
        },
    };
    let pos = Point::new(gpui::px(170.), gpui::px(50.));
    assert_eq!(
        detect_split_zone(&bounds, pos),
        SplitZone::Placement(DockPlacement::Right)
    );
}

#[test]
fn test_split_zone_top() {
    let bounds = Bounds {
        origin: Point::new(gpui::px(0.), gpui::px(0.)),
        size: gpui::Size {
            width: gpui::px(200.),
            height: gpui::px(100.),
        },
    };
    let pos = Point::new(gpui::px(100.), gpui::px(10.));
    assert_eq!(
        detect_split_zone(&bounds, pos),
        SplitZone::Placement(DockPlacement::Top)
    );
}

#[test]
fn test_split_zone_bottom() {
    let bounds = Bounds {
        origin: Point::new(gpui::px(0.), gpui::px(0.)),
        size: gpui::Size {
            width: gpui::px(200.),
            height: gpui::px(100.),
        },
    };
    let pos = Point::new(gpui::px(100.), gpui::px(90.));
    assert_eq!(
        detect_split_zone(&bounds, pos),
        SplitZone::Placement(DockPlacement::Bottom)
    );
}

#[test]
fn test_drag_payload() {
    let payload = DragPayload {
        panel_id: EntityId::from(1u64),
        panel_name: "Test",
        source_tab_panel: None,
    };
    assert_eq!(payload.panel_name, "Test");
}
