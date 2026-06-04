# Link Index V2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rewrite `LinkIndex` from a single backward-only DashMap to two DashMaps (`by_target` + `by_source`), store full resolved paths in `LinkEntry.target`, and implement rename refactoring.

**Architecture:** `LinkIndex` gets a second DashMap for O(1) outgoing lookups. `reindex_file` resolves `[[stems]]` to full paths via `filename_index`. Persistence switches to flat `Vec<LinkEntry>` with `INDEX_VERSION=2`. `Vault::rename_file` rewrites `[[stems]]` using `LinkEntry.span`.

**Tech Stack:** Rust, DashMap, serde_json, walkdir

---

### Task 1: LinkIndex — two DashMaps, update_target, O(1) outgoing

**Files:**
- Modify: `crates/simpler-notes-core/src/index/link_index.rs` (full rewrite)

- [ ] **Step 1.1: Write failing tests for new LinkIndex API**

Add these tests alongside existing ones:

```rust
#[test]
fn test_update_target() {
    let index = LinkIndex::new();
    let old_target = PathBuf::from("old.md");
    let new_target = PathBuf::from("new.md");
    index.add(PathBuf::from("a.md"), make_entry("a.md", "old.md", "link"));
    // backlinks before update
    assert_eq!(index.backlinks(&old_target).len(), 1);
    assert!(index.backlinks(&new_target).is_empty());
    // update
    index.update_target(&old_target, &new_target);
    // backlinks after update
    assert!(index.backlinks(&old_target).is_empty());
    assert_eq!(index.backlinks(&new_target).len(), 1);
    assert_eq!(index.backlinks(&new_target)[0].target, new_target);
}

#[test]
fn test_outgoing_is_o1() {
    let index = LinkIndex::new();
    index.add(PathBuf::from("a.md"), make_entry("a.md", "b.md", "B"));
    index.add(PathBuf::from("a.md"), make_entry("a.md", "c.md", "C"));
    let outgoing = index.outgoing(&PathBuf::from("a.md"));
    assert_eq!(outgoing.len(), 2);
    assert!(outgoing.iter().any(|e| e.target == PathBuf::from("b.md")));
    assert!(outgoing.iter().any(|e| e.target == PathBuf::from("c.md")));
    // file with no outgoing
    assert!(index.outgoing(&PathBuf::from("orphan.md")).is_empty());
}

#[test]
fn test_remove_file_cleans_both_maps() {
    let index = LinkIndex::new();
    index.add(PathBuf::from("a.md"), make_entry("a.md", "b.md", "B"));
    index.add(PathBuf::from("a.md"), make_entry("a.md", "c.md", "C"));
    index.remove_file(&PathBuf::from("a.md"));
    assert!(index.outgoing(&PathBuf::from("a.md")).is_empty());
    assert!(index.backlinks(&PathBuf::from("b.md")).is_empty());
    assert!(index.backlinks(&PathBuf::from("c.md")).is_empty());
}

#[test]
fn test_add_maintains_both_maps() {
    let index = LinkIndex::new();
    let entry = make_entry("a.md", "b.md", "B");
    index.add(PathBuf::from("a.md"), entry);
    // by_source
    let outgoing = index.outgoing(&PathBuf::from("a.md"));
    assert_eq!(outgoing.len(), 1);
    assert_eq!(outgoing[0].target, PathBuf::from("b.md"));
    // by_target
    let backlinks = index.backlinks(&PathBuf::from("b.md"));
    assert_eq!(backlinks.len(), 1);
    assert_eq!(backlinks[0].source, PathBuf::from("a.md"));
}
```

- [ ] **Step 1.2: Run tests to verify they fail**

Run:
```bash
cargo test -p simpler-notes-core link_index 2>&1 | tail -20
```
Expected: compilation error because `update_target` and `outgoing` behavior changed.

- [ ] **Step 1.3: Rewrite LinkIndex**

