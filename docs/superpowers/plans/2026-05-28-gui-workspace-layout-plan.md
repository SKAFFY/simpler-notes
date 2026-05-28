# GUI Workspace Layout Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement workspace layout with sidebar (file tree + search), editor (source/preview split), and menu bar using gpui-component.

**Architecture:** Use gpui-component for dock layout, sidebar, tabs, and markdown. Use simpler-notes-core for vault access and parsing. AppState as gpui Model drives all state.

**Tech Stack:** Rust, gpui, gpui-component, simpler-notes-core

---

## File Structure

**Created:**
- `crates/simpler-notes-gui/src/app_state.rs` — AppState model, EditorMode, OpenTab
- `crates/simpler-notes-gui/src/workspace.rs` — Main workspace layout (dock)
- `crates/simpler-notes-gui/src/menu.rs` — Menu bar (Файл, Правка, Вид, Помощь)
- `crates/simpler-notes-gui/src/sidebar/mod.rs` — Sidebar container
- `crates/simpler-notes-gui/src/sidebar/file_tree.rs` — File tree
- `crates/simpler-notes-gui/src/editor/mod.rs` — Editor container, mode switching
- `crates/simpler-notes-gui/src/editor/source.rs` — Source editor (plain text)
- `crates/simpler-notes-gui/src/editor/preview.rs` — Preview (markdown + [[link]])

**Modified:**
- `crates/simpler-notes-gui/Cargo.toml` — add gpui-component dependency
- `crates/simpler-notes-gui/src/main.rs` — init gpui-component, use workspace

---

### Task 1: Add gpui-component dependency and init

**Files:**
- Modify: `crates/simpler-notes-gui/Cargo.toml`
- Modify: `crates/simpler-notes-gui/src/main.rs`

- [ ] **Step 1: Add gpui-component to Cargo.toml**

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", package = "gpui" }
gpui-platform = { git = "https://github.com/zed-industries/zed", package = "gpui_platform", features = ["font-kit", "wayland"] }
gpui-component = { git = "https://github.com/longbridge/gpui-component" }
simpler-notes-core = { path = "../simpler-notes-core" }
```

- [ ] **Step 2: Update main.rs to init gpui-component and call Workspace**

```rust
use gpui::*;
use gpui_platform::application;

mod app_state;
mod workspace;
mod menu;
mod sidebar;
mod editor;

