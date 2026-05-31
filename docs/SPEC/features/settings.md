---
priority: P2
layer: gui
depends: [vault]
---

- [x]

# Settings

Настройки приложения. Хранятся в стандартной директории конфигурации ОС.

## Путь

- **macOS:** `~/Library/Application Support/simpler-notes/settings.json`
- **Linux:** `~/.config/simpler-notes/settings.json`
- **Windows:** `%APPDATA%/simpler-notes/settings.json`

## Формат

```json
{
  "last_vault_path": "/home/user/notes",
  "project_panel_visible": true,
  "project_panel_width": 250,
  "window_bounds": {"x": 100, "y": 100, "width": 1024, "height": 768},
  "theme": "dark",
  "git_auto_commit": true,
  "git_auto_commit_interval_minutes": 10,
  "autosave_interval_secs": 30
}
```

## Структура Rust

```rust
pub struct Settings {
    pub last_vault_path: Option<PathBuf>,
    pub recent_vaults: Vec<PathBuf>,
    pub project_panel_visible: bool,
    pub project_panel_width: f32,
    pub window_bounds: Option<Bounds>,
    pub theme: Theme,
    pub git_auto_commit: bool,
    pub git_auto_commit_interval_minutes: u64,
    pub autosave_interval_secs: u64,
}
```

## API

- `Settings::load()` — загрузить из конфигурационной директории (или дефолт)
- `Settings::save(&self)` — сохранить
- `Settings::update_vault(path)` — обновить last_vault_path и recent_vaults

## Поведение

- При первом запуске — дефолтные настройки
- При сохранении — файл перезаписывается
- Валидация не требуется (значения по умолчанию безопасны)
- `autosave_interval_secs` (по умолчанию 30) — интервал автосохранения документа. Хранится в `Document`.
- `git_auto_commit_interval_minutes` (по умолчанию 10) — интервал авто-коммита в git. Передаётся в `GitBackend::open()`.