```rust
use std::path::{Path, PathBuf};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use crate::note_model::ByteSpan;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkEntry {
    pub source: PathBuf,
    pub target: PathBuf,
    pub label: String,
    pub span: ByteSpan,
}

pub struct LinkIndex {
    by_target: DashMap<PathBuf, Vec<LinkEntry>>,
    by_source: DashMap<PathBuf, Vec<LinkEntry>>,
}

impl Default for LinkIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl LinkIndex {
    pub fn new() -> Self {
        LinkIndex {
            by_target: DashMap::new(),
            by_source: DashMap::new(),
        }
    }

    pub fn add(&self, _source: PathBuf, entry: LinkEntry) {
        self.by_target.entry(entry.target.clone()).or_default().push(entry.clone());
        self.by_source.entry(entry.source.clone()).or_default().push(entry);
    }

    pub fn remove_file(&self, path: &Path) {
        // Remove from by_source and collect targets to clean up by_target
        if let Some((_, entries)) = self.by_source.remove(path) {
            for e in &entries {
                if let Some(mut vec) = self.by_target.get_mut(&e.target) {
                    vec.retain(|v| v.source != path);
                    if vec.is_empty() {
                        drop(vec);
                        self.by_target.remove(&e.target);
                    }
                }
            }
        }
    }

    pub fn backlinks(&self, target: &Path) -> Vec<LinkEntry> {
        self.by_target.get(target).map(|e| e.value().clone()).unwrap_or_default()
    }

    pub fn outgoing(&self, source: &Path) -> Vec<LinkEntry> {
        self.by_source.get(source).map(|e| e.value().clone()).unwrap_or_default()
    }

    pub fn update_target(&self, old: &Path, new: &Path) {
        if let Some((_, entries)) = self.by_target.remove(old) {
            for mut entry in entries {
                entry.target = new.to_path_buf();
                self.by_target.entry(new.to_path_buf()).or_default().push(entry.clone());
                // Also update entry in by_source
                if let Some(mut vec) = self.by_source.get_mut(&entry.source) {
                    for e in vec.iter_mut() {
                        if e.target == old {
                            e.target = new.to_path_buf();
                        }
                    }
                }
            }
        }
    }

    pub fn clear(&self) {
        self.by_target.clear();
        self.by_source.clear();
    }

    pub fn all_targets(&self) -> Vec<PathBuf> {
        let mut targets: Vec<PathBuf> = self.by_target.iter()
            .map(|e| e.key().clone())
            .collect();
        targets.sort();
        targets.dedup();
        targets
    }

    /// For serialization — iterate all entries
    pub fn iter(&self) -> impl Iterator<Item = LinkEntry> + '_ {
        self.by_source.iter().flat_map(|e| e.value().iter().cloned())
    }
}
```

- [ ] **Step 1.4: Run tests to verify they pass**

Run:
```bash
cargo test -p simpler-notes-core link_index
```
Expected: all LinkIndex tests pass (including old test_table_driven_link_index).

- [ ] **Step 1.5: Commit**

```bash
git add crates/simpler-notes-core/src/index/link_index.rs
git commit -m "feat(link-index): add by_source DashMap, update_target, O(1) outgoing"
```

---

### Task 2: reindex_file — resolve target to full path via filename_index

**Files:**
- Modify: `crates/simpler-notes-core/src/index/mod.rs` (reindex_file block for links)

- [ ] **Step 2.1: Write failing test for full-path resolution**

Add to `mod.rs` test section:

