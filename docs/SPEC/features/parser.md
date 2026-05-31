---
priority: P0
layer: core
depends: [note-model]
---

- [x]

# Парсер Markdown

Извлекает из содержимого .md файла три сущности: `[[вики-ссылки]]`, `@теги` и даты `!DD.MM.YYYY`.

Парсер **не знает** о файловой системе. Он принимает только текст и возвращает байтовые оффсеты.

## Вход

- Содержимое файла (строка UTF-8)

## Сигнатура

```rust
/// Парсит содержимое .md и возвращает результат с байтовыми оффсетами.
pub fn parse_content(text: &str) -> ParseResult;

pub struct ParseResult {
    pub links: Vec<LinkSpan>,
    pub tags: Vec<TagSpan>,
    pub dates: Vec<DateSpan>,
    pub errors: Vec<ParseError>,
}

pub struct LinkSpan {
    pub file_name: String,
    pub label: String,
    pub span: ByteSpan,
}

pub struct TagSpan {
    pub name: String,
    pub span: ByteSpan,
}

pub struct DateSpan {
    pub date: NaiveDate,
    pub raw: String,
    pub span: ByteSpan,
}

pub struct ByteSpan {
    pub offset: usize,     // байт от начала текста
    pub length: usize,     // длина в байтах
}

pub struct ParseError {
    pub span: ByteSpan,
    pub message: String,
    pub kind: ParseErrorKind,
}

pub enum ParseErrorKind {
    InvalidDate,    // !32.13.2000
    EmptyLink,      // [[]]
}
```

## Правила парсинга

### `[[вики-ссылки]]`
1. Регулярное выражение: `\[\[(.+?)(?:\|.+?)?\]\]`
2. Извлекается текст между `[[` и `]]`
3. Алиас `[[Note Name|Label]]` — сохраняется и `file_name`, и `label`
4. Пустые `[[]]` — ошибка `EmptyLink` с позицией
5. Многострочные `[[` не поддерживаются

### `@теги`
1. Слово после `@`, может содержать буквы (включая кириллицу), цифры, дефис, подчёркивание
2. Регулярное выражение: `(?m:^|\s)@([a-zA-Zа-яА-Я0-9_\-]+)`
3. Регистр сохраняется как в исходнике
4. Теги не должны содержать пробелы
5. Дубликаты сохраняются — каждый `@тег` добавляется со своей позицией
6. `@@` (двойной @) не считается тегом
7. `@` считается тегом только если перед ним пробел или начало строки — `user@host.com` не сработает

### Даты `!DD.MM.YYYY`
1. Обязательный префикс `!` перед датой
2. Регулярное выражение: `(?m:^|\s)!(\d{2})\.(\d{2})\.(\d{4})\b`
3. Проверка валидности: `NaiveDate::from_ymd_opt(year, month, day)`
4. Невалидные даты (`!32.13.2000`) — ошибка `InvalidDate` с позицией
5. Другие форматы дат не распознаются

## Когда парсинг происходит

- При открытии файла (`Document::open`)
- При сохранении файла (`Buffer::save`)
- Не происходит при каждом нажатии клавиши

## Производительность

- Парсер stateless — можно параллелить
- Один проход по тексту
- Результат — оффсеты, которые Document конвертирует в Anchor

## Тесты

- `[[Простая ссылка]]` — offset, file_name
- `[[]]` — ошибка `EmptyLink` с offset
- `[[Ссылка|С алиасом]]` — file_name vs label
- `@project @todo @project` — два TagSpan для `project`, один для `todo`
- `!21.07.2003` и `!01.01.2024` (валидные даты)
- `!32.13.2000` — ошибка `InvalidDate` с offset
- `user@host.com` (не считается тегом)
- Пустой текст — пустые списки, errors = []
- Строка без тегов и дат — errors = []
- `@@` не считается тегом
- Многострочный текст — проверка offset
