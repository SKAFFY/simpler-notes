---
priority: P1
layer: gui
depends: [workspace-layout, parser, document]
---

- [x]

# Preview Editor

Режим предпросмотра Markdown с кликабельными `[[вики-ссылками]]`. Использует `document.cached_metadata` для навигации.

## Рендеринг

- Markdown рендерится через `gpui_component::Markdown`
- `[[Note Name]]` заменяются на кликабельные элементы
- `@теги` и даты пока как обычный текст

## Получение метаданных

При переключении на Preview (или Split):
1. Если `document.is_dirty()` — можно показать stale данные (Anchor всё ещё корректны)
2. Опционально: `document.parse_and_update()` для актуальных Anchor
3. `document.cached_metadata` содержит `AnchoredMetadata` со всеми ссылками, тегами, датами

## Обработка `[[вики-ссылок]]`

1. Из `cached_metadata.links` берём все `LinkRef`
2. Для каждого: заменяем `[[file_name]]` на markdown-ссылку `[label](note://file_name)`
3. При клике:
   - Декодировать `note://file_name`
   - Найти файл `file_name.md` в vault
   - Если найден → открыть в новой вкладке (или переключиться на существующую)
   - Если не найден → серый текст, клик ничего не делает

## Ошибки

- Diagnostics берутся из `vault.get_diagnostics(path)` (те же EmptyLink, InvalidDate, BrokenLink)
- Если есть diagnostics — показать индикатор ошибки (красная волнистая линия или иконка) на соответствующих span
- Позиция ошибки: `diagnostic.span.offset` → line/column через `document.anchor_to_line_col()`
- Preview обновляет diagnostics при `document.parse_and_update()` (после каждой перерисовки)

## Переключение режимов

| Состояние | Нажатие Preview | Нажатие Source |
|-----------|----------------|----------------|
| Source (default) | → Split | — |
| Split | → Preview | → Source |
| Preview | → Split | → Source |

## Не входит в MVP

- Подсветка `@тегов` и дат в preview
- Подсветка синтаксиса кода в Markdown