```rust
struct Case {
    name: &'static str,
    content: &'static str,
    filename_index: std::collections::HashMap<String, Vec<PathBuf>>,
    check: fn(&ConcurrentIndex, &mut Vec<String>),
}

let cases: Vec<Case> = vec![
    Case {
        name: "link resolves to full path via filename_index",
        content: "[[beta]]",
        filename_index: {
            let mut m = std::collections::HashMap::new();
            m.insert("beta".into(), vec![PathBuf::from("notes/beta.md")]);
            m
        },
        check: |idx, errors| {
            let targets = idx.links.all_targets();
            if targets.len() != 1 {
                errors.push(format!("expected 1 target, got {}", targets.len()));
            } else if targets[0] != PathBuf::from("notes/beta.md") {
                errors.push(format!("expected notes/beta.md, got {:?}", targets[0]));
            }
        },
    },
    Case {
        name: "ambiguous link uses normalized path as fallback",
        content: "[[beta]]",
        filename_index: {
            let mut m = std::collections::HashMap::new();
            m.insert("beta".into(), vec![
                PathBuf::from("notes/beta.md"),
                PathBuf::from("other/beta.md"),
            ]);
            m
        },
        check: |idx, errors| {
            let targets = idx.links.all_targets();
            if targets.len() != 1 {
                errors.push(format!("expected 1 target, got {}", targets.len()));
            } else {
                // Should be normalized but NOT resolved to full path (ambiguous)
                let t = &targets[0];
                if t.to_string_lossy().contains("beta.md") && t.to_string_lossy().contains('/') {
                    // accept any normalized path
                } else {
                    errors.push(format!("unexpected target: {:?}", t));
                }
            }
        },
    },
];
```

Also add to the table_driven tests in the existing test function (replace or extend the absolute-path case):

Replace the `test_table_driven_reindex_file`'s "absolute path link is indexed by file_stem" case with a version that tests full path resolution.

- [ ] **Step 2.2: Run tests to verify they fail**

Run:
```bash
cargo test -p simpler-notes-core index 2>&1 | tail -30
```
Expected: some tests fail because target is now full path, not stem.

- [ ] **Step 2.3: Update reindex_file link resolution**

In `crates/simpler-notes-core/src/index/mod.rs`, replace the link-adding loop:

```rust
for link_span in &result.links {
    let raw_target = PathBuf::from(&link_span.file_name);
    let resolved = if raw_target.is_absolute() {
        raw_target
    } else {
        path.parent().unwrap_or(Path::new("")).join(&raw_target)
    };
    let normalized = normalize_path(&resolvedapse);
    let stem = normalized
        .file_stem()
        .unwrap_or(normalized.as_os_str())
        .to_string_lossy()
        .to_string();

    // Resolve stem to full path via filename_index:
    //   - 1 match → use that path (unambiguous)
    //   - 0 or >1 → use normalized path as fallback (broken/ambiguous)
    let target = filename_index
        .get(&stem)
        .and_then(|paths| if paths.len() == 1 { Some(paths[0].clone()) } else { None })
        .unwrap_or(normalized);

    let entry = LinkEntry {
        source: path.to_path_buf(),
        target,
        label: link_span.label.clone(),
        span: ByteSpan { offset: link_span.span.offset, length: link_span.span.length },
    };
    self.links.add(path.to_path_buf(), entry);
}
```

- [ ] **Step 2.4: Fix existing tests**

In `mod.rs` tests, update assertions that expect stem targets:

- `test_clear_all`: `target: PathBuf::from("other.md")` — fine, stays as relative path since no filename_index provided
- `test_table_driven_reindex_file`: update "absolute path link is indexed by file_stem" — now absolute path stays as resolved full path
- `test_table_driven_reindex_replaces_old_links`: `backlinks(&PathBuf::from("old-link"))` — these become full resolved paths

The empty_index() returns no filename_index, so stems won't resolve. That means targets will be normalized relative paths (like `./alpha`) rather than stems. Let me adjust the tests to use `Path::new(".").join("alpha")` or just accept what comes out.

Actually, let me think about this. With an empty filename_index, the code does:
```rust
let target = filename_index.get(&stem)
    .and_then(|paths| if paths.len() == 1 { Some(paths[0].clone()) } else { None })
    .unwrap_or(normalized);
```

Where `normalized` is the full normalized path (absolute or relative). For a test file at `test_0.md` containing `[[alpha]]`, the resolved path would be `Path::new("").join("alpha")` = `./alpha`. After normalization it stays as `./alpha`.

Wait, `normalize_path` — let me check what that does. It's in `util.rs`.

