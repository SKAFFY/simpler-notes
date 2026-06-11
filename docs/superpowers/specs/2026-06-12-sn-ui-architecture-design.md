# sn-ui: Архитектура UI-слоя для Simpler Notes

> **Дата:** 2026-06-12
> **Статус:** черновик
> **Крейт:** `sn-ui` (новый, `crates/sn-ui/`)

---

- [x] Зависимость от gpui и gpui-platform (переиспользуем)
- [x] Зависимость от gpui-component (Tab, TabBar, Tree, Sidebar, Resizable, Theme, Scroll, Input)
- [x] Layer 1: App (Window factory + lifecycle)
- [x] Layer 2: Workspace (корневой контейнер)
- [x] Layer 3: Panel (трейт + PanelView)
- [x] Layer 4: DockItem (рекурсивное дерево)
- [x] Layer 5: DockArea (управление layout)
- [x] Layer 6: StackPanel (split + resize)
- [x] Layer 7: TabPanel (tabs + drag-and-drop)
- [x] Layer 8: Command (actions + keybindings)
- [x] Layer 9: Testing (test utilities)
- [x] Layer 10: Integration (переход simpler-notes на sn-ui)

---

## 1. Мотивация и принципы

### 1.1 Зачем sn-ui?

Текущий `simpler-notes-gui` держит всю раскладку в `workspace.rs` (~580 строк) с ручным flex,
хардкодом панелей и без перетаскивания. Добавление новой панели — это копипаста flex-секций.

sn-ui — это **архитектурный слой** поверх gpui, который даёт:

- **Workspace с произвольным числом панелей** (не только project/editor/lower)
- **Drag-and-drop вкладок** между панелями
- **Выравнивание с snapping** через resize handle'ы
- **Сериализацию layout** (dump/load)
- **Единый PanelView трейт** для всех панелей
- **Команды и keybindings** в единой системе

### 1.2 Границы ответственности

| Делает | Не делает |
|--------|-----------|
| Управляет деревом панелей (DockItem) | Не рендерит контент панелей |
| Создаёт/закрывает окна через App | Не управляет файловой системой |
| Обрабатывает DnD между панелями | Не знает про .md файлы |
| Сохраняет/восстанавливает layout | Не хранит содержимое редактора |
| Предоставляет PanelView трейт | Не определяет бизнес-логику |

### 1.3 Что переиспользуем

| Компонент | Откуда | Назначение |
|-----------|--------|------------|
| `gpui` core | Zed | Entity, Context, App, Window, Element tree, Div, Styled, IntoElement, Render, Focus, Actions, Subscriptions, DnD |
| `gpui-platform` | Zed | platform::Platform, PlatformWindow, PlatformDisplay — окно, event loop, рендеринг |
| `gpui-component` | longbridge | Tab, TabBar, Tree, Sidebar, Resizable, Theme, Scroll, Input, Root, Tooltip, Sheet, Modal |

---

## 2. Структура крейта

```
crates/sn-ui/
├── Cargo.toml
└── src/
    ├── lib.rs                    # Публичное API, реэкспорты
    ├── app.rs                    # App — фабрика окна + lifecycle
    ├── workspace.rs              # Workspace — корневой container
    ├── panel/
    │   ├── mod.rs                # PanelView трейт + Registry
    │   └── traits.rs             # Вспомогательные трейты (Closable, Zoomable)
    ├── dock/
    │   ├── mod.rs                # DockItem (рекурсивный enum)
    │   ├── area.rs               # DockArea — владелец layout'а
    │   ├── stack.rs              # StackPanel — split с resize
    │   ├── tab.rs                # TabPanel — вкладки + DnD
    │   ├── tiles.rs              # Tiles — плавающие окна
    │   └── drag.rs               # DragPanel — ghost DnD view
    ├── command/
    │   ├── mod.rs                # Action-ы + Keybindings
    │   └── palette.rs            # Command Palette (заглушка)
    ├── theme.rs                  # Тема sn-ui (обёртка над theme gpui-component)
    └── testing.rs                # Test utilities: stub window, event helpers
```

---

## 3. Layer 1: App (app.rs)

### 3.1 Назначение

Фабрика окна + lifecycle. Скрывает рутину `gpui::application().with_assets().run()`.

### 3.2 API

