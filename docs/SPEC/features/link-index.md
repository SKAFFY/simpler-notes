---
priority: P0
layer: core
depends: [parser]
---

- [x]

# Link Index

Индекс связей между заметками (`[[wiki-link]]`). Позволяет узнать, какие файлы ссылаются на target (backlinks), куда ссылается source (outgoing), и поддерживает rename refactoring.

## Структура

```rust
pub struct LinkIndex {
    /// Полный путь цели → обратные ссылки (O(1) для backlinks)
    by_target: DashMap<PathBuf, Vec<LinkEntry>>,
    /// Полный путь источника → исходящие ссылки (O(1) для outgoing)
    by_source: DashMap<PathBuf, Vec<LinkEntry>>,
}

pub struct LinkEntry {
    pub source: PathBuf,     // полный путь к файлу-источнику
    pub target: PathBuf,     // полный путь к файлу-цели (resolved)
    pub label: String,       // отображаемый текст ссылки
    pub span: ByteSpan,      // позиция в исходном файле
}
```

Две мапы: `by_target` — для обратных ссылок, `by_source` — для исходящих. Обе O(1).

`LinkEntry.target` — это полный путь к файлу, на который ссылаются (не stem!). При индексации происходит резолв через `filename_index`.

## API

| Метод | Сигнатура | Описание |
|-------|-----------|----------|
| `add` | `(&self, source: PathBuf, entry: LinkEntry)` | Добавить entry в обе мапы |
| `remove_file` | `(&self, path: &Path)` | Удалить все entry с source == path из обеих мап |
| `backlinks` | `(&self, target: &Path) -> Vec<LinkEntry>` | Кто ссылается на файл — O(1) |
| `outgoing` | `(&self, source: &Path) -> Vec<LinkEntry>` | Куда ссылается файл — O(1) |
| `update_target` | `(&self, old: &Path, new: &Path)` | Перенести ключ в by_target и обновить entry.target |
| `all_targets` | `(&self) -> Vec<PathBuf>` | Все уникальные target пути (для autocomplete) |
| `clear` | `(&self)` | Очистить обе мапы |

### add

```rust
pub fn add(&self, _source: PathBuf, entry: LinkEntry) {
    self.by_target.entry(entry.target.clone()).or_default().push(entry.clone());
    self.by_source.entry(entry.source.clone()).or_default().push(entry);
}
```

### remove_file

Удаляет все записи, где `source == path`, из обеих мап. После удаления чистит пустые ключи.

### update_target

При rename файла — переносит ключ в `by_target` и обновляет `entry.target` у всех записей:

```rust
pub fn update_target(&self, old: &Path, new: &Path) {
    if let Some((_, entries)) = self.by_target.remove(old) {
        for mut entry in entries {
            entry.target = new.to_path_buf();
            self.by_target.entry(new.to_path_buf()).or_default().push(entry);
        }
    }
}
```

## Индексация

При парсинге файла (в `reindex_file()`):

1. Удалить старые ссылки из этого файла: `index.links.remove_file(path)`
2. Для каждой распарсенной ссылки:
   - Взять `link_span.file_name` (raw target из `[[...]]`)
   - Зарезолвить относительный путь относительно папки source-файла
   - Нормализовать (`normalize_path`)
   - Извлечь file_stem
   - Попытаться зарезолвить stem в полный путь через `filename_index`:
     - `[path]` → используем этот путь (unambiguous)
     - `[]` или `[a, b, ...]` → используем нормализованный путь как fallback
   - Создать `LinkEntry` с resolved `target`
   - Вызвать `index.links.add(source_path, entry)`
3. Запустить диагностику (`check_file`)

```rust
for link_span in &result.links {
    let raw = PathBuf::from(&link_span.file_name);
    let resolved = if raw.is_absolute() { raw }
        else { path.parent().unwrap_or(Path::new("")).join(&raw) };
    let normalized = normalize_path(&resolved);
    let stem = normalized.file_stem()
        .unwrap_or(normalized.as_os_str())
        .to_string_lossy().to_string();

    let full_target = match filename_index.get(&stem) {
        Some(paths) if paths.len() == 1 => paths[0].clone(),
        _ => normalized,
    };

    let entry = LinkEntry {
        source: path.to_path_buf(),
        target: full_target,
        label: link_span.label.clone(),
        span: ByteSpan { offset: link_span.span.offset, length: link_span.span.length },
    };
    self.links.add(path.to_path_buf(), entry);
}
```

## Rename refactoring

`Vault::rename_file(old_rel, new_rel)` — полный rename файла с переписыванием ссылок:

1. Получить все файлы, ссылающиеся на `old_path`: `links.backlinks(&old_path)`
2. Для каждого такого файла: прочитать содержимое, заменить `[[old_stem]]` → `[[new_stem]]` через span'ы в LinkEntry, записать обратно
3. Переместить файл: `std::fs::rename(&old_path, &new_path)`
4. Обновить индекс:
   - `links.update_target(&old_path, &new_path)` — перенести ключ в by_target
   - `links.remove_file(&old_path)` — убрать source-записи старого пути
   - `reindex_file(new_path)` — переиндексировать renamed-файл
   - Для каждого изменённого файла: `reindex_file(source)` — переиндексировать
5. Сохранить индекс на диск

## Персистентность

Формат `links.json` — плоский `Vec<LinkEntry>`. Версия индекса: INDEX_VERSION = 2.

```rust
// save
let entries: Vec<LinkEntry> = self.by_source.iter()
    .flat_map(|e| e.value().iter().cloned())
    .collect();
serde_json::to_string_pretty(&entries)

// load
for entry in entries {
    index.links.add(entry.source.clone(), entry);
}
```

Миграция со старого формата не поддерживается — при несовпадении версии индекса происходит полная переиндексация.

## Взаимодействие с Watcher

При ручном rename через Finder/Terminal watcher видит `Remove(old) + Create(new)`. Refactoring ссылок не происходит — это conscious action пользователя через GUI или MCP. Watcher корректно обновляет индекс для Create/Modify, а Remove чистит старые записи.

## Конкурентность

`DashMap` для обеих мап, можно читать и писать одновременно.

## Изменения в зависимых компонентах

| Компонент | Изменение |
|-----------|-----------|
| `vault.rs` | `get_backlinks` — без изменений. `get_outgoing_links` — теперь O(1). `autocomplete_links` — `all_targets()` возвращает `PathBuf` (полные пути), фильтр по file_stem. |
| `search.rs` | Поиск `link:<target>` — использует `backlinks`, без изменений. |
| `mcp/get_backlinks.rs` | target в ответе — полный путь, не stem. |
| `mcp/get_outgoing_links.rs` | target в ответе — полный путь, не stem. |