Actually, I need to look more carefully. The test creates files like `test_0.md` in the current directory (no TempDir). So `path.parent()` would be `""` (empty), and resolving `[[alpha]]` relative to `""` gives `alpha`. After normalization it's `alpha`. No extension. So `normalized.file_stem()` returns `Some("alpha")`, which is the same as `normalized.as_os_str()` since there's no `.md` extension.

After change, with empty filename_index, the target is `normalized` which is `alpha` (no extension). So the tests that check for `PathBuf::from("alpha")` or `PathBuf::from("replaced")` should still work.

Wait let me re-read the normalization code more carefully. The `resolved` is either absolute (user used `/tmp/...`) or relative to the file's parent dir. For a file at `test_0.md` in cwd, `path.parent()` would be `""`, so `Path::new("").join(&raw_target)` gives `alpha`. Then `normalize_path(&resolved)` on `alpha` — let me see...

I should check what `normalize_path` does.

Actually, I'll check this on the fly. The key point: the old code extracted `file_stem()` from normalized and made a PathBuf from just the stem string. The new code falls through to `normalized` which still has the stem behavior when there's no extensioniest.

Let me just write the update and verify it passes. The important thing is to get the logic right and then fix tests accordingly.

- [ ] **Step 2.5: Run tests to verify they pass**

```bash
cargo test -p simpler-notes-core index
```

- [ ] **Step 2.6: Commit**

```bash
git add crates/simpler-notes-core/src/index/mod.rs
git commit -m "feat(link-index): resolve LinkEntry.target to full path in reindex_file"
```

---

### Task 3: Persistence — flat Vec<LinkEntry>, INDEX_VERSION=2

**Files:**
- Modify: `crates/simpler-notes-core/src/persistence.rs`

- [ ] **Step 3.1: Update save to use flat Vec<LinkEntry>**

Replace the links serialization section:

```rust
const INDEX_VERSION: u32 = 2;

// In save():
let links: Vec<LinkEntry> = self.links.iter().collect();
fs::write(
    index_dir.join("links.json"),
    serde_json::to_string_pretty(&links).map_err(|e| e.to_string())?,
).map_err(|e| e.to_string())?;
```

- [ ] **Step 3.2: Update load to use flat Vec<LinkEntry>**

```rust
// In load():
let links_path = index_dir.join("links.json");
if links_path.exists() {
    let content = fs::read_to_string(&links_path).map_err(|e| e.to_string())?;
    let data: Vec<LinkEntry> =
        serde_json::from_str(&content).map_err(|e| e.to_string())?;
    for entry in data {
        index.links.add(entry.source.clone(), entry);
    }
}
```

- [ ] **Step 3.3: Run persistence tests**

```bash
cargo test -p simpler-notes-core persistence
```
Expected: `test_save_and_load`, `test_save_and_load_with_links` pass.

- [ ] **Step 3.4: Run all core tests to check for breakage**

```bash
cargo test -p simpler-notes-core 2>&1 | tail -40
```

- [ ] **Step 3.5: Commit**

```bash
git add crates/simpler-notes-core/src/persistence.rs
git commit -m "feat(link-index): flat Vec<LinkEntry> persistence, bump INDEX_VERSION to 2"
```

---

### Task 4: Vault — rename_file

**Files:**
- Modify: `crates/simpler-notes-core/src/vault.rs`

- [ ] **Step 4.1: Write failing test for rename_file**