```rust
use sn_ui::App;

App::new()
    .with_size(Size { width: px(1024.), height: px(768.) })
    .with_title("Simpler Notes")
    .run(|cx: &mut AppContext| {
        // Инициализация gpui-component, state, и т.д.
        let workspace = cx.new(|_| Workspace::new());
        workspace
    });
```

### 3.3 Внутренности

- `App::run()` принимает замыкание, которое возвращает `Entity<impl Render>` (корневая View)
- Под капотом: `gpui::application().with_assets(...)`, затем `cx.open_window(...)` с `Root::new(view, window, cx)`
- `App` — это builder: `with_size`, `with_title`, `with_min_size`, `with_menu`

### 3.4 Тесты

- `test_app_creates_window` — проверка что окно создаётся без паники (через test platform)
- `test_app_calls_init_callback` — init callback вызывается

---

## 4. Layer 2: Workspace (workspace.rs)

### 4.1 Назначение

Корневой контейнер. Содержит всё приложение. Имплементирует `Render`.

### 4.2 API

```rust
pub struct Workspace {
    state: Entity<WorkspaceState>,
    subscriptions: Vec<Subscription>,
}

pub struct WorkspaceState {
    pub title: SharedString,
    pub dock: DockItem,               // корень dock-дерева
    pub left_dock: Option<Entity<Dock>>,
    pub right_dock: Option<Entity<Dock>>,
    pub bottom_dock: Option<Entity<Dock>>,
    pub lock_panels: bool,
    pub zoomed_panel: Option<EntityId>,
}
```

### 4.3 Поведение

- `Workspace` — единственная View приложения
- Содержит все панели через систему Dock
- Пересоздаётся только при смене vault
- `Workspace::add_panel(panel, placement)` — добавить панель в dock
- `Workspace::remove_panel(panel_id)` — удалить панель
- `WorkspaceState` — Entity, уведомляет через `cx.notify()` при изменениях

### 4.4 Тесты

- `test_workspace_empty_state` — рендер без панелей
- `test_workspace_with_panel` — рендер с одной панелью
- `test_workspace_add_remove_panel` — добавление и удаление панелей

---

## 5. Layer 3: Panel (panel/mod.rs)

### 5.1 Назначение

Единый трейт для всех панелей приложения. Аналог `DockItem::Panel` в gpui-component, но свой.

### 5.2 PanelView trait

```rust
/// Объектно-безопасный трейт для встраиваемых панелей.
pub trait PanelView: Send + Sync {
    /// Уникальный идентификатор панели (совпадает с EntityId).
    fn panel_id(&self, cx: &App) -> EntityId;

    /// Имя панели для сериализации (например, "FileTree", "Editor").
    fn panel_name(&self, cx: &App) -> &'static str;

    /// Имя на вкладке.
    fn tab_name(&self, cx: &App) -> Option<SharedString>;

    /// Иконка на вкладке (опционально).
    fn tab_icon(&self, cx: &App) -> Option<IconName>;

    /// Заголовок окна (для TitleBar area).
    fn title(&self, window: &mut Window, cx: &mut AppContext) -> impl IntoElement;

    /// Контент панели (главное).
    fn render(&self, window: &mut Window, cx: &mut AppContext) -> impl IntoElement;

    /// Можно ли закрыть панель.
    fn closable(&self, cx: &App) -> bool { true }

    /// Можно ли zoom (full-screen внутри Workspace).
    fn zoomable(&self, cx: &App) -> bool { false }

    /// Активна ли панель (получает фокус).
    fn set_active(&self, active: bool, cx: &mut AppContext) {}

    /// Вызывается при добавлении в панель (TabPanel).
    fn on_added(&self, tab_panel: EntityId, cx: &mut AppContext) {}

    /// Вызывается при удалении.
    fn on_removed(&self, cx: &mut AppContext) {}

    /// Сериализация состояния панели.
    fn dump(&self, cx: &App) -> Option<serde_json::Value> { None }
}
```

### 5.3 PanelRegistry

```rust
pub struct PanelRegistry {
    builders: HashMap<&'static str, Box<dyn Fn(...) -> Box<dyn PanelView>>>,
}
```

Глобальный реестр для десериализации: `register_panel(name, builder)` и `build_panel(name) -> Option<Box<dyn PanelView>>`.

### 5.4 Тесты

