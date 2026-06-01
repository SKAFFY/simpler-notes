---
priority: P0
layer: core
depends: [parser, note-model, tag-index, date-index, link-index, index-persistence, query-language, document]
---

- [x]

# Vault

Оркестратор — центральная точка доступа к хранилищу заметок. Открывает директорию, строит/загружает индекс, предоставляет API для чтения, записи, поиска и открытия документов.

## Структура

```rust
pub struct Vault {
    pub path: PathBuf,
    pub index: Arc<ConcurrentIndex>,
}
```

- **path** — корневая директория заметок
- **index** — индекс (теги, даты, file_states)

## API

```rust
impl Vault {
    /// Открыть vault из директории.
    /// Загружает персистентный индекс или строит новый в фоне.
    pub fn open(path: &Path) -> Result<Self, String>;

    /// Открыть .md файл как Buffer (читает с диска в String).
    /// Валидирует [[ссылки]]: если target не существует — добавляет в diagnostics BrokenLink.
    pub fn open_buffer(&self, path: &Path) -> Result<Buffer, String>;

    /// Сохранить buffer на диск и переиндексировать.
    pub fn save_buffer(&self, buf: &mut Buffer) -> Result<(), String>;

    /// Поиск по query language.
    pub fn search(&self, query: &str) -> Result<Vec<SearchResult>, String>;

    /// Прочитать содержимое файла как строку.
    pub fn read_note(&self, path: &Path) -> Result<String, String>;

    /// Записать содержимое в файл.
    pub fn write_note(&self, path: &Path, content: &str) -> Result<(), String>;

    /// Все теги в индексе. Резерв: может использоваться для UI поиска по тегам
    /// (пустой фильтр → список всех тегов). Если не понадобится — удалить.
    pub fn get_all_tags(&self) -> Vec<String>;

    /// Даты в диапазоне [from, to] для таймлайна.
    pub fn get_dates_in_range(&self, from: NaiveDate, to: NaiveDate) -> Vec<(NaiveDate, Vec<DateEntry>)>;

    /// Отчёт о состоянии индекса.
    pub fn validate_indexes(&self) -> IndexReport;

    /// Список всех .md файлов в vault (рекурсивно).
    pub fn list_md_files(&self) -> Vec<PathBuf>;

    // -- Автокомплит --

    /// Автокомплит для @тегов.
    pub fn autocomplete_tags(&self, prefix: &str) -> Vec<TagCompletion>;

    /// Fuzzy поиск по @тегам (подстрока, для случаев когда точное имя забыто).
    pub fn fuzzy_search_tags(&self, query: &str) -> Vec<TagCompletion>;

    /// Автокомплит для [[: список .md файлов, начинающихся с prefix.
    pub fn autocomplete_links(&self, prefix: &str) -> Vec<String>;

    /// Автокомплит для !: сегодня + топ-5 дат из индекса.
    pub fn autocomplete_dates(&self, prefix: &str) -> Vec<String>;

    // -- Управление индексом --

    /// Полная перестройка индекса: очистить, перепарсить все .md файлы, сохранить на диск.
    pub fn reindex_all(&self) -> Result<IndexReport, String>;

    // -- Ссылки --

    /// Какие файлы ссылаются на target.
    pub fn get_backlinks(&self, target: &Path) -> Vec<LinkEntry>;

    /// Куда ссылается source (поиск по backward, O(N)).
    pub fn get_outgoing_links(&self, source: &Path) -> Vec<LinkEntry>;

    /// Разрешить плоское имя ссылки (file_stem) в полный путь к файлу.
    /// Ошибка если 0 или >1 совпадений.
    pub fn resolve_link(&self, target: &str) -> Result<PathBuf, String>;

    // -- Diagnostics --

    /// Diagnostics для одного файла.
    pub fn get_diagnostics(&self, path: &Path) -> Vec<Diagnostic>;

    /// Diagnostics для всех файлов (для UI списка ошибок).
    pub fn all_diagnostics(&self) -> Vec<(PathBuf, Vec<Diagnostic>)>;
}
```

## Индекс

- `ConcurrentIndex` (tag, date, links, diagnostics, file_states) — как в [tag-index](./tag-index.md), [date-index](./date-index.md), [link-index](./link-index.md), [diagnostics](./diagnostics.md)
- Индекс хранит `тег/дата → [(путь, [позиции])]` и `файл → [[ссылки на файл]]` — с позициями для подсветки и навигации
- `file_states` — обратный индекс: `файл → {tags, dates}` для инкрементальной переиндексации
- `filename_index` — маппинг `file_stem → [пути]` для проверки коллизий имён при диагностике ссылок
- Fulltext-поиск выполняется внешним инструментом (`ripgrep`), см. [query-language](./query-language.md)

## Индексация

### Валидация ссылок и diagnostics

Парсер не знает о файловой системе, поэтому `BrokenLink` добавляется на уровне Diagnostics. При каждом `reindex_file()` вызывается `diagnostics.check_file()`, который проверяет все `[[ссылки]]` на существование файла и собирает все ошибки (EmptyLink, InvalidDate, BrokenLink). См. [diagnostics](./diagnostics.md).

### Первичная (при открытии vault)

При первой загрузке vault все .md файлы парсятся и заносятся в индекс:

```rust
for path in all_md_files {
    let content = fs::read_to_string(&path)?;
    index.reindex_file(&path, &content);
}
index.save(self.path);
```

### Инкрементальная (при сохранении файла)

При `save_buffer()` вызывается `ConcurrentIndex::reindex_file()`:

