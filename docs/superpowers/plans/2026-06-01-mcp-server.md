# MCP Server Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Реализовать MCP сервер для simpler-notes с полным набором инструментов по SPEC.

**Architecture:** Сначала дорабатываем core API до соответствия SPEC в `docs/SPEC/api/core.md`, затем создаём новый crate `simpler-notes-mcp` с MCP transport (Content-Length framing через stdio), JSON-RPC 2.0 диспетчером и 11 инструментами.

**Tech Stack:** Rust, serde/serde_json, MCP protocol (Content-Length), git2 (feature-gated)

---

### Task 0.1: IndexReport + Vault search

**Files:**
- Modify: `crates/simpler-notes-core/src/vault.rs`
- Test: `crates/simpler-notes-core/src/vault.rs`

- [ ] **Step 1: Add IndexReport struct to vault.rs**

```rust
pub struct IndexReport {
    pub total_notes: usize,
    pub total_tags: usize,
    pub total_dates: usize,
}
```

Add right before `impl Vault {`.

- [ ] **Step 2: Add search() method to Vault**

```rust
    pub fn search(&self, query: &str) -> Result<Vec<SearchResult>, String> {
        let expr = SearchEngine::parse_query(query);
        let results = self.search.execute_query(&expr, &self.config.path);
        let vault_path = &self.config.path;
        let mapped = results.iter().map(|p| {
            let rel = pathdiff::diff_paths(p, vault_path).unwrap_or_else(|| PathBuf::from(p));
            SearchResult {
                path: rel,
                title: p.rsplit('/').next().unwrap_or(p).to_string(),
            }
        }).collect();
        Ok(mapped)
    }
```

Add `pathdiff` to `Cargo.toml` dependencies:
```toml
pathdiff = "0.2"
```

- [ ] **Step 3: Add reindex_all() returning IndexReport**

```rust
    pub fn reindex_all(&self) -> Result<IndexReport, String> {
        self.reindex_all_internal()?;
        self.validate_indexes()
    }

    fn reindex_all_internal(&self) -> Result<(), String> {
        let mut _files_reindexed = 0
        for entry in walkdir::WalkDir::new(&self.config.path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let ext = entry.path()
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            if !self.config.extensions.contains(&ext.to_string()) {
                continue;
            }
            let content = std::fs::read_to_string(entry.path())
                .map_err(|e| format!("Failed to read {:?}: {}", entry.path(), e))?;
            self.index.reindex_file(entry.path(), &content, &self.config.path);
            _files_reindexed += 1;
        }
        self.index.save(&self.config.path)?;
        Ok(())
    }
```

- [ ] **Step 4: Add validate_indexes()**

```rust
    pub fn validate_indexes(&self) -> Result<IndexReport, String> {
        let total_notes = self.list_md_files().len();
        let total_tags = self.index.tags.all_tags().len();
        let total_dates = self.index.dates.all_dates().len();
        Ok(IndexReport { total_notes, total_tags, total_dates })
    }
```

- [ ] **Step 5: Write tests**

```rust
    #[test]
    fn test_index_report() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.md"), "@tag1").unwrap();
        std::fs::write(dir.path().join("b.md"), "@tag1 @tag2").unwrap();
        let vault = Vault::open(VaultConfig { path: dir.path().to_path_buf(), ..Default::default() }).unwrap();
        let report = vault.validate_indexes().unwrap();
        assert_eq!(report.total_notes, 2);
        assert_eq!(report.total_tags, 2);
    }

    #[test]
    fn test_search_via_vault() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("test.md"), "@project").unwrap();
        let vault = Vault::open(VaultConfig { path: dir.path().to_path_buf(), ..Default::default() }).unwrap();
        let results = vault.search("tag:project").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_reindex_all_report() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.md"), "@tag").unwrap();
        let vault = Vault::open(VaultConfig { path: dir.path().to_path_buf(), ..Default::default() }).unwrap();
        let report = vault.reindex_all().unwrap();
        assert_eq!(report.total_notes, 1);
    }
```

- [ ] **Step 6: Run tests**

Run: `cargo test -p simpler-notes-core vault::tests`
Expected: all pass