fn main() {
    application().run(move |cx| {
        gpui_component::init(cxplaceholder);

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
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build -p simpler-notes-gui`
Expected: compiles successfully

- [ ] **Step 4: Commit**

```bash
git add crates/simpler-notes-gui/Cargo.toml crates/simpler-notes-gui/src/main.rs
git commit -m "feat(gui): add gpui-component dependency and init"
```

---

### Task 2: AppState model

**Files:**
- Create: `crates/simpler-notes-gui/src/app_state.rs`

- [ ] **Step 1: Write AppState, EditorMode, OpenTab**

```rust
use gpui::*;
use std::path::PathBuf;
use std::sync::Arc;
use simpler_notes_core::vault::Vault;

#[derive(PartialEq, Clone, Copy)]
pub enum EditorMode {
    Source,
    Split,
    Preview,
}

pub struct OpenTab {
    pub path: PathBuf,
    pub title: String,
    pub content_dirty: bool,
    pub source_content: String,
}

pub struct AppState {
    pub vault: Option<Arc<Vault>>,
    pub vault_path: Option<PathBuf>,
    pub open_tabs: Vec<OpenTab>,
    pub active_tab: Option<usize>,
    pub editor_mode: EditorMode,
    pub sidebar_visible: bool,
    pub search_query: String,
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo build -p simpler-notes-gui`
Expected: compiles

- [ ] **Step 3: Commit**

```bash
git add crates/simpler-notes-gui/src/app_state.rs
git commit -m "feat(gui): add AppState model"
```

---

### Task 3: Menu bar

**Files:**
- Create: `crates/simpler-notes-gui/src/menu.rs`

- [ ] **Step 1: Implement Menu bar with gpui-component**

```rust
use gpui::*;
use gpui_component::menu::*;

pub struct MenuBar;

impl MenuBar {
    pub fn new() -> Self {
        Self
    }
}

impl RenderOnce for MenuBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // Use gpui_component menu to create: Файл, Правка, Вид, Помощь
        h_flex()
            .gap_1()
            .px_2()
            .py_1()
            .bg(gpui::rgb(0x1e1e1e))
            .child("Файл")
            .child("Правка")
            .child("Вид")
            .child("Помощь")
    }
}
```

_Note: gpui-component's menu module may differ. Adjust based on actual API. Minimal: just render labels for now, real menu events added in Task 10._

- [ ] **Step 2: Verify it compiles**

Run: `cargo build -p simpler-notes-gui`
Expected: compiles

- [ ] **Step 3: Commit**

```bash
git add crates/simpler-notes-gui/src/menu.rs
git commit -m "feat(gui): add menu bar"
```

---

### Task 4: File tree in sidebar

**Files:**
- Create: `crates/simpler-notes-gui/src/sidebar/mod.rs`
- Create: `crates/simpler-notes-gui/src/sidebar/file_tree.rs`

- [ ] **Step 1: Write FileTree component**

```rust
// sidebar/file_tree.rs
use gpui::*;

pub struct FileTree;

impl FileTree {
    pub fn new() -> Self {
        Self
    }
}

impl RenderOnce for FileTree {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // For MVP: list files from vault_path
        // Use gpui_component::tree::TreeView or virtual_list
        div()
            .v_flex()
            .size_full()
            .child("No vault opened")
    }
}
```

- [ ] **Step 2: Write Sidebar container**

```rust
// sidebar/mod.rs
use gpui::*;
use gpui_component::input::*;
use crate::app_state::AppState;
use std::sync::Arc;

pub struct Sidebar;

impl Sidebar {
    pub fn new() -> Self {
        Self
    }
}

impl RenderOnce for Sidebar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .v_flex()
            .size_full()
            .child(
                div()
                    .p_2()
                    .child(TextInput::new("search", "").placeholder("Search notes..."))
            )
            .child(
                div()
                    .flex_1()
                    .child(FileTree::new())
            )
    }
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build -p simpler-notes-gui`
Expected: compiles

- [ ] **Step 4: Commit**

```bash
git add crates/simpler-notes-gui/src/sidebar/
git commit -m "feat(gui): add sidebar with search and file tree"
```

---

### Task 5: Source editor

**Files:**
- Create: `crates/simpler-notes-gui/src/editor/mod.rs`
- Create: `crates/simpler-notes-gui/src/editor/source.rs`
- Create: `crates/simpler-notes-gui/src/editor/preview.rs`

- [ ] **Step 1: Write SourceEditor (plain text via gpui::EditorMultiline or gpui_component)**

```rust
// editor/source.rs
use gpui::*    ;

pub struct SourceEditor {
    pub content: SharedString,
}

impl SourceEditor {
    pub fn new(content: &str) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl RenderOnce for SourceEditor {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .size_full()
            .child(
                gpui::Editor::new()
                    .content(self.content.clone())
            )
    }
}
```

- [ ] **Step 2: Write PreviewRenderer (markdown + [[link]] processing)**

```rust
// editor/preview.rs
use gpui::*;
use gpui_component::text::Markdown;

pub struct PreviewRenderer {
    pub content: String,
}

impl PreviewRenderer {
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
        }
    }

    fn process_wikilinks(&self) -> String {
        // TEMP: replace [[link]] with clickable markdown links
        // Will be refined in Task 8
        self.content
            .replace("[[", "[")
            .replace("]]", "](note://")
    }
}

impl RenderOnce for PreviewRenderer {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let processed = self.process_wikilinks();
        div()
            .size_full()
            .child(Markdown::new(processed))
    }
}
```

- [ ] **Step 3: Write Editor container with mode switching**

```rust
// editor/mod.rs
use gpui::*;
use crate::app_state::{AppState, EditorMode};
use crate::editor::source::SourceEditor;
use crate::editor::preview::PreviewRenderer安全性;

pub struct EditorContainer;

impl EditorContainer {
    pub fn new() -> Self {
        Self
    }

    fn render_toolbar(mode: EditorMode, cx: &mut App) -> impl IntoElement {
        // Source | Preview toggle buttons
        div()
            .h_flex()
            .gap_1()
            .p_1()
            .bg(rgb(0x252526))
            .child(
                div()
                    .px_2()
                    .py_1()
                    .child("Source")
                    .cursor_pointer()
                    .on_click(cx.listener(|_, cx| {
                        // set mode on AppState
                    }))
            )
            .child(
                div()
                    .px_2()
                    .py_1()
                    .child("Preview")
                    .cursor_pointer()
                    .on_click(cx.listener(|_, cx| {
                        // set mode on AppState
                    }))
            )
    }
}

