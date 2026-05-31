use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;

use crate::buffer::Buffer;
use crate::diagnostics::Diagnostic;
use crate::index::ConcurrentIndex;
use crate::index::{DateEntry, LinkEntry, TagCompletion};
use crate::search::SearchEngine;
use crate::diagnostics::Diagnostics;
use chrono::NaiveDate;

pub struct VaultConfig {
    pub path: PathBuf,
    pub extensions: Vec<String>,
}

impl Default for VaultConfig {
    fn default() -> Self {
        VaultConfig {
            path: PathBuf::from("."),
            extensions: vec!["md".to_string()],
        }
    }
}

pub struct Vault {
    pub config: VaultConfig,
    pub index: Arc<ConcurrentIndex>,
    pub buffer: Arc<RwLock<Buffer>>,
    pub search: SearchEngine,
}

pub struct IndexReport {
    pub total_notes: usize,
    pub total_tags: usize,
    pub total_dates: usize,
}

/// A search result with relative path and title.
#[derive(Debug)]
pub struct VaultSearchResult {
    pub path: PathBuf,
    pub title: String,
}

impl Vault {
    pub fn open(config: VaultConfig) -> Result<Self, String> {
        let path = config.path.clone();
        if !path.exists() {
            return Err(format!("Vault path does not exist: {:?}", path));
        }

        let index = if let Ok(loaded) = ConcurrentIndex::load(&path) {
            Arc::new(loaded)
        } else {
            Arc::new(ConcurrentIndex::new())
        };

        let vault = Vault {
            config,
            index: index.clone(),
            buffer: Arc::new(RwLock::new(Buffer::new())),
            search: SearchEngine::new(index.clone()),
        };

        vault.reindex_all_internal()?;
        Ok(vault)
    }

    pub fn diagnostics(&self) -> &Diagnostics {
        &self.index.diagnostics
    }

    pub fn search(&self, query: &str) -> Result<Vec<VaultSearchResult>, String> {
        let expr = SearchEngine::parse_query(query);
        let results = self.search.execute_query(&expr, &self.config.path);
        let vault_path = &self.config.path;
        let mapped = results.iter().map(|p| {
            let rel = pathdiff::diff_paths(p, vault_path).unwrap_or_else(|| PathBuf::from(p));
            let title = p.rsplit('/').next().unwrap_or(p).to_string();
            VaultSearchResult {
                path: rel,
                title,
            }
        }).collect();
        Ok(mapped)
    }

    pub fn reindex_all(&self) -> Result<IndexReport, String> {
        self.reindex_all_internal()?;
        Ok(self.validate_indexes())
    }

    pub fn validate_indexes(&self) -> IndexReport {
        let total_notes = self.list_md_files().len();
        let total_tags = self.index.tags.all_tags().len();
        let total_dates = self.index.dates.all_dates().len();
        IndexReport { total_notes, total_tags, total_dates }
    }

    fn reindex_all_internal(&self) -> Result<(), String> {

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
        }

