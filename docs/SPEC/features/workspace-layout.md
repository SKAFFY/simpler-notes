---
priority: P1
layer: gui
depends: [vault, document]
---

- [x]

# Workspace Layout

Главное окно GUI-приложения. Layout ориентирован на Zed: левая панель (Project Panel), центральная панель (Editor Panel) с табами, нижняя панель (Lower Panel) с переключаемыми вкладками.

## Структура окна

```
┌─────────────────────────────────────────────────────────┐
│  Title Bar: Simpler Notes — Файл Правка Вид Помощь      │
├──────────┬──────────────────────────────────────────────┤
│ Project  │  Editor Panel                                │
│ Panel    │  ┌─Tab Bar────────────────┐                  │
│ ──────── │  │ note1 │ note2 │ +    ✕ │                  │
│ Tree     │  ├────────────────────────┤                  │
│          │  │                        │                  │
│          │  │  Source  │  Preview    │                  │
│          │  │                        │                  │
│          │  │                        │                  │
├──────────┴──────────────────────────────────────────────┤
│  [Search]  [Timeline]  [Graph]  [Diagnostics]           │
└─────────────────────────────────────────────────────────┘
```

- **Title Bar** — строка заголовка с выпадающим меню (без нативной строки меню macOS)
- **Project Panel** — левая панель: файловое дерево (collapsible, `Cmd+B`)
- **Editor Panel** — правая панель: табы + контент
- **Lower Panel** — нижняя панель с табами: Search, Timeline, Graph, Diagnostics (collapsible, `Cmd+J`)

## Компоненты

