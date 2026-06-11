use gpui::*;
use crate::dock::Axis;

pub struct StackPanel {
    axis: Axis,
    min_size: Pixels,
    _subscriptions: Vec<Subscription>,
}

impl StackPanel {
    pub fn new(axis: Axis) -> Self {
        Self {
            axis,
            min_size: px(100.),
            _subscriptions: Vec::new(),
        }
    }

    pub fn axis(&self) -> Axis {
        self.axis
    }

    pub fn set_axis(&mut self, axis: Axis, _cx: &mut Context<Self>) {
        self.axis = axis;
    }

    pub fn min_size(&self) -> Pixels {
        self.min_size
    }

    pub fn set_min_size(&mut self, size: Pixels, _cx: &mut Context<Self>) {
        self.min_size = size;
    }
}

impl Render for StackPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let flex_dir = match self.axis {
            Axis::Horizontal => div().flex().flex_row().size_full(),
            Axis::Vertical => div().flex().flex_col().size_full(),
        };
        flex_dir.child("StackPanel")
    }
}
