# MCP API

Инструменты, доступные через MCP протокол. JSON-RPC 2.0 через stdio.

## Инструменты

### `search_notes`

Поиск заметок по query language.

**Arguments:**
```json
{"query": "tags contain \"project\" and date before 01.01.2025"}
```

**Result:**
```json
[
  {"path": "notes/project.md", "title": "My Project"},
  {"path": "notes/old.md", "title": "Old Note"}
]
```

### `read_note`

Чтение содержимого файла.

**Arguments:**
```json
{"path": "notes/project.md"}
```

**Result:**
```json
{"content": "# My Project\n\nSome content..."}
```

### `write_note`

Создание или обновление заметки.

**Arguments:**
```json
{"path": "notes/new.md", "content": "# New Note\n\nHello world"}
```

**Result:**
```json
{"ok": true}
```

### `list_notes`

Дерево файлов в vault.

**Arguments (опционально):**
```json
{"path": "subdir/"}
```

**Result:**
```json
[
  {"name": "notes", "type": "directory"},
  {"name": "project.md", "type": "file"},
  {"name": "subdir", "type": "directory"}
]
```

### `get_tags`

Все теги с количеством файлов (не вхождений).

**Arguments:** нет

**Result:**
```json
[
  {"tag": "project", "count": 5},
  {"tag": "todo", "count": 3}
]
```

### `get_dates`

Все даты. Опционально — фильтр по диапазону.

**Arguments:**
```json
{}  // все даты
// или
{"from": "01.01.2024", "to": "01.06.2024"}
```

**Result:**
```json
[
  {"date": "2024-01-15", "notes": ["notes/meeting.md"]}
]
```

### `git_push`

Stage все изменения, commit (если dirty), squash unpushed commits в один, pull rebase, push.

**Arguments:** нет

**Result:**
```json
{"ok": true}
```

Поведение:
1. `git add -A`
2. Если есть staged → `git commit -m "sync: manual push"`
3. Если `unpushed_count > 1` → `git reset --soft @{u}` + `git commit -m "sync: manual push"`
4. `git pull --rebase`
5. `git push`

### `git_pull`

Pull из remote.

**Arguments:** нет

**Result:**
```json
{"ok": true}
```

### `validate_indexes`

Проверка целостности индексов.

**Arguments:** нет

**Result:**
```json
{
  "total_notes": 42,
  "total_tags": 15,
  "total_dates": 30
}
```

### `reindex`

Полная перестройка индекса (очистить, перепарсить все файлы, сохранить).

**Arguments:** нет

**Result:**
```json
{
  "ok": true,
  "total_notes": 42,
  "total_tags": 15,
  "total_dates": 30
}
```

### `get_diagnostics`

Вернуть warnings для одного или всех файлов.

**Arguments:**
```json
{}  // все файлы
// или
{"path": "notes/project.md"}  // один файл
```

**Result (все файлы):**
```json
{
  "files": [
    {
      "path": "notes/project.md",
      "diagnostics": [
        {"span": {"offset": 0, "length": 2}, "message": "Empty link", "severity": "warning"},
        {"span": {"offset": 100, "length": 15}, "message": "Broken link: NonExistent — file not found", "severity": "warning"}
      ]
    }
  ]
}
```

**Result (один файл):**
```json
{
  "diagnostics": []
}
```

### `get_backlinks`

Какие файлы ссылаются на указанную заметку.

**Arguments:**
```json
{"path": "notes/project.md"}
```

**Result:**
```json
[
  {"source": "notes/reference.md", "target": "project", "label": "My Project"},
  {"source": "notes/index.md", "target": "project", "label": "project"}
]
```

`target` — плоское имя файла (file_stem) без расширения и без пути.

### `resolve_link`

Разрешить плоское имя ссылки (file_stem) в полный путь к файлу на диске.

**Arguments:**
```json
{"target": "beta"}
```

**Result (success):**
```json
{"path": "/vault/notes/beta.md"}
```

**Result (broken link):**
```json
{"error": {"code": -32000, "message": "Broken link: ghost — file not found"}}
```

**Result (ambiguous link):**
```json
{"error": {"code": -32000, "message": "Ambiguous link: note — multiple files: /vault/note.md, /vault/sub/note.md"}}
```

### `get_outgoing_links`

На какие файлы ссылается указанная заметка.

**Arguments:**
```json
{"path": "notes/source.md"}
```

**Result:**
```json
[
  {"source": "notes/source.md", "target": "project", "label": "My Project"},
  {"source": "notes/source.md", "target": "todo", "label": "todo"}
]
```