- `test_panel_view_trait` — создание мок-панели, проверка всех методов
- `test_panel_registry_register_and_build` — регистрация и создание
- `test_panel_registry_nonexistent` — запрос несуществующей панели

---

## 6. Layer 4: DockItem (dock/mod.rs)

### 6.1 Назначение

Рекурсивное дерево, описывающее раскладку внутри области (центральной).

### 6.2 DockItem enum

```rust
pub enum DockItem {
    /// Разделитель (split) с resize handle'ами.
    Split {
        axis: Axis,
        children: Vec<DockItem>,
        sizes: Vec<Option<Pixels>>,
        state: Entity<StackPanel>,
    },
    /// Вкладки.
    Tabs {
        children: Vec<Box<dyn PanelView>>,
        active: usize,
        state: Entity<TabPanel>,
    },
    /// Одиночная панель.
    Panel {
        view: Box<dyn PanelView>,
    },
    /// Плавающие окна.
    Tiles {
        children: Vec<TileItem>,
        state: Entity<TilesState>,
    },
}
```

### 6.3 Операции

```rust
impl DockItem {
    // Навигация
    fn find_panel(&self, id: EntityId) -> Option<&Box<dyn PanelView>>;
    fn find_panel_mut(&mut self, id: EntityId) -> Option<&mut Box<dyn PanelView>>;

    // Мутация
    fn add_panel(&mut self, panel: Box<dyn PanelView>, placement: DockPlacement, cx: &mut AppContext);
    fn remove_panel(&mut self, id: EntityId, cx: &mut AppContext) -> Option<Box<dyn PanelView>>;

    // Сериализация
    fn dump(&self, cx: &App) -> DockItemState;
    fn load(state: DockItemState, registry: &PanelRegistry, cx: &mut AppContext) -> Self;
}
```

### 6.4 DockPlacement

```rust
pub enum DockPlacement {
    /// Слева от центра (split вертикально).
    Left,
    /// Справа от центра (split вертикально).
    Right,
    /// Сверху от центра (split горизонтально).
    Top,
    /// Снизу от центра (split горизонтально).
    Bottom,
    /// В центр (как новая вкладка).
    Center,
}
```

### 6.5 Тесты

- `test_dock_item_panel_roundtrip` — создание Panel -> dump -> load -> panel_name совпадает
- `test_dock_item_tabs_dump_load` — Tabs с 2 panel -> dump -> load -> 2 panel
- `test_dock_item_split_dump_load` — Split с 2 children -> dump -> load
- `test_dock_item_add_panel` — добавление панели в разные места
- `test_dock_item_remove_panel` — удаление существующей и несуществующей панели
- `test_dock_item_find_panel` — поиск по id

---

## 7. Layer 5: DockArea (dock/area.rs)

### 7.1 Назначение

Владелец раскладки окна. Содержит центральную область + левый/правый/нижний доки.

### 7.2 API

```rust
pub struct DockArea {
    id: SharedString,
    center: DockItem,
    left_dock: Option<Entity<Dock>>,
    right_dock: Option<Entity<Dock>>,
    bottom_dock: Option<Entity<Dock>>,
    zoomed_panel: Option<EntityId>,
    locked: bool,
}

impl DockArea {
    pub fn new(id: impl Into<SharedString>, cx: &mut AppContext) -> Self;

    // Управление центральной областью
    pub fn set_center(&mut self, item: DockItem, cx: &mut AppContext);
    pub fn center(&self) -> &DockItem;

    // Управление боковыми доками
    pub fn set_left_dock(&mut self, panel: DockItem, size: Pixels, open: bool, cx: &mut AppContext);
    pub fn set_right_dock(&mut self, panel: DockItem, size: Pixels, open: bool, cx: &mut AppContext);
    pub fn set_bottom_dock(&mut self, panel: DockItem, size: Pixels, open: bool, cx: &mut AppContextctors);
    pub fn remove_left_dock(&mut self, cx: &mut AppContext);
    pub fn remove_right_dock(&mut self, cx: &mut AppContext);
    pub fn remove_bottom_dock(&mut self, cx: &mut AppContext);

    // Добавление панели в раскладку
    pub fn add_panel(&mut self, panel: Box<dyn PanelView>, placement: DockPlacement, cx: &mut AppContext);
    pub fn remove_panel(&mut self, id: EntityId, cx: &mut AppContext) -> bool;

    // Zoom
    pub fn zoom_in(&mut self, panel: EntityId, cx: &mut AppContext);
    pub fn zoom_out(&mut self, cx: &mut AppContext);

    // Lock
    pub fn set_locked(&mut self, locked: bool);
    pub fn is_locked(&self) -> bool;

    // Сериализация
    pub fn dump(&self, cx: &App) -> DockAreaState;
    pub fn load(&mut self, state: DockAreaState, cx: &mut AppContext);
}
```

