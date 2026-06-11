use gpui::*;

pub struct SnApp {
    size: Option<Size<Pixels>>,
    title: Option<SharedString>,
    window_options_modifier: Option<Box<dyn FnOnce(WindowOptions) -> WindowOptions>>,
}

impl Default for SnApp {
    fn default() -> Self {
        Self::new()
    }
}

impl SnApp {
    pub fn new() -> Self {
        Self {
            size: None,
            title: None,
            window_options_modifier: None,
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

    pub fn with_window_options(
        mut self,
        f: impl FnOnce(WindowOptions) -> WindowOptions + 'static,
    ) -> Self {
        self.window_options_modifier = Some(Box::new(f));
        self
    }

    pub fn run<V: 'static + Render>(
        self,
        init: impl FnOnce(&mut Window, &mut App) -> Entity<V> + 'static,
    ) {
        let size = self.size.unwrap_or(Size {
            width: px(1024.),
            height: px(768.),
        });
        let window_options_modifier = self.window_options_modifier;

        let app = gpui_platform::application();

        app.run(move |app: &mut App| {
            let window_options = WindowOptions {
                window_bounds: Some(WindowBounds::centered(size, app)),
                ..Default::default()
            };
            let window_options = if let Some(modifier) = window_options_modifier {
                modifier(window_options)
            } else {
                window_options
            };

            app.open_window::<V>(window_options, |window, app| init(window, app))
                .ok();
        })
    }
}
