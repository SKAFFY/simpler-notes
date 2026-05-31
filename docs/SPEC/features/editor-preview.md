---
priority: P1
layer: gui
depends: [workspace-layout, parser]
---

- [x]

# Preview Editor

## Source Editor

Используется нативный `gpui::Editor` от Zed (из крейта `gpui`). Это полноценный редактор кода, адаптированный для Markdown.

### Возможности (из коробки gpui)

- Rope-буфер (эффективные вставки/удаления)
- Undo/redo (полный стек)
- Множественные курсоры
- IME-поддержка
- Скроллинг больших файлов
- Буфер обмена (копировать/вставить)

### Настройки для Markdown

- Soft wrap включён (в отличие от кодовых редакторов)
- Номера строк выключены (опционально, можно включить)
- Gutter минимальный

### Получение/запись текста

```rust
// При открытии:
let buffer = vault.open_buffer(path)?;
editor.set_text(buffer.text);

// При сохранении:
buffer.text = editor.text();
vault.save_buffer(&mut buffer)?;
```

### Подсветка

- Markdown-specific подсветка реализуется через `editor.set_highlighted_ranges()` с диапазонами из парсера
- `[[вики-ссылки]]`, `@теги`, даты `!DD.MM.YYYY` подсвечиваются цветом
- Для MVP: без подсветки (gpui::Editor в plain text mode)

### Автокомплит

`@`, `[[`, `!` перехватываются через `cx.on_keystroke()` на gpui::Editor. При вводе триггерного символа:

1. Определить контекст (перед курсором)
2. Вызвать `vault.autocomplete_*()` с префиксом
3. Показать `AutocompletePopup` (см. [autocomplete](./autocomplete.md))
4. При выборе — вставить полную конструкцию вместо триггерного текста

### Режимы переключения

| Текущий | Нажатие Source | Нажатие Preview |
|---------|----------------|-----------------|
| Source | — | Split |
| Split | Source | Preview |
| Preview | Source | Split |

### Не входит в MVP

- Встроенный Markdown-preview внутри редактора (только переключение режимов)

---

Режим предпросмотра Markdown с кликабельными `[[вики-ссылками]]`. Использует `parser::parse_content()` для получения метаданных.

## Рендеринг

- Markdown рендерится через `gpui_component::Markdown`
- `[[Note Name]]` заменяются на кликабельные элементы
- `@теги` и даты пока как обычный текст

## Получение метаданных

При переключении на Preview (или Split):
1. Если `buffer.is_dirty()` — метаданные могут быть stale. Для свежих: парсинг `buffer.text` заново
2. Парсинг не происходит при каждом нажатии клавиши — только при сохранении или переключении на Preview
3. `parse_content(buffer.text)` возвращает `ParseResult` со всеми ссылками, тегами, датами

## Обработка `[[вики-ссылок]]`

1. Из `ParseResult.links` берём все `LinkSpan`
2. Для каждого: заменяем `[[file_name]]` на markdown-ссылку `[label](note://file_name)`
3. При клике:
   - Декодировать `note://file_name`
   - Найти файл `file_name.md` в vault
   - Если найден → открыть в новой вкладке (или переключиться на существующую)
   - Если не найден → серый текст, клик ничего не делает

## Ошибки

- Diagnostics берутся из `vault.get_diagnostics(path)` (те же EmptyLink, InvalidDate, BrokenLink)
- Если есть diagnostics — показать индикатор ошибки (красная волнистая линия или иконка) на соответствующих span
- Позиция ошибки: `diagnostic.span.offset` → line/column через конвертацию offset в строку
- Preview обновляет diagnostics через `vault.get_diagnostics(path)` при каждой перерисовке

## Переключение режимов

| Состояние | Нажатие Preview | Нажатие Source |
|-----------|----------------|----------------|
| Source (default) | → Split | — |
| Split | → Preview | → Source |
| Preview | → Split | → Source |

## Не входит в MVP

- Подсветка `@тегов` и дат в preview
- Подсветка синтаксиса кода в Markdown