### 7.3 Render

```rust
impl Render for DockArea {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Если zoomed — рендерим только zoomed панель
        if let Some(id) = self.zoomed_panel {
            return self.render_zoomed(id, window, cx);
        }

        div().flex().flex_row().size_full()
            .when_some(self.left_dock.as_ref(), |this, dock| {
                this.child(dock.clone())
            })
            .child(
                div().flex().flex_col().flex_1().min_w_0()
                    .child(self.center.render(window, cx))
                    .when_some(self.bottom_dock.as_ref(), |this, dock| {
                        this.child(dock.clone())
                    })
            )
            .when_some(self.right_dock.as_ref(), |this, dock| {
                this.child(dock.clone())
            })
    }
}
```

### 7.4 Важный нюанс: боковые доки vs DockPlacement

`DockArea` различает два типа "слева":

- **`left_dock`** — фиксированная боковая панель (как Project Panel в Zed). Управляется через `set_left_dock()` / `remove_left_dock()`. Это flex-элемент с фиксированной шириной.
- **`add_panel(panel, DockPlacement::Left)`** — добавляет панель в **центр**, создавая Split с центром. Это внутри центральной области.

Проще: `left_dock` = внешняя панель всего окна, `placement::Left` = split внутри центра.

При `add_panel(panel, Left)`, если left_dock открыт — панель добавляется в split внутри центральной области, не затрагивая левый док. Если нужно добавить в левый док — используется `set_left_dock()`.

### 7.5 Тесты

- `test_dock_area_empty` — DockArea без панелей
- `test_dock_area_with_center` — одна центральная панель
- `test_dock_area_with_left_dock` — + левый док
- `test_dock_area_add_panel_center` — добавление в центр как вкладку
- `test_dock_area_add_panel_left_split` — добавление слева (split)
- `test_dock_area_remove_panel` — удаление
- `test_dock_area_dump_load_empty` — dump/load пустого
- `test_dock_area_dump_load_full` — dump/load со всеми доками
- `test_dock_area_zoom_in_out` — zoom + unzoom

---

## 8. Layer 6: StackPanel (dock/stack.rs)

### 8.1 Назначение

Split-панель с resize handle'ами. Использует `ResizablePanelGroup` из gpui-component.

### 8.2 API

```rust
pub struct StackPanel {
    axis: Axis,
    panels: Vec<Box<dyn PanelView>>,
    resizable_state: Entity<ResizableState>,
    min_size: Pixels,
}

impl StackPanel {
    pub fn new(axis: Axis, cx: &mut AppContext) -> Self;
    pub fn add_panel(&mut self, panel: Box<dyn PanelView>, cx: &mut AppContext);
    pub fn insert_panel(&mut self, panel: Box<dyn PanelView>, index: usize, cx: &mut AppContext);
    pub fn remove_panel(&mut self, id: EntityId, cx: &mut AppContext) -> Option<Box<dyn PanelView>>;
    pub fn set_min_size(&mut self, size: Pixels);
}
```

### 8.3 Render

```rust
impl Render for StackPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        ResizablePanelGroup::new("sn-stack")
            .with_state(&self.resizable_state)
            .axis(self.axis)
            .children(self.panels.iter().map(|panel| {
                resizable_panel()
                    .min_size(self.min_size)
                    .child(panel.render(window, cx))
            }))
    }
}
```

### 8.4 Snapping

- Минимальный размер панели: `PANEL_MIN_SIZE = 100px` (константа)
- Resize handle: `on_drag` с проверкой минимальных размеров
- Алгоритм resize: при перетаскивании handle'а изменяет размер текущей панели, соседняя сжимается/расширяется пропорционально
- Если панель достигла min_size — handle перестаёт двигаться (snapping)

### 8.5 Тесты

