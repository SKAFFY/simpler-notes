mod app;
mod workspace;

use app::App;

pub fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .theme(App::theme)
        .run()
}
