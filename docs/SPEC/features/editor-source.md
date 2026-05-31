---
priority: P1
layer: gui
depends: [workspace-layout, document, vault]
---

- [x]

# Source Editor

Режим редактирования Markdown как plain text. Работает через `Document.text` (Rope).

## Реализация

- Использует `gpui::EditorMultiline` или `gpui_component::TextArea`
- Читает/пишет через `document.text` (Rope — эффективные вставки/удаления)

## Поведение

| Действие | Результат |
|----------|-----------|
| Ввод текста | `document.apply_edit(start, end, text)` → Rope обновляется |
| Изменение содержимого | `document.is_dirty() == true` |
| Переключение на Preview | Контент не теряется — cached_metadata устарел, но Anchor корректны |
| Закрытие вкладки с dirty | Подтверждение (TODO после MVP) |
| Ввод `@` / `[[` / `!` | Открывается попап автокомплита |

## Подсветка синтаксиса

Подсветка применяется на основе `cached_metadata` и `diagnostics`. При каждой перерисовке текста редактор применяет стили к диапазонам:

| Элемент | Стиль | Источник |
|---------|-------|----------|
| `@тег` | цветной (theme accent) | `cached_metadata.tags` |
| `!дата` | цветной (theme date) | `cached_metadata.dates` |
| `[[ссылка]]` | цветной (theme link) | `cached_metadata.links` |
| EmptyLink, InvalidDate, BrokenLink | красный волнистый underline | `vault.get_diagnostics(path)` |

### Реализация

```rust
fn apply_highlighting(text: &str, metadata: &AnchoredMetadata, diagnostics: &[Diagnostic]) -> Vec<HighlightSpan> {
    let mut spans = Vec::new();
    for tag in &metadata.tags {
        spans.push(HighlightSpan {
            start: tag.span.start.offset,
            end: tag.span.end.offset,
            style: Style::Tag,
        });
    }
    for date in &metadata.dates {
        spans.push(HighlightSpan {
            start: date.span.start.offset,
            end: date.span.end.offset,
            style: Style::Date,
        });
    }
    for link in &metadata.links {
        spans.push(HighlightSpan {
            start: link.span.start.offset,
            end: link.span.end.offset,
            style: Style::Link,
        });
    }
    for diag in diagnostics {
        spans.push(HighlightSpan {
            start: diag.span.offset,
            end: diag.span.offset + diag.span.length,
            style: Style::Error,
        });
    }
    spans
}
```

- Подсветка обновляется при каждом изменении текста (после `apply_edit`)
- `cached_metadata` не перепарсивается на каждый символ — Anchor сдвигаются автоматически
- Если `cached_metadata` устарел (не перепарсивался после редактирования) — подсветка может быть неточной, но не ломается

## Синхронизация с Document

- SourceEditor не копирует текст — он работает напрямую с `document.text`
- При каждом изменении — `apply_edit` на Rope
- Anchor в `cached_metadata` автоматически сдвигаются при редактировании

## Автокомплит

### Триггеры

| Ввод | Показывается |
|------|-------------|
| `@` + буквы | Список тегов (из TagIndex), отфильтрованных по введённому префиксу |
| `[[` + буквы | Список .md файлов в vault, отфильтрованных по введённому префиксу |
| `!` | Сегодняшняя дата + топ-5 дат из DateIndex |

### UI

- Попап появляется над/под курсором сразу после ввода триггера
- При дальнейшем вводе список фильтруется
- Навигация: см. [autocomplete](./autocomplete.md) — `Tab`/`Shift+Tab`, `Enter`, `Right Arrow`, `Esc`
- При выборе — вставляется полная конструкция:
  - @ + project + пробел → `@project `
  - [[ + Note + Enter → `[[Note Name]]`
  - ! + Enter → `!31.05.2026 `

### Данные

Автокомплит получает данные через `Vault`:

```rust
// source_editor.rs
fn on_input(text: &str, cursor: usize, vault: &Vault) -> Option<AutocompleteState> {

    match prefix_before {
        ("@", partial) => {
            let completions = vault.autocomplete_tags(partial);
            Some(AutocompleteState::Tags(completions, cursor))
        }
        ("[[", partial) => {
            let completions = vault.autocomplete_links(partial);
            Some(AutocompleteState::Links(completions, cursor))
        }
        ("!", partial) => {
            let completions = vault.autocomplete_dates(partial);
            Some(AutocompleteState::Dates(completions, cursor))
        }
        _ => None,
    }
}
```

## Не входит в MVP

- Сворачивание заголовков
