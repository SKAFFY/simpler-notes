---
priority: P0
layer: core
depends: []
---

- [x]

# Note Model

Модели данных для заметок: позиции в тексте (Anchor), сущности (ссылки, теги, даты) и модель заметки.

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

## Note

```rust
pub struct Note {
    pub path: PathBuf,
    pub metadata: NoteMetadata,
}

pub struct NoteMetadata {
    pub links: Vec<LinkRef>,    // [[вики-ссылки]]
    pub tags: Vec<TagRef>,      // @теги
    pub dates: Vec<DateRef>,    // !даты
}
```

## Свойства

- **path** — путь к .md файлу
- **links** — все [[вики-ссылки]] с Anchor-позициями
- **tags** — все @теги с Anchor-позициями (включая дубликаты)
- **dates** — все !даты с Anchor-позициями

## Идентификация

Заметка идентифицируется по пути к файлу. Две заметки с одинаковым путём считаются одной и той же заметкой.

## Создание

`Note` создаётся через `parser::parse_content(text)`. Полученные байтовые оффсеты используются напрямую для индексации и diagnostics.