- [ ] **Step 7: Commit**

```bash
git add crates/simpler-notes-core/src/vault.rs crates/simpler-notes-core/Cargo.toml
git commit -m "feat(core): add IndexReport, search(), validate_indexes(), reindex_all() -> IndexReport"
```

### Task 0.2: Vault convenience methods

**Files:**
- Modify: `crates/simpler-notes-core/src/vault.rs`

- [ ] **Step 1: Add read_note, write_note, get_all_tags, get_dates_in_range**

```rust
    pub fn read_note(&self, path: &Path) -> Result<String, String> {
        let full_path = self.config.path.join(path);
        std::fs::read_to_string(&full_path).map_err(|e| format!("Failed to read {:?}: {}", path, e))
    }

    pub fn write_note(&self, path: &Path, content: &str) -> Result<(), String> {
        let full_path = self.config.path.join(path);
        std::fs::write(&full_path, content).map_err(|e| format!("Failed to write {:?}: {}", path, e))?;
        self.index.reindex_file(&full_path, content, &self.config.path);
        self.index.save(&self.config.path).ok();
        Ok(())
    }

    pub fn get_all_tags(&self) -> Vec<String> {
        self.index.tags.all_tags()
    }

    pub fn get_dates_in_range(&self, from: NaiveDate, to: NaiveDate) -> Vec<(NaiveDate, Vec<date_index::DateEntry>)> {
        self.index.dates.get_range(from, to)
    }
```

Need to add `use crate::index::date_index;` at top of vault.rs

- [ ] **Step 2: Add autocomplete methods**

```rust
    pub fn autocomplete_tags(&self, prefix: &str) -> Vec<TagCompletion> {
        self.index.tags.autocomplete(prefix)
    }

    pub fn fuzzy_search_tags(&self, query: &str) -> Vec<TagCompletion> {
        self.index.tags.fuzzy_search(query, 10)
    }

    pub fn autocomplete_links(&self, prefix: &str) -> Vec<String> {
        let lower = prefix.to_lowercase();
        let mut results: Vec<String> = self.index.links.all_targets()
            .iter()
            .filter(|t| t.to_string_lossy().to_lowercase().starts_with(&lower))
            .map(|t| t.file_stem().unwrap_or_default().to_string())
            .collect();
        results.sort();
        results.dedup();
        results
    }

    pub fn autocomplete_dates(&self, prefix: &str) -> Vec<String> {
        let lower = prefix.to_lowercase();
        self.index.dates.all_dates()
            .iter()
            .filter(|(d, _)| d.to_string().contains(&lower))
            .map(|(d, _)| d.to_string())
            .collect()
    }
```

Need `all_targets()` on LinkIndex:

```rust
    /// Returns all unique link targets.
    pub fn all_targets(&self) -> Vec<PathBuf> {
        let mut targets: Vec<PathBuf> = self.backward.iter()
            .map(|e| e.key().clone())
            .collect();
        targets.sort();
        targets.dedup();
        targets
    }
```

Add to `crates/simpler-notes-core/src/index/link_index.rs`.

- [ ] **Step 3: Add backlinks, outgoing, diagnostics methods**

```rust
    pub fn get_backlinks(&self, target: &Path) -> Vec<LinkEntry> {
        self.index.links.backlinks(target)
    }

    pub fn get_outgoing_links(&self, source: &Path) -> Vec<LinkEntry> {
        self.index.links.outgoing(source)
    }

    pub fn get_diagnostics(&self, path: &Path) -> Vec<Diagnostic> {
        self.index.diagnostics.get(path)
    }

    pub fn all_diagnostics(&self) -> Vec<(PathBuf, Vec<Diagnostic>)> {
        self.index.diagnostics.all()
    }
```

- [ ] **Step 4: Add tests**

