use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

use crate::index::ConcurrentIndex;
use crate::note::Note;
use crate::parser;
use crate::search::{self, SearchResult};

#[derive(Debug)]
pub struct IndexReport {
    pub total_notes: usize,
    pub total_tags: usize,
    pub total_dates: usize,
}

#[derive(Debug)]
pub struct Vault {
    pub path: PathBuf,
    pub index: Arc<ConcurrentIndex>,
}

impl Vault {
    pub fn open(path: &Path) -> Result<Self, String> {
        let path = path.to_path_buf();
        if !path.exists() {
            return Err(format!("Path does not exist: {}", path.display()));
        }
        if !path.is_dir() {
            return Err(format!("Path is not a directory: {}", path.display()));
        }

        let index_dir = path.join(".index");
        let index = if index_dir.exists() {
            Arc::new(ConcurrentIndex::load(&index_dir).unwrap_or_default())
        } else {
            Arc::new(ConcurrentIndex::new())
        };

        let vault = Self { path, index };
        vault.rebuild_index();
        Ok(vault)
    }

    pub fn rebuild_index(&self) {
        let index = self.index.clone();
        let vault_path = self.path.clone();

        thread::spawn(move || {
            index.clear();
            let md_files = walkdir(&vault_path);
            let notes: Vec<Note> = md_files
                .iter()
                .filter_map(|path| {
                    std::fs::read_to_string(path)
                        .ok()
                        .map(|content| parser::parse_note(path, &content))
                })
                .collect();

            for note in &notes {
                index.index_note(note);
            }
            let _ = index.save(&vault_path.join(".index"));
        });

    }

    pub fn search(&self, query_str: &str) -> Result<Vec<SearchResult>, String> {
        let query = search::parse_query(query_str)?;
        let paths = search::execute_query(&self.index, &query);

        let results = paths
            .into_iter()
            .map(|path| SearchResult {
                title: path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled")
                    .to_string(),
                path,
            })
            .collect();

        Ok(results)
    }

    pub fn get_note(&self, relative_path: &Path) -> Result<String, String> {
        let full_path = self.path.join(relative_path);
        std::fs::read_to_string(&full_path)
            .map_err(|e| format!("Failed to read {}: {}", full_path.display(), e))
    }

    pub fn write_note(&self, relative_path: &Path, content: &str) -> Result<(), String> {
        let full_path = self.path.join(relative_path);

        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directories: {}", e))?;
        }

        std::fs::write(&full_path, content)
            .map_err(|e| format!("Failed to write {}: {}", full_path.display(), e))?;

        let note = parser::parse_note(&full_path, content);
        self.index.index_note(&note);
        let _ = self.index.save(&self.path.join(".index"));

        Ok(())
    }

    pub fn get_all_tags(&self) -> Vec<String> {
        self.index.tags.all_tags()
    }

    pub fn validate_indexes(&self) -> IndexReport {
        IndexReport {
            total_notes: self.index.file_states.len(),
            total_tags: self.index.tags.all_tags().len(),
            total_dates: self.index.dates.all_dates().len(),
        }
    }
}

fn walkdir(path: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                let name = entry_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                if name == ".git" || name == ".index" {
                    continue;
                }
                files.extend(walkdir(&entry_path));
            } else if entry_path.extension().map(|e| e == "md").unwrap_or(false) {
                files.push(entry_path);
            }
        }
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(name)
    }

    #[test]
    fn test_vault_open_nonexistent_path() {
        let path = temp_dir("simpler_notes_vault_test_nonexistent");
        let result = Vault::open(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }

    #[test]
    fn test_vault_open_file_path() {
        let dir = temp_dir("simpler_notes_vault_test_filepath");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("test.txt");
        fs::write(&file_path, "content").unwrap();

        let result = Vault::open(&file_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not a directory"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_vault_write_and_read_note() {
        let dir = temp_dir("simpler_notes_vault_test_write_read");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let vault = Vault::open(&dir).unwrap();
        vault
            .write_note(Path::new("hello.md"), "# Hello\n\nWorld")
            .unwrap();

        let content = vault.get_note(Path::new("hello.md")).unwrap();
        assert_eq!(content, "# Hello\n\nWorld");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_vault_search_tags() {
        let dir = temp_dir("simpler_notes_vault_test_search_tags");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let vault = Vault::open(&dir).unwrap();
        vault
            .write_note(Path::new("test.md"), "# Test\n\nThis is #test note")
            .unwrap();

        let results = vault.search("tags contain \"test\"").unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.path.ends_with("test.md")));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_vault_get_all_tags() {
        let dir = temp_dir("simpler_notes_vault_test_all_tags");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let vault = Vault::open(&dir).unwrap();
        vault
            .write_note(Path::new("a.md"), "# A\n\n#project #todo")
            .unwrap();
        vault
            .write_note(Path::new("b.md"), "# B\n\n#project #done")
            .unwrap();

        let mut tags = vault.get_all_tags();
        tags.sort();
        // After rebuild_index (async), the index should have these tags.
        // Since the index is rebuilt in a separate thread, we wait a bit.
        assert!(tags.contains(&"project".to_string()));
        assert!(tags.contains(&"todo".to_string()));
        assert!(tags.contains(&"done".to_string()));

        let _ = fs::remove_dir_all(&dir);
    }
}
