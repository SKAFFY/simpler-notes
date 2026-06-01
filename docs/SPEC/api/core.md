# simpler-notes-core API

Публичный API крейта `simpler-notes-core`. Описание модулей, структур и методов.

## Модули

### `note_model`

```rust
pub struct Anchor { pub offset: usize, pub bias: Bias }
pub enum Bias { Left, Right }
pub struct Span { pub start: Anchor, pub end: Anchor }

pub struct LinkRef { pub file_name: String, pub label: String, pub span: Span }
pub struct TagRef { pub name: String, pub span: Span }
pub struct DateRef { pub date: NaiveDate, pub raw: String, pub span: Span }

pub struct Note { pub path: PathBuf, pub metadata: NoteMetadata }
pub struct NoteMetadata { pub links: Vec<LinkRef>, pub tags: Vec<TagRef>, pub dates: Vec<DateRef> }
```

### `parser`

```rust
pub fn parse_content(text: &str) -> ParseResult;

pub struct ParseResult { pub links: Vec<LinkSpan>, pub tags: Vec<TagSpan>, pub dates: Vec<DateSpan>, pub errors: Vec<ParseError> }
pub struct LinkSpan { pub file_name: String, pub label: String, pub span: ByteSpan }
pub struct TagSpan { pub name: String, pub span: ByteSpan }
pub struct DateSpan { pub date: NaiveDate, pub raw: String, pub span: ByteSpan }
pub struct ByteSpan { pub offset: usize, pub length: usize }
pub struct ParseError { pub span: ByteSpan, pub message: String, pub kind: ParseErrorKind }
pub enum ParseErrorKind { InvalidDate, EmptyLink }
```

### `buffer`

Простая модель файла в памяти. Не зависит от GUI. Используется как MCP (headless), так и GUI.

```rust
pub struct Buffer {
    pub path: Option<PathBuf>,
    pub text: String,
    pub saved_revision: u64,
    pub current_revision: u64,
}

impl Buffer {
    pub fn open(path: &Path) -> Result<Self>;
    pub fn save(&mut self) -> Result<()>;
    pub fn is_dirty(&self) -> bool;
}
```

| Метод | Описание |
|-------|----------|
| `open(path)` | Читает файл с диска в `text`, выставляет `saved_revision = 0, current_revision = 0` |
| `save()` | Пишет `text` на диск, `saved_revision = current_revision` |
| `is_dirty()` | `current_revision != saved_revision` |

### `editor` (GUI-only)

В GUI-крейте Buffer оборачивается в `gpui::Editor`:

```rust
// simpler-notes-gui, не в core
pub struct OpenTab {
    pub path: PathBuf,
    pub title: String,
    pub editor: gpui::View<gpui::Editor>,
    pub buffer: Arc<RwLock<Buffer>>,
}
```

Поток данных:
1. `vault.open_buffer(path)` → `Buffer` (core)
2. `editor.set_text(buffer.text)` (GUI)
3. Пользователь редактирует через gpui::Editor (GUI)
4. При сохранении: `buffer.text = editor.text()`, `vault.save_buffer(buffer)` (core)

gpui::Editor сам управляет Anchor, undo/redo, selection. Core ничего не знает об этом.

### `diagnostics`

```rust
pub struct Diagnostics { /* DashMap<PathBuf, Vec<Diagnostic>> */ }
pub struct Diagnostic { pub span: ByteSpan, pub message: String, pub severity: Severity }
pub enum Severity { Warning }

impl Diagnostics {
    pub fn new() -> Self;
    pub fn check_file(&self, path: &Path, content: &str, vault_path: &Path, filename_index: &HashMap<String, Vec<PathBuf>>);
    pub fn get(&self, path: &Path) -> Vec<Diagnostic>;
    pub fn all(&self) -> Vec<(PathBuf, Vec<Diagnostic>)>;
    pub fn remove(&self, path: &Path);
    pub fn clear(&self);
}
```

### `index`

```rust
pub struct TagIndex { /* DashMap<String, Vec<TagEntry>> */ }
pub struct TagEntry { pub path: PathBuf, pub spans: Vec<ByteSpan> }
pub struct TagCompletion { pub name: String, pub count: usize }

pub struct DateIndex { /* DashMap<NaiveDate, Vec<DateEntry>> */ }
pub struct DateEntry { pub path: PathBuf, pub spans: Vec<ByteSpan> }

pub struct ConcurrentIndex { /* tags, dates, links, diagnostics, file_states */ }
```

