# GUI Workspace Layout — Designer spec

> Дата: 2026-05-28
> Статус: черновик
> Ветка: feat/gui-app

## 1. Цель

Реализовать базовый workspace layout GUI-приложения Simpler Notes: боковая панель (sidebar) с файловым деревом и поиском, центральный редактор с переключением source/preview, меню-бар.

## 2. Зависимости

- `simpler-notes-core` — vault, note, parser
- `gpui` — фреймворк окна
- `gpui-component` — UI-компоненты (dock layout, кнопки, иконки, Markdown рендеринг)

## 3. Архитектура

```
┌─────────────────────────────────────────────────┐
│  Menu Bar: Файл | Правка | Вид | Помощь         │
├──────────┬──────────────────────────────────────┤
│ Sidebar  │  Editor                              │
│ ┌──────┐ │  ┌─────┬─────┐                       │
│ │Search │ │  │ Src │ Prev│  ← кликабельные      │
│ └──────┘ │  ├─────┴─────┤                       │
│ ┌──────┐ │  │           │                       │
│ │Tree   │ │  │ Content  │                       │
│ │files  │ │  │           │                       │
│ └──────┘ │  └───────────┘                       │
└──────────┴──────────────────────────────────────┘
```

### 3.1 Компоненты

| Компонент | Файл | Назначение |
|-----------|------|------------|
| `AppState` | `app_state.rs` | gpui Model: vault, open_tabs, editor_mode |
| `Workspace` | `workspace.rs` | Главный layout: sidebar + editor |
| `Menu` | `menu.rs` | Menu Bar |
| `Sidebar` | `sidebar/mod.rs` | Боковая панель |
| `SearchBox` | `sidebar/search_box.rs` | Поиск |
| `FileTree` | `sidebar/file_tree.rs` | Файловое дерево |
| `EditorModel` | `editor/mod.rs` | Управление режимами редактора |
| `SourceEditor` | `editor/source.rs` | Plain text редактирование |
| `PreviewRenderer` | `editor/preview.rs` | Markdown + кликабельные [[link]] |

### 3.2 AppState

```rust
struct AppState {
    vault: Option<Arc<Vault>>,
    vault_path: Option<PathBuf>,
    open_tabs: Vec<OpenTab>,
    active_tab: usize,
    editor_mode: EditorMode,    // Source | Split | Preview
    sidebar_visible: bool,
    sidebar_width: f32,
}

enum EditorMode {
    Source,
    Split,
    Preview,
}

struct OpenTab {
    path: PathBuf,
    title: String,
    content_dirty: bool,
}
```

### 3.3 Запуск

- При первом запуске (нет settings.json) — пустой экран с подсказкой «Откройте папку с заметками через Файл → Открыть Папку»
- Если vault_path сохранён — открываем vault и показываем sidebar с файловым деревом
- Vault открывается асинхронно (в фоновом потоке), UI показывает спиннер/состояние загрузки

### 3.4 Settings

Хранятся в `dirs::config_dir()/simpler-notes/settings.json`:
- `vault_path` — последний открытый путь
- `sidebar_visible` — видимость боковой панели
- `sidebar_width` — ширина боковой панели
- `window_bounds` — последняя позиция/размер окна

## 4. Sidebar

- Верхняя часть: поле поиска (TextInput). Фильтрация файлового дерева в реальном времени.
- Нижняя часть: файловое дерево (список из gpui-component или кастомный).

### 4.1 Sidebar Search

- При вводе текста фильтрует файловое дерево
- Поле пустое — показываем все файлы
- Поиск по имени файла (не по содержимому)

### 4.2 File Tree

- Показывает файлы .md в vault рекурсивно
- Директории раскрываются/схлопываются
- Клик по файлу → открыть в редакторе
- Двойной клик → не используется (как в Zed — одиночный открывает)

## 5. Editor

### 5.1 Переключение Source / Preview

Две кнопки в верхней части редактора:

| Состояние | Нажатие Preview | Нажатие Source |
|-----------|----------------|----------------|
| Source (по умолчанию) | → Split | — |
| Split | → Preview | → Source |
| Preview | → Split | → Source |

Split: источник слева, превью справа (50/50).

### 5.2 Source Editor

- Plain text editing через `gpui::EditorMultiline` или `gpui_component::TextArea`
- Пока без подсветки синтаксиса
- При изменении — маркировка dirty на вкладке

### 5.3 Preview

- Рендеринг содержимого через `gpui_component::markdown::Markdown`
- Проход по `[[Note Name]]`: заменяем на кликабельную кнопку/ссылку
  - Если заметка существует → открывает её
  - Если не существует → визуально отличить (серый текст)
- `#tag` и даты пока как обычный текст (будут подсвечены позже)

## 6. Menu Bar

Стандартное меню сверху:

- **Файл**: Открыть Папку, Открыть Файл, Выход
- **Правка**: заглушка
- **Вид**: Toggle Sidebar
- **Помощь**: заглушка

## 7. Навигация по [[link]]

- Парсер `simpler-notes-core::parse()` находит все `[[Note Name]]`
- Preview заменяет их на кликабельные spans
- Обработчик клика:
  1. Ищем файл с именем `Note Name.md` в vault
  2. Если найден → открываем как новый tab (или переключаемся на существующий)
  3. Если не найден → ничего не делаем (или показываем toast/статус)

## 8. Первый запуск

```
┌──────────────────────────────────┐
│  Menu Bar                        │
├──────────────────────────────────┤
│                                  │
│      Добро пожаловать!           │
│                                  │
│   Откройте папку с заметками     │
│   через Файл → Открыть Папку     │
│                                  │
│         [Open Vault]             │
│                                  │
└──────────────────────────────────┘
```

## 9. Testing

- Unit tests для AppState (логика смены режимов, открытие/закрытие вкладок)
- Unit tests для Preview: парсинг `[[link]]` → кликабельные элементы
- Интеграционные тесты: открытие vault, навигация по файлам
- Тестирование вручную на CachyOS Linux

## 10. Не входит в MVP

- Подсветка `#tag` и дат в source редакторе
- Автокомплит `[[` и `#`
- Timeline / Graph View
- Темы оформления
- Resize sidebar через drag (пока фиксированная или через gpui-component dock)
- Drag to reorder вкладок