        self.index.save(&self.config.path)?;
        Ok(())
    }

    pub fn list_md_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        for entry in walkdir::WalkDir::new(&self.config.path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let ext = entry.path()
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            if self.config.extensions.contains(&ext.to_string()) {
                files.push(entry.path().to_path_buf());
            }
        }
        files
    }

    pub fn list_markdown_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        for entry in walkdir::WalkDir::new(&self.config.path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let ext = entry.path()
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            if self.config.extensions.contains(&ext.to_string()) {
                files.push(entry.path().to_path_buf());
            }
        }
        files
    }

    pub fn read_note(&self, path: &Path) -> Result<String, String> {
        let full_path = self.config.path.join(path);
        std::fs::read_to_string(&full_path)
            .map_err(|e| format!("Failed to read {:?}: {}", full_path, e))
    }

    pub fn write_note(&self, path: &Path, content: &str) -> Result<(), String> {
        let full_path = self.config.path.join(path);
        std::fs::write(&full_path, content)
            .map_err(|e| format!("Failed to write {:?}: {}", full_path, e))?;
        self.index.reindex_file(&full_path, content, &self.config.path);
        self.index.save(&self.config.path)?;
        Ok(())
    }

    pub fn get_all_tags(&self) -> Vec<String> {
        self.index.tags.all_tags()
    }

    pub fn get_dates_in_range(&self, from: NaiveDate, to: NaiveDate) -> Vec<(NaiveDate, Vec<DateEntry>)> {
        self.index.dates.get_range(from, to)
    }

    pub fn autocomplete_tags(&self, prefix: &str) -> Vec<TagCompletion> {
        self.index.tags.autocomplete(prefix)
    }

    pub fn fuzzy_search_tags(&self, query: &str) -> Vec<TagCompletion> {
        self.index.tags.fuzzy_search(query, usize::MAX)
    }

    pub fn autocomplete_links(&self, prefix: &str) -> Vec<String> {
        let lower = prefix.to_lowercase();
        self.index.links.all_targets().into_iter()
            .filter(|t| t.to_string_lossy().to_lowercase().contains(&lower))
            .map(|t| t.to_string_lossy().to_string())
            .collect()
    }

    pub fn autocomplete_dates(&self, prefix: &str) -> Vec<String> {
        let lower = prefix.to_lowercase();
        self.index.dates.all_dates().into_iter()
            .filter(|(d, _)| d.to_string().contains(&lower))
            .map(|(d, _)| d.to_string())
            .collect()
    }

    pub fn get_backlinks(&self, target: &Path) -> Vec<LinkEntry> {
        self.index.links.backlinks(target)
    }

    pub fn get_outgoing_links(&self, source: &Path) -> Vec<LinkEntry> {
        self.index.links.outgoing(source)
    }

    pub fn get_diagnostics(&self, path: &Path) -> Vec<Diagnostic> {
        let full_path = self.config.path.join(path);
        self.diagnostics().get(&full_path)
    }

    pub fn all_diagnostics(&self) -> Vec<(PathBuf, Vec<Diagnostic>)> {
        self.diagnostics().all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_vault_open_nonexistent() {
        let config = VaultConfig {
            path: PathBuf::from("/nonexistent/path"),
            ..Default::default()
        };
        let result = Vault::open(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_vault_open_empty() {
        let dir = TempDir::new().unwrap();
        let config = VaultConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };
        let vault = Vault::open(config).unwrap();
        assert!(vault.list_markdown_files().is_empty());
    }

    #[test]
    fn test_vault_reindex() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.md");
        std::fs::write(&file_path, "# Hello @tag").unwrap();

        let config = VaultConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };
        let vault = Vault::open(config).unwrap();
        let tags = vault.index.tags.get("tag");
        assert!(!tags.is_empty(), "Expected tag 'tag' to be indexed");
        assert!(vault.list_markdown_files().len() == 1);
    }

    #[test]
    fn test_vault_reindex_with_link_diagnostic() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("note.md");
        std::fs::write(&file_path, "[[NonExistent]]").unwrap();

        let config = VaultConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };
        let vault = Vault::open(config).unwrap();
        let diags = vault.diagnostics().get(&file_path);
        assert!(!diags.is_empty(), "Expected broken link diagnostic");
    }

    #[test]
    fn test_index_report() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.md"), "@tag1").unwrap();
        std::fs::write(dir.path().join("b.md"), "@tag1 @tag2").unwrap();
        let vault = Vault::open(VaultConfig { path: dir.path().to_path_buf(), ..Default::default() }).unwrap();
        let report = vault.validate_indexes();
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
    fn test_vault_reopen_loads_persisted_index() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("test.md"), "# Hello @tag").unwrap();
        let config = VaultConfig { path: dir.path().to_path_buf(), ..Default::default() };

        // First open — creates .index/
        let vault = Vault::open(config).unwrap();
        assert!(!vault.index.tags.get("tag").is_empty());

        // Second open — loads from persisted .index/
        let config2 = VaultConfig { path: dir.path().to_path_buf(), ..Default::default() };
        let vault2 = Vault::open(config2).unwrap();
        assert!(!vault2.index.tags.get("tag").is_empty(), "Index should load from disk");
    }

    #[test]
    fn test_reindex_all_report() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.md"), "@tag").unwrap();
        let vault = Vault::open(VaultConfig { path: dir.path().to_path_buf(), ..Default::default() }).unwrap();
        let report = vault.reindex_all().unwrap();
        assert_eq!(report.total_notes, 1);
    }

    #[test]
    fn test_vault_skip_non_md() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("notes.txt"), "some text").unwrap();

        let config = VaultConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };
        let vault = Vault::open(config).unwrap();
        assert!(vault.list_markdown_files().is_empty());
    }

    #[test]
    fn test_read_write_note() {
        let dir = TempDir::new().unwrap();
        let vault = Vault::open(VaultConfig { path: dir.path().to_path_buf(), ..Default::default() }).unwrap();
        vault.write_note(&PathBuf::from("test.md"), "hello @tag").unwrap();
        let content = vault.read_note(&PathBuf::from("test.md")).unwrap();
        assert_eq!(content, "hello @tag");
        let tags = vault.get_all_tags();
        assert!(tags.contains(&"tag".to_string()));
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

    #[test]
    fn test_get_dates_in_range() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("note.md"), "!21.06.2027\n\ncontent @test").unwrap();
        let vault = Vault::open(VaultConfig { path: dir.path().to_path_buf(), ..Default::default() }).unwrap();
        let from = NaiveDate::from_ymd_opt(2027, 5, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2027, 7, 1).unwrap();
        let dates = vault.get_dates_in_range(from, to);
        assert!(!dates.is_empty(), "Expected dates in range");
        assert_eq!(dates[0].0, NaiveDate::from_ymd_opt(2027, 6, 21).unwrap());
        let out_of_range = vault.get_dates_in_range(
            NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2020, 12, 31).unwrap(),
        );
        assert!(out_of_range.is_empty(), "Expected no dates in empty range");
    }

    #[test]
    fn test_fuzzy_search_tags() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.md"), "@project-alpha @project-beta").unwrap();
        let vault = Vault::open(VaultConfig { path: dir.path().to_path_buf(), ..Default::default() }).unwrap();
        let results = vault.fuzzy_search_tags("project");
        assert!(results.iter().any(|c| c.name == "project-alpha"), "Should find by substring");
    }

    #[test]
    fn test_autocomplete_links() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.md"), "[[target-note]]").unwrap();
        let vault = Vault::open(VaultConfig { path: dir.path().to_path_buf(), ..Default::default() }).unwrap();
        let results = vault.autocomplete_links("target");
        assert!(!results.is_empty(), "Expected autocomplete results");
        assert!(results[0].contains("target-note"));
    }

    #[test]
    fn test_autocomplete_dates() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("note.md"), "!25.12.2027\n\ntext").unwrap();
        let vault = Vault::open(VaultConfig { path: dir.path().to_path_buf(), ..Default::default() }).unwrap();
        let results = vault.autocomplete_dates("2027-12");
        assert!(!results.is_empty(), "Expected date autocomplete results");
        assert!(results.iter().any(|d| d == "2027-12-25"));
    }

    #[test]
    fn test_get_backlinks() {
        let dir = TempDir::new().unwrap();
        let a_path = dir.path().join("a.md");
        let b_path = dir.path().join("b.md");
        std::fs::write(&a_path, "[[b]]").unwrap();
        std::fs::write(&b_path, "content").unwrap();
        let vault = Vault::open(VaultConfig { path: dir.path().to_path_buf(), ..Default::default() }).unwrap();
        let backlinks = vault.get_backlinks(&PathBuf::from("b"));
        assert!(!backlinks.is_empty(), "Expected backlinks");
        assert_eq!(backlinks[0].source, a_path);
    }

    #[test]
    fn test_get_outgoing_links() {
        let dir = TempDir::new().unwrap();
        let a_path = dir.path().join("a.md");
        let b_path = dir.path().join("b.md");
        std::fs::write(&a_path, "[[b]]").unwrap();
        std::fs::write(&b_path, "content").unwrap();
        let vault = Vault::open(VaultConfig { path: dir.path().to_path_buf(), ..Default::default() }).unwrap();
        let outgoing = vault.get_outgoing_links(&a_path);
        assert!(!outgoing.is_empty(), "Expected outgoing links");
        assert_eq!(outgoing[0].target, PathBuf::from("b"));
    }

    #[test]
    fn test_all_diagnostics() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.md"), "[[]]").unwrap();
        std::fs::write(dir.path().join("b.md"), "clean").unwrap();
        let vault = Vault::open(VaultConfig { path: dir.path().to_path_buf(), ..Default::default() }).unwrap();
        let all = vault.all_diagnostics();
        assert!(!all.is_empty());
    }
}
