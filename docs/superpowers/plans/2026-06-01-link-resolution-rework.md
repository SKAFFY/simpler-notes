# Link Resolution Rework — Flat Names, Ambiguous Links, Утилита normalize_path

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Перевести логику ссылок с относительных путей на плоские имена (filename без расширения), добавить проверку коллизий имён, вынести `normalize_path` в утилиту, написать полные табличные тесты.

**Architecture:**
1. `LinkEntry.target` — всегда только имя файла без расширения (`note2`, а не `sub/note2` или `../note2`)
2. `Diagnostics::check_file` получает маппинг `file_stem → [full_paths]` для детекции:
   - 0 совпадений → `BrokenLink`
   - 1 совпадение → OK
   - 2+ совпадения → новая диагностика `AmbiguousLink`
3. `normalize_path` выносится из дубликатов в `src/util.rs`
4. Все тесты переписываются на table-driven

**Затрагиваемые файлы:**
- `crates/simpler-notes-core/src/util.rs` — новый
- `crates/simpler-notes-core/src/index/mod.rs` — flatten target, normalize_path → util
- `crates/simpler-notes-core/src/diagnostics.rs` — AmbiguousLink, normalize_path → util, маппинг имён
- `crates/simpler-notes-core/src/parser.rs` — без изменений
- `crates/simpler-notes-core/src/search.rs` — fix link: query
- `crates/simpler-notes-core/src/vault.rs` — build_filename_index, передача маппинга в reindex_file/diagnostics
- `crates/simpler-notes-core/src/lib.rs` — добавить mod util
- `crates/simpler-notes-mcp/src/tools/mod.rs` — register get_backlinks, get_outgoing_links
- `crates/simpler-notes-mcp/src/tools/get_backlinks.rs` — новый
- `crates/simpler-notes-mcp/src/tools/get_outgoing_links.rs` — новый
- `docs/SPEC/features/parser.md` — обновить
- `docs/SPEC/features/link-index.md` — обновить
- `docs/SPEC/features/diagnostics.md` — обновить
- `docs/SPEC/features/vault.md` — обновить
- `docs/SPEC/features/mcp-server.md` — обновить
- `docs/SPEC/api/core.md` — обновить
- `docs/SPEC/api/mcp.md` — обновить

---

### Task 0: Обновить SPEC документацию

