---
priority: P0
layer: core
depends: [parser]
---

- [x]

# Date Index

Индекс для поиска заметок по датам `!DD.MM.YYYY`. Хранит все вхождения дат с их позициями в файле. Шардированная хэш-таблица (`DashMap`) для конкурентного доступа.

## Структура

```rust
use std::path::PathBuf;
use chrono::NaiveDate;
use dashmap::DashMap;
use serde::{Serialize, Deserialize};

pub struct DateIndex {
    /// дата → [(путь к файлу, [позиции даты в файле])]
    dates: DashMap<NaiveDate, Vec<DateEntry>>,
}

pub struct DateEntry {
    pub path: PathBuf,
    pub spans: Vec<ByteSpan>,
}

pub struct ByteSpan {
    pub offset: usize,
    pub length: usize,
}
```

Ключ: дата (`NaiveDate`).
Значение: список записей даты — каждая содержит путь к файлу и все позиции, где встречается эта дата.

## API

| Метод | Сигнатура | Описание |
|-------|-----------|----------|
| `add` | `(&self, path: PathBuf, date: NaiveDate, span: ByteSpan)` | Добавить одно вхождение даты |
| `remove` | `(&self, path: &Path, date: NaiveDate)` | Удалить все вхождения даты для указанного файла |
| `get` | `(&self, date: NaiveDate) -> Vec<DateEntry>` | Все файлы с этой датой и их позиции |
| `get_range` | `(&self, from: NaiveDate, to: NaiveDate) -> Vec<(NaiveDate, Vec<DateEntry>)>` | Все даты в диапазоне [from, to] включительно |
| `all_dates` | `(&self) -> Vec<(NaiveDate, Vec<DateEntry>)>` | Все даты с файлами |
| `clear` | `(&self)` | Очистить индекс |

## Особенности

- Дата — точное значение, без учёта времени
- Одна заметка может содержать несколько дат — все вхождения одной даты группируются в один `DateEntry`
- `all_dates()` используется для запросов "date before" и "date after" (фильтрация на стороне поиска)

## Индексация

```rust
for date_span in parse_result.dates {
    date_index.add(path.clone(), date_span.date, date_span.span);
}
```

Реализация `add` — один файл = одна запись со всеми spans:
```rust
fn add(&self, path: PathBuf, date: NaiveDate, span: ByteSpan) {
    let mut entries = self.dates.entry(date).or_default();
    match entries.iter_mut().find(|e| e.path == path) {
        Some(entry) => entry.spans.push(span),
        None => entries.push(DateEntry { path, spans: vec![span] }),
    }
}
```

Реализация `remove`:
```rust
fn remove(&self, path: &Path, date: NaiveDate) {
    if let Some(mut entries) = self.dates.get_mut(&date) {
        entries.retain(|e| e.path != path);
        if entries.is_empty() {
            drop(entries);
            self.dates.remove(&date);
        }
    }
}
```

## Конкурентность

Аналогично TagIndex — `DashMap`, можно читать и писать одновременно.