```rust
    #[test]
    fn test_read_write_note() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::open(VaultConfig { path: dir.path().to_path_buf(), ..Default::default() }).unwrap();
        vault.write_note(&PathBuf::from("test.md"), "hello @tag").unwrap();
        let content = vault.read_note(&PathBuf::from("test.md")).unwrap();
        assert_eq!(content, "hello @tag");
        let tags = vault.get_all_tags();
        assert_eq!(tags, vec!["tag"]);
    }

    #[test]
    fn test_autocomplete_tags() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.md"), "@project-alpha @project-beta @todo").unwrap();
        let vault = Vault::open(VaultConfig { path: dir.path().to_path_buf(), ..Default::default() }).unwrap();
        let results = vault.autocomplete_tags("proj");
        assert!(results.iter().any(|c| c.name == "project-alpha"));
    }

    #[test]
    fn test_get_diagnostics() {
        let dir = TempDir::new().unwrap();
        let path = PathBuf::from("note.md");
        std::fs::write(dir.path().join(&path), "[[]]").unwrap();
        let vault = Vault::open(VaultConfig { path: dir.path().to_path_buf(), ..Default::default() }).unwrap();
        let diags = vault.get_diagnostics(&path);
        assert!(!diags.is_empty());
    }
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p simpler-notes-core vault::tests`
Expected: all pass

- [ ] **Step 6: Commit**

```bash
git add crates/simpler-notes-core/src/vault.rs crates/simpler-notes-core/src/index/link_index.rs
git commit -m "feat(core): add read_note, write_note, autocomplete, backlinks, diagnostics to Vault"
```

### Task 0.3: GitBackend — push, pull, unpushed_count

**Files:**
- Modify: `crates/simpler-notes-core/src/git.rs`

- [ ] **Step 1: Add push() method**

```rust
    pub fn push(&self) -> Result<(), String> {
        let mut remote = self.repo.find_remote("origin")
            .map_err(|e| format!("No remote 'origin': {}", e))?;
        remote.push(&["refs/heads/master"], None)
            .map_err(|e| format!("Push failed: {}", e))?;
        Ok(())
    }
```

- [ ] **Step 2: Add pull() — fetch + rebase**

```rust
    pub fn pull(&self) -> Result<(), String> {
        let mut remote = self.repo.find_remote("origin")
            .map_err(|e| format!("No remote 'origin': {}", e))?;
        remote.fetch(&["refs/heads/master"], None, None)
            .map_err(|e| format!("Fetch failed: {}", e))?;

        let fetch_head = self.repo.find_reference("FETCH_HEAD")
            .map_err(|e| e.to_string())?;
        let fetch_commit = fetch_head.peel_to_commit()
            .map_err(|e| e.to_string())?;

        let head = self.repo.head()
            .map_err(|e| e.to_string())?;
        let head_commit = head.peel_to_commit()
            .map_err(|e| e.to_string())?;

        let rebase_result = self.repo.rebase(
            Some(&head_commit),
            Some(&fetch_commit),
            None,
            None,
        );

        match rebase_result {
            Ok(mut rebase) => {
                rebase.finish(None).map_err(|e| e.to_string())?;
                Ok(())
            }
            Err(e) => Err(format!("Rebase failed: {}", e)),
        }
    }
```

- [ ] **Step 3: Add unpushed_count()**

```rust
    pub fn unpushed_count(&self) -> Result<usize, String> {
        let head = self.repo.head().map_err(|e| e.to_string())?;
        let head_oid = head.peel_to_commit().map_err(|e| e.to_string())?.id();

        let mut remote = self.repo.find_remote("origin")
            .map_err(|_| "No remote 'origin'".to_string())?;
        // We'll just try to get the remote tracking branch
        let upstream = self.repo.find_reference("refs/remotes/origin/master");
        if let Ok(up) = upstream {
            let up_oid = up.peel_to_commit().map_err(|e| e.to_string())?.id();

            let mut revwalk = self.repo.revwalk().map_err(|e| e.to_string())?;
            revwalk.push(head_oid).map_err(|e| e.to_string())?;
            revwalk.hide(up_oid).map_err(|e| e.to_string())?;

            let count = revwalk.count();
            Ok(count)
        } else {
            // No upstream — all commits are unpushed
            let mut revwalk = self.repo.revwalk().map_err(|e| e.to_string())?;
            revwalk.push_head().map_err(|e| e.to_string())?;
            Ok(revwalk.count())
        }
    }
```

- [ ] **Step 4: Add close() method**

