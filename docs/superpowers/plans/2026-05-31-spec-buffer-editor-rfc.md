# SPEC: Buffer вместо Document, Anchor убрать из core

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Привести SPEC в соответствие с решением: core не знает про Rope/Anchor/Bias — это детали GUI.

**Architecture:** `Document` → `Buffer` (простая структура с String), `Anchor`/`Bias`/`Span` удаляются из core API, gpui::Editor становится source-редактором в GUI.

**Tech Stack:** Rust, gpui, simpler-notes-core, simpler-notes-gui

---

### Task 1: Buffer в api/core.md

**Files:**
- Modify: `docs/SPEC/api/core.md`

Заменить `Document` на `Buffer` в секции `document`. Убрать всё про Anchor, Rope, AnchoredMetadata. Buffer — простая структура с String.

- [ ] **Step 1: Заменить секцию `document` на `buffer` и `editor`**

Старый текст (строки 38-63):
```
### `document`

```rust
pub struct Document {
    pub path: Option<PathBuf>,
    pub text: Rope,
    pub cached_metadata: Option<AnchoredMetadata>,
    pub saved_revision: u64,
    pub current_revision: u64,
}

impl Document {
    pub fn open(path: &Path) -> Result<Self>;
    pub fn snapshot(&self) -> RopeSlice;
    pub fn apply_edit(&mut self, start: usize, end: usize, text: &str) -> u64;
    pub fn parse_and_update(&mut self);
    pub fn save(&mut self) -> Result<()>;
    pub fn is_dirty(&self) -> bool;
    pub fn anchor_to_line_col(&self, anchor: &Anchor) -> (usize, usize);
}

pub struct AnchoredMetadata {
    pub links: Vec<AnchoredLink>,
    pub tags: Vec<AnchoredTag>,
    pub dates: Vec<AnchoredDate>,
    pub errors: Vec<AnchoredError>,
}
```
```

Новый текст:
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
```

- [ ] **Step 2: Убрать AnchoredMetadata из api/core.md**

Найти строки 57-63:
```rust
pub struct AnchoredMetadata {
    pub links: Vec<AnchoredLink>,
    pub tags: Vec<AnchoredTag>,
    pub dates: Vec<AnchoredDate>,
    pub errors: Vec<AnchoredError>,
}
```

Удалить целиком. Вместо этого в секции `parser` уже есть `ParseResult` с `ByteSpan` — его достаточно. Диагностики хранятся отдельно в `Diagnostics`.

Проверить, нет ли упоминаний `AnchoredMetadata` в других файлах — если есть, заменить на `ParseResult`.

- [ ] **Step 3: Вычистить Anchor из всех секций api/core.md**

Удалить секцию с Anchor/Bias/Span (сейчас её нет в api/core.md — ок). Убедиться что `Vault.open_document` и `Vault.save_document` переименованы в `open_buffer` / `save_buffer`.

Найти:
```rust
pub fn open_document(&self, path: &Path) -> Result<Document, String>;
pub fn save_document(&self, doc: &mut Document) -> Result<(), String>;
```

Заменить на:
```rust
pub fn open_buffer(&self, path: &Path) -> Result<Buffer, String>;
pub fn save_buffer(&self, buf: &mut Buffer) -> Result<(), String>;
```

- [ ] **Step 4: Убрать упоминание Rope из api/core.md**

Строки с `Rope`, `RopeSlice` убрать (их не должно остаться после замены Document на Buffer).

- [ ] **Step 5: Commit**

```bash
git add docs/SPEC/api/core.md
git commit -m "docs(spec): replace Document/Rope with Buffer in api/core.md"
```

---

### Task 2: note-model.md — убрать Anchor/Bias

**Files:**
- Modify: `docs/SPEC/features/note-model.md`

Anchor/Bias/Span убрать. NoteMetadata хранит только `ByteSpan` (byte offset, length).

- [ ] **Step 1: Заменить секцию Anchor на ByteSpan**

Старый текст (строки 13-49):
```
## Anchor — ссылка на позицию в тексте

```rust
/// Ссылка на позицию в Document.text, устойчивая к вставкам/удалениям.
/// При изменении текста до этой позиции offset автоматически корректируется.
pub struct Anchor {
    pub offset: usize,    // байт от начала документа
    pub bias: Bias,       // поведение при вставке прямо на этой позиции
}

pub enum Bias {
    Left,   // Anchor остаётся слева от нового текста
    Right,  // Anchor сдвигается за новый текст
}

pub struct Span {
    pub start: Anchor,
    pub end: Anchor,
}
```

### Как работает Bias

Если пользователь вставляет текст на offset, где находится Anchor:

- `Bias::Left` — Anchor остаётся на месте, новый текст вставляется после
- `Bias::Right` — Anchor сдвигается за новый текст

### Преобразование из байтового оффсета

