---
priority: P0
layer: core
depends: [tag-index, date-index]
---

- [x]

# Query Language

Язык поисковых запросов для фильтрации заметок по тегам, датам и тексту.
Fulltext-часть выполняется через `ripgrep` (rg) по файловой системе — без инвертированного индекса.

## Грамматика

```
query       = expression
expression  = condition (("and" | "or") condition)*
condition   = "tags contain" string
            | "date before" date
            | "date after" date
            | "date between" date ";" date
            | "text contains" string

string      = '"' ... '"'
date        = DD.MM.YYYY
```

## Примеры запросов

```
tags contain "project" and date before 01.01.2025
date after 01.01.2024 and date before 01.06.2024
date between 01.01.2024;01.06.2024
tags contain "project" or tags contain "todo"
text contains "hello world"
tags contain "project" and text contains "database schema"
```

## Типы условий

### `tags contain "тег"`
- Регистрозависимо (как в исходнике)
- Исполняется через `TagIndex::get()`

### `date before DD.MM.YYYY` / `date after DD.MM.YYYY` / `date between DD.MM.YYYY and DD.MM.YYYY`

Все date-условия **включают границы** (inclusive):
- `date before 01.01.2025` → дата `<= 01.01.2025`
- `date after 01.01.2024` → дата `>= 01.01.2024`
- `date between 01.01.2024;01.06.2024` → `01.01.2024 <= дата <= 01.06.2024`

### `text contains "строка"`
- Исполняется через `ripgrep` (`rg -l --null -F`)
- Строка в кавычках, регистрозависимо (пока, TODO: `-i`)
- Если в запросе несколько слов — вся строка передаётся как один шаблон (точное совпадение фразы)

## Логические операторы

| Оператор | Поведение |
|----------|-----------|
| `and` | Пересечение результатов (intersection) |
| `or` | Объединение результатов (union, dedup) |

Приоритет: `and` выполняется раньше `or`. Для изменения приоритета используются скобки (TODO).

## AST

```rust
pub enum Query {
    TagsContain(String),
    DateBefore(NaiveDate),
    DateAfter(NaiveDate),
    DateBetween(NaiveDate, NaiveDate),
    Text(String),
    And(Box<Query>, Box<Query>),
    Or(Box<Query>, Box<Query>),
}
```

## Результат поиска

```rust
pub struct SearchResult {
    pub path: PathBuf,
    pub title: String,
}
```

## Исполнение

```rust
pub fn execute_search(index: &ConcurrentIndex, query: &Query, vault_path: &Path) -> Result<Vec<SearchResult>>;
```

Логика исполнения — рекурсивный обход AST:

| Query variant | Исполнение |
|---------------|------------|
| `TagsContain(tag)` | `index.tags.get(tag)` → список путей + позиции |
| `DateBefore(date)` | фильтр по `index.dates.all_dates()`, `d <= date` |
| `DateAfter(date)` | фильтр по `index.dates.all_dates()`, `d >= date` |
| `DateBetween(from, to)` | фильтр по `index.dates.all_dates()`, `from <= d <= to` |
| `Text(term)` | `rg -l --null -F "{term}" {vault_path}` |
| `And(a, b)` | пересечение результатов a и b |
| `Or(a, b)` | объединение результатов a и b |

### Гибридная оптимизация

Если `Text` комбинируется через `And` с `TagsContain` или датами — `Text` исполняется **только по отфильтрованному подмножеству файлов**, а не по всему vault:

```
TagsContain("project") AND Text("database")
→ paths = index.tags.get("project")          // по индексу
→ filtered = rg("database", paths)          // rg только по paths
```

Это достигается передачей `--filelist` (или stdin) в rg:

```rust
fn execute_text(term: &str, vault_path: &Path, filter: Option<&[PathBuf]>) -> Result<Vec<PathBuf>> {
    let mut cmd = Command::new("rg");
    cmd.args(["-l", "--null", "-F", term]);
    if let Some(paths) = filter {
        let stdin = paths.iter()
            .map(|p| p.to_string_lossy())
            .collect::<Vec<_>>()
            .join("\n");
        cmd.arg("--filelist").arg("-");
        // общий случай: paths загружаются в stdin
    }
    cmd.arg(vault_path);
    // парсим stdout: каждая строка — null-terminated путь
}
```

Если `Text` без фильтра — просто `rg -l --null vault/`.

Если запрос состоит только из тегов/дат — rg не вызывается.

## Парсер

Рекурсивный спуск:
1. Разделить на top-level по `and`, затем по `or`
2. Каждая часть — условие с префиксом
3. Распарсить условие по префиксу (`tags contain`, `text contains`, `date before`, `date after`)
4. После префикса — строка в кавычках или дата без кавычек

All conditions are explicit — plain text without a prefix is a syntax error.

## План (позже)
- Скобки `(...)` для группировки
- `tags contain any ["a", "b"]` — любой из тегов
- `tags contain all ["a", "b"]` — все теги
- `-i` (case insensitive) для `text contains`
- `-E` (regex) для `text contains`