- `test_stack_panel_empty` — пустой StackPanel не паникует
- `test_stack_panel_add_remove` — добавление/удаление панелей
- `test_stack_panel_remove_empty` — удаление последней панели удаляет StackPanel
- `test_stack_panel_min_size` — resize не меньше min_size
- `test_stack_panel_multiple_sizes` — корректное распределение размеров

---

## 9. Layer 7: TabPanel (dock/tab.rs)

### 9.1 Назначение

Вкладки с контентом. Самая сложная часть — drag-and-drop вкладок между панелями.

### 9.2 API

```rust
pub struct TabPanel {
    panels: Vec<Box<dyn PanelView>>,
    active_ix: usize,
    closable: bool,
    dock_area: WeakEntity<DockArea>,
    will_split_placement: Option<DockPlacement>,
}
```

### 9.3 Render

```rust
impl Render for TabPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let panels = &self.panels;
        let active = self.active_ix;

        div().flex().flex_col().size_full()
            .child(self.render_tab_bar(window, cx))
            .child(
                div().flex_1().min_h_0()
                    .child(panels[active].render(window, cx))
            )
    }
}
```

### 9.4 Drag-and-drop вкладок

**Инициирование драга:**
```
.on_drag(
    DragPayload { panel, tab_panel: self },
    |payload, offset, window, cx| {
        cx.new(|_| DragGhost::new(payload))
    }
)
```

**Drop цели (на Tab-элементе TabPanel):**
1. На существующий таб — вставить до/после него
2. На панель контента — split zone (слева/справа/сверху/снизу/центр)

**Split zone (из gpui-component TabPanel):**
- < 35% ширины → Left
- > 65% ширины → Right
- < 35% высоты → Top
- > 65% высоты → Bottom
- Иначе → Center (merge)

**Алгоритм дропа:**
1. Определить `will_split_placement`
2. Если split — создать новый StackPanel с нужной осью
3. Если center — вставить как новую вкладку
4. `detach_panel` из исходного TabPanel (если не тот же)
5. Обновить `cx.notify()`

### 9.5 DragGhost

```rust
pub struct DragGhost {
    payload: DragPayload,
}

impl Render for DragGhost {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px_2()
            .py_1()
            .bg(cx.theme().background)
            .opacity(0.8)
            .border_1()
            .border_color(cx.theme().border)
            .rounded_md()
            .child(self.payload.panel.tab_name(cx))
    }
}
```

### 9.6 Тесты

- `test_tab_panel_empty` — пустой 
- `test_tab_panel_add_panel` — добавление
- `test_tab_panel_switch_tab` — переключение
- `test_tab_panel_remove_panel` — удаление активной/неактивной
- `test_tab_panel_remove_last` — удаление последней удаляет TabPanel
- `test_tab_panel_drag_start` — создание DragPayload
- `test_tab_panel_split_zone` — проверка split zone logic
- `test_tab_panel_drop_center` — merge вкладок

---

## 10. Layer 8: Command (command/mod.rs)

### 10.1 Назначение

Единый центр обработки действий и клавиатурных сочетаний.

### 10.2 API

```rust
pub use gpui::Action;

/// Определение действия.
/// Использует gpui::actions! макрос.
public_actions!(WorkspaceActions, [
    OpenVault,
    CloseVault,
    ToggleProjectPanel,
    ToggleLowerPanel,
    TogglePreview,
    SaveFile,
    CloseTab,
    NextTab,
    PrevTab,
]);

/// Регистрация keybindings.
pub fn register_default_keybindings(cx: &mut AppContext) {
    cx.bind_keys([
        KeyBinding::new("cmd-o", OpenVault, None),
        KeyBinding::new("cmd-s", SaveFile, Some("Workspace")),
        KeyBinding::new("cmd-b", ToggleProjectPanel, Some("Workspace")),
        KeyBinding::new("cmd-j", ToggleLowerPanel, Some("Workspace")),
        KeyBinding::new("cmd-w", CloseTab, Some("Workspace")),
        KeyBinding::new("cmd-shift-]", NextTab, Some("Workspace")),
        KeyBinding::new("cmd-shift-[", PrevTab, Some("Workspace")),
    ]);
}

/// Обработчик действий для Workspace.
impl actions::Handler<ToggleProjectPanel> for Workspace {
    fn handle(&mut self, _: &ToggleProjectPanel, window: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.left_dock_open = !state.left_dock_open;
            cx.notify();
        });
    }
}
```

