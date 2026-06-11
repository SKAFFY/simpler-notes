use gpui::*;

pub struct SnApp {
    size: Option<Size<Pixels>>,
    title: Option<SharedString>,
}

impl SnApp {
    pub fn new() -> Self {
        Self {
            size: None,
            title: None,
        }
    }

    pub fn with_size(mut self, size: Size<Pixels>) -> Self {
        self.size = Some(size);
        self
    }

    pub fn with_title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn run<V: 'static + Render>(
        self,
        init: impl FnOnce(&mut App) -> Entity<V> + 'static,
    ) {
        let size = self.size.unwrap_or(Size {
            width: px(1024.),
            height: px(768.),
        });

        let app = gpui_platform::application();

        app.run(move |app: &mut App| {
            let window_options = WindowOptions {
                window_bounds: Some(WindowBounds::centered(size, app)),
                ..Default::default()
            };

            app.open_window::<V>(window_options, |_window, app| {
                init(app)
            })
            .ok();
        })
    }
}
