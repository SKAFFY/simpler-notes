use std::path::PathBuf;

use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::list::ListItem;
use gpui_component::{
    menu::AppMenuBar,
    scroll::ScrollableElement,
    sidebar::SidebarToggleButton,
    tab::{Tab, TabBar, TabVariant},
    tree::{tree, TreeItem, TreeState},
    ActiveTheme, Icon, IconName, Selectable, TitleBar,
};

use crate::app_state::{AppState, EditorMode, LowerPanelTab};

actions!(simpler_notes, [Save, OpenVault, CloseVault, Exit, ToggleProjectPanel, ToggleLowerPanel, TogglePreview]);

pub struct Workspace {
    state: Entity<AppState>,
    editor: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
    last_active_path: Option<PathBuf>,
    app_menu_bar: Entity<AppMenuBar>,
    tree_state: Entity<TreeState>,
    last_vault_path: Option<PathBuf>,
}

impl Workspace {
    pub fn new(state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let editor = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .soft_wrap(true)
                .placeholder("Start typing...")
        });

        let weak_editor = editor.downgrade();
        let weak_state = state.downgrade();

        let _subscriptions = vec![cx.subscribe(
            &editor,
            move |_this, _editor, event: &InputEvent, cx| {
                if let InputEvent::Change = event {
                    let path = weak_state
                        .upgrade()
                        .and_then(|s| s.read(cx).active_tab_path());
                    if let Some(path) = path {
                        if let Some(editor) = weak_editor.upgrade() {
                            let content = editor.read(cx).text().to_string();
                            if let Some(state) = weak_state.upgrade() {
                                if let Some(vault) = &state.read(cx).vault {
                                    vault.buffer.write().update(&path, content);
                                }
                            }
                        }
                    }
                }
            },
        )];

        let app_menu_bar = AppMenuBar::new(cx);

        let tree_state = cx.new(|cx| TreeState::new(cx));

        Self {
            state,
            editor,
            _subscriptions,
            last_active_path: None,
            app_menu_bar,
            tree_state,
            last_vault_path: None,
        }
    }

    fn save_current(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let path = self.state.read(cx).active_tab_path();
        if let Some(path) = path {
            let content = self.editor.read(cx).text().to_string();
            let state = self.state.read(cx);
            if let Some(vault) = &state.vault {
                vault.buffer.write().update(&path, content.clone());
                vault.buffer.write().save(&path);

                let rel_path = pathdiff::diff_paths(&path, &vault.config.path)
                    .unwrap_or_else(|| path.clone());
                if let Err(e) = vault.write_note(&rel_path, &content) {
                    eprintln!("Failed to save: {}", e);
                }
            }
        }
    }

    fn sync_editor_content(&mut self, path: &std::path::Path, window: &mut Window, cx: &mut Context<Self>) {
        let target_path = path.to_path_buf();
        self.editor.update(cx, |editor, cx| {
            let content = std::fs::read_to_string(&target_path).unwrap_or_default();
            editor.set_value(content, window, cx);
        });
    }

    fn save_editor_to_buffer(&self, path: &std::path::Path, cx: &mut Context<Self>) {
        let content = self.editor.read(cx).text().to_string();
        if let Some(vault) = &self.state.read(cx).vault {
            vault.buffer.write().update(path, content);
        }
    }
}

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current_path = self.state.read(cx).active_tab_path();

        if current_path != self.last_active_path {
            if let Some(old_path) = &self.last_active_path {
                self.save_editor_to_buffer(old_path, cx);
            }
            if let Some(path) = &current_path {
                self.sync_editor_content(path, window, cx);
            }
            self.last_active_path = current_path.clone();
        }

        let vault_path: Option<PathBuf> = {
            let s = self.state.read(cx);
            s.vault.as_ref().map(|v| v.config.path.clone())
        };

        if vault_path != self.last_vault_path {
            let items = match &vault_path {
                Some(path) => build_file_tree(path),
                None => Vec::new(),
            };
            self.tree_state.update(cx, |t, cx| t.set_items(items, cx));
            self.last_vault_path = vault_path;
        }

        let state = self.state.read(cx);

        let vault_path_display = state
            .vault
            .as_ref()
            .map(|v| v.config.path.to_string_lossy().to_string());

        let editor_mode = state.editor_mode;
        let lower_panel_visible = state.lower_panel_visible;
        let lower_panel_active_tab = state.lower_panel_active_tab;

        let tab_items: Vec<Tab> = state
            .open_tabs
            .iter()
            .enumerate()
            .map(|(ix, tab)| {
                let selected = state.active_tab == Some(ix);
                let weak = self.state.clone().downgrade();
                let close_weak = self.state.clone().downgrade();
                let close_hover_bg = cx.theme().colors.selection;
                Tab::new()
                    .label(tab.title.as_str())
                    .selected(selected)
                    .on_click(move |_, _window, cx| {
                        _window.prevent_default();
                        if let Some(state) = weak.upgrade() {
                            state.update(cx, |s, cx| s.select_tab(ix, cx));
                        }
                    })
                    .suffix(
                        div()
                            .id(("close", ix))
                            .cursor_pointer()
                            .rounded_sm()
                            .p_0p5()
                            .hover(move |s| s.bg(close_hover_bg))
                            .child(Icon::new(IconName::Close).size_3())
                            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                                window.prevent_default();
                                cx.stop_propagation();
                                if let Some(state) = close_weak.upgrade() {
                                    state.update(cx, |s, cx| s.close_tab(ix, cx));
                                }
                            }),
                    )
            })
            .collect();

        let has_active_tab = current_path.is_some();

        let editor_area: AnyElement = if has_active_tab {
            let input_element = Input::new(&self.editor)
                .bordered(false)
                .p_0()
                .h_full()
                .font_family(cx.theme().mono_font_family.clone())
                .text_size(cx.theme().mono_font_size)
                .focus_bordered(false);
            match editor_mode {
                EditorMode::Source => input_element.into_any_element(),
                EditorMode::Preview => {
                    let content = self.editor.read(cx).text().to_string();
                    div()
                        .p_4()
                        .size_full()
                        .overflow_y_scrollbar()
                        .child(content)
                        .into_any_element()
                }
                EditorMode::Split => {
                    let content = self.editor.read(cx).text().to_string();
                    div()
                        .flex()
                        .flex_row()
                        .flex_1()
                        .child(div().flex_1().min_w_0().child(input_element))
                        .child(div().w(px(1.)).bg(cx.theme().border))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .p_4()
                                .overflow_y_scrollbar()
                                .child(content),
                        )
                        .into_any_element()
                }
            }
        } else if state.vault.is_some() {
            empty_editor(cx)
        } else {
            welcome_screen(cx)
        };

        let lower_panel = if lower_panel_visible {
            let search_content = div()
                .p_4()
                .child("Search (coming soon)")
                .into_any_element();
            let diagnostics_content: AnyElement = div()
                .p_4()
                .child("No diagnostics")
                .into_any_element();
            let panel_content = match lower_panel_active_tab {
                LowerPanelTab::Search => search_content,
                LowerPanelTab::Diagnostics => diagnostics_content,
                _ => div().into_any_element(),
            };
            let weak_search = self.state.clone().downgrade();
            let weak_diag = self.state.clone().downgrade();
            Some(
                div()
                    .flex()
                    .flex_col()
                    .bg(cx.theme().background)
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .h(px(state.lower_panel_height))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .h(px(28.))
                            .items_center()
                            .px_2()
                            .gap_1()
                            .bg(cx.theme().title_bar)
                            .child(
                                Tab::new()
                                    .label("Search")
                                    .selected(lower_panel_active_tab == LowerPanelTab::Search)
                                    .with_variant(TabVariant::Pill)
                                    .on_click(move |_, _, cx| {
                                        if let Some(s) = weak_search.upgrade() {
                                            s.update(cx, |s, cx| {
                                                s.set_lower_panel_tab(LowerPanelTab::Search, cx);
                                            });
                                        }
                                    }),
                            )
                            .child(
                                Tab::new()
                                    .label("Diagnostics")
                                    .selected(lower_panel_active_tab == LowerPanelTab::Diagnostics)
                                    .with_variant(TabVariant::Pill)
                                    .on_click(move |_, _, cx| {
                                        if let Some(s) = weak_diag.upgrade() {
                                            s.update(cx, |s, cx| {
                                                s.set_lower_panel_tab(LowerPanelTab::Diagnostics, cx);
                                            });
                                        }
                                    }),
                            ),
                    )
                    .child(div().flex_1().child(panel_content)),
            )
        } else {
            None
        };

        div()
            .flex()
            .flex_col()
            .size_full()
            .key_context("Workspace")
            .bg(cx.theme().background)
            .on_action(cx.listener(|this, _: &Save, window, cx| {
                this.save_current(window, cx);
            }))
            .on_action(cx.listener(|this, _: &OpenVault, _, cx| {
                let weak = this.state.downgrade();
                cx.spawn(|_this: WeakEntity<Workspace>, app: &mut AsyncApp| {
                    let mut app = app.clone();
                    async move {
                        if let Some(path) = rfd::AsyncFileDialog::new().pick_folder().await {
                            if let Some(state) = weak.upgrade() {
                                state.update(&mut app, |s, cx| {
                                    s.open_vault(path.path().into(), cx);
                                });
                            }
                        }
                    }
                })
                .detach();
            }))
            .on_action(cx.listener(|this, _: &CloseVault, _, cx| {
                this.state.update(cx, |s, cx| s.close_vault(cx));
            }))
            .on_action(cx.listener(|_, _: &Exit, window, _| {
                window.remove_window();
            }))
            .on_action(cx.listener(|this, _: &ToggleProjectPanel, _, cx| {
                this.state.update(cx, |s, cx| s.toggle_project_panel(cx));
            }))
            .on_action(cx.listener(|this, _: &ToggleLowerPanel, _, cx| {
                this.state.update(cx, |s, cx| s.toggle_lower_panel(cx));
            }))
            .on_action(cx.listener(|this, _: &TogglePreview, _, cx| {
                this.state.update(cx, |s, cx| s.cycle_editor_mode(EditorMode::Preview, cx));
            }))
            .child(
                TitleBar::new()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .h_full()
                            .child(self.app_menu_bar.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .h_full()
                            .gap_2()
                            .child(
                                SidebarToggleButton::new()
                                    .collapsed(!state.project_panel_visible)
                                    .on_click(cx.listener(|this, _, _window, cx| {
                                        this.state.update(cx, |s, cx| {
                                            s.toggle_project_panel(cx);
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
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .min_h_0()
                    .when(state.project_panel_visible, |this| {
                        this.child(
                            div()
                                .flex()
                                .flex_col()
                                .flex_shrink_0()
                                .h_full()
                                .overflow_hidden()
                                .bg(cx.theme().sidebar)
                                .text_color(cx.theme().sidebar_foreground)
                                .border_r_1()
                                .border_color(cx.theme().sidebar_border)
                                .w(px(state.project_panel_width))
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .gap_2()
                                        .pt_3()
                                        .px_3()
                                        .child(Icon::new(IconName::BookOpen))
                                        .child("Simpler Notes"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .size_full()
                                        .px_3()
                                        .pt_3()
                                        .gap_y_3()
                                        .overflow_y_scrollbar()
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .flex_1()
                                                .min_h_0()
                                                .child(
                                                    ListItem::new("open-folder")
                                                        .child(Icon::new(IconName::FolderOpen).size_3())
                                                        .child(" ")
                                                        .child(if vault_path_display.is_some() {
                                                            "Change folder..."
                                                        } else {
                                                            "Open folder..."
                                                        })
                                                        .on_click({
                                                            let weak = self.state.clone().downgrade();
                                                            move |_, _window, cx| {
                                                                let h = weak.clone();
                                                                cx.spawn(async move |cx| {
                                                                    if let Some(path) =
                                                                        rfd::AsyncFileDialog::new().pick_folder().await
                                                                    {
                                                                        if let Some(state) = h.upgrade() {
                                                                            state.update(cx, |s, cx| {
                                                                                s.open_vault(path.path().into(), cx);
                                                                            });
                                                                        }
                                                                    }
                                                                })
                                                                .detach();
                                                            }
                                                        }),
                                                )
                                                .child({
                                                    let state_entity = self.state.clone();
                                                    tree(&self.tree_state, move |ix, entry, _selected, _window, cx| {
                                                        let state = state_entity.read(cx);
                                                        let vault_root = match &state.vault {
                                                            Some(v) => v.config.path.clone(),
                                                            None => return ListItem::new(ix).child(""),
                                                        };
                                                        let rel_path = std::path::Path::new(entry.item().id.as_str());
                                                        let full_path = vault_root.join(rel_path);
                                                        let is_dir = full_path.is_dir();

                                                        let mut item = ListItem::new(entry.item().id.clone())
                                                            .pl(px(12.) * entry.depth() as f32);

                                                        if is_dir {
                                                            let icon = if entry.is_expanded() {
                                                                IconName::FolderOpen
                                                            } else {
                                                                IconName::FolderClosed
                                                            };
                                                            item = item
                                                                .child(Icon::new(icon).size_3())
                                                                .child(" ")
                                                                .child(entry.item().label.clone());
                                                        } else {
                                                            let has_diag = state
                                                                .vault
                                                                .as_ref()
                                                                .map(|v| !v.get_diagnostics(rel_path).is_empty())
                                                                .unwrap_or(false);
                                                            let file_icon = if has_diag {
                                                                IconName::CircleX
                                                            } else {
                                                                IconName::File
                                                            };
                                                            let weak = state_entity.downgrade();
                                                            let fp = full_path.clone();
                                                            item = item
                                                                .child(Icon::new(file_icon).size_3())
                                                                .child(" ")
                                                                .child(entry.item().label.clone())
                                                                .on_click(move |_, window, cx| {
                                                                    window.prevent_default();
                                                                    if let Some(state) = weak.upgrade() {
                                                                        state.update(cx, |s, cx| {
                                                                            s.open_file(fp.clone(), cx);
                                                                        });
                                                                    }
                                                                });
                                                        }
                                                        item
                                                    })
                                                }),
                                        ),
                                ),
                        )
                    })
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .child(div().flex_1().child(editor_area))
                            .when_some(lower_panel, |this, panel| this.child(panel)),
                    ),
            )
    }
}

fn build_file_tree(root: &std::path::Path) -> Vec<TreeItem> {
    let mut entries = Vec::new();
    let Ok(dir) = std::fs::read_dir(root) else {
        return Vec::new();
    };

    for entry in dir.filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let id = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        if path.is_dir() {
            let children = build_file_tree(&path);
            entries.push(TreeItem::new(id, name).expanded(true).children(children));
        } else {
            entries.push(TreeItem::new(id, name));
        }
    }

    entries.sort_by(|a, b| {
        let a_is_dir = a.is_folder();
        let b_is_dir = b.is_folder();
        if a_is_dir != b_is_dir {
            b_is_dir.cmp(&a_is_dir)
        } else {
            a.label.to_lowercase().cmp(&b.label.to_lowercase())
        }
    });

    entries
}

fn empty_editor(cx: &App) -> AnyElement {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .text_color(cx.theme().muted_foreground)
        .child("Open a file to start editing")
        .into_any_element()
}

fn welcome_screen(cx: &App) -> AnyElement {
    div()
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_4()
        .text_color(cx.theme().muted_foreground)
        .child(div().text_xl().child("Welcome to Simpler Notes!"))
        .child(div().child("Open a folder with markdown notes to get started."))
        .into_any_element()
}