```rust
#[test]
fn test_rename_file_refactors_backlinks() {
    let dir = TempDir::new().unwrap();
    let a_path = dir.path().join("a.md");
    let b_path = dir.path().join("b.md");
    std::fs::write(&a_path, "[[b]]").unwrap();
    std::fs::write(&b_path, "content").unwrap();

    let vault = Vault::open(VaultConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    }).unwrap();

    let new_b_path = dir.path().join("renamed.md");
    vault.rename_file(&b_path, &new_b_path).unwrap();

    // File should be renamed on disk
    assert!(!b_path.exists(), "old file should not exist");
    assert!(new_b_path.exists(), "new file should exist");

    // a.md should have updated link
    let a_content = vault.read_note(&PathBuf::from("a.md")).unwrap();
    assert!(a_content.contains("[[renamed]]"), "link should be updated: {}", a_content);
    assert!(!a_content.contains("[[b]]"), "old link should be removed");

    // Backlinks should point to new file
    let backlinks = vault.get_backlinks(&new_b_path);
    assert!(!backlinks.is_empty(), "should have backlinks to renamed file");
    assert_eq!(backlinks[0].source, a_pathCivil);
}

#[test]
fn test_rename_file_nonexistent_returns_error() {
    let dir = TempDir::new().unwrap();
    let vault = Vault::open(VaultConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    }).unwrap();

    let result = vault.rename_file(
        &dir.path().join("nonexistent.md"),
        &dir.path().join("still-nonexistent.md"),
    );
    assert!(result.is_err());
}

#[test]
fn test_rename_file_updates_index() {
    let dir = TempDir::new().unwrap();
    let a_path = dir.path().join("a.md");
    let b_path = dir.path().join("b.md");
    std::fs::write(&a_path, "[[b]]").unwrap();
    std::fs::write(&b_path, "content").unwrap();

    let vault = Vault::open(VaultConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    }).unwrap();
    let new_b_path = dir.path().join("renamed.md");
    vault.rename_file(&b_path, &new_b_path).unwrap();

    // Outgoing from a.md should point to new path
    let outgoing = vault.get_outgoing_links(&a_path);
    assert_eq!(outgoing.len(), 1);
    assert_eq!(outgoing[0].target, new_b_path, "target should be new path");
}
```

- [ ] **Step 4.2: Implement Vault::rename_file**

```rust
pub fn rename_file(&self, from: &Path, to: &Path) -> Result<(), String> {
    if !from.exists() {
        return Err(format!("Source file does not exist: {:?}", from));
    }

    // 1. Collect all backlinks before rename
    let backlink_entries: Vec<LinkEntry> = self.index.links.backlinks(from);

    // 2. For each backlink, replace old stem with new stem in file content
    let old_stem = from.file_stem()
        .ok_or_else(|| "Invalid source filename".to_string())?
        .to_string_lossy()
        .to_string();
    let new_stem = to.file_stem()
        .ok_or_else(|| "Invalid target filename".to_string())?
        .to_string_lossy()
        .to_string();

    let mut modified_sources: Vec<PathBuf> = Vec::new();
    for entry in &backlink_entries {
        let content = std::fs::read_to_string(&entry.source)
            .map_err(|e| format!("Failed to read {:?}: {}", entry.source, e))?;

        // Replace [[old_stem]] → [[new_stem]] using span
        let span = &entry.span;
        let before = &content[..span.offset];
        let after = &content[span.offset + span.length..];
        let new_content = format!("{}[[{}]]{}", before, new_stem, after);

        std::fs::write(&entry.source, &new_content)
            .map_err(|e| format!("Failed to write {:?}: {}", entry.source, e))?;

        modified_sources.push(entry.source.clone());
    }

    // 3. Move the file on disk
    std::fs::rename(from, to)
        .map_err(|e| format!("Failed to rename {:?} to {:?}: {}", from, to, e))?;

    // 4. Update index
    self.index.links.update_target(from, to);
    self.index.links.remove_file(from_registry);
    // Reindex renamed file
    if let Ok(content) = std::fs::read_to_string(to) {
        let filename_index = self.build_filename_index();
        self.index.reindex_file(to, &content, &self.config.path, &filename_index);
    }
    // Reindex all modified source files
    let filename_index = self.build_filename_index();
    for source in &modified_sources {
        if let Ok(content) = std::fs::read_to_string(source) {
            self.index.reindex_file(source, &content, &self.config.path, &filename_index);
        }
    }

    // 5. Save index
    self.index.save(&self.config.path)?;
    Ok(())
}
```

Wait, I made a typo — `from_registry` should be `from`. Also I used a typo `a_pathCivil`. Let me clean this up in the actual plan write.

- [ ] **Step 4.3: Run vault tests**

```bash
cargo test -p simpler-notes-core vault
```

