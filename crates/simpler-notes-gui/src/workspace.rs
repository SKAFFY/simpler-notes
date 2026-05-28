use std::path::PathBuf;

use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::{
    ActiveTheme, IconName, Selectable,
    sidebar::{
        Sidebar, SidebarCollapsible, SidebarGroup, SidebarHeader, SidebarMenu,
        SidebarMenuItem, SidebarToggleButton,
    },
    tab::{Tab, TabBar},
};

use crate::app_state::AppState;

pub struct Workspace {
    state: Entity<AppState>,
}

impl Workspace {
    pub fn new(state: Entity<AppState>, _cx: &mut Context<Self>) -> Self {
        Self { state }
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let files: Vec<PathBuf> = if state.vault_path.is_some() {
            state.list_markdown_files()
        } else {
            Vec::new()
        };

        let icon_collapsed = state.collapsed;

        let menu_items: Vec<SidebarMenuItem> = files
            .into_iter()
            .map(|path| {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                let weak = self.state.clone().downgrade();
                SidebarMenuItem::new(name)
                    .on_click(move |_, _window, cx| {
                        _window.prevent_default();
                        if let Some(state) = weak.upgrade() {
                            let _ = state.update(cx, |s, cx| {
                                s.open_file(path.clone(), cx);
                            });
                        }
                    })
            })
            .collect();

        let tab_items: Vec<Tab> = state
            .open_tabs
            .iter()
            .enumerate()
            .map(|(ix, tab)| {
                let selected = state.active_tab == Some(ix);
                let weak = self.state.clone().downgrade();
                Tab::new()
                    .label(tab.title.as_str())
                    .selected(selected)
                    .on_click(move |_, _window, cx| {
                        _window.prevent_default();
                        if let Some(state) = weak.upgrade() {
                            let _ = state.update(cx, |s, cx| s.select_tab(ix, cx));
                        }
                    })
            })
            .collect();

        let editor_content: gpui::AnyElement = match state.active_tab {
            Some(idx) => match state.open_tabs.get(idx) {
                Some(tab) => {
                    let processed =
                        crate::editor::preview::process_wikilinks(&tab.source_content);
                    div()
                        .size_full()
                        .text_color(cx.theme().foreground)
                        .child(processed)
                        .into_any_element()
                }
                None => div().size_full().into_any_element(),
            },
            None => div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(cx.theme().muted_foreground)
                .child("Open a file to start editing")
                .into_any_element(),
        };

        let open_vault_weak = self.state.clone().downgrade();

        div()
            .flex()
            .flex_row()
            .size_full()
            .bg(cx.theme().background)
            .child(
                Sidebar::new("file-tree")
                    .collapsible(SidebarCollapsible::Icon)
                    .collapsed(icon_collapsed)
                    .w(px(240.))
                    .header(
                        SidebarHeader::new().child(
                            div()
                                .flex()
                                .flex_row()
                                .gap_2()
                                .child(IconName::BookOpen)
                                .when(!icon_collapsed, |this| this.child("Simpler Notes")),
                        ),
                    )
                    .child(
                        SidebarGroup::new("Vault").child({
                            let mut items: Vec<SidebarMenuItem> = Vec::new();
                            items.push(
                                SidebarMenuItem::new(if state.vault_path.is_some() {
                                    "Change folder..."
                                } else {
                                    "Open folder..."
                                })
                                .icon(IconName::FolderOpen)
                                .on_click(move |_, _window, cx| {
                                    let h = open_vault_weak.clone();
                                    cx.spawn(async move |cx| {
                                        if let Some(path) = rfd::AsyncFileDialog::new().pick_folder().await {
                                            if let Some(state) = h.upgrade() {
                                                let _ = state.update(cx, |s, cx| {
                                                    s.open_vault(&path.path().to_path_buf(), cx);
                                                });
                                            }
                                        }
                                    })
                                    .detach();
                                }),
                            );
                            items.extend(menu_items);
                            SidebarMenu::new().children(items)
                        })
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .h_full()
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .h(px(34.))
                            .items_center()
                            .px_2()
                            .gap_2()
                            .bg(cx.theme().background)
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .child(
                                SidebarToggleButton::new()
                                    .collapsed(icon_collapsed)
                                    .on_click(cx.listener(|this, _, _window, cx| {
                                        this.state.update(cx, |s, cx| {
                                            s.toggle_collapsed(cx);
                                        });
                                    })),
                            )
                            .when(!tab_items.is_empty(), |this| {
                                this.child(
                                    TabBar::new("open-tabs")
                                        .children(tab_items)
                                        .selected_index(state.active_tab.unwrap_or(0)),
                                )
                            }),
                    )
                    .child(
                        div()
                            .flex_1()
                            .p_4()
                            .child(editor_content),
                    ),
            )
    }
}