| Компонент | Файл | Назначение |
|-----------|------|------------|
| `AppState` | `app_state.rs` | gpui Model: vault, open_tabs, editor_mode, panel visibility |
| `Workspace` | `workspace.rs` | Главный layout (3-panel) |
| `TitleBar` | `title_bar.rs` | Строка заголовка + выпадающее меню |
| `ProjectPanel` | `project_panel.rs` | Левая панель: файловое дерево |
| `FileTree` | `project_panel/file_tree.rs` | Файловое дерево с diagnostics |
| `EditorPanel` | `editor/editor_panel.rs` | Правая панель: TabBar + Content |
| `TabBar` | `editor/tab_bar.rs` | Вкладки открытых файлов |
| `SourceEditor` | `editor/source.rs` | Source редактирование |
| `PreviewRenderer` | `editor/preview.rs` | Preview |
| `AutocompletePopup` | `editor/autocomplete.rs` | Автокомплит @, [[, ! (см. [autocomplete](./autocomplete.md)) |
| `LowerPanel` | `lower_panel.rs` | Нижняя панель |
| `LowerTabBar` | `lower_panel/tab_bar.rs` | Табы: Search, Timeline, Graph, Diagnostics |
| `SearchBox` | `lower_panel/search.rs` | Поиск по query language |
| `QueryAutocomplete` | `lower_panel/query_autocomplete.rs` | Автокомплит для query language |
| `TimelineView` | `lower_panel/timeline.rs` | Таймлайн |
| `GraphView` | `lower_panel/graph.rs` | Граф связей |
| `DiagnosticsList` | `lower_panel/diagnostics.rs` | Список diagnostics по всем файлам |

## AppState

```rust
pub struct AppState {
    pub vault: Option<Arc<Vault>>,
    pub open_tabs: Vec<OpenTab>,
    pub active_tab: Option<usize>,
    pub editor_mode: EditorMode,     // Source | Split | Preview
    pub project_panel_visible: bool,
    pub project_panel_width: f32,
    pub search_query: String,
    pub lower_panel_visible: bool,
    pub lower_panel_active_tab: LowerPanelTab,
    pub lower_panel_height: f32,
}

pub enum EditorMode {
    Source,
    Split,
    Preview,
}

pub enum LowerPanelTab {
    Search,
    Timeline,
    Graph,
    Diagnostics,
}

pub struct OpenTab {
    pub path: PathBuf,
    pub title: String,
    pub buffer: Arc<RwLock<Buffer>>,
    pub editor: gpui::View<gpui::Editor>,
}
```

### Состояние автокомплита

Управляется внутри `SourceEditor`/`AutocompletePopup`, но может быть частью `EditorModel`:

```rust
pub enum AutocompleteState {
    Hidden,
    Tags { items: Vec<TagCompletion>, selected: usize },
    Links { items: Vec<String>, selected: usize },
    Dates { items: Vec<String>, selected: usize },
    QueryKeywords { items: Vec<(String, usize)>, selected: usize },
}

pub enum AutocompleteLocation {
    SourceEditor,
    QuerySearch,
}
```

- `buffer.text` — синхронизируется с gpui::Editor при открытии и сохранении
- `buffer.is_dirty()` — флаг несохранённых изменений
- Редактирование текста — через gpui::Editor напрямую
- Парсинг для Preview: `parse_content(editor.text())` на момент переключения

## TitleBar

Без нативной строки меню. Всё меню — выпадающее из TitleBar.

```rust
pub struct TitleBar {
    pub title: String,    // "Simpler Notes" или имя папки vault
    pub menu_open: bool,
}
```

| Меню | Пункты |
|------|--------|
| Файл | Open Vault, Close Vault, Exit |
| Правка | _(заглушка)_ |
| Вид | Toggle Project Panel (`Cmd+B`), Toggle Lower Panel (`Cmd+J`) |
| Помощь | About |

При нажатии на пункт меню — меню закрывается и выполняется действие.

## Project Panel

Левая панель. Collapsible по `Cmd+B`. Ширина изменяется drag (дефолт 250px).

- `FileTree` — все .md файлы в vault
- Иконка `FileWarning` для файлов с diagnostics (см. [file-tree](./file-tree.md))
- При первом открытии vault — все файлы загружены, фокус на первом файле в дереве

## Editor Panel

Правая панель. Содержит `TabBar` (вкладки) и контент (Source / Split / Preview).

### TabBar

```rust
pub struct TabBar {
    pub tabs: Vec<OpenTab>,
    pub active_idx: usize,
}
```

- Каждый таб: название файла (без `.md`), иконка close (✕)
- Активный таб — выделен
- Клик по табу → переключение
- Клик по ✕ → закрыть (если dirty → подтверждение)
- Клик по `+` → Quick Open (после MVP) или Open Vault если vault не открыт
- Если нет открытых вкладок — показывать пустой экран (как при первом запуске)
- Drag to reorder — после MVP

### Контент

- Source: `SourceEditor` — редактирование текста с подсветкой
- Split: Source слева, Preview справа (50/50)
- Preview: `PreviewRenderer` — рендер Markdown

## Lower Panel

Нижняя панель. Collapsible по `Cmd+J` или клик по активному табу. Высота изменяется drag (дефолт 200px).

### LowerTabBar

Четыре таба: **Search**, **Timeline**, **Graph**, **Diagnostics**:

```rust
pub struct LowerTabBar {
    pub active_tab: LowerPanelTab,
}
```

- Клик по неактивному табу → переключение контента
- Клик по активному табу → collapse если панель открыта, uncollapse если закрыта
- Активный таб выделен

### Search

Поле ввода для query language запросов. Автокомплит ключевых слов, тегов и дат (см. [lower-panel-search](./lower-panel-search.md)).

- `Cmd+F` → открыть Search таб (если Lower Panel скрыта — показать)
- `query_language` всегда — Files mode упразднён (его заменяет FileTree в Project Panel)

### DiagnosticsList

```rust
/// Показывает все diagnostics в vault.
fn render_diagnostics_list(vault: &Vault) -> Vec<DiagnosticRow> {
    vault.all_diagnostics().into_iter().map(|(path, diagnostics)| {
        // path:line — message
    }).collect()
}
```

- Строка: `path:line: message`
- Клик по строке → открыть файл в Editor, сфокусироваться на span ошибки
- Если diagnostics пуст — пустой список или сообщение «No issues found»

## Первый запуск

Если vault не открыт — показывается пустой экран в Editor Panel:
- Текст: «Добро пожаловать! Откройте папку с заметками через Файл → Open Vault»
- Кнопка: [Open Vault]

## Размеры

- Окно по умолчанию: 1024x768
- Project Panel: 250px ширины (default), изменяется drag
- Lower Panel: 200px высоты (default), изменяется drag
- Split режим: 50/50