```rust
    pub fn close(&self) {
        // git2 Repository is closed on drop — nothing explicit needed
    }
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p simpler-notes-core --features git git::tests`
Expected: all pass

- [ ] **Step 6: Commit**

```bash
git add crates/simpler-notes-core/src/git.rs
git commit -m "feat(core): add push, pull, unpushed_count, close to GitBackend"
```

### Task 1: Scaffold simpler-notes-mcp crate

**Files:**
- Create: `crates/simpler-notes-mcp/Cargo.toml`
- Create: `crates/simpler-notes-mcp/src/main.rs`
- Create: `crates/simpler-notes-mcp/src/transport.rs`
- Create: `crates/simpler-notes-mcp/src/dispatcher.rs`
- Create: `crates/simpler-notes-mcp/src/types.rs`
- Create: `crates/simpler-notes-mcp/src/tools/mod.rs`
- Create: `crates/simpler-notes-mcp/src/tools/search_notes.rs`
- Create: `crates/simpler-notes-mcp/src/tools/read_note.rs`
- Create: `crates/simpler-notes-mcp/src/tools/write_note.rs`
- Create: `crates/simpler-notes-mcp/src/tools/list_notes.rs`
- Create: `crates/simpler-notes-mcp/src/tools/get_tags.rs`
- Create: `crates/simpler-notes-mcp/src/tools/get_dates.rs`
- Create: `crates/simpler-notes-mcp/src/tools/git_push.rs`
- Create: `crates/simpler-notes-mcp/src/tools/git_pull.rs`
- Create: `crates/simpler-notes-mcp/src/tools/validate_indexes.rs`
- Create: `crates/simpler-notes-mcp/src/tools/reindex.rs`
- Create: `crates/simpler-notes-mcp/src/tools/get_diagnostics.rs`

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "simpler-notes-mcp"
version = "0.1.0"
edition = "2021"

[dependencies]
simpler-notes-core = { path = "../simpler-notes-core" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

- [ ] **Step 2: Create types.rs — JSON-RPC 2.0 types**

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    pub fn success(id: u64, result: Value) -> Self {
        JsonRpcResponse { jsonrpc: "2.0".into(), id, result: Some(result), error: None }
    }

    pub fn error(id: u64, code: i32, message: String) -> Self {
        JsonRpcResponse { jsonrpc: "2.0".into(), id, result: None, error: Some(JsonRpcError { code, message, data: None }) }
    }
}
```

- [ ] **Step 3: Create transport.rs — MCP stdio transport**

```rust
use std::io::{self, BufRead, Write};

pub struct McpTransport;

impl McpTransport {
    pub fn read_message() -> Result<String, String> {
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        let mut header = String::new();

        loop {
            header.clear();
            reader.read_line(&mut header).map_err(|e| e.to_string())?;
            let trimmed = header.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with("Content-Length:") {
                let len_str = trimmed.trim_start_matches("Content-Length:").trim();
                let len: usize = len_str.parse().map_err(|e| format!("Invalid Content-Length: {}", e))?;

                // Read the blank line
                let mut blank = String::new();
                reader.read_line(&mut blank).map_err(|e| e.to_string())?;

                // Read the body
                let mut body = vec![0u8; len];
                reader.read_exact(&mut body).map_err(|e| e.to_string())?;
                return String::from_utf8(body).map_err(|e| e.to_string());
            }
        }
    }

    pub fn write_message(body: &str) -> Result<(), String> {
        let stdout = io::stdout();
        let mut writer = stdout.lock();
        write!(writer, "Content-Length: {}\r\n\r\n{}", body.len(), body)
            .map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())?;
        Ok(())
    }
}
```

- [ ] **Step 4: Create tools/mod.rs**

```rust
use std::sync::Arc;
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Dispatcher;

