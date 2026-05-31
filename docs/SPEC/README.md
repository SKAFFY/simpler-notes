# Simpler Notes — Спецификации

Документация проекта **Simpler Notes** — нативное десктоп приложение для заметок на Rust (gpui).

## Структура

```
docs/SPEC/
├── README.md                  # Навигация (этот файл)
├── roadmap.md                 # Приоритеты и порядок реализации
├── features/                  # Функциональные спецификации
│   ├── parser.md              # Парсинг MD: [[ссылки]], @теги, даты
│   ├── note-model.md          # Модель данных заметки
│   ├── tag-index.md           # Индекс по тегам
│   ├── date-index.md          # Индекс по датам
│   ├── link-index.md          # Индекс обратных ссылок
│   ├── diagnostics.md         # Diagnostics: ошибки и предупреждения
│   ├── index-persistence.md   # Сохранение индекса в .index/
│   ├── query-language.md      # Язык поисковых запросов (гибрид: индекс + ripgrep)
│   ├── autocomplete.md         # Автокомплит (общий компонент)
│   ├── vault.md               # Vault — оркестратор заметок
│   ├── watcher.md             # File system watcher
│   ├── git-sync.md            # Git синхронизация
│   ├── mcp-server.md          # MCP сервер (JSON-RPC)
│   ├── workspace-layout.md    # Workspace GUI layout
│   ├── editor-source.md       # Source редактор
│   ├── editor-preview.md      # Source (gpui::Editor) и Preview режимы
│   ├── lower-panel-search.md    # Поиск по query language (нижняя панель)
│   ├── file-tree.md           # Файловое дерево
│   ├── open-vault-dialog.md   # Диалог открытия vault
│   ├── timeline.md            # Таймлайн
│   ├── graph-view.md          # MindMap / граф связей
│   └── settings.md            # Настройки приложения
└── api/
    ├── core.md                # Публичный API simpler-notes-core
    └── mcp.md                 # MCP инструменты (JSON-RPC методы)
```

## Связанные документы

- `DESIGN-DOCUMENT.md` — концепт и хотелки
- `docs/architecture-review.md` — архитектурные решения
- `docs/superpowers/plans/` — планы реализации
