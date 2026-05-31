# Roadmap

Приоритеты и порядок реализации фич. P0 — обязательно для MVP, P1 — нужно для базовой работы, P2 — полировка.

## Core Engine (P0)

| Фича | Уровень | Приоритет | Зависимости |
|------|---------|-----------|-------------|
| Парсер | core | P0 | — |
| Note model | core | P0 | — |
| Tag index | core | P0 | Парсер |
| Date index | core | P0 | Парсер |
| Link index | core | P0 | Парсер |
| Diagnostics | core | P0 | Парсер |
| Index persistence | core | P0 | Tag Index, Date Index, Link Index |
| Document | core | P0 | Note Model, Парсер |
| Query language | core | P0 | — |
| Vault | core | P0 | Парсер, Note Model, Document, Tag/Date/Link Index, Diagnostics, Index persistence, Query Language |

## Core Engine (P1)

| Фича | Уровень | Приоритет | Зависимости |
|------|---------|-----------|-------------|
| File watcher | core | P1 | Vault |
| Git sync | core | P1 | Vault, Settings |

## MCP Server

| Фича | Уровень | Приоритет | Зависимости |
|------|---------|-----------|-------------|
| MCP сервер и инструменты | mcp | P1 | Vault |

## GUI — Базовый интерфейс (P1)

| Фича | Уровень | Приоритет | Зависимости |
|------|---------|-----------|-------------|
| Workspace layout | gui | P1 | Vault |
| AppState model | gui | P1 | — |
| File tree | gui | P1 | Workspace layout |
| Source editor | gui | P1 | Workspace layout |
| Preview editor | gui | P1 | Workspace layout, Парсер |
| Переключение Source / Split / Preview | gui | P1 | Workspace layout |
| Вкладки (tabs) | gui | P1 | Workspace layout |
| Поиск в сайдбаре | gui | P1 | File tree |
| Open vault dialog | gui | P1 | Workspace layout |
| Первый запуск | gui | P1 | Workspace layout |
| Навигация по [[link]] | gui | P1 | Preview editor |

## GUI — Продвинутые вьюхи (P2)

| Фича | Уровень | Приоритет | Зависимости |
|------|---------|-----------|-------------|
| Timeline | gui | P2 | Vault |
| Graph View (MindMap) | gui | P2 | Vault |
| Quick Open (Cmd+P) | gui | P2 | Vault |
| Completion popup [[ и # | gui | P2 | Source editor |

## Полировка (P2)

| Фича | Уровень | Приоритет | Зависимости |
|------|---------|-----------|-------------|
| Настройки приложения | gui | P2 | Vault |
| Сохранение состояния окна | gui | P2 | Настройки |
| Подсветка @тегов и дат | gui | P2 | Source editor |
| Resize project panel | gui | P2 | Workspace layout |
| Drag to reorder вкладок | gui | P2 | Вкладки |
