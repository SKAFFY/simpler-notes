use gpui::*;
use gpui_platform::application;

fn main() {
    application().run(move |cx| {
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(
                        Bounds::centered(None, size(px(1024.), px(768.)), cx),
                    )),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|_| workspace::Workspace);
                    cx.new(|cx| gpui_component::Root::new(view, window, cx))
                },
            )
            .unwrap();
        })
        .detach();
    });
}