**Files:** docs/SPEC/features/*.md, docs/SPEC/api/*.md

- [ ] **Step 1: Обновить `docs/SPEC/features/link-index.md`**
  - `LinkEntry.target` — только имя файла (file_stem) без расширения и без пути
  - Убрать упоминание относительных путей в target

- [ ] **Step 2: Обновить `docs/SPEC/features/diagnostics.md`**
  - Новая диагностика: `AmbiguousLink` — `Ambiguous link: note2 — multiple files: note2.md, sub/note2.md`
  - `check_file` принимает `file_names: &HashMap<String, Vec<PathBuf>>` (маппинг file_stem → пути)
  - Логика: для каждого link target → поиск по маппингу → 0 = BrokenLink, 1 = OK, >1 = AmbiguousLink

- [ ] **Step 3: Обновить `docs/SPEC/features/vault.md`**
  - `reindex_file` → flatten target до file_stem
  - `Vault` строит `filename_index: HashMap<String, Vec<PathBuf>>` перед вызовом `reindex_file`/`check_file`

- [ ] **Step 4: Обновить `docs/SPEC/features/mcp-server.md`**
  - Добавить `get_backlinks(path)` и `get_outgoing_links(path)` в таблицу инструментов

- [ ] **Step 5: Обновить `docs/SPEC/api/core.md`**
  - Синхронизировать сигнатуры: `Diagnostics::check_file` с `file_names`, `LinkEntry.target` = file_stem

- [ ] **Step 6: Обновить `docs/SPEC/api/mcp.md`**
  - Добавить документацию для `get_backlinks` и `get_outgoing_links`

---

### Task 1: Core — вынести normalize_path в util.rs

**Files:**
- Create: `crates/simpler-notes-core/src/util.rs`
- Modify: `crates/simpler-notes-core/src/lib.rs`
- Modify: `crates/simpler-notes-core/src/index/mod.rs`
- Modify: `crates/simpler-notes-core/src/diagnostics.rs`

- [ ] **Step 1: Создать `src/util.rs`**
  ```rust
  use std::path::{Path, PathBuf, Component};

  pub fn normalize_path(path: &Path) -> PathBuf {
      let mut components = Vec::new();
      for component in path.components() {
          match component {
              Component::CurDir => continue,
              Component::ParentDir => { components.pop(); }
              other => components.push(other),
          }
      }
      components.iter().collect()
  }

  #[cfg(test)]
  mod tests {
      use super::*;
      // test cases
  }
  ```

- [ ] **Step 2: Добавить `pub mod util;` в `src/lib.rs`**

- [ ] **Step 3: В `src/index/mod.rs` — удалить `normalize_path`, импортировать из `crate::util::normalize_path`**

- [ ] **Step 4: В `src/diagnostics.rs` — удалить `normalize_path`, импортировать из `crate::util::normalize_path`**

- [ ] **Step 5: Запустить тесты**
  ```bash
  cargo test -p simpler-notes-core
  ```

- [ ] **Step 6: Commit**

---

### Task 2: Core — flatten target в reindex_file

**Files:**
- Modify: `crates/simpler-notes-core/src/index/mod.rs`

- [ ] **Step 1: Изменить `ConcurrentIndex::reindex_file` — flatten target до file_stem**
  После normalize_path взять `.file_stem()`.

- [ ] **Step 2: Запустить тесты**

- [ ] **Step 3: Commit**

---

### Task 3: Core — AmbiguousLink в diagnostics + filename_index

**Files:**
- Modify: `crates/simpler-notes-core/src/diagnostics.rs`
- Modify: `crates/simpler-notes-core/src/vault.rs`
- Modify: `crates/simpler-notes-core/src/index/mod.rs`

- [ ] **Step 1: Добавить логику AmbiguousLink в Diagnostics::check_file**
  - Принимает `filename_index: &HashMap<String, Vec<PathBuf>>`
  - Для каждого link: flatten до file_stem → поиск в filename_index
  - 0 → BrokenLink, 1 → OK, >1 → AmbiguousLink

- [ ] **Step 2: В vault.rs построить filename_index и передавать в reindex_file**

- [ ] **Step 3: Обновить сигнатуру `ConcurrentIndex::reindex_file` — добавить параметр filename_index**

- [ ] **Step 4: Запустить тесты**

- [ ] **Step 5: Commit**

---

### Task 4: Core — fix search.rs для flat names

**Files:**
- Modify: `crates/simpler-notes-core/src/search.rs`

- [ ] **Step 1: Исправить `execute_query` для `QueryExpr::Link`**
  Заменить `vault_path.join(target).with_extension("md")` на `PathBuf::from(target)`.

- [ ] **Step 2: Запустить тесты**

- [ ] **Step 3: Commit**

---

### Task 5: MCP — добавить get_backlinks и get_outgoing_links

**Files:**
- Create: `crates/simpler-notes-mcp/src/tools/get_backlinks.rs`
- Create: `crates/simpler-notes-mcp/src/tools/get_outgoing_links.rs`
- Modify: `crates/simpler-notes-mcp/src/tools/mod.rs`

- [ ] **Step 1-3: Создать файлы и зарегистрировать в tools/mod.rs**

- [ ] **Step 4: Собрать и запустить тесты**

- [ ] **Step 5: Commit**

---

### Task 6: Core — табличные тесты

**Files:**
- Modify: `crates/simpler-notes-core/src/parser.rs`
- Modify: `crates/simpler-notes-core/src/index/link_index.rs`
- Modify: `crates/simpler-notes-core/src/index/mod.rs`
- Modify: `crates/simpler-notes-core/src/diagnostics.rs`
- Modify: `crates/simpler-notes-core/src/vault.rs`

- [ ] **Step 1: parser.rs — табличные тесты на parse_content**
- [ ] **Step 2: link_index.rs — табличные тесты**
- [ ] **Step 3: index/mod.rs — табличные тесты на reindex_file**
- [ ] **Step 4: diagnostics.rs — табличные тесты**
- [ ] **Step 5: vault.rs — табличные тесты**
- [ ] **Step 6: Запустить все тесты**
- [ ] **Step 7: Commit**

---

### Task 7: MCP e2e тесты

**Files:**
- Create: `crates/simpler-notes-mcp/tests/e2e.rs`
- Modify: `crates/simpler-notes-mcp/Cargo.toml`

- [ ] **Step 1: Добавить dev-dependencies**
- [ ] **Step 2: Создать tests/e2e.rs с McpClient и 11+ сценариями**
- [ ] **Step 3: Запустить e2e тесты**
- [ ] **Step 4: Commit**
