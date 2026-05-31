# Архитектурный ревью: Simpler Notes

> Дата: 2026-05-27
> Язык: Rust
> UI: gpui (github.com/zed-industries/zed/tree/main/crates/gpui)
> UI-компоненты: github.com/longbridge/gpui-component

---

## Принятые решения (из обсуждения)

| Решение | Значение |
|---------|----------|
| Язык | Rust |
| UI фреймворк | gpui напрямую + gpui-component |
| WebView | Не используем |
| Фронтенд/бэкенд разделение | core-крейт, два отдельных бинарника |
| Формат бинарников | `simpler-notes-gui` (gpui app), `simpler-notes-mcp` (headless) |
| Хранение файлов | PlainText, файловая структура 1:1 с файловым менеджером |
| Git синхронизация | Гибрид: авто-коммиты, ручной push/pull. Репозиторий = директория заметок |
| `git init` | Ответственность пользователя |
| Формат тегов | Inline: `#tag` в тексте |
| Формат дат | DD.MM.YYYY (пока один формат) |
| Wiki-ссылки | `[[Note Name]]` как в Obsidian |
| MindMap | Граф связей (узлы=заметки, рёбра=ссылки) |
| Индекс проекта | Персистентный, хранится в `.index/` в корне проекта заметок |
| Редактор | 3 режима: source, preview, WYSIWYG (WYSIWYG в roadmap после MVP) |
| MCP протокол | JSON-RPC 2.0 через stdio (стандартный MCP transport) |

---

## 1. Структура проекта

```
simpler-notes/
├── Cargo.toml                       # workspace
├── crates/
│   ├── simpler-notes-core/          # Бизнес-логика: парсинг, индексы, поиск, git
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── vault.rs             # Vault — точка входа: открыть/создать хранилище
│   │       ├── note.rs              # Note — модель заметки: parsed content, метаданные
│   │       ├── parser.rs            # Парсинг MD: [[ссылки]], #теги, DD.MM.YYYY
│   │       ├── index/
│   │       │   ├── mod.rs           # IndexManager: build, load, save, search
│   │       │   ├── tag_index.rs     # Индекс по тегам
│   │       │   └── date_index.rs    # Индекс по датам
│   │       ├── search.rs            # Query Language: парсер AND/OR условий
│   │       ├── git.rs               # Git: auto-commit, push, pull (feature gate)
│   │       └── watcher.rs           # File system watcher (notify)
│   ├── simpler-notes-gui/           # Десктоп приложение на gpui
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs              # Точка входа: gpui App, init окна
│   │       ├── app_state.rs         # AppState: Vault handle, текущий note, выделение
│   │       ├── workspace.rs         # Workspace — главное окно
│   │       ├── sidebar/
│   │       │   ├── mod.rs           # Боковая панель
│   │       │   └── file_tree.rs     # Файловое дерево
│   │       ├── editor/
│   │       │   ├── mod.rs           # EditorModel: переключение режимов
│   │       │   ├── source.rs        # Plain text editing
│   │       │   └── preview.rs       # Rendered preview
│   │       ├── views/
│   │       │   ├── timeline.rs      # Таймлайн с датами
│   │       │   └── graph.rs         # MindMap / граф связей
│   │       └── settings.rs          # Настройки
│   └── simpler-notes-mcp/           # MCP сервер (headless)
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs              # tokio runtime, stdio JSON-RPC
│           └── handlers.rs          # Маппинг MCP-инструментов на core API
├── assets/                          # Иконки, шрифты
└── README.md
```

---

## 2. Core-крейт (simpler-notes-core)

### 2.1 Нефункциональные требования к производительности

Vault спроектирован для многопоточного доступа:
- Индекс на `DashMap` (шардированный, lock-free чтение)
- Watcher в отдельном потоке, коммуникация через канал
- Поиск и парсинг параллелятся через `rayon` или tokio tasks
- GUI поток читает индекс одновременно с записью от watcher

### 2.2 Vault

Центральный объект — `Arc<Vault>`.

```rust
pub struct Vault {
    path: PathBuf,
    index: Arc<ConcurrentIndex>,
    parser: Parser,                     // Stateless, можно параллелить
    git: Option<GitBackend>,
    watcher_handle: JoinHandle<()>,     // Фоновый поток
    change_tx: Sender<FileEvent>,       // Канал: watcher -> index updater
}

struct ConcurrentIndex {
    tags: Arc<DashMap<String, Vec<PathBuf>>>,
    dates: Arc<DashMap<NaiveDate, Vec<PathBuf>>>,
    fulltext: Arc<InvertedIndex>,
    file_states: Arc<DashMap<PathBuf, FileState>>,   // хэш/модификация файла
}

impl Vault {
    pub fn open(path: &Path) -> Result<Self>;
    pub fn search(&self, query: &str) -> Result<Vec<SearchResult>>;
    pub fn open_buffer(&self, path: &Path) -> Result<Buffer>;
    pub fn write_note(&self, path: &Path, content: &str) -> Result<()>;
    pub fn get_all_tags(&self) -> Vec<&str>;
    pub fn get_all_dates(&self) -> Vec<DateEntry>;
    pub fn validate_indexes(&self) -> Result<IndexReport>;
}
```