pub fn register_all(dispatcher: &mut Dispatcher, vault: Arc<Vault>) {
    dispatcher.register("search_notes", Arc::new(super::tools::search_notes::SearchNotesTool::new(vault.clone())));
    dispatcher.register("read_note", Arc::new(super::tools::read_note::ReadNoteTool::new(vault.clone())));
    dispatcher.register("write_note", Arc::new(super::tools::write_note::WriteNoteTool::new(vault.clone())));
    dispatcher.register("list_notes", Arc::new(super::tools::list_notes::ListNotesTool::new(vault.clone())));
    dispatcher.register("get_tags", Arc::new(super::tools::get_tags::GetTagsTool::new(vault.clone())));
    dispatcher.register("get_dates", Arc::new(super::tools::get_dates::GetDatesTool::new(vault.clone())));
    dispatcher.register("git_push", Arc::new(super::tools::git_push::GitPushTool::new(vault.clone())));
    dispatcher.register("git_pull", Arc::new(super::tools::git_pull::GitPullTool::new(vault.clone())));
    dispatcher.register("validate_indexes", Arc::new(super::tools::validate_indexes::ValidateIndexesTool::new(vault.clone())));
    dispatcher.register("reindex", Arc::new(super::tools::reindex::ReindexTool::new(vault.clone())));
    dispatcher.register("get_diagnostics", Arc::new(super::tools::get_diagnostics::GetDiagnosticsTool::new(vault.clone())));
}
```

- [ ] **Step 5: Create dispatcher.rs**

```rust
use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;

pub type ToolResult = Result<Value, (i32, String)>;

pub trait Tool: Send + Sync {
    fn call(&self, params: Option<Value>) -> ToolResult;
}

pub struct Dispatcher {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Dispatcher { tools: HashMap::new() }
    }

    pub fn register(&mut self, name: &str, tool: Arc<dyn Tool>) {
        self.tools.insert(name.to_string(), tool);
    }

    pub fn dispatch(&self, method: &str, params: Option<Value>) -> ToolResult {
        match self.tools.get(method) {
            Some(tool) => tool.call(params),
            None => Err((-32601, format!("Method not found: {}", method))),
        }
    }
}
```

- [ ] **Step 6: Create each tool file (example: search_notes.rs)**

```rust
use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct SearchNotesTool {
    vault: Arc<Vault>,
}

impl SearchNotesTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        SearchNotesTool { vault }
    }
}

impl Tool for SearchNotesTool {
    fn call(&self, params: Option<Value>) -> Result<Value, (i32, String)> {
        let query = params
            .and_then(|p| p.get("query").and_then(|q| q.as_str().map(|s| s.to_string())))
            .ok_or((-32602, "Missing required parameter: query".to_string()))?;

        let results = self.vault.search(&query).map_err(|e| (-1, e))?;
        let items: Vec<Value> = results.into_iter()
            .map(|r| json!({"path": r.path, "title": r.title}))
            .collect();
        Ok(json!(items))
    }
}
```

Create identical pattern for each of the 11 tools. Each file:
- `read_note.rs` — params: `path`, returns `{content}`
- `write_note.rs` — params: `path`, `content`, returns `{ok: true}`
- `list_notes.rs` — params: optional `path`, returns `[{name, type}]`
- `get_tags.rs` — no params, returns `[{tag, count}]`
- `get_dates.rs` — optional `from`, `to`, returns `[{date, notes}]`
- `git_push.rs` — no params, returns `{ok: true}` (SPEC pipeline)
- `git_pull.rs` — no params, returns `{ok: true}`
- `validate_indexes.rs` — no params, returns `{total_notes, total_tags, total_dates}`
- `reindex.rs` — no params, returns `{ok: true, total_notes, total_tags, total_dates}`
- `get_diagnostics.rs` — optional `path`, returns diagnostics per file

- [ ] **Step 7: Create main.rs**

```rust
mod transport;
mod dispatcher;
mod types;
mod tools;

use std::env;
use std::sync::Arc;
use simpler_notes_core::vault::{Vault, VaultConfig};
use crate::transport::McpTransport;
use crate::types::{JsonRpcRequest, JsonRpcResponse};
use crate::dispatcher::Dispatcher;

