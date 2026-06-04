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
    ├── tags.json           # тег → [{path, spans}]
    ├── dates.json          # дата → [{path, spans}]
    ├── links.json          # файл → [{source, target, label, span}]
    ├── file-hashes.json    # (inode, size, mtime) для каждого .md файла
    └── metadata.json       # Версия формата, время перестройки
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

### file-hashes.json

```json
[
  {"path": "notes/foo.md", "inode": 12345, "size": 1024, "mtime": 1717516800},
  {"path": "notes/bar.md", "inode": 12346, "size": 512, "mtime": 1717516900}
]
```

Формат: массив `FileHashEntry`. Каждый содержит:

| Поле | Тип | Описание |
|------|-----|----------|
| `path` | `PathBuf` | Относительный путь от корня vault |
| `inode` | `u64` | Номер inode файла (через `std::fs::Metadata`) |
| `size` | `u64` | Размер файла в байтах |
| `mtime` | `u64` | Unix timestamp последней модификации (секунды) |

Файл перезаписывается целиком при каждом `save()`.

### metadata.json

```json
{"version": 2, "last_rebuild": "2026-05-31T12:00:00Z"}
```

### links.json

```json
[
  {"source": "notes/project.md", "target": "notes/meeting.md", "label": "Meeting", "span": {"offset": 10, "length": 18}},
  {"source": "notes/project.md", "target": "notes/ideas.md", "label": "Ideas", "span": {"offset": 40, "length": 15}}
]
```

Формат: плоский массив `LinkEntry`. Каждый `LinkEntry` — объект с source (полный путь), target (полный путь), label и ByteSpan.
Ранее хранилось как `Vec<(PathBuf, Vec<LinkEntry>)>` (группировка по target), теперь плоский список для упрощения save/load.

## API

```rust
impl ConcurrentIndex {
    pub fn save(&self, path: &Path) -> Result<()>;
    pub fn load(path: &Path) -> Result<Self>;
    pub fn set_file_hashes(&self, hashes: Vec<FileHashEntry>);  // для Vault
}
```

## Процесс

**Сохранение:**
1. Создать `.index/` если не существует
2. Сериализовать TagIndex в `tags.json` (Vec<(String, Vec<TagEntry>)>)
3. Сериализовать DateIndex в `dates.json` (Vec<(NaiveDate, Vec<DateEntry>)>)
4. Сериализовать LinkIndex в `links.json` (flat Vec<LinkEntry>)
5. Сериализовать `file_hashes` в `file-hashes.json` (Vec<FileHashEntry>)
6. Записать `metadata.json`

**Загрузка:**
1. Проверить что `.index/` существует
2. Прочитать `metadata.json` для проверки версии
3. Загрузить `tags.json`, `dates.json`, `links.json`
4. Загрузить `file-hashes.json` (если существует)

## Когда сохраняется

- После полной перестройки индекса при открытии vault
- При каждом `save_buffer()` — сразу после переиндексации файла

## Когда загружается

- При открытии vault: если `.index` существует и актуален — загружаем данные индекса и `file-hashes.json`

## Инкрементальный ребилд (будущая оптимизация)

`file-hashes.json` сохраняется при каждом `save()`, но **пока не используется для оптимизации загрузки**. На данный момент `Vault::open()` всегда выполняет полный реиндекс всех `.md` файлов.

В будущем при `Vault::open()`:

1. Загрузить `file-hashes.json` с диска
2. Пройти `walkdir` по всем `.md` файлам, для каждого сделать `stat()`
3. Сравнить `(inode, size, mtime)` с сохранёнными значениями:
   - Совпало → файл не изменился, не индексировать
   - Не совпало → прочитать файл, вызвать `reindex_file()`, обновить хэш
4. Файлы, которые были в `file-hashes.json` но пропали из `walkdir` → `index.remove_file()` + удалить из хэшей
5. Новые файлы (есть в `walkdir`, нет в `file-hashes.json`) → `reindex_file()` + добавить в хэши
6. Сохранить обновлённый `file-hashes.json` и перезаписать `tags.json`/`dates.json`/`links.json`

**Триггер полного реиндекса:**
- `metadata.version` не совпал
- `file-hashes.json` не существует
- Пользователь явно вызвал `reindex_all()`

**Композитный хэш `(inode, size, mtime)` гарантирует:**
- `inode` — файл пересоздан (даже если содержимое то же)
- `size` — изменился размер содержимого
- `mtime` — изменилась дата модификации
- `touch` без изменения содержимого → ложное срабатывание (безвредно: лишняя переиндексация одного файла)

## Версионирование

`metadata.version` позволяет в будущем менять формат индекса. При несовпадении версии индекс перестраивается с нуля.

Добавление `file-hashes.json` не меняет версию формата — это вспомогательные данные, не влияющие на корректность загрузки.
