---
priority: P1
layer: gui
depends: [vault, workspace-layout]
---

- [x]

# Lower Panel Search

Поиск по query language в нижней панели (вкладка Search). Всегда Query mode — Files mode упразднён (FileTree в Project Panel).

## Поле ввода

Поле ввода для query language запросов. При вводе — автокомплит ключевых слов, тегов и дат.

## Автокомплит запроса

### Ключевые слова

Статический список, подставляется как текст:

```
tags contain "❙
date before ❙
date after ❙
date between ❙
text contains "❙
and
or
```

Каждый вариант вставляется в поле ввода, курсор ставится на `❙`.

### Контекстный автокомплит

После ввода ключевого слова с `"` — автокомплит из индекса. Список открывается после ввода первого символа:

| В поле | Поведение |
|--------|-----------|
| `tags contain "` | ждём 1 символ → `vault.autocomplete_tags(prefix)` |
| `tags contain "p` | сразу показывает `autocomplete_tags("p")` |
| `date before ` / `date after ` / `date between ` | ждём 1 символ → `vault.autocomplete_dates(prefix)` |
| `text contains "` | нет автокомплита |

### UI

- Используется общий `AutocompletePopup` (см. [autocomplete](./autocomplete.md))
- Список фильтруется при дальнейшем вводе
- Навигация: см. [autocomplete](./autocomplete.md) — `Tab`/`Shift+Tab`, `Enter`, `Right Arrow`, `Esc`
- При выборе тега/даты — вставляется значение

## Поведение

| Состояние | Отображение |
|-----------|-------------|
| Поле пустое | Пустой список |
| Введён запрос | Результаты `vault.search(query)` |
| Нет результатов | Пустой список |

## UI

- `TextInput` с placeholder "Search..."
- Автокомплит при каждом вводе (с debounce 150ms для вызова search)
- `Cmd+F` → открыть Search таб в Lower Panel, фокус на поле ввода
