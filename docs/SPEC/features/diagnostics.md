---
priority: P0
layer: core
depends: [parser]
---

- [x]

# Diagnostics

Фоновый анализ всех .md файлов в vault на ошибки. Работает как LSP — при каждом изменении файла diagnostics обновляются.

## Структура

```rust
use std::path::PathBuf;
use dashmap::DashMap;

pub struct Diagnostics {
    /// файл → [проблемы в этом файле]
    file_diagnostics: DashMap<PathBuf, Vec<Diagnostic>>,
}

pub struct Diagnostic {
    pub span: ByteSpan,
    pub message: String,
    pub severity: Severity,
}

pub enum Severity {
    Warning,
}
```

## Типы диагностик

| Ситуация | Сообщение | Severity |
|----------|-----------|----------|
| `[[]]` (пустая ссылка) | `Empty link` | Warning |
| `!DD.MM.YYYY` невалидная дата | `Invalid date: 32.13.2000` | Warning |
| `[[NonExistent]]` — файл не найден | `Broken link: NonExistent — file not found` | Warning |

## API

```rust
impl Diagnostics {
    /// Создать пустой набор diagnostics.
    pub fn new() -> Self;

    /// Проверить один файл и сохранить результат.
    pub fn check_file(&self, path: &Path, content: &str, vault_path: &Path);

    /// Получить diagnostics для одного файла.
    pub fn get(&self, path: &Path) -> Vec<Diagnostic>;

    /// Все diagnostics (для UI — показать все ошибки в проекте).
    pub fn all(&self) -> Vec<(PathBuf, Vec<Diagnostic>)>;

    /// Удалить diagnostics для файла (при удалении файла).
    pub fn remove(&self, path: &Path);

    /// Очистить все diagnostics.
    pub fn clear(&self);
}
```

### check_file

```rust
fn check_file(&self, path: &Path, content: &str, vault_path: &Path) {
    let parse_result = parse_content(content);

    let mut diagnostics: Vec<Diagnostic> = parse_result.errors.into_iter().map(|e| Diagnostic {
        span: e.span,
        message: e.message,
        severity: Severity::Warning,
    }).collect();

    // Broken link — проверка существования файла
    for link in &parse_result.links {
        let full_path = vault_path.join(&link.file_name).with_extension("md");
        if !full_path.exists() {
            diagnostics.push(Diagnostic {
                span: link.span,
                message: format!("Broken link: {} — file not found", link.file_name),
                severity: Severity::Warning,
            });
        }
    }

    self.file_diagnostics.insert(path.to_path_buf(), diagnostics);
}
```

## Жизненный цикл

1. При открытии vault — `reindex_all()` вызывает `check_file()` для каждого файла
2. При каждом `save_document()` — `reindex_file()` вызывает `check_file()` для одного файла
3. При удалении файла — `remove()` вызывается watcher'ом
4. GUI читает diagnostics при перерисовке
