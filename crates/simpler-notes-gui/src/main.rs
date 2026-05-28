mod app_state;
mod editor;
mod workspace;

use gpui::*;
use gpui_platform::application;
use gpui_component_assets::Assets;

fn main() {
    let app = application().with_assets(Assets);

    app.run(move |cx| {
        gpui_component::init(cx);

        let state = cx.new(|_| app_state::AppState::new());

        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(1024.), px(768.)), cx)),
            ..Default::default()
        };

        cx.spawn(async move |cx| {
            cx.open_window(window_options, |window, cx| {
                let view = cx.new(|cx| workspace::Workspace::new(state, cx));
                cx.new(|cx| gpui_component::Root::new(view, window, cx))
            })
            .expect("Failed to open window");
        })
        .detach();
    });
}