Парсер возвращает `Span { offset: usize, length: usize }`. Document конвертирует его в Anchored Span:

```
start = Anchor { offset, bias: Bias::Left }
end = Anchor { offset + length, bias: Bias::Right }
```
```

Новый текст:
```
## ByteSpan — ссылка на позицию в тексте

Парсер возвращает байтовые оффсеты. Они стабильны до следующего редактирования файла. GUI использует `gpui::Editor` для управления позициями курсора — Anchor не нужен.

```rust
/// Байтовая позиция в тексте.
/// offset отсчитывается от начала файла.
pub struct ByteSpan {
    pub offset: usize,     // байт от начала файла
    pub length: usize,     // длина в байтах
}
```
```

- [ ] **Step 2: Заменить LinkRef/TagRef/DateRef — убрать Anchor, оставить ByteSpan**

Старый текст (строки 51-73):
```
## Сущности

```rust
/// [[вики-ссылка]]
pub struct LinkRef {
    pub file_name: String,  // имя заметки (без алиаса)
    pub label: String,      // отображаемый текст (с алиасом или то же имя)
    pub span: Span,         // позиция в Anchor
}

/// @тег
pub struct TagRef {
    pub name: String,       // название тега (без @)
    pub span: Span,         // позиция в Anchor
}

/// !дата
pub struct DateRef {
    pub date: NaiveDate,    // распарсенная дата
    pub raw: String,        // исходная строка "!21.07.2003" (с префиксом !)
    pub span: Span,         // позиция в Anchor
}
```
```

Новый текст:
```
## Сущности

Все сущности используют байтовые оффсеты (`ByteSpan`), а не Anchor.

```rust
/// [[вики-ссылка]]
pub struct LinkRef {
    pub file_name: String,
    pub label: String,
    pub span: ByteSpan,
}

/// @тег
pub struct TagRef {
    pub name: String,
    pub span: ByteSpan,
}

/// !дата
pub struct DateRef {
    pub date: NaiveDate,
    pub raw: String,
    pub span: ByteSpan,
}
```
```

- [ ] **Step 3: Убрать упоминание Document::parse_and_update()**

Старый текст (строка 102):
```
`Note` создаётся через `parser::parse_content(text)`. Полученные байтовые оффсеты затем конвертируются в Anchor через `Document::parse_and_update()`.
```

Новый текст:
```
`Note` создаётся через `parser::parse_content(text)`. Полученные байтовые оффсеты используются напрямую для индексации и diagnostics.
```

- [ ] **Step 4: Commit**

```bash
git add docs/SPEC/features/note-model.md
git commit -m "docs(spec): replace Anchor/Bias with ByteSpan in note-model.md"
```

---

### Task 3: vault.md — Document → Buffer

**Files:**
- Modify: `docs/SPEC/features/vault.md`

- [ ] **Step 1: Заменить open_document/save_document на open_buffer/save_buffer**

Найти все вхождения `open_document`, `save_document`, `Document` в сигнатурах и тексте. Заменить:

```rust
// Было:
pub fn open_document(&self, path: &Path) -> Result<Document, String>;
pub fn save_document(&self, doc: &mut Document) -> Result<(), String>;

// Стало:
pub fn open_buffer(&self, path: &Path) -> Result<Buffer, String>;
pub fn save_buffer(&self, buf: &mut Buffer) -> Result<(), String>;
```

- [ ] **Step 2: Обновить секцию жизненного цикла открытия файла**

Старый текст (строки 231-238):
```
## Жизненный цикл открытия файла

1. `open_document(path)` → `Document::open(full_path)` → читает диск, парсит, кеширует метаданные
2. GUI хранит `Document` в `OpenTab`, работает с ним через Rope
3. При редактировании: `doc.apply_edit()` → `current_revision++`, `autosave_timer` сбрасывается
4. При явном сохранении (Ctrl+S): `vault.save_document(doc)` → пишет диск + переиндексация
5. При автосохранении: то же самое, что и явное сохранение
6. Индекс на диске обновляется при каждом `save_document()`
```

Новый текст:
```
## Жизненный цикл открытия файла

1. `vault.open_buffer(path)` → `Buffer` (core, читает файл с диска)
2. GUI: `editor.set_text(buffer.text)`, сохраняет ссылку на `Arc<RwLock<Buffer>>`
3. При редактировании: `buffer.current_revision++` (через gpui::Editor)
4. При сохранении (Ctrl+S): `buffer.text = editor.text()`, `vault.save_buffer(&mut buffer)` → пишет диск + переиндексация
5. При автосохранении: то же, что и явное сохранение
6. Индекс на диске обновляется при каждом `save_buffer()`

