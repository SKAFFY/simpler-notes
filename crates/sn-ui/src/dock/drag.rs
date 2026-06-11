use gpui::{Bounds, EntityId, Point, Pixels};

use crate::dock::DockPlacement;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SplitZone {
    Placement(DockPlacement),
    Merge,
}

#[derive(Debug, Clone)]
pub struct DragPayload {
    pub panel_id: EntityId,
    pub panel_name: &'static str,
    pub source_tab_panel: Option<EntityId>,
}

pub fn detect_split_zone(bounds: &Bounds<Pixels>, cursor: Point<Pixels>) -> SplitZone {
    let rel_x = (cursor.x - bounds.origin.x).as_f32() / bounds.size.width.as_f32();
    let rel_y = (cursor.y - bounds.origin.y).as_f32() / bounds.size.height.as_f32();

    if rel_x < 0.35 {
        return SplitZone::Placement(DockPlacement::Left);
    }
    if rel_x > 0.65 {
        return SplitZone::Placement(DockPlacement::Right);
    }
    if rel_y < 0.35 {
        return SplitZone::Placement(DockPlacement::Top);
    }
    if rel_y > 0.65 {
        return SplitZone::Placement(DockPlacement::Bottom);
    }

    SplitZone::Merge
}
