---
priority: P1
layer: core
depends: [vault]
---

- [x]

# File Watcher

Отслеживает изменения файлов в vault через крейт `notify`. Работает в фоновом потоке.

## Архитектура

```
notify (kqueue/inotify)
  → mpsc канал
  → FileWatcher.receiver (FileEvent)
  → Vault.reindex_file()
    → инкрементальное обновление индекса и diagnostics
```

## Структура

```rust
pub struct FileWatcher {
    pub receiver: mpsc::Receiver<FileEvent>,
    _handle: thread::JoinHandle<()>,
}

pub enum FileEvent {
    Created(String),
    Modified(String),
    Deleted(String),
}
```

## Правила

- Отслеживаются все файлы рекурсивно
- **Игнорируются:** `.git/`, `.index/`
- Debounce: события группируются (300ms) — если за 300мс пришли новые события, таймер сбрасывается
- После debounce: триггер на переиндексацию изменённых файлов

## Типы событий

| Событие | Действие |
|---------|----------|
| `Created` | Вызвать `vault.reindex_file(path, content)` |
| `Modified` | Вызвать `vault.reindex_file(path, content)` |
| `Deleted` | Вызвать `vault.reindex_file(path, None)` |

## Инкрементальное обновление

При получении события:
1. Прочитать файл (если не Deleted)
2. Вызвать `vault.reindex_file(path, content)` — он сам обновляет теги, даты, ссылки, diagnostics
3. Не перестраивать весь индекс

## Зависимости

- Крейт `notify` (v7)
- `mpsc` канал для коммуникации между watcher потоком и Vault