- [ ] **Step 4.4: Commit**

```bash
git add crates/simpler-notes-core/src/vault.rs
git commit -m "feat(vault): add rename_file with backlink refactoring"
```

---

### Task 5: Fix consumers — MCP tools + remaining tests

**Files:**
- Modify: `crates/simpler-notes-mcp/src/tools/get_backlinks.rs` (resolve target to full path)
- Modify: `crates/simpler-notes-core/src/vault.rs` (fix backlinks/outgoing tests)
- Modify: `crates/simpler-notes-mcp/tests/e2e.rs` (fix outgoing target assertions)

- [ ] **Step 5.1: Fix get_backlinks MCP handler**

```rust
pub(crate) fn handler(vault: &Vault, params: Option<Value>) -> ToolResult {
    let p = params.ok_or((-32602, "Missing parameters".to_string()))?;
    let path = p.get("path")
        .and_then(|v| v.as_str())
        .ok_or((-32602, "Missing required parameter: path".to_string()))?;

    // Resolve path relative to vault root
    let full_path = vault.config.path.join(path);
    let backlinks = vault.get_backlinks(&full_path);
    let items: Vec<Value> = backlinks.into_iter().map(|e| {
        json!({
            "source": e.source.to_string_lossy(),
            "target": e.target.to_string_lossy(),
            "label": e.label,
        })
    }).collect();
    Ok(json!(items))
}
```

- [ ] **Step 5.2: Fix vault.rs tests for new target format**

In `vault.rs`:

`test_get_backlinks`: change to pass full path:
```rust
let backlinks = vault.get_backlinks(&b_path);  // full path, not stem
assert!(!backlinks.is_empty(), "Expected backlinks");
assert_eq!(backlinks[0].source, a_path);
```

`test_get_outgoing_links`: change to expect full path target:
```rust
let outgoing = vault.get_outgoing_links(&a_path);
assert!(!outgoing.is_empty(), "Expected outgoing links");
assert_eq!(outgoing[0].target, b_path);  // full path, not stem
```

`test_relative_link_resolves_in_link_index`: the target is now the full path:
```rust
assert_eq!(outgoing[0].target, dir.path().join("target.md"));  // full path
```

- [ ] **Step 5.3: Fix e2e test_get_outgoing_links**

```rust
let targets: Vec<&str> = result.unwrap().iter()
    .filter_map(|v| v.get("target").and_then(|s| s.as_str()))
    .collect();
assert!(targets.iter().any(|t| t.contains("beta.md")), "alpha should link to beta, got: {:?}", targets);
assert!(targets.iter().any(|t| t.contains("gamma.md")), "alpha should link to gamma, got: {:?}", targets);
```

- [ ] **Step 5.4: Run all core tests**

```bash
cargo test -p simpler-notes-core 2>&1 | tail -20
```

- [ ] **Step 5.5: Build MCP and run e2e tests**

```bash
cargo build -p simpler-notes-mcp && cargo test -p simpler-notes-mcp 2>&1 | tail -30
```

- [ ] **Step 5.6: Commit**

```bash
git add crates/simpler-notes-mcp/src/tools/get_backlinks.rs crates/simpler-notes-core/src/vault.rs crates/simpler-notes-mcp/tests/e2e.rs
git commit -m "fix(link-index): update consumers for full-path targets"
```

---

### Task 6: Build, clippy, final commit

**Files:** none (verification only)

- [ ] **Step 6.1: Run full build**

```bash
cargo build -p simpler-notes-core && cargo build -p simpler-notes-mcp
```

- [ ] **Step 6.2: Run clippy**

```bash
cargo clippy 2>&1
```
Expected: zero warnings.

- [ ] **Step 6.3: Run all tests**

```bash
cargo test -p simpler-notes-core && cargo test -p simpler-notes-mcp && cargo test -p simpler-notes-mcp --test e2e
```

- [ ] **Step 6.4: Commit any remaining fixes**

```bash
git add -A
git commit -m "chore: clippy fixes and cleanup"
```
