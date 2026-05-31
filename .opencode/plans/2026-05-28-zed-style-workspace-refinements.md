# Zed-Style Workspace Refinements

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bring the simpler-notes GUI look closer to the Zed editor: replace the gpui-component `Sidebar` with gpui-component `DockArea` (with built-in resize handle), adopt Zed-like colors/spacing in the tab bar, and restyle the editor area.

**Architecture:** Use gpui-component's `DockArea` as the top-level layout container. The file tree lives in a left Dock, the editor area (with a gpui-component `TabBar`) lives in the Center dock. All docks get default sizes and resize handles for free. Colors come from `cx.theme()` (gpui-component's ActiveTheme).

**Tech Stack:** Rust, gpui, gpui-component (DockArea, DockItem, Panel, TabBar, Tab), gpui-component-assets.

**Constraints:**
- Still single editor (no split pane yet)
- No status bar yet
- AppState stays as-is
- gpui-component TabBar is fine; just restyle it

---

### Task 1: Adopt gpui-component DockArea for workspace layout

**Files:**
- Modify: `crates/simpler-notes-gui/src/workspace.rs`
- Modify: `crates/simpler-notes-gui/src/main.rs`

Replace the current `Sidebar + div` toolbar layout with gpui-component `DockArea`.

- [ ] **Step 1: Update imports in workspace.rs**

```rust
use std::path::PathBuf;
use std::sync::Arc;

use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::{
    ActiveTheme, Focusable,
    dock::{
        DockArea, DockItem, DockPlacement, Panel, PanelControl, PanelEvent, PanelView,
    },
    tab::{Tab, TabBar},
    sidebar::{SidebarGroup, SidebarMenu, SidebarMenuItem},
    IconName,
};

use crate::app_state::AppState;
```

- [ ] **Step 2: Create FileTreePanel struct implementing `Panel` trait**

Add to `workspace.rs`:

```rust
pub struct FileTreePanel {
    state: Entity<AppState>,
}

impl FileTreePanel {
    pub fn new(state: Entity<AppState>, _cx: &mut Context<Self>) -> Self {
        Self { state }
    }
}

impl EventEmitter<PanelEvent> for FileTreePanel {}
impl Focusable for FileTreePanel {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        cx.focus_handle(&self.state)
    }
}

impl Render for FileTreePanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);

        let vault_button = SidebarMenuItem::new(if state.vault_path.is_some() {
            "Change folder..."
        } else {
            "Open folder..."
        })
        .icon(IconName::FolderOpen);

        let file_items: Vec<SidebarMenuItem> = state
            .list_markdown_files()
            .into_iter()
            .map(|path| {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                let weak = self.state.clone().downgrade();
                SidebarMenuItem::new(name)
                    .on_click(move |_, _window, cx| {
                        if let Some(state) = weak.upgrade() {
                            let _ = state.update(cx, |s, cx| {
                                s.open_file(path.clone(), cx);
                            });
                        }
                    })
            })
            .collect();

        let mut all_items = vec![vault_button];
        all_items.extend(file_items);

        div()
            .size_full()
            .overflow_y_scroll()
            .bg(cx.theme().panel_background)
            .child(
                SidebarGroup::new("Vault")
                    .child(SidebarMenu::new().children(all_items))
            )
    }
}

impl Panel for FileTreePanel {
    fn panel_name(&self) -> &'static str {
        "FileTree"
    }

    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        "Files".to_string()
    }

    fn closable(&self, _cx: &App) -> bool {
        false
    }

    fn zoomable(&self, _cx: &App) -> Option<PanelControl> {
        None
    }
}
```

- [ ] **Step 3: Create EditorPanel struct implementing `Panel` trait**

Add to `workspace.rs`:

```rust
pub struct EditorPanel {
    state: Entity<AppState>,
}

impl EditorPanel {
    pub fn new(state: Entity<AppState>, _cx: &mut Context<Self>) -> Self {
        Self { state }
    }
}

impl EventEmitter<PanelEvent> for EditorPanel {}
impl Focusable for EditorPanel {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        cx.focus_handle(&self.state)
    }
}

impl Render for EditorPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);

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
                        if let Some(state) = weak.upgrade() {
                            let _ = state.update(cx, |s, cx| s.select_tab(ix, cx));
                        }
                    })
                    .closable(true)
                    .on_close(move |_, _window, cx| {
                        if let Some(state) = weak.upgrade() {
                            let _ = state.update(cx, |s, cx| s.close_tab(ix, cx));
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
                        .p_4()
                        .font_family("JetBrains Mono, Fira Code, monospace".into())
                        .text_size(px(14.))
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

        let has_tabs = !tab_items.is_empty();
        let active_idx = state.active_tab.unwrap_or(0);

        div()
            .flex()
            .flex_col()
            .size_full()
            .child(
                div().when(has_tabs, |this| {
                    this.child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .h(px(34.))
                            .bg(cx.theme().tab_bar_background)
                            .border_b_1()
                            .border_color(cx.theme().border_variant)
                            .child(
                                TabBar::new("open-tabs")
                                    .children(tab_items)
                                    .selected_index(active_idx),
                            ),
                    )
                }),
            )
            .child(
                div()
                    .flex_1()
                    .bg(cx.theme().background)
                    .child(editor_content),
            )
    }
}

impl Panel for EditorPanel {
    fn panel_name(&self) -> &'static str {
        "Editor"
    }

    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        "Editor".to_string()
    }

    fn closable(&self, _cx: &App) -> bool {
        false
    }

    fn zoomable(&self, _cx: &App) -> Option<PanelControl> {
        None
    }
}
```

