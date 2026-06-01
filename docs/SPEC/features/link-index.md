---
priority: P0
layer: core
depends: [parser]
---

- [x]

# Link Index

Индекс обратных ссылок между заметками (`[[note]]`). Позволяет узнать, какие файлы ссылаются на target (backlinks) и куда ссылается source (outgoing). Работает аналогично TagIndex и DateIndex.

## Структура

```rust
use std::path::PathBuf;
use dashmap::DashMap;

pub struct LinkIndex {
    /// backward: target → [ссылки на этот файл]
    backward: DashMap<PathBuf, Vec<LinkEntry>>,
}

pub struct LinkEntry {
    pub source: PathBuf,     // откуда ссылка
    pub target: PathBuf,     // куда ссылка (только имя файла, file_stem, без расширения и без пути)
    pub label: String,       // отображаемый текст
    pub span: ByteSpan,      // позиция в исходном файле
}
```

Ключ: **target** — имя файла (file_stem без расширения), на который ссылаются (PathBuf).
Значение: список `LinkEntry` — все файлы, которые ссылаются на target + их позиции.

Forward-связи (source → target) не хранятся в индексе — их можно получить через `parse_content()` для каждого открытого файла при необходимости.

## API

| Метод | Сигнатура | Описание |
|-------|-----------|----------|
| `add` | `(&self, source: PathBuf, entry: LinkEntry)` | Добавить одну ссылку (source → target) |
| `remove_file` | `(&self, path: &Path)` | Удалить все ссылки, где source == path |
| `backlinks` | `(&self, target: &Path) -> Vec<LinkEntry>` | Какие файлы ссылаются на target |
| `outgoing` | `(&self, source: &Path) -> Vec<LinkEntry>` | Куда ссылается source (через backward lookup — по всем записям, где source передан) |
| `clear` | `(&self)` | Очистить индекс |

### add

Добавляет запись backward: `target → { source, label, span }`.

### remove_file

Удаляет все записи backward, где `source == path`. Используется при переиндексации файла.

### outgoing

`outgoing()` проходится по всему backward и собирает записи, где `source == path`. Это неэффективно для большого индекса, но forward не дублируется (прямые ссылки можно получить через `parse_content()`).

## Индексация

При парсинге файла (в `reindex_file()`):

```rust
// 1. Удалить старые ссылки из этого файла
index.links.remove_file(path);

// 2. Добавить новые
for link_span in parse_result.links {
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
    index.links.add(path.to_path_buf(), entry);
}
```

`target` всегда сохраняется как плоское имя файла (file_stem) без расширения, без пути. Относительные пути (`../note`, `sub/file.md`) резолвятся и нормализуются до чистого имени файла.

## Конкурентность

Аналогично TagIndex — `DashMap`, можно читать и писать одновременно.

## Ограничения

- `outgoing()` — линейный проход по всему backward (O(N)). Для графа связей используется `backlinks()` (O(1)). Если в будущем нужен быстрый outgoing — добавить forward-индекс.
- Ссылка на несуществующий файл (broken link) — не валидируется, просто хранится в индексе.