Методы `TagIndex`:
- `new()`, `add(path, tag, span)`, `remove(path, tag)`, `get(tag)`, `all_tags()`, `autocomplete(prefix)`, `clear()`

Методы `DateIndex`:
- `new()`, `add(path, date, span)`, `remove(path, date)`, `get(date)`, `get_range(from, to)`, `all_dates()`, `clear()`

Методы `LinkIndex`:
- `new()`, `add(source, entry)`, `remove_file(path)`, `backlinks(target)`, `outgoing(source)`, `clear()`

```rust
pub struct LinkEntry {
    pub source: PathBuf,     // откуда ссылка
    pub target: PathBuf,     // куда ссылка (file_stem, без .md, без пути)
    pub label: String,       // отображаемый текст
    pub span: ByteSpan,
}
```

Методы `ConcurrentIndex`:
- `new()`, `reindex_file(path, content, vault_path, filename_index)`, `save(path)`, `load(path)`, `clear()`

```rust
pub struct FileIndexState {
    pub tags: Vec<String>,
    pub dates: Vec<NaiveDate>,
}

### `search`

```rust
pub enum Query { TagsContain(String), DateBefore(NaiveDate), DateAfter(NaiveDate), DateBetween(NaiveDate, NaiveDate), Text(String), And(Box<Query>, Box<Query>), Or(Box<Query>, Box<Query>) }
pub struct SearchResult { pub path: PathBuf, pub title: String }
pub fn parse_query(input: &str) -> Result<Query, String>;
pub fn execute_search(index: &ConcurrentIndex, query: &Query, vault_path: &Path) -> Result<Vec<SearchResult>, String>;
```

### `vault`

```rust
pub struct Vault { pub path: PathBuf, pub index: Arc<ConcurrentIndex> }
pub struct IndexReport { pub total_notes, pub total_tags, pub total_dates }

impl Vault {
    pub fn open(path: &Path) -> Result<Self, String>;
    pub fn list_md_files(&self) -> Vec<PathBuf>;
    pub fn open_buffer(&self, path: &Path) -> Result<Buffer, String>;
    pub fn save_buffer(&self, buf: &mut Buffer) -> Result<(), String>;
    pub fn search(&self, query: &str) -> Result<Vec<SearchResult>, String>;
    pub fn read_note(&self, path: &Path) -> Result<String, String>;
    pub fn write_note(&self, path: &Path, content: &str) -> Result<(), String>;
    // Reserved for future UI (список всех тегов)
    pub fn get_all_tags(&self) -> Vec<String>;
    pub fn get_dates_in_range(&self, from: NaiveDate, to: NaiveDate) -> Vec<(NaiveDate, Vec<DateEntry>)>;
    pub fn validate_indexes(&self) -> IndexReport;

    // Автокомплит
    pub fn autocomplete_tags(&self, prefix: &str) -> Vec<TagCompletion>;
    pub fn fuzzy_search_tags(&self, query: &str) -> Vec<TagCompletion>;
    pub fn autocomplete_links(&self, prefix: &str) -> Vec<String>;
    pub fn autocomplete_dates(&self, prefix: &str) -> Vec<String>;

    // Ссылки
    pub fn get_backlinks(&self, target: &Path) -> Vec<LinkEntry>;
    pub fn get_outgoing_links(&self, source: &Path) -> Vec<LinkEntry>;
    pub fn resolve_link(&self, target: &str) -> Result<PathBuf, String>;

    // Управление индексом
    pub fn reindex_all(&self) -> Result<IndexReport, String>;

    // Diagnostics
    pub fn get_diagnostics(&self, path: &Path) -> Vec<Diagnostic>;
    pub fn all_diagnostics(&self) -> Vec<(PathBuf, Vec<Diagnostic>)>;
}
```

### `watcher`

```rust
pub enum FileEvent { Created(String), Modified(String), Deleted(String) }
pub struct FileWatcher { pub receiver: mpsc::Receiver<FileEvent>, /* handle */ }
impl FileWatcher { pub fn start(path: &Path) -> Result<Self, String>; }
```

### `git` (feature gate)

```rust
pub struct GitBackend { /* repo_path, timer_handle */ }
impl GitBackend {
    pub fn open(path: &Path, auto_commit: bool, interval_minutes: u64) -> Result<Self, String>;
    pub fn close(&self);
    pub fn commit(&self, message: &str) -> Result<(), String>;
    pub fn push(&self) -> Result<(), String>;
    pub fn pull(&self) -> Result<(), String>;
    pub fn status(&self) -> Result<String, String>;
    pub fn unpushed_count(&self) -> Result<usize, String>;
}
```
