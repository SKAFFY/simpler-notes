---
priority: P1
layer: gui
depends: [workspace-layout]
---

- [x]

# Autocomplete Popup

Общий компонент автокомплита для Source Editor и Search. Показывает выпадающий список вариантов при вводе триггерных символов.

## Контексты

```rust
pub enum AutocompleteContext {
    SourceEditor,   // @, [[, !
    SearchQuery,    // ключевые слова, теги, даты
}
```

`AutocompleteLocation` (из [workspace-layout](./workspace-layout.md)) определяет где показывать попап на экране.

## Клавиши

| Клавиша | Действие |
|---------|----------|
| `Tab` | Следующий пункт |
| `Shift+Tab` | Предыдущий пункт |
| `Enter` | Выбрать текущий пункт |
| `Right Arrow` | Закрыть попап (без выбора) |
| `Esc` | Закрыть попап (без выбора) |

- `Right Arrow` закрывает попап, курсор остаётся на месте (ввод текста продолжается)
- `Esc` закрывает попап, курсор остаётся на месте

## Общее состояние

```rust
pub struct AutocompletePopup {
    pub visible: bool,
    pub context: AutocompleteContext,
    pub items: Vec<String>,
    pub selected: usize,
    pub location: AutocompleteLocation,
}

impl AutocompletePopup {
    pub fn navigate_next(&mut self);       // Tab
    pub fn navigate_prev(&mut self);       // Shift+Tab
    pub fn dismiss(&mut self);             // Esc, Right Arrow
    pub fn select(&self) -> Option<&str>;  // Enter
}
```

## Поведение

- Попап появляется после ввода триггерного символа
- Список фильтруется при дальнейшем вводе (в Source Editor — по префиксу после `@`, `[[`, `!`; в Search — по ключевым словам и кавычкам)
- При выборе — вставляется полная конструкция:
  - Source Editor: `@project`, `[[Note Name]]`, `!31.05.2026`
  - Search: `tags contain "project"`, `date before 01.01.2024`