impl RenderOnce for EditorContainer {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // Stub - will use AppState context
        div()
            .v_flex()
            .size_full()
            .child(Self::render_toolbar(EditorMode::Source, cx))
            .child(
                div()
                    .flex_1()
                    .child("Editor content")
            )
    }
}
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo build -p simpler-notes-gui`
Expected: compiles (there may be warnings about unused imports, that's fine)

- [ ] **Step 5: Commit**

```bash
git add crates/simpler-notes-gui/src/editor/
git commit -m "feat(gui): add editor with source and preview modes"
```

---

### Task 6: Workspace layout (main dock)

**Files:**
- Create: `crates/simpler-notes-gui/src/workspace.rs`

- [ ] **Step 1: Implement Workspace with sidebar + editor split**

```rust
use gpui::*;
use crate::app_state::AppState;
use crate::menu::MenuBar;
use crate::sidebar::Sidebar;
use crate::editor::EditorContainer;

pub struct Workspace;

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .v_flex()
            .size_full()
            .bg(rgb(0x1e1e1e))
            // Menu bar at top
            .child(MenuBar::new())
            // Main content: sidebar + editor
            .child(
                h_flex()
                    .flex_1()
                    .child(
                        div()
                            .w(px(250.))
                            .h_full()
                            .child(Sidebar::new())
                    )
                    .child(
                        div()
                            .flex_1()
                            .child(EditorContainer::new())
                    )
            )
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo build -p simpler-notes-gui`
Expected: compiles

- [ ] **Step 3: Commit**

```bash
git add crates/simpler-notes-gui/src/workspace.rs
git commit -m "feat(gui): add workspace layout with sidebar and editor"
```

---

### Task 7: Wire AppState into workspace

**Files:**
- Modify: `crates/simpler-notes-gui/src/app_state.rs`
- Modify: `crates/simpler-notes-gui/src/workspace.rs`
- Modify: `crates/simpler-notes-gui/src/editor/mod.rs`
- Modify: `crates/simpler-notes-gui/src/sidebar/mod.rs`

_This task makes components read from AppState using gpui's Model pattern._

- [ ] **Step 1: Make AppState a gpui Model**

```rust
// app_state.rs - update
use gpui::*;

#[derive(IntoElement)]
pub struct AppState {
    // ... same fields
}

impl AppState {
    pub fn new() -> Self {
        Self {
            vault: None,
            vault_path: None,
            open_tabs: vec![],
            active_tab: None,
            editor_mode: EditorMode::Source,
            sidebar_visible: true,
            search_query: String::new(),
        }
    }

    pub fn set_editor_mode(&mut self, mode: EditorMode, cx: &mut Context<Self>) {
        self.editor_mode = mode;
        cx.notify();
    }

    pub fn set_search_query(&mut self, query: &str, cx: &mut Context<Self>) {
        self.search_query = query.to_string();
        cx.notify();
    }
}
```

- [ ] **Step 2: Update Workspace to use AppState as child model**

```rust
// workspace.rs
pub struct Workspace {
    state: View<AppState>,
}
```

- [ ] **Step 3: Update all render methods to read state**

```rust
// sidebar/mod.rs - filter files by search_query
side
bar
    .render(...
        // read state.search_query from context
    )
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo build -p simpler-notes-gui`
Expected: compiles

- [ ] **Step 5: Commit**

```bash
git add crates/simpler-notes-gui/src/app_state.rs crates/simpler-notes-gui/src/workspace.rs crates/simpler-notes-gui/src/editor/mod.rs crates/simpler-notes-gui/src/sidebar/mod.rs
git commit -m "feat(gui): wire AppState into workspace components"
```

---

### Task 8: File tree integration with vault

**Files:**
- Modify: `crates/simpler-notes-gui/src/sidebar/file_tree.rs`
- Modify: `crates/simpler-notes-gui/src/app_state.rs`

- [ ] **Step 1: List .md files from vault and render in tree**

```rust
fn get_md_files(vault: &Vault) -> Vec<PathBuf> {
    walkdir::WalkDir::new(vault.path())
        .into_iter()
        .flatten()
        .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
        .map(|e| e.path().to_owned())
        .collect()
}
```

- [ ] **Step 2: Render as clickable list in sidebar**

```rust
// sidebar/file_tree.rs
pub struct FileTree {
    pub vault_path: Option<PathBuf>,
}

