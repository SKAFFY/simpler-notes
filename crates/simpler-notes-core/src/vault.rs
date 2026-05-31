use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;

use crate::buffer::Buffer;
use crate::index::ConcurrentIndex;
use crate::search::SearchEngine;
use crate::diagnostics::Diagnostics;

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
        let mut _files_reindexed = 0;

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
}