```rust
pub fn reindex_file(&self, path: &Path, content: &str) {
    let parse_result = parse_content(content);

    // 1. Удалить старые записи для этого файла
    if let Some(old_state) = self.file_states.get(path) {
        for tag in &old_state.tags {
            self.tags.remove(path, tag);
        }
        for date in &old_state.dates {
            self.dates.remove(path, date);
        }
    }
    self.links.remove_file(path);
    self.diagnostics.remove(path);

    // 2. Добавить новые записи
    for tag_span in &parse_result.tags {
        self.tags.add(path.to_path_buf(), &tag_span.name, tag_span.span);
    }
    for date_span in &parse_result.dates {
        self.dates.add(path.to_path_buf(), date_span.date, date_span.span);
    }
    for link_span in &parse_result.links {
        let raw_target = PathBuf::from(&link_span.file_name);
        let resolved = if raw_target.is_absolute() {
            raw_target
        } else {
            path.parent().unwrap_or(Path::new("")).join(&raw_target)
        };
        let normalized = normalize_path(&resolved);
        let file_stem = normalized
            .file_stem()
            .unwrap_or(normalized.as_os_str())
            .to_string_lossy()
            .to_string();
        let target = PathBuf::from(file_stem);
        let entry = LinkEntry {
            source: path.to_path_buf(),
            target: target.clone(),
            label: link_span.label.clone(),
            span: link_span.span,
        };
        self.links.add(path.to_path_buf(), entry);
    }

    // 3. Diagnostics
    self.diagnostics.check_file(path, content, &self.path, &filename_index);

    // 4. Сохранить новое состояние файла
    self.file_states.insert(path.to_path_buf(), FileIndexState {
        tags: parse_result.tags.iter().map(|t| t.name.clone()).collect(),
        dates: parse_result.dates.iter().map(|d| d.date).collect(),
    });
}
```

### Полная перестройка

`reindex_all()` очищает индекс и парсит все .md файлы заново. Перед индексацией строится `filename_index`:

```rust
fn build_filename_index(&self) -> HashMap<String, Vec<PathBuf>> {
    let mut map = HashMap::new();
    for entry in self.list_md_files() {
        if let Some(stem) = entry.file_stem() {
            let name = stem.to_string_lossy().to_string();
            map.entry(name).or_default().push(entry);
        }
    }
    map
}
```

```rust
impl Vault {
    pub fn reindex_all(&self) -> Result<IndexReport, String> {
        self.index.clear();
        let filename_index = self.build_filename_index();
        let md_files = self.list_md_files()?;
        for path in &md_files {
            let content = std::fs::read_to_string(path)
                .map_err(|e| e.to_string())?;
            self.index.reindex_file(path, &content, &filename_index);
        }
        self.index.save(&self.path)?;
        Ok(self.validate_indexes())
    }
}
```

## Сохранение buffer

```rust
impl Vault {
    pub fn save_buffer(&self, buf: &mut Buffer) -> Result<(), String> {
        let path = buf.path.clone()
            .ok_or_else(|| "Buffer has no path".to_string())?;

        buf.save()?;

        // Переиндексация
        self.index.reindex_file(&path, &buf.text);

        // Сохранить индекс на диск
        self.index.save(&self.path)?;

        Ok(())
    }
}
```

## Автосохранение

### Триггеры

| Триггер | Описание |
|---------|----------|
| Таймер | Каждый tick GUI проверяет `buffer.is_dirty()` для всех dirty buffer'ов |
| Потеря фокуса | При сворачивании окна — все dirty buffer'ы автосохраняются |

### Процесс

1. GUI проверяет `buffer.is_dirty()` для всех открытых buffer'ов
2. Если buffer dirty — вызывает `vault.save_buffer(&mut buffer)`
3. `save_buffer()` пишет файл на диск, переиндексирует, сохраняет индекс

## Жизненный цикл открытия файла

1. `vault.open_buffer(path)` → `Buffer` (core, читает файл с диска)
2. GUI: `editor.set_text(buffer.text)`, сохраняет ссылку на `Arc<RwLock<Buffer>>`
3. При редактировании: `buffer.current_revision++` (через gpui::Editor)
4. При явном сохранении (Ctrl+S): `buffer.text = editor.text()`, `vault.save_buffer(&mut buffer)` → пишет диск + переиндексация
5. При автосохранении: то же, что и явное сохранение
6. Индекс на диске обновляется при каждом `save_buffer()`

Для MCP (headless):
1. `vault.open_buffer(path)` → `Buffer`
2. Агент читает `buffer.text` целиком
3. Агент пишет новый `buffer.text` целиком
4. `vault.save_buffer(&mut buffer)`

## Автокомплит

### tags
- `autocomplete_tags("pro")` → фильтрует `TagIndex::all_tags()` по префиксу
- Сортировка: по убыванию count, потом по алфавиту

### links
- `autocomplete_links("Pro")` → ищет .md файлы в vault, чьё имя начинается с prefix
- Сканирование файлового дерева (кешируется или на лету)

### dates
- `autocomplete_dates("19.05")` → сегодняшняя дата + топ-5 дат, начинающихся с `19.05`
- Формат: `DD.MM.YYYY`

## Поиск

```rust
pub struct SearchResult {
    pub path: PathBuf,
    pub title: String,
}
```

`title` вычисляется при поиске: первый H1 из файла или имя файла.

## Ошибки

- Все методы возвращают `Result<_, String>`
- `open` — если путь не существует
- `open_buffer` — если файл не найден или не читается
- `save_buffer` — если у Buffer нет path

## После MVP

- Поиск по тегам (фильтр с автокомплитом)
- Поиск по датам (фильтр с автокомплитом)
- `get_all_tags()` и `get_all_dates()` остаются в API как резерв для этой функциональности