### 10.3 Тесты

- `test_command_binding` — keybinding вызывает action
- `test_command_handler` — handler меняет состояние
- `test_command_palette` — палитра показывает действия (заглушка)

---

## 11. Layer 9: Testing (testing.rs)

### 11.1 Назначение

Утилиты для тестирования sn-ui компонентов.

### 11.2 API

```rust
/// Тестовое окно с gpui test platform.
pub struct SnTestContext {
    pub app: TestAppContext,
    pub window: TestWindow,
}

impl SnTestContext {
    pub fn new() -> Self;
    
    /// Создать PanelView для тестов.
    pub fn create_mock_panel(name: &'static str) -> MockPanel;
}
```

### 11.3 MockPanel

```rust
pub struct MockPanel {
    id: EntityId,
    name: &'static str,
    content: SharedString,
}

impl PanelView for MockPanel { ... }
```

### 11.4 Тесты

- `test_context_creates_window` — создание контекста не паникует
- `test_mock_panel_roundtrip` — dump/load мок-панели

---

## 12. Layer 10: Integration

### 12.1 План перехода simpler-notes на sn-ui

1. Добавить `sn-ui = { path = "../sn-ui" }` в `Cargo.toml` simpler-notes-gui
2. Заменить `AppState` на `Workspace` + `PanelView` реализации для каждой панели
3. FileTree — обернуть в `PanelView`
4. Editor — обернуть в `PanelView` (с Source/Split/Preview как состояния)
5. Lower panel — обернуть в `PanelView` (Search/Timeline/Graph/Diagnostics)
6. TitleBar — переиспользовать из gpui-component (уже есть)
7. Keybindings — перевести на command-систему
8. Сохранение layout — `DockArea::dump()` в настройки

### 12.2 Критерии готовности

- Все существующие тесты проходят
- Визуал не изменился (те же панели, те же размеры)
- Можно перетащить вкладку из Editor в Lower Panel и обратно
- Layout сохраняется между запусками

---

## 13. Тестирование (общие принципы)

### 13.1 Покрытие

- Каждый публичный метод — минимум 2 теста (happy path + ошибка/граница)
- DnD — тесты на split zone logic, drag start, drop, отмену
- DockItem dump/load — roundtrip для всех вариантов
- PanelRegistry — регистрация + создание + несуществующая

### 13.2 Формат тестов

```rust
#[gpui::test]
fn test_dock_area_add_remove_panel(cx: &mut TestAppContext) {
    let mut area = DockArea::new("test", cx);
    let panel = SnTestContext::create_mock_panel("TestPanel");

    area.add_panel(panel.clone(), DockPlacement::Center, cx);
    assert!(area.center().find_panel(panel.panel_id(cx)).is_some());

    area.remove_panel(panel.panel_id(cx), cx);
    assert!(area.center().find_panel(panel.panel_id(cx)).is_none());
}
```

---

## 14. Зависимости

```toml
[package]
name = "sn-ui"
version = "0.1.0"
edition = "2021"

[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", package = "gpui" }
gpui-platform = { git = "https://github.com/zed-industries/zed", package = "gpui_platform" }
gpui-component = { git = "https://github.com/longbridge/gpui-component" }
gpui-component-assets = { git = "https://github.com/longbridge/gpui-component" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"

[dev-dependencies]
gpui = { git = "https://github.com/zed-industries/zed", package = "gpui", features = ["test-support"] }

[features]
default = []
linux   = ["gpui-platform/font-kit", "gpui-platform/wayland", "gpui-platform/x11"]
macos   = ["gpui-platform/font-kit"]
windows = []
```

---

## 15. Риски и компромиссы

| Риск | Решение |
|------|---------|
| Зависимость от gpui-platform может сломаться при обновлении | Пин версии в Cargo.lock; тесты CI |
| gpui-component Dock можно не успеть адаптировать | Начинаем с упрощённого DockItem, потом наращиваем |
| Drag-and-drop в табах сложен в тестировании | Выносим split zone logic в чистую функцию без gpui |
| Производительность рендера с большим числом панелей | Используем uniform_list для списков; defer для невидимых |
| Конфликт версий gpui (sn-ui и simpler-notes-core) | Все крейты workspace используют один gpui коммит |
