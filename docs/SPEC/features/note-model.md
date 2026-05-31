---
priority: P0
layer: core
depends: []
---

- [x]

# Note Model

Модели данных для заметок: позиции в тексте (Anchor), сущности (ссылки, теги, даты) и модель заметки.

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

`Note` создаётся через `parser::parse_content(text)`. Полученные байтовые оффсеты затем конвертируются в Anchor через `Document::parse_and_update()`.