- [ ] **Step 4: Rewrite Workspace struct to use DockArea**

```rust
pub struct Workspace {
    dock_area: Entity<DockArea>,
    _file_tree_panel: Entity<FileTreePanel>,
    _editor_panel: Entity<EditorPanel>,
}

impl Workspace {
    pub fn new(state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let dock_area = cx.new(|cx| DockArea::new("main", None, window, cx));
        let file_tree = cx.new(|cx| FileTreePanel::new(state.clone(), cx));
        let editor = cx.new(|cx| EditorPanel::new(state.clone(), cx));

        let weak = dock_area.downgrade();
        dock_area.update(cx, |dock, cx| {
            let center = DockItem::tabs(vec![Arc::new(editor.clone())], &weak, window, cx);
            dock.set_center(center, window, cx);

            let left = DockItem::tab(Arc::new(file_tree.clone()), &weak, window, cx);
            dock.set_left_dock(left, Some(px(240.)), true, window, cx2);
        });

        Self {
            dock_area,
            _file_tree_panel: file_tree,
            _editor_panel: editor,
        }
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.dock_area.clone())
    }
}
```

- [ ] **Step 5: Update main.rs — pass window to Workspace::new**

```rust
mod app_state;
mod editor;
mod workspace;

use gpui::*;
use gpui_platform::application;
use gpui_component_assets::Assets;

fn main() {
    let app = application().with_assets(Assetsende;

    app.run(move |cx| {
        gpui_component::init(cx);

        let state = cx.new(|_| app_state::AppState::new());
        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(1024.), px(768.)), cx)),
            ..Default::default()
        };

        cx.spawn(async move |cx| {
            cx.open_window(window_options, |window, cx| {
                let view = cx.new(|cx| workspace::Workspace::new(state, window, cx));
                cx.new(|cx| gpui_component::Root::new(view, window, cx))
            })
            .expect("Failed to open window");
        })
        .detach();
    });
}
```

- [ ] **Step 6: Sync to Linux, build, fix compilation errors**

```bash
scp + ssh "cargo build -p simpler-notes-gui 2>&1"
```

Typical issues to fix:
- `cx2` is a typo — should be `cx` (the closure parameter)
- `PanelView` needs `Arc<dyn PanelView>` — check if `Entity<T>` implements `PanelView` (it does per gpui-component: `impl<T: Panel> PanelView for Entity<T>`)
- `DockItem::tab` and `DockItem::tabs` signatures — verify from example
- `Focusable` import path

Fix errors until 0 errors.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(gui): adopt DockArea for workspace layout with resize handles"
```

---

### Task 2: Enable dock collapsing to icon strip

**Files:**
- Modify: `crates/simpler-notes-gui/src/workspace.rs`

- [ ] **Step 1: Set left dock collapsible and toggle buttons visible**

In `Workspace::new`, after setting docks:

```rust
dock.set_dock_collapsible(
    gpui::Edges { left: true, ..Default::default() },
    window,
    cx,
);
dock.set_toggle_button_visible(true, cx);
```

- [ ] **Step 2: Build and test on Linux**

Run: `cargo build -p simpler-notes-gui`

- [ ] **Step 3: Commit**

```bash
git commit -a -m "feat(gui): enable dock collapsing with icon strip"
```

---

### Task 3: Wire vault open button click in FileTreePanel

**Files:**
- Modify: `crates/simpler-notes-gui/src/workspace.rs`

The vault button in Step 2 is created without `on_click`. Wire it up properly.

- [ ] **Step 1: Add on_click to vault button**

```rust
let vault_button = {
    let h = self.state.clone().downgrade();
    SidebarMenuItem::new(if state.vault_path.is_some() {
        "Change folder..."
    } else {
        "Open folder..."
    })
    .icon(IconName::FolderOpen)
    .on_click(move |_, _window, cx| {
        let weak = h.clone();
        cx.spawn(async move |cx| {
            if let Some(path) = rfd::AsyncFileDialog::new().pick_folder().await {
                if let Some(state) = weak.upgrade() {
                    let _ = state.update(cx, |s, cx| {
                        s.open_vault(&path.path().to_path_buf(), cx);
                    });
                }
            }
        })
        .detach();
    })
};
```

- [ ] **Step 2: Build and verify**

- [ ] **Step 3: Commit**

```bash
git commit -a -m "fix(gui): wire vault open button in file tree panel"
```

---

### Task 4: Remove unused code and clean up warnings

**Files:**
- Modify: `crates/simpler-notes-gui/src/app_state.rs`
- Modify: `crates/simpler-notes-gui/src/workspace.rs`

- [ ] **Step 1: Remove EditorMode::Split and EditorMode::Preview**

```rust
#[derive(PartialEq, Clone, Copy)]
pub enum EditorMode {
    Source,
}
```

Remove `EditorMode` import from workspace.rs if no longer used there (it won't be — only `EditorMode::Source` is set in `open_file`).

- [ ] **Step 2: Remove `collapsed` from AppState if no longer used**

Check if anything reads `state.collapsed` — after DockArea takes over, the manual `collapsed` flag is obsolete. Remove it and `toggle_collapsed`.

- [ ] **Step 3: Remove unused imports**

Check for unused `SidebarToggleButton`, `SidebarCollapsible`, etc.

- [ ] **Step 4: Build with 0 warnings**

Run: `cargo build -p simpler-notes-gui 2>&1 | grep -c warning`
Expected: `0`

- [ ] **Step 5: Commit**

```bash
git commit -a -m "chore: remove dead code and unused imports"
```