Для MCP (headless):
1. `vault.open_buffer(path)` → `Buffer`
2. Агент читает `buffer.text` целиком
3. Агент пишет новый `buffer.text` целиком
4. `vault.save_buffer(&mut buffer)`
```

- [ ] **Step 3: Обновить секцию автосохранения**

Найти:
```
1. GUI проверяет `doc.should_autosave()` (прошло >= `autosave_interval_secs` с последнего edit)
2. Если пора — вызывает `vault.save_document(doc)`
3. `save_document()` пишет файл на диск, переиндексирует, сохраняет индекс
```

Заменить:
```
1. Таймер в GUI проверяет dirty-буферы каждые `autosave_interval_secs`
2. Если `buffer.is_dirty()`, вызывает `vault.save_buffer(&mut buffer)`
3. `save_buffer()` пишет файл на диск, переиндексирует, сохраняет индекс
```

- [ ] **Step 4: Обновить секцию сохранения документа**

Старый текст (строки 195-213):
```rust
impl Vault {
    pub fn save_document(&self, doc: &mut Document) -> Result<(), String> {
        let path = doc.path.clone()
            .ok_or_else(|| "Document has no path".to_string())?;
        doc.save()?;
        let content = doc.text.to_string();
        self.index.reindex_file(&path, &content);
        self.index.save(&self.path)?;
        Ok(())
    }
}
```

Новый текст:
```rust
impl Vault {
    pub fn save_buffer(&self, buf: &mut Buffer) -> Result<(), String> {
        let path = buf.path.clone()
            .ok_or_else(|| "Buffer has no path".to_string())?;
        buf.save()?;
        self.index.reindex_file(&path, &buf.text);
        self.index.save(&self.path)?;
        Ok(())
    }
}
```

- [ ] **Step 5: Commit**

```bash
git add docs/SPEC/features/vault.md
git commit -m "docs(spec): replace Document with Buffer in vault.md"
```

---

### Task 4: editor-preview.md — добавить gpui::Editor

**Files:**
- Modify: `docs/SPEC/features/editor-preview.md`

Добавить секцию про source-редактор. Сейчас документ только про preview. Надо описать, что source — это `gpui::Editor`.

- [ ] **Step 1: Добавить секцию Source Editor перед Preview**

После строки 6 (`---`) и заголовка, вставить новую секцию:

```
## Source Editor

Используется нативный `gpui::Editor` от Zed (из крейта `gpui`). Это полноценный редактор кода, адаптированный для Markdown.

### Возможности (из коробки gpui)

- Rope-буфер (эффективные вставки/удаления)
- Undo/redo (полный стек)
- Множественные курсоры
- IME-поддержка
- Скроллинг больших файлов
- Буфер обмена (копировать/вставить)

### Настройки для Markdown

- Soft wrap включён (в отличие от кодовых редакторов)
- Номера строк выключены (опционально, можно включить)
- Gutter минимальный

### Получение/запись текста

```rust
// При открытии:
let buffer = vault.open_buffer(path)?;
editor.set_text(buffer.text);

// При сохранении:
buffer.text = editor.text();
vault.save_buffer(&mut buffer)?;
```

### Подсветка

- Markdown-specific подсветка реализуется через `editor.set_highlighted_ranges()` с диапазонами из парсера
- `[[вики-ссылки]]`, `@теги`, даты `!DD.MM.YYYY` подсвечиваются цветом
- Для MVP: без подсветки (gpui::Editor в plain text mode)

### Автокомплит

`@`, `[[`, `!` перехватываются через `cx.on_keystroke()` на gpui::Editor. При вводе триггерного символа:

1. Определить контекст (перед курсором)
2. Вызвать `vault.autocomplete_*()` с префиксом
3. Показать `AutocompletePopup` (см. [autocomplete](./autocomplete.md))
4. При выборе — вставить полную конструкцию вместо триггерного текста

### Режимы переключения

| Текущий | Нажатие Source | Нажатие Preview |
|---------|----------------|-----------------|
| Source | — | Split |
| Split | Source | Preview |
| Preview | Source | Split |

### Не входит в MVP