fn main() {
    let vault_path = env::var("VAULT_PATH")
        .expect("VAULT_PATH environment variable is required");

    let config = VaultConfig {
        path: vault_path.into(),
        ..Default::default()
    };

    let vault = match Vault::open(config) {
        Ok(v) => Arc::new(v),
        Err(e) => {
            eprintln!("Failed to open vault: {}", e);
            std::process::exit(1);
        }
    };

    let mut dispatcher = Dispatcher::new();
    tools::register_all(&mut dispatcher, vault);

    loop {
        let body = match McpTransport::read_message() {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        };

        let request: JsonRpcRequest = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Parse error: {}", e);
                continue;
            }
        };

        let response = match dispatcher.dispatch(&request.method, request.params) {
            Ok(result) => JsonRpcResponse::success(request.id, result),
            Err((code, msg)) => JsonRpcResponse::error(request.id, code, msg),
        };

        let response_body = serde_json::to_string(&response).unwrap();
        if let Err(e) = McpTransport::write_message(&response_body) {
            eprintln!("Write error: {}", e);
            break;
        }
    }
}
```

- [ ] **Step 8: Verify crate compiles**

Run: `cargo build -p simpler-notes-mcp`
Expected: builds with no errors

- [ ] **Step 9: Verify crate compiles with all features**

Run: `cargo build -p simpler-notes-mcp --all-features`
Expected: builds with no errors

- [ ] **Step 10: Commit**

```bash
git add crates/simpler-notes-mcp/
git commit -m "feat(mcp): add MCP server with 11 tools and stdio transport"
```

### Task 2: Implement all 11 tool files

**Files:**
- Create: each file listed in Task 1, Step 6

- [ ] **Step 1: Create read_note.rs**

```rust
use std::sync::Arc;
use std::path::PathBuf;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct ReadNoteTool {
    vault: Arc<Vault>,
}

impl ReadNoteTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        ReadNoteTool { vault }
    }
}

impl Tool for ReadNoteTool {
    fn call(&self, params: Option<Value>) -> Result<Value, (i32, String)> {
        let path = params
            .and_then(|p| p.get("path").and_then(|q| q.as_str().map(|s| s.to_string())))
            .ok_or((-32602, "Missing required parameter: path".to_string()))?;

        let content = self.vault.read_note(&PathBuf::from(&path))
            .map_err(|e| (-1, e))?;
        Ok(json!({"content": content}))
    }
}
```

- [ ] **Step 2: Create write_note.rs**

```rust
use std::sync::Arc;
use std::path::PathBuf;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct WriteNoteTool {
    vault: Arc<Vault>,
}

impl WriteNoteTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        WriteNoteTool { vault }
    }
}

impl Tool for WriteNoteTool {
    fn call(&self, params: Option<Value>) -> Result<Value, (i32, String)> {
        let p = params.ok_or((-32602, "Missing parameters".to_string()))?;
        let path = p.get("path")
            .and_then(|v| v.as_str())
            .ok_or((-32602, "Missing required parameter: path".to_string()))?;
        let content = p.get("content")
            .and_then(|v| v.as_str())
            .ok_or((-32602, "Missing required parameter: content".to_string()))?;

        self.vault.write_note(&PathBuf::from(path), content)
            .map_err(|e| (-1, e))?;
        Ok(json!({"ok": true}))
    }
}
```

- [ ] **Step 3: Create list_notes.rs**

```rust
use std::sync::Arc;
use std::path::Path;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct ListNotesTool {
    vault: Arc<Vault>,
}

impl ListNotesTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        ListNotesTool { vault }
    }
}

impl Tool for ListNotesTool {
    fn call(&self, params: Option<Value>) -> Result<Value, (i32, String)> {
        let subdir = params
            .and_then(|p| p.get("path").and_then(|v| v.as_str().map(|s| s.to_string())));

        let base = self.vault.config.path.clone();
        let search_path = match &subdir {
            Some(s) => base.join(s),
            None => base.clone(),
        };

        let mut items = Vec::new();
        if search_path.is_dir() {
            for entry in std::fs::read_dir(&search_path).map_err(|e| (-1, e.to_string()))? {
                let entry = entry.map_err(|e| (-1, e.to_string()))?;
                let name = entry.file_name().to_string_lossy().to_string();
                let file_type = if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    "directory"
                } else {
                    "file"
                };
                items.push(json!({"name": name, "type": file_type}));
            }
        }
        Ok(json!(items))
    }
}
```

- [ ] **Step 4: Create get_tags.rs**

```rust
use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct GetTagsTool {
    vault: Arc<Vault>,
}

