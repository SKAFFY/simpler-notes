---
priority: P1
layer: core
depends: [vault, settings]
---

- [x]

# Git Sync

Автоматическая синхронизация vault с git remote.

## Структура

```rust
pub struct GitBackend {
    repo_path: PathBuf,
    timer_handle: Option<JoinHandle<()>>,
}
```

## Поведение

### Автокоммит по таймеру

- Таймер запускается при открытии vault
- Интервал: `git_auto_commit_interval_minutes` из настроек (по умолчанию 10 минут)
- При каждом тике:
  1. `git add -A` (stage все изменения)
  2. Если есть staged → `git commit -m "Auto-commit {timestamp}"`
  3. Если нет staged → ничего не делать
- Таймер не привязан к autosave или explicit save — работает независимо

### Push

При `git_push()`:
1. Stage все изменения (`git add -A`)
2. Если есть staged → commit с сообщением `"sync: manual push"`
3. Если есть unpushed commits (больше 1) → squash в один:
   - `git reset --soft <merge-base с remote>`
   - `git commit -m "sync: manual push"`
4. `git pull --rebase`
5. `git push`
6. Успех или ошибка → результат

### Pull

- `git pull --rebase`
- Если есть конфликты → ошибка (пользователь решает вручную)

## API

```rust
impl GitBackend {
    /// Открыть git репозиторий. Если `auto_commit` включён — запустить таймер.
    pub fn open(path: &Path, auto_commit: bool, interval_minutes: u64) -> Result<Self, String>;

    /// Остановить таймер (при закрытии vault).
    pub fn close(&self);

    /// Push с предварительным commit (если есть dirty) и squash (если >1 unpushed).
    pub fn push(&self) -> Result<(), String>;

    /// Pull с rebase.
    pub fn pull(&self) -> Result<(), String>;

    /// Статус репозитория.
    pub fn status(&self) -> Result<String, String>;

    /// Количество unpushed commits относительно `@{u}`.
    pub fn unpushed_count(&self) -> Result<usize, String>;
}
```

## Реализация

### squash

```rust
fn squash_to_one(&self, message: &str) -> Result<(), String> {
    let base = self.get_merge_base("@{u}")?;
    self.git(&["reset", "--soft", &base])?;
    self.git(&["commit", "-m", message])?;
    Ok(())
}

fn get_merge_base(&self, target: &str) -> Result<String, String> {
    let output = self.git(&["merge-base", "HEAD", target])?;
    Ok(output.trim().to_string())
}
```

### timer

```rust
fn start_timer(&self, interval: Duration) {
    let backend = self.clone();
    self.timer_handle = Some(std::thread::spawn(move || {
        loop {
            thread::sleep(interval);
            let _ = backend.auto_commit();
        }
    }));
}
```

### auto_commit

```rust
fn auto_commit(&self) -> Result<(), String> {
    self.git(&["add", "-A"])?;
    let status = self.git(&["status", "--porcelain"])?;
    if status.is_empty() {
        return Ok(());  // ничего не изменилось
    }
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
    self.git(&["commit", "-m", &format!("Auto-commit {}", timestamp)])?;
    Ok(())
}
```

## Зависимости

- Крейт `git2` (libgit2) или вызов CLI-команд через `std::process::Command`
- `chrono` для таймстампов в сообщениях коммитов
