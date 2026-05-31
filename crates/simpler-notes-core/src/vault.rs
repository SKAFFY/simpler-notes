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

        vault.reindex_all()?;
        Ok(vault)
    }

    pub fn diagnostics(&self) -> &Diagnostics {
        &self.index.diagnostics
    }

    pub fn reindex_all(&self) -> Result<(), String> {
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

        // Persist index after initial reindex
        self.index.save(&self.config.path)?;
        Ok(())
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