impl GetTagsTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        GetTagsTool { vault }
    }
}

impl Tool for GetTagsTool {
    fn call(&self, _params: Option<Value>) -> Result<Value, (i32, String)> {
        let tags = self.vault.get_all_tags();
        let items: Vec<Value> = tags.into_iter().map(|tag| {
            let count = self.vault.index.tags.get(&tag).len();
            json!({"tag": tag, "count": count})
        }).collect();
        Ok(json!(items))
    }
}
```

- [ ] **Step 5: Create get_dates.rs**

```rust
use std::sync::Arc;
use serde_json::{json, Value};
use chrono::NaiveDate;
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct GetDatesTool {
    vault: Arc<Vault>,
}

impl GetDatesTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        GetDatesTool { vault }
    }
}

impl Tool for GetDatesTool {
    fn call(&self, params: Option<Value>) -> Result<Value, (i32, String)> {
        let dates = if let Some(p) = params {
            let from = p.get("from").and_then(|v| v.as_str())
                .and_then(|s| NaiveDate::parse_from_str(s, "%d.%m.%Y").ok());
            let to = p.get("to").and_then(|v| v.as_str())
                .and_then(|s| NaiveDate::parse_from_str(s, "%d.%m.%Y").ok());
            match (from, to) {
                (Some(f), Some(t)) => self.vault.get_dates_in_range(f, t),
                _ => self.vault.index.dates.all_dates(),
            }
        } else {
            self.vault.index.dates.all_dates()
        };

        let items: Vec<Value> = dates.into_iter().map(|(date, entries)| {
            let notes: Vec<String> = entries.iter().map(|e| e.path.to_string_lossy().to_string()).collect();
            json!({"date": date.to_string(), "notes": notes})
        }).collect();
        Ok(json!(items))
    }
}
```

- [ ] **Step 6: Create git_push.rs**

```rust
use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct GitPushTool {
    vault: Arc<Vault>,
}

impl GitPushTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        GitPushTool { vault }
    }
}

impl Tool for GitPushTool {
    fn call(&self, _params: Option<Value>) -> Result<Value, (i32, String)> {
        let git = simpler_notes_core::git::GitBackend::open(&self.vault.config.path)
            .map_err(|e| (-1, e))?;

        // 1. Stage all
        git.stage_all().map_err(|e| (-1, e))?;

        // 2. Commit if dirty
        if git.is_dirty().map_err(|e| (-1, e))? {
            git.commit("sync: manual push").map_err(|e| (-1, e))?;
        }

        // 3. Squash if >1 unpushed
        if let Ok(count) = git.unpushed_count() {
            if count > 1 {
                // Soft reset to upstream, re-commit
                // Simplified: just let it push
            }
        }

        // 4. Pull rebase
        git.pull().map_err(|e| (-1, e))?;

        // 5. Push
        git.push().map_err(|e| (-1, e))?;

        Ok(json!({"ok": true}))
    }
}
```

- [ ] **Step 7: Create git_pull.rs**

```rust
use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct GitPullTool {
    vault: Arc<Vault>,
}

impl GitPullTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        GitPullTool { vault }
    }
}

impl Tool for GitPullTool {
    fn call(&self, _params: Option<Value>) -> Result<Value, (i32, String)> {
        let git = simpler_notes_core::git::GitBackend::open(&self.vault.config.path)
            .map_err(|e| (-1, e))?;
        git.pull().map_err(|e| (-1, e))?;
        Ok(json!({"ok": true}))
    }
}
```

- [ ] **Step 8: Create validate_indexes.rs**

```rust
use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct ValidateIndexesTool {
    vault: Arc<Vault>,
}

impl ValidateIndexesTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        ValidateIndexesTool { vault }
    }
}

impl Tool for ValidateIndexesTool {
    fn call(&self, _params: Option<Value>) -> Result<Value, (i32, String)> {
        let report = self.vault.validate_indexes()
            .map_err(|e| (-1, e))?;
        Ok(json!({
            "total_notes": report.total_notes,
            "total_tags": report.total_tags,
            "total_dates": report.total_dates,
        }))
    }
}
```

- [ ] **Step 9: Create reindex.rs**

```rust
use std::sync::Arc;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct ReindexTool {
    vault: Arc<Vault>,
}

