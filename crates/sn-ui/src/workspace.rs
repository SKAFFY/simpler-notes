use std::sync::atomic::{AtomicBool, Ordering};
use gpui::*;

pub struct Workspace {
    pub state: Entity<WorkspaceState>,
    _subscriptions: Vec<Subscription>,
    initialized: bool,
}

pub struct WorkspaceState {
    pub title: SharedString,
    pub left_dock_open: bool,
    initialized: AtomicBool,
}

impl WorkspaceState {
    pub fn new() -> Self {
        Self {
            title: SharedString::from("sn-ui"),
            left_dock_open: true,
            initialized: AtomicBool::new(false),
        }
    }

    pub fn mark_initialized(&self) {
        self.initialized.store(true, Ordering::SeqCst);
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::SeqCst)
    }
}

impl Workspace {
    pub fn new(app: &mut App) -> Self {
        let state = app.new(|_| WorkspaceState::new());
        Self {
            state,
            _subscriptions: Vec::new(),
            initialized: false,
        }
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.initialized {
            self.state.read(cx).mark_initialized();
            self.initialized = true;
        }

        div()
            .flex()
            .flex_col()
            .size_full()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .items_center()
                                    .justify_center()
                                    .size_full()
                                    .child("sn-ui Workspace"),
                            ),
                    ),
            )
    }
}
