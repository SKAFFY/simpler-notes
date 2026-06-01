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
| `[[note2]]` — 2+ файла с именем `note2.md` | `Ambiguous link: note2 — multiple files: note2.md, sub/note2.md` | Warning |

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
fn check_file(&self, path: &Path, content: &str, vault_path: &Path,
              filename_index: &HashMap<String, Vec<PathBuf>>) {
    let parse_result = parse_content(content);

    let mut diagnostics: Vec<Diagnostic> = parse_result.errors.into_iter().map(|e| Diagnostic {
        span: e.span,
        message: e.message,
        severity: Severity::Warning,
    }).collect();

    // Broken link / Ambiguous link — проверка через filename_index
    for link in &parse_result.links {
        let raw_target = PathBuf::from(&link.file_name);
        let resolved = if raw_target.is_absolute() {
            raw_target
        } else {
            path.parent().unwrap_or(Path::new("")).join(&raw_target)
        };
        let normalized = normalize_path(&resolved);
        let link_name = normalized
            .file_stem()
            .unwrap_or(normalized.as_os_str())
            .to_string_lossy()
            .to_string();

        match filename_index.get(&link_name) {
            None => {
                diagnostics.push(Diagnostic {
                    span: link.span,
                    message: format!("Broken link: {} — file not found", link.file_name),
                    severity: Severity::Warning,
                });
            }
            Some(files) if files.len() > 1 => {
                let file_list = files.iter()
                    .map(|f| f.to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                diagnostics.push(Diagnostic {
                    span: link.span,
                    message: format!("Ambiguous link: {} — multiple files: {}", link_name, file_list),
                    severity: Severity::Warning,
                });
            }
            _ => {} // exactly 1 match — OK
        }
    }

    self.file_diagnostics.insert(path.to_path_buf(), diagnostics);
}
```

`filename_index` — маппинг `file_stem → Vec<PathBuf>`, строится Vault'ом на основе списка .md файлов.

Логика:
1. Link target резолвится и нормализуется (resolve относительного пути + normalize_path)
2. Из нормализованного пути берётся `file_stem()`
3. По file_stem ищем в filename_index:
   - **0** вхождений → `BrokenLink` (файл не существует)
   - **1** вхождение → OK (ровно один файл)
   - **2+** вхождений → `AmbiguousLink` (коллизия имён)

## Жизненный цикл

1. При открытии vault — `reindex_all()` вызывает `check_file()` для каждого файла
2. При каждом `save_buffer()` — `reindex_file()` вызывает `check_file()` для одного файла
3. При удалении файла — `remove()` вызывается watcher'ом
4. GUI читает diagnostics при перерисовке
