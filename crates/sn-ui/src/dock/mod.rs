pub mod area;
pub mod tab;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockPlacement {
    Left,
    Right,
    Top,
    Bottom,
    Center,
}

pub fn dock_placement_to_axis(placement: DockPlacement) -> Axis {
    match placement {
        DockPlacement::Left | DockPlacement::Right => Axis::Horizontal,
        DockPlacement::Top | DockPlacement::Bottom => Axis::Vertical,
        DockPlacement::Center => Axis::Horizontal,
    }
}
