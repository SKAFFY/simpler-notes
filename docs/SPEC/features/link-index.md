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
    pub target: PathBuf,     // куда ссылка (имя файла без .md)
    pub label: String,       // отображаемый текст
    pub span: ByteSpan,      // позиция в исходном файле
}
```

Ключ: **target** — файл, на который ссылаются (PathBuf).
Значение: список `LinkEntry` — все файлы, которые ссылаются на target + их позиции.

Forward-связи (source → target) не хранятся в индексе — они уже есть в `Document.cached_metadata.links` для каждого открытого файла.

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

`outgoing()` проходится по всему backward и собирает записи, где `source == path`. Это неэффективно для большого индекса, но forward не дублируется (он уже в `Document.cached_metadata`).

## Индексация

При парсинге файла (в `reindex_file()`):

```rust
// 1. Удалить старые ссылки из этого файла
index.links.remove_file(path);

// 2. Добавить новые
for link_span in parse_result.links {
    let target = PathBuf::from(&link_span.file_name);
    let entry = LinkEntry {
        source: path.to_path_buf(),
        target: target.clone(),
        label: link_span.label.clone(),
        span: link_span.span,
    };
    index.links.add(path.to_path_buf(), entry);
}
```

## Конкурентность

Аналогично TagIndex — `DashMap`, можно читать и писать одновременно.

## Ограничения

- `outgoing()` — линейный проход по всему backward (O(N)). Для графа связей используется `backlinks()` (O(1)). Если в будущем нужен быстрый outgoing — добавить forward-индекс.
- Ссылка на несуществующий файл (broken link) — не валидируется, просто хранится в индексе.
