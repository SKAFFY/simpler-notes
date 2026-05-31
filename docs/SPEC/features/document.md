---
priority: P0
layer: core
depends: [note-model, parser]
---

- [x]

# Document

Представление открытого файла в памяти. Хранит текст в виде Rope для эффективных вставок/удалений и кеширует распарсенные метаданные в Anchor-координатах.

## Структура

```rust
use ropey::Rope;
use std::path::PathBuf;
use std::time::Instant;

pub struct Document {
    pub path: Option<PathBuf>,
    pub text: Rope,
    pub cached_metadata: Option<AnchoredMetadata>,
    pub saved_revision: u64,
    pub current_revision: u64,
    pub autosave_timer: Option<Instant>,
    pub autosave_interval_secs: u64,
}
```

- **path** — путь к файлу (None если новый несохранённый документ)
- **text** — содержимое файла в Rope (сбалансированное дерево строк)
- **cached_metadata** — кешированные метаданные (ссылки, теги, даты) в Anchor-координатах
- **saved_revision** — версия текста на момент последнего сохранения
- **current_revision** — текущая версия (инкрементируется при каждом edit)
- **autosave_timer** — таймер последнего edit (для автосохранения)
- **autosave_interval_secs** — задержка автосохранения (из конфига, по умолчанию 30)

## API

```rust
impl Document {
    /// Открыть файл с диска, прочитать в Rope.
    pub fn open(path: &Path) -> Result<Self>;

    /// Получить снимок текста для потокобезопасного чтения.
    pub fn snapshot(&self) -> ropey::RopeSlice;

    /// Применить изменение: удалить [start..end], вставить text.
    /// Возвращает новую ревизию.
    /// Сбрасывает autosave_timer на текущее время.
    pub fn apply_edit(&mut self, start: usize, end: usize, text: &str) -> u64;

    /// Распарсить содержимое через parser::parse_content и кешировать.
    /// Конвертирует байтовые оффсеты парсера в Anchor.
    pub fn parse_and_update(&mut self);

    /// Сохранить на диск (если path есть) и обновить saved_revision.
    /// Сбросить autosave_timer.
    pub fn save(&mut self) -> Result<()>;

    /// Флаг: есть ли несохранённые изменения.
    pub fn is_dirty(&self) -> bool {
        self.current_revision != self.saved_revision
    }

    /// Конвертировать Anchor в (line, column) для GUI.
    pub fn anchor_to_line_col(&self, anchor: &Anchor) -> (usize, usize);

    /// Проверить, пора ли автосохранять (с момента последнего edit прошло >= interval).
    pub fn should_autosave(&self) -> bool;
}
```

## Жизненный цикл

### Открытие файла
1. `Document::open(path)` — читает файл с диска в Rope
2. `saved_revision = 0`, `current_revision = 0`
3. `parse_and_update()` — парсит контент, конвертирует в Anchor (без валидации ссылок — её делает Vault)

### Редактирование
1. `apply_edit(start, end, text)` — изменяет Rope
2. `current_revision++`
3. Метаданные **не** обновляются автоматически (сохраняют старые Anchor, которые корректно "едут" вместе с текстом)

### Сохранение
1. `save()` — записывает `text` на диск
2. `parse_and_update()` — перепарсивает, обновляет Anchor
3. `saved_revision = current_revision`
4. `autosave_timer = None`

### Автосохранение
1. При каждом `apply_edit()` — `autosave_timer = Instant::now()`
2. Регулярно (каждый tick GUI или по таймеру) вызывается `should_autosave()`
3. Если `is_dirty()` и `Instant::now() - autosave_timer >= autosave_interval_secs` — пора
4. Автосохранение также срабатывает при потере фокуса окном (см. Vault)
5. Vault обрабатывает автосохранение: вызывает `doc.save()`, затем `index.reindex_file()`

## Конвертация Anchor

```rust
pub struct AnchoredMetadata {
    pub links: Vec<AnchoredLink>,
    pub tags: Vec<AnchoredTag>,
    pub dates: Vec<AnchoredDate>,
    pub errors: Vec<AnchoredError>,
}

impl AnchoredMetadata {
    /// Создать из ParseResult, где все оффсеты — байтовые.
    /// Каждый offset → Anchor { offset, bias: Left }
    /// Каждый offset+length → Anchor { offset + length, bias: Right }
    pub fn from_parse_result(result: ParseResult, text_len: usize) -> Self;
}
```

Парсер возвращает простые оффсеты (`Span { offset, length }`). Document при `parse_and_update()` создаёт Anchor:

- `start = Anchor { offset: span.offset, bias: Bias::Left }`
- `end = Anchor { offset: span.offset + span.length, bias: Bias::Right }`

## Зависимости

- Крейт `ropey` для Rope
- Крейт `parser` (локальный)
- Крейт `note-model` (локальный, для Anchor/Note)

## Когда не используется

- MCP сервер не использует Document — он читает/пишет файлы напрямую через Vault
- Индекс не использует Document — он хранит только метаданные для поиска
