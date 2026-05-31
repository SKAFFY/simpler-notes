---
priority: P0
layer: core
depends: [parser]
---

- [x]

# Tag Index

Индекс для поиска заметок по `@тегам`. Хранит все вхождения тега с их позициями в файле. Шардированная хэш-таблица (`DashMap`) для конкурентного доступа.

## Структура

```rust
use std::path::PathBuf;
use dashmap::DashMap;
use serde::{Serialize, Deserialize};

pub struct TagIndex {
    /// тег → [(путь к файлу, [позиции тега в файле])]
    tags: DashMap<String, Vec<TagEntry>>,
}

pub struct TagEntry {
    pub path: PathBuf,
    pub spans: Vec<ByteSpan>,
}

pub struct ByteSpan {
    pub offset: usize,
    pub length: usize,
}

pub struct TagCompletion {
    pub name: String,    // имя тега (без @)
    pub count: usize,    // общее количество вхождений тега (сумма spans.len() по всем файлам)
}
```

Ключ: тег (строка как в исходнике).
Значение: список записей тега — каждая содержит путь к файлу и все позиции, где встречается этот тег.

## API

| Метод | Сигнатура | Описание |
|-------|-----------|----------|
| `add` | `(&self, path: PathBuf, tag: &str, span: ByteSpan)` | Добавить одно вхождение тега |
| `remove` | `(&self, path: &Path, tag: &str)` | Удалить все вхождения тега для указанного файла |
| `get` | `(&self, tag: &str) -> Vec<TagEntry>` | Все файлы с этим тегом и их позиции |
| `all_tags` | `(&self) -> Vec<String>` | Список всех тегов в индексе |
| `autocomplete` | `(&self, prefix: &str) -> Vec<TagCompletion>` | Автокомплит по префиксу (регистронезависимо) |
| `fuzzy_search` | `(&self, query: &str, max_results: usize) -> Vec<TagCompletion>` | Поиск по подстроке, сортировка по релевантности |
| `clear` | `(&self)` | Очистить индекс |

### autocomplete

- Фильтрует `all_tags()` по префиксу (регистронезависимо)
- `count` вычисляется как сумма длин `spans` по всем `TagEntry` для тега
- Сортирует: по убыванию count, потом по алфавиту
- Для сотен тегов — фильтрация на лету за O(N)

### fuzzy_search

- Возвращает теги, где `query` — подстрока имени тега или наоборот (регистронезависимо)
- Сортировка: сначала точное совпадение, потом по убыванию count, потом по алфавиту
- `max_results` ограничивает результат (по умолчанию 10)

## Индексация

При парсинге файла `parse_content()` возвращает `Vec<TagSpan>`. Каждый TagSpan добавляется в индекс.
Если `TagEntry` для данного файла уже существует — `span` добавляется в существующий список:

```rust
for tag_span in parse_result.tags {
    tag_index.add(path.clone(), &tag_span.name, tag_span.span);
}
```

Реализация `add`:
```rust
fn add(&self, path: PathBuf, tag: &str, span: ByteSpan) {
    let mut entries = self.tags.entry(tag.to_string()).or_default();
    match entries.iter_mut().find(|e| e.path == path) {
        Some(entry) => entry.spans.push(span),
        None => entries.push(TagEntry { path, spans: vec![span] }),
    }
}
```

Реализация `remove`:
```rust
fn remove(&self, path: &Path, tag: &str) {
    if let Some(mut entries) = self.tags.get_mut(tag) {
        entries.retain(|e| e.path != path);
        if entries.is_empty() {
            drop(entries);
            self.tags.remove(tag);
        }
    }
}
```

Один тег может встречаться в файле несколько раз — все вхождения попадают в один `TagEntry` с разными `ByteSpan`.

## Конкурентность

- `add()` и `get()` можно вызывать одновременно из разных потоков
- `DashMap` блокирует только один шард, а не весь индекс
- GUI читает индекс, пока watcher пишет — без блокировок

## Ограничения

- Не хранит количество вхождений тега (вычисляется на лету: `entries.iter().map(|e| e.spans.len()).sum()`)
- Регистр не нормализуется — `@Project` и `@project` разные теги
