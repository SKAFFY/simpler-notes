mod app_state;
mod workspace;

use gpui::*;
use gpui_platform::application;
use gpui_component_assets::Assets;
use gpui_component::GlobalState;

fn main() {
    let app = application().with_assets(Assets);

    app.run(move |cx| {
        gpui_component::init(cx);

        use gpui::{Menu, MenuItem};

        let file_menu = Menu::new("Файл").items(vec![
            MenuItem::action("Open Vault...", workspace::OpenVault),
            MenuItem::action("Close Vault", workspace::CloseVault),
            MenuItem::separator(),
            MenuItem::action("Exit", workspace::Exit),
        ]);
        let edit_menu = Menu::new("Правка").items(vec![]);
        let view_menu = Menu::new("Вид").items(vec![
            MenuItem::action("Toggle Project Panel", workspace::ToggleProjectPanel),
            MenuItem::action("Toggle Lower Panel", workspace::ToggleLowerPanel),
            MenuItem::separator(),
            MenuItem::action("Toggle Preview", workspace::TogglePreview),
        ]);
        let help_menu = Menu::new("Помощь").items(vec![]);

        GlobalState::global_mut(cx).set_app_menus(vec![
            file_menu.owned(),
            edit_menu.owned(),
            view_menu.owned(),
            help_menu.owned(),
        ]);

        cx.bind_keys([
            KeyBinding::new("cmd-s", workspace::Save, None),
            KeyBinding::new("cmd-b", workspace::ToggleProjectPanel, None),
            KeyBinding::new("cmd-j", workspace::ToggleLowerPanel, None),
            KeyBinding::new("cmd-shift-s", workspace::TogglePreview, None),
        ]);

        let state = cx.new(|_| app_state::AppState::new());

        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(1024.), px(768.)), cx)),
            titlebar: Some(gpui_component::TitleBar::title_bar_options()),
            ..Default::default()
        };

        cx.spawn(async move |cx| {
            cx.open_window(window_options, |window, cx| {
                let view =
                    cx.new(|cx| workspace::Workspace::new(state, window, cx));
                cx.new(|cx| gpui_component::Root::new(view, window, cx))
            })
            .expect("Failed to open window");
        })
        .detach();
    });
}
