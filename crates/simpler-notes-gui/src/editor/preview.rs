use gpui::*;

use crate::app_state::AppState;

fn process_wikilinks(content: &str) -> String {
    let re = regex::Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    re.replace_all(content, |caps: &regex::Captures| {
        let target = &caps[1];
        format!("[{}](note://{})", target, urlencoding::encode(target))
    })
    .to_string()
}

pub struct PreviewRenderer {
    state: View<AppState>,
}

impl PreviewRenderer {
    pub fn new(state: View<AppState>) -> Self {
        Self { state }
    }
}

impl Render for PreviewRenderer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let content = match state.active_tab {
            Some(idx) => state.open_tabs.get(idx).map(|t| t.source_content.clone()),
            None => None,
        };
        let state_handle = self.state.clone();

        match content {
            Some(text) => {
                let processed = process_wikilinks(&text);
                div()
                    .flex_1()
                    .p(8.)
                    .child(gpui_component::text::markdown(processed))
            }
            None => div()
                .flex_1()
                .p(8.)
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x888888))
                        .child("Preview"),
                ),
        }
    }
}