### 2.2 Парсер

Извлекает из Markdown-файла три сущности:

- **Ссылки** `[[Note Name]]` — в том числе с alias `[[Note|Display]]`
- **Теги** `#tag` — слово после `#`, может содержать буквы и цифры
- **Даты** `DD.MM.YYYY` — только этот формат, регистронезависимо

Парсер streaming-based: читает строку, ищет паттерны, возвращает структуру `ParsedNote`.

### 2.3 Индекс

Персистентный индекс в `.index/`:
- `index/tags.bin` — маппинг тег → список файлов
- `index/dates.bin` — маппинг дата → список файлов
- `index/fulltext.bin` — инвертированный индекс для полнотекстового поиска
- `index/metadata.json` — версия формата, время последней перестройки

При старте: если `.index` существует и актуален — загружаем, иначе перестраиваем.
Watcher отслеживает изменения файлов и инкрементально обновляет индекс.

### 2.4 Query Language

Простой парсер для строки поиска:

```
tags contain "inProgress" and date before 01.01.2025
tags contain "project"
date after 01.01.2024 and date before 01.06.2024
```

Грамматика:
- `and` / `or` — логические операторы
- `tags contain "value"` — фильтр по тегам
- `date before DD.MM.YYYY`, `date after DD.MM.YYYY` — фильтр по датам
- Голый текст — полнотекстовый поиск

### 2.5 Git

Feature gate: `features = ["git"]`.

- `auto_commit(message)` — git add -A && git commit
- `push()` — git push
- `pull()` — git pull
- `status()` — проверка состояния (чисто/грязно)
- `is_repo(path)` — проверка, есть ли .git

Auto-commit: при изменении файла запускается debounce таймер (30s без изменений → commit).

### 2.6 Watcher

Использует крейт `notify` (или `inotify`/`kqueue` напрямую):
- Подписывается на события: create, modify, delete файлов в vault
- Debounce (300ms) для группировки изменений
- Триггер: перестроить индекс + авто-коммит (если git включён)
- Игнорирует `.git/` и `.index/`

---

## 3. GUI-приложение (simpler-notes-gui)

### 3.1 Архитектура

```
┌─────────────────────────────────────────────┐
│  Workspace                                  │
│  ┌──────────┬──────────────────────────────┐│
│  │ Sidebar   │  Editor                     ││
│  │ ┌──────┐  │  ┌────────────────────────┐ ││
│  │ │Search│  │  │ ┌──────────────────────┐│ ││
│  │ └──────┘  │  │ │ Tab: Note.md  ✕     ││ ││
│  │ ┌──────┐  │  │ └──────────────────────┘│ ││
│  │ │Tree  │  │  │ ┌──────────────────────┐│ ││
│  │ │views │  │  │ │ Source/Preview toggle││ ││
│  │ │      │  │  │ │ Content area         ││ ││
│  │ └──────┘  │  │ │                      ││ ││
│  └──────────┴──┘  └────────────────────────┘ ││
│                                               ││
│  ┌──────────────────────────────────────────┐ ││
│  │ Timeline / Graph View (toggle bottom)    │ ││
│  └──────────────────────────────────────────┘ ││
└─────────────────────────────────────────────┘
```

### 3.2 AppState (gpui Model)

```rust
#[derive(Model)]
pub struct AppState {
    pub vault: Arc<RwLock<Vault>>,
    pub open_tabs: Vec<OpenTab>,
    pub active_tab: usize,
    pub editor_mode: EditorMode,  // Source | Split | Preview
    pub sidebar_focus: SidebarFocus, // Tree | Search
    pub lower_panel_visible: bool,
    pub lower_panel_active_tab: LowerPanelTab,
}

pub struct OpenTab {
    pub path: PathBuf,
    pub title: String,
    pub buffer: Arc<RwLock<Buffer>>,
    pub editor: gpui::View<gpui::Editor>,
    pub editor_mode: EditorMode,
}
```

### 3.3 Окна и вьюхи

