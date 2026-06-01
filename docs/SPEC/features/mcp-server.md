---
priority: P1
layer: mcp
depends: [vault]
---

- [x]

# MCP Server

Headless-сервер для доступа к vault через MCP (Model Context Protocol) от Anthropic.

## Запуск

```bash
simpler-notes-mcp --vault /path/to/notes [--git]
```

## Транспорт

- **Протокол:** JSON-RPC 2.0
- **Транспорт:** stdio (stdin/stdout)
- **Кодировка:** UTF-8
- **Разделитель:** `\n` (построчно)

## Жизненный цикл

### 1. Initialize
Агент отправляет `initialize`, сервер отвечает со своими capabilities.

### 2. Initialized
Агент отправляет `notifications/initialized` — готов к работе.

### 3. Инструменты
Агент вызывает `tools/list` → получает список инструментов.
Агент вызывает `tools/call` с именем инструмента и аргументами.

### 4. Завершение
Закрытие stdin → сервер завершает работу.

## Инструменты

| Инструмент | Аргументы | Описание |
|-----------|-----------|----------|
| `search_notes` | `query: string` | Поиск по query language |
| `read_note` | `path: string` | Содержимое файла |
| `write_note` | `path: string, content: string` | Полная перезапись файла |
| `list_notes` | `path?: string` | Дерево файлов |
| `get_tags` | — | Все теги с количеством файлов |
| `get_dates` | `from?: string, to?: string` | Все даты или в диапазоне |
| `get_backlinks` | `path: string` | Какие файлы ссылаются на path |
| `get_outgoing_links` | `path: string` | Куда ссылается path |
| `resolve_link` | `target: string` | Разрешить плоское имя ссылки в полный путь к файлу |
| `git_push` | — | Stage, commit (если dirty), squash (если >1 unpushed), pull rebase, push |
| `git_pull` | — | Pull из remote |
| `validate_indexes` | — | Проверка целостности |
| `reindex` | — | Полная перестройка индекса |
| `get_diagnostics` | `path?: string` | Diagnostics для одного или всех файлов |

## Обработка ошибок

- Все ошибки возвращаются через JSON-RPC error объект
- Код ошибки: `-32000` (application error)
- Сообщение: текст ошибки на английском

## Рекомендации для разработчиков агентов

- `write_note` выполняет полную перезапись файла. Перед записью всегда читайте содержимое через `read_note`, вносите изменения в памяти, затем пишите целиком.
- `get_dates` без аргументов возвращает все даты. Используйте `from`/`to` для фильтрации, если нужен только диапазон.
