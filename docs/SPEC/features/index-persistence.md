---
priority: P0
layer: core
depends: [tag-index, date-index, link-index]
---

- [x]

# Index Persistence

Сохранение и загрузка индекса на диск в директорию `.index/` внутри vault.

## Формат хранения

```
.vault/
└── .index/
    ├── tags.json       # тег → [{path, spans}]
    ├── dates.json      # дата → [{path, spans}]
    ├── links.json      # файл → [{source, target, label, span}]
    └── metadata.json   # Версия формата, время перестройки
```

**Diagnostics не сохраняется** — при каждой загрузке перестраивается через `reindex_all()` или `reindex_file()`. Это осознанное решение: diagnostics дёшевы (один парсинг + проверка `path.exists()` на каждый файл) и всегда свежие при старте.

### tags.json

```json
[
  ["project", [{"path": "notes/project.md", "spans": [{"offset": 15, "length": 8}, {"offset": 120, "length": 8}]}]],
  ["todo",    [{"path": "notes/project.md", "spans": [{"offset": 45, "length": 5}]},
               {"path": "notes/ideas.md",   "spans": [{"offset": 10, "length": 5}, {"offset": 88, "length": 5}]}]]
]
```

Формат: массив пар `[тег, [TagEntry]]`. Каждый `TagEntry` — объект с путём и списком `ByteSpan`.

### dates.json

```json
[
  ["2024-01-15", [{"path": "notes/meeting.md", "spans": [{"offset": 0, "length": 10}]}]],
  ["2024-03-20", [{"path": "notes/ideas.md",   "spans": [{"offset": 5, "length": 10}, {"offset": 200, "length": 10}]}]]
]
```

Формат: массив пар `[дата (ISO), [DateEntry]]`.

### links.json

```json
[
  ["notes/meeting.md", [{"source": "notes/project.md", "target": "notes/meeting.md", "label": "Meeting", "span": {"offset": 10, "length": 18}}]],
  ["notes/ideas.md",   [{"source": "notes/project.md", "target": "notes/ideas.md", "label": "Ideas", "span": {"offset": 40, "length": 15}}]]
]
```

Формат: массив пар `[target, [LinkEntry]]`. Каждый `LinkEntry` — объект с source, target, label и ByteSpan.

### metadata.json

```json
{"version": 1, "last_rebuild": "2026-05-31T12:00:00Z"}
```

## API

```rust
impl ConcurrentIndex {
    pub fn save(&self, path: &Path) -> Result<()>;
    pub fn load(path: &Path) -> Result<Self>;
}
```

## Процесс

**Сохранение:**
1. Создать `.index/` если не существует
2. Сериализовать TagIndex в `tags.json` (Vec<(String, Vec<TagEntry>)>)
3. Сериализовать DateIndex в `dates.json` (Vec<(NaiveDate, Vec<DateEntry>)>)
4. Сериализовать LinkIndex в `links.json` (Vec<(PathBuf, Vec<LinkEntry>)>)
5. Записать `metadata.json`

**Загрузка:**
1. Проверить что `.index/` существует
2. Прочитать `metadata.json` для проверки версии
3. Загрузить `tags.json`, `dates.json`, `links.json`

## Когда сохраняется

- После полной перестройки индекса при открытии vault
- При каждом `save_buffer()` — сразу после переиндексации файла

## Когда загружается

- При открытии vault: если `.index` существует и актуален — загружаем

## Версионирование

`metadata.version` позволяет в будущем менять формат индекса. При несовпадении версии индекс перестраивается с нуля.