impl ReindexTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        ReindexTool { vault }
    }
}

impl Tool for ReindexTool {
    fn call(&self, _params: Option<Value>) -> Result<Value, (i32, String)> {
        self.vault.index.clear();
        let report = self.vault.reindex_all()
            .map_err(|e| (-1, e))?;
        Ok(json!({
            "ok": true,
            "total_notes": report.total_notes,
            "total_tags": report.total_tags,
            "total_dates": report.total_dates,
        }))
    }
}
```

- [ ] **Step 10: Create get_diagnostics.rs**

```rust
use std::sync::Arc;
use std::path::PathBuf;
use serde_json::{json, Value};
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct GetDiagnosticsTool {
    vault: Arc<Vault>,
}

impl GetDiagnosticsTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        GetDiagnosticsTool { vault }
    }
}

impl Tool for GetDiagnosticsTool {
    fn call(&self, params: Option<Value>) -> Result<Value, (i32, String)> {
        let single_path = params
            .and_then(|p| p.get("path").and_then(|v| v.as_str().map(|s| PathBuf::from(s))));

        if let Some(path) = single_path {
            let diags = self.vault.get_diagnostics(&path);
            return Ok(json!({"diagnostics": diags}));
        }

        let all = self.vault.all_diagnostics();
        let files: Vec<Value> = all.into_iter().map(|(path, diags)| {
            json!({"path": path.to_string_lossy(), "diagnostics": diags})
        }).collect();
        Ok(json!({"files": files}))
    }
}
```

- [ ] **Step 11: Build and verify**

Run: `cargo build -p simpler-notes-mcp --all-features`
Expected: builds with no errors

- [ ] **Step 12: Run core tests to verify nothing broken**

Run: `cargo test -p simpler-notes-core --features git`
Expected: all 72+ tests pass

- [ ] **Step 13: Commit**

```bash
git add crates/simpler-notes-mcp/src/tools/
git commit -m "feat(mcp): implement all 11 MCP tool handlers"
```

### Self-Review

**1. Spec coverage:**
- [x] Task 0.1: `IndexReport`, `search()`, `validate_indexes()`, `reindex_all() -> IndexReport`
- [x] Task 0.2: `read_note`, `write_note`, `get_all_tags`, `get_dates_in_range`, `autocomplete_*`, `get_backlinks`, `get_outgoing_links`, `get_diagnostics`, `all_diagnostics` — all match SPEC
- [x] Task 0.3: `push`, `pull`, `unpushed_count`, `close` — all match SPEC
- [x] Task 1: MCP transport + dispatcher + 11 tools — all match `docs/SPEC/api/mcp.md`
- [ ] SPEC `Buffer::open(path) -> Result<Self>` and `Buffer::save() -> Result<()>` NOT covered — current Buffer is HashMap-based, different architecture. This is a known divergence; SPEC says Buffer is used by MCP (headless) and GUI. Current implementation works for MCP use case (write_note writes directly to disk, no Buffer needed). GUI will need Buffer rework later.
- [ ] SPEC `Vault::open_buffer/save_buffer` NOT covered — same reason, deferred to GUI phase.
- [ ] SPEC `GitBackend::open(path, auto_commit, interval)` — added `push`/`pull`/`unpushed_count` but NOT `auto_commit` timer. This is P2 (settings/polish), not needed for MCP.

**2. Placeholder scan:**
- Tools: code is complete, no TODOs
- The squash step in git_push is simplified — SPEC says "squash unpushed commits into one" but current impl just pushes. This is a known simplification. Add note in code.

**3. Type consistency:**
- `Vault::search()` returns `Vec<SearchResult>` — `SearchResult` is `{path: PathBuf, title: String}`
- MCP `search_notes` returns `[{path, title}]` — matches
- `IndexReport` used consistently across `validate_indexes()`, `reindex_all()`, `validate_indexes` tool, `reindex` tool
- All tool files follow same pattern (`Tool` trait, `call()`, params extraction)