- **Workspace** — главное окно, делит экран на sidebar + editor + нижняя панель
- **Sidebar:**
  - Поиск (сверху, cmd+F фокус)
  - Файловое дерево (ниже, как в VS Code)
  - Фильтрация по тегам (как clickable pills под поиском)
- **Editor:**
  - Вкладки (tabs) для открытых файлов — как в редакторах
    - Двойной клик по файлу в дереве → открыть в новой вкладке
    - Если файл уже открыт → переключиться на существующую
    - Close button ✕ (Cmd+W закрыть активную)
    - Точка/индикатор на вкладке при `content_dirty: true`
    - Drag to reorder (опционально, после MVP)
  - Source mode: редактирование как plain text (gpui EditorMultiline или своё)
  - Preview mode: парсим MD → рендерим в gpui элементы (кликабельные ссылки, подсвеченные теги и даты)
  - `[[` триггерит completion popup с именами заметок
  - `#` триггерит completion popup с существующими тегами
- **Нижняя панель:**
  - Timeline — точки на временной шкале, клик → открыть заметку
  - Graph View — граф связей с force-directed layout (переключение между Timeline и Graph)

### 3.4 Режимы редактора (MVP)

1. **Source** — редактирование plain text (gpui `Editor` с подсветкой синтаксиса или своей минимальной реализацией)
2. **Preview** — read-only rendered view

WYSIWYG — отдельная задача после MVP.

### 3.5 Настройки

```rust
pub struct Settings {
    pub vault_path: Option<PathBuf>,     // Последний открытый vault
    pub recent_vaults: Vec<PathBuf>,     // Список недавних
    pub theme: Theme,                    // Темная/светлая тема
    pub date_format: String,             // Будущее: отображение дат
    pub git_auto_commit: bool,
    pub git_auto_commit_interval: u64,   // Секунд
}
```

Хранятся в стандартной директории конфигурации ОС (dirs::config_dir()).

---

## 4. MCP сервер (simpler-notes-mcp)

### 4.1 Протокол

Стандартный MCP (Model Context Protocol) от Anthropic:

- **Transport:** stdio (stdin/stdout)
- **Формат:** JSON-RPC 2.0
- **Жизненный цикл:**
  1. Initialize — агент шлёт `initialize`, сервер отвечает capabilities
  2. `tools/list` — сервер возвращает список доступных инструментов
  3. `tools/call` — агент вызывает инструмент с аргументами
  4. Штатное завершение — `notifications/initialized`, затем закрытие stdin

### 4.2 Запуск

```bash
simpler-notes-mcp --vault /path/to/notes
```

Аргументы:
- `--vault <path>` — путь к хранилищу заметок (обязательный)
- `--git` — включить git-функции (опционально)

### 4.3 Инструменты

| Метод | Аргументы | Описание |
|-------|-----------|----------|
| `search_notes` | `query: string` | Поиск по query language |
| `read_note` | `path: string` | Содержимое файла |
| `write_note` | `path: string, content: string` | Создание/обновление заметки |
| `list_notes` | `path?: string` | Дерево файлов |
| `get_tags` | — | Все теги с количеством заметок |
| `get_dates` | — | Все даты |
| `git_push` | — | Push в remote |
| `git_pull` | — | Pull из remote |
| `validate_indexes` | — | Проверка целостности индексов и заметок |

---

## 5. Порядок реализации (MVP)

### Фаза 1: Core (без GUI)

1. **Cargo workspace** + структура крейтов
2. **Parser**: извлечение [[ссылок]], #тегов, дат из MD
3. **Note model**: структура данных для заметки
4. **Index**: in-memory + persist в `.index/`
5. **Search**: query language парсер
6. **Vault**: open, read, write, search
7. **Watcher**: notify для отслеживания изменений
8. **Git**: auto-commit

### Фаза 2: MCP сервер

1. JSON-RPC поверх stdio
2. MCP lifecycle (initialize, tools/list, tools/call)
3. Все инструменты из таблицы выше

### Фаза 3: GUI (MVP)

1. Окно gpui, Workspace layout
2. File Tree в сайдбаре
3. Source редактор
4. Preview редактор
5. Поиск в сайдбаре
6. Timeline
7. Graph View (минимальный — force-directed)

### Фаза 4: Полировка

1. Настройки (тема, vault path)
2. Quick Open (Cmd+P)
3. Completion popups ([[ и #)
4. Навигация по ссылкам
5. WYSIWYG (опционально)

---

## 6. Открытые вопросы (future)

- Поддержка вложений (картинки, схемы) рядом с .md файлом
- Self-hosted sync server (альтернатива git)
- Расширяемый query language
- Кастомные .vault с базой знаний
- Динамическое сужение mindMap при поиске