- Встроенный Markdown-preview внутри редактора (только переключение режимов)
```

- [ ] **Step 2: Обновить секцию про получение метаданных**

Старый текст (строки 21-24):
```
1. Если `document.is_dirty()` — можно показать stale данные (Anchor всё ещё корректны)
2. Опционально: `document.parse_and_update()` для актуальных Anchor
3. `document.cached_metadata` содержит `AnchoredMetadata` со всеми ссылками, тегами, датами
```

Новый текст:
```
1. Метаданные хранятся в `Buffer` через `ParseResult` (полученные при последнем парсинге)
2. Если `buffer.is_dirty()` — метаданные могут быть stale. Для свежих данных: парсинг `buffer.text` заново
3. Парсинг не происходит при каждом нажатии клавиши — только при сохранении или переключении на Preview
```

- [ ] **Step 3: Commit**

```bash
git add docs/SPEC/features/editor-preview.md
git commit -m "docs(spec): add gpui::Editor source editor section to editor-preview.md"
```

---

### Task 5: workspace-layout.md — OpenTab с gpui::Editor

**Files:**
- Modify: `docs/SPEC/features/workspace-layout.md`

- [ ] **Step 1: Обновить OpenTab в AppState**

Старый текст (строки 88-92):
```rust
pub struct OpenTab {
    pub path: PathBuf,
    pub title: String,
    pub document: Arc<RwLock<Document>>,
}
```

Новый текст:
```rust
pub struct OpenTab {
    pub path: PathBuf,
    pub title: String,
    pub buffer: Arc<RwLock<Buffer>>,
    pub editor: gpui::View<gpui::Editor>,
}
```

- [ ] **Step 2: Обновить комментарии про document → buffer**

Старый текст (строки 114-117):
```
- `document.text` — текущее содержимое (Rope), редактируется в Source
- `document.cached_metadata` — кешированные метаданные для Preview
- `document.is_dirty()` — флаг несохранённых изменений
- `document.parse_and_update()` — вызывается при сохранении или при переключении на Preview
```

Удалить или заменить на:
```
- `buffer.text` — синхронизируется с gpui::Editor при открытии и сохранении
- `buffer.is_dirty()` — флаг несохранённых изменений
- Редактирование текста — через gpui::Editor напрямую
- Парсинг для Preview: `parse_content(editor.text())` на момент переключения
```

- [ ] **Step 3: Commit**

```bash
git add docs/SPEC/features/workspace-layout.md
git commit -m "docs(spec): update OpenTab to use Buffer and gpui::Editor"
```

---

### Task 6: architecture-review.md — обновить раздел про Document

**Files:**
- Modify: `docs/architecture-review.md`

- [ ] **Step 1: Обновить AppState секцию**

Старый текст (строки 208-224):
```rust
pub struct AppState {
    pub vault: Arc<RwLock<Vault>>,
    pub open_tabs: Vec<OpenTab>,
    pub active_tab: usize,
    pub editor_mode: EditorMode,
    pub sidebar_focus: SidebarFocus,
}

pub struct OpenTab {
    pub path: PathBuf,
    pub title: String,
    pub content_dirty: bool,
    pub editor_mode: EditorMode,
}
```

Новый текст:
```rust
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

- [ ] **Step 2: Убрать упоминание Document из раздела "Vault" в architecture-review.md**

Найти:
```
impl Vault {
    pub fn get_note(&self, path: &Path) -> Result<Note>;
```

Заменить на:
```
impl Vault {
    pub fn open_buffer(&self, path: &Path) -> Result<Buffer>;
```

- [ ] **Step 3: Commit**

```bash
git add docs/architecture-review.md
git commit -m "docs: update architecture-review with Buffer and gpui::Editor"
```

---

### Task 7: parser.md и autocomplete.md — мелкие правки

**Files:**
- Modify: `docs/SPEC/features/parser.md`
- Modify: `docs/SPEC/features/autocomplete.md`

- [ ] **Step 1: parser.md — убрать упоминание Document**

Старый текст (строка 96):
```
При сохранении файла (`Document::save`)
```

Заменить на:
```
При сохранении файла (`Buffer::save`)
```

- [ ] **Step 2: parser.md — убрать "Конвертацию в Anchor выполняет Document"**

Старый текст (строки 12-13):
```
Парсер **не знает** о файловой системе и Anchor. Он принимает только текст и возвращает байтовые оффсеты. Конвертацию в Anchor выполняет `Document::parse_and_update()`.
```

Новый текст:
```
Парсер **не знает** о файловой системе. Он принимает только текст и возвращает байтовые оффсеты.
```

- [ ] **Step 3: autocomplete.md — уточнить контекст SourceEditor**

Старый текст (строка 17):
```
SourceEditor,   // @, [[, !
```

Оставить как есть — это корректно. Только заменить `SourceEditor` на `gpui::Editor`:

```
gpui::Editor,   // @, [[, !
```

- [ ] **Step 4: Commit**

```bash
git add docs/SPEC/features/parser.md docs/SPEC/features/autocomplete.md
git commit -m "docs(spec): minor fixes for parser and autocomplete"
```

---

### Task 8: README.md — обновить навигацию

**Files:**
- Modify: `docs/SPEC/README.md`

- [ ] **Step 1: Проверить список feature-файлов**

Убедиться, что `editor-preview.md` упомянут (строка 28):
```
├── editor-preview.md      # Preview режим
```

Добавить примечание, что Source Editor описан в том же файле:
```
├── editor-preview.md      # Source (gpui::Editor) и Preview режимы
```

- [ ] **Step 2: Commit**

```bash
git add docs/SPEC/README.md
git commit -m "docs(spec): update README navigation"
```