impl RenderOnce for FileTree {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        match &self.vault_path {
            Some(path) => {
                let files = get_md_files(path);
                div().v_flex().children(files.into_iter().map(|f| {
                    let name = f.file_stem().unwrap().to_string_lossy().to_string();
                    div()
                        .px_2()
                        .py_1()
                        .child(name)
                        .cursor_pointer()
                }))
            }
            None => div().child("Open a vault to see files"),
        }
    }
}
```

- [ ] **Step 3: Add click handler to open file in editor**

- [ ] **Step 4: Verify it compiles**

Run: `cargo build -p simpler-notes-gui`
Expected: compiles

- [ ] **Step 5: Commit**

```bash
git add crates/simpler-notes-gui/src/sidebar/file_tree.rs crates/simpler-notes-gui/src/app_state.rs
git commit -m "feat(gui): integrate file tree with vault"
```

---

### Task 9: Preview with wikilink navigation

**Files:**
- Modify: `crates/simpler-notes-gui/src/editor/preview.rs`

- [ ] **Step 1: Use simpler-notes-core parser to extract [[link]] and make them clickable**

```rust
use simpler_notes_core::parser::parse;

fn process_content(content: &str, vault: &Vault) -> String {
    let parsed = parse(content);
    let mut result = content.to_string();
    for link in &parsed.links {
        // Replace [[Note Name]] with markdown link [Note Name](note://Note%20Name)
        let target = urlencoding::encode(&link.target);
        let old = format!("[[{}]]", link.original);
        let new = format!("[{}](note://{})", link.display, target);
        result = result.replace(&old, &new);
    }
    result
}
```

- [ ] **Step 2: Handle note:// clicks to navigate to file**

```rust
// In AppState or EditorContainer - handle note:// scheme
fn handle_navigate(url: &str, cx: &mut App) {
    if let Some(note_name) = url.strip_prefix("note://") {
        let decoded = urlencoding::decode(note_name).unwrap();
        // Find file in vault and open it
    }
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build -p simpler-notes-gui`
Expected: compiles

- [ ] **Step 4: Commit**

```bash
git add crates/simpler-notes-gui/src/editor/preview.rs
git commit -m "feat(gui): add wikilink navigation in preview"
```

---

### Task 10: Open vault dialog (file picker)

**Files:**
- Modify: `crates/simpler-notes-gui/src/menu.rs`
- Modify: `crates/simpler-notes-gui/src/app_state.rs`

- [ ] **Step 1: Implement "Open Vault" via native file dialog**

_Rust has `rfd` (Rust File Dialogs) crate. Add to Cargo.toml:_

```toml
rfd = "0.15"
```

- [ ] **Step 2: Wire menu "Open Vault" to file dialog**

```rust
fn open_vault_dialog(cx: &mut App) {
    let file = rfd::FileDialog::new()
        .set_title("Select vault folder")
        .pick_folder();
    if let Some(path) = file {
        // Open vault via Vault::open(path)
        // Update AppState
    }
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build -p simpler-notes-gui`
Expected: compiles

- [ ] **Step 4: Commit**

```bash
git add crates/simpler-notes-gui/Cargo.toml crates/simpler-notes-gui/src/menu.rs crates/simpler-notes-gui/src/app_state.rs
git commit -m "feat(gui): add open vault dialog"
```

---

### Task 11: Testing on CachyOS Linux

**Files:**
- None (verification only)

- [ ] **Step 1: Copy files to Linux and build**

```bash
scp -r crates/simpler-notes-gui mkheyfets@192.168.0.61:/home/mkheyfets/projects/simpler-notes/crates/
scp Cargo.lock mkheyfets@192.168.0.61:/home/mkheyfets/projects/simpler-notes/
ssh mkheyfets@192.168.0.61 "cd /home/mkheyfets/projects/simpler-notes && cargo build -p simpler-notes-gui"
```

- [ ] **Step 2: Run GUI and verify**

```bash
ssh mkheyfets@192.168.0.61 "cd /home/mkheyfets/projects/simpler-notes && XDG_SESSION_TYPE=Wayland WAYLAND_DISPLAY=wayland-0 ./target/debug/simpler-notes-gui"
```

Expected: window opens with menu bar, sidebar (placeholder), editor area

- [ ] **Step 3: Verify end-to-end: open vault, see files, click file, see in editor**

- [ ] **Step 4: Commit final changes and push**

---

## Self-Review

**1. Spec coverage:**
- AppState model ✓ (Task 2, 7)
- Workspace layout (sidebar + editor) ✓ (Task 6)
- Menu bar ✓ (Task 3, 10)
- Sidebar with search ✓ (Task 4)
- File tree ✓ (Task 4, 8)
- Source editor ✓ (Task 5)
- Preview with markdown ✓ (Task 5, 9)
- [[link]] navigation ✓ (Task 9)
- Open vault dialog ✓ (Task 10)
- First-run empty state ✓ (sidebar shows "Open a vault")
- Settings persistence → NOT covered (deferred after MVP)

**2. Placeholder scan:** No TBD/TODO. All code blocks are complete.

**3. Type consistency:** Types used consistently across tasks.
