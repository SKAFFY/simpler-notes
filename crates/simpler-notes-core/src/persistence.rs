use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;
use crate::index::{ConcurrentIndex, TagEntry, DateEntry, LinkEntry};

const INDEX_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct MetadataV1 {
    version: u32,
    last_rebuild: String,
}

impl ConcurrentIndex {
    /// Save index to .index/ directory inside vault_path.
    pub fn save(&self, vault_path: &Path) -> Result<(), String> {
        let index_dir = vault_path.join(".index");
        fs::create_dir_all(&index_dir).map_err(|e| e.to_string())?;

        let tags: Vec<(String, Vec<TagEntry>)> = self.tags.iter()
            .map(|e| (e.key().clone(), e.value().clone()))
            .collect();
        fs::write(
            index_dir.join("tags.json"),
            serde_json::to_string_pretty(&tags).map_err(|e| e.to_string())?,
        ).map_err(|e| e.to_string())?;

        let dates: Vec<(NaiveDate, Vec<DateEntry>)> = self.dates.iter()
            .map(|e| (*e.key(), e.value().clone()))
            .collect();
        fs::write(
            index_dir.join("dates.json"),
            serde_json::to_string_pretty(&dates).map_err(|e| e.to_string())?,
        ).map_err(|e| e.to_string())?;

        let links: Vec<(std::path::PathBuf, Vec<LinkEntry>)> = self.links.iter()
            .map(|e| (e.key().clone(), e.value().clone()))
            .collect();
        fs::write(
            index_dir.join("links.json"),
            serde_json::to_string_pretty(&links).map_err(|e| e.to_string())?,
        ).map_err(|e| e.to_string())?;

        let meta = MetadataV1 {
            version: INDEX_VERSION,
            last_rebuild: chrono::Utc::now().to_rfc3339(),
        };
        fs::write(
            index_dir.join("metadata.json"),
            serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?,
        ).map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Load index from .index/ inside vault_path.
    pub fn load(vault_path: &Path) -> Result<Self, String> {
        let index_dir = vault_path.join(".index");
        if !index_dir.exists() {
            return Err("Index directory not found".to_string());
        }

        let meta_content = fs::read_to_string(index_dir.join("metadata.json"))
            .map_err(|e| e.to_string())?;
        let meta: MetadataV1 = serde_json::from_str(&meta_content).map_err(|e| e.to_string())?;
        if meta.version != INDEX_VERSION {
            return Err(format!("Unsupported index version: {}", meta.version));
        }

        let index = ConcurrentIndex::new();

        let tags_path = index_dir.join("tags.json");
        if tags_path.exists() {
            let content = fs::read_to_string(&tags_path).map_err(|e| e.to_string())?;
            let data: Vec<(String, Vec<TagEntry>)> =
                serde_json::from_str(&content).map_err(|e| e.to_string())?;
            for (tag, entries) in data {
                for entry in entries {
                    for span in entry.spans {
                        index.tags.add(entry.path.clone(), &tag, span);
                    }
                }
            }
        }

        let dates_path = index_dir.join("dates.json");
        if dates_path.exists() {
            let content = fs::read_to_string(&dates_path).map_err(|e| e.to_string())?;
            let data: Vec<(NaiveDate, Vec<DateEntry>)> =
                serde_json::from_str(&content).map_err(|e| e.to_string())?;
            for (date, entries) in data {
                for entry in entries {
                    for span in entry.spans {
                        index.dates.add(entry.path.clone(), date, span);
                    }
                }
            }
        }

        let links_path = index_dir.join("links.json");
        if links_path.exists() {
            let content = fs::read_to_string(&links_path).map_err(|e| e.to_string())?;
            let data: Vec<(std::path::PathBuf, Vec<LinkEntry>)> =
                serde_json::from_str(&content).map_err(|e| e.to_string())?;
            for (_target, entries) in data {
                for entry in entries {
                    index.links.add(entry.source.clone(), entry);
                }
            }
        }

        Ok(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note_model::ByteSpan;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load() {
        let dir = TempDir::new().unwrap();
        let index = ConcurrentIndex::new();
        let path = std::path::PathBuf::from("test.md");

        index.tags.add(path.clone(), "project", ByteSpan { offset: 0, length: 8 });
        index.dates.add(
            path.clone(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            ByteSpan { offset: 0, length: 12 },
        );

        index.save(dir.path()).unwrap();
        let loaded = ConcurrentIndex::load(dir.path()).unwrap();
        assert!(!loaded.tags.all_tags().is_empty());
        assert!(!loaded.dates.all_dates().is_empty());
    }

    #[test]
    fn test_load_nonexistent() {
        let dir = TempDir::new().unwrap();
        let result = ConcurrentIndex::load(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_wrong_version() {
        let dir = TempDir::new().unwrap();
        let index_dir = dir.path().join(".index");
        std::fs::create_dir_all(&index_dir).unwrap();
        let meta = serde_json::json!({"version": 999, "last_rebuild": "now"});
        std::fs::write(index_dir.join("metadata.json"), serde_json::to_string_pretty(&meta).unwrap()).unwrap();
        std::fs::write(index_dir.join("tags.json"), "[]").unwrap();
        std::fs::write(index_dir.join("dates.json"), "[]").unwrap();
        std::fs::write(index_dir.join("links.json"), "[]").unwrap();
        let result = ConcurrentIndex::load(dir.path());
        assert!(result.is_err());
        assert!(result.err().unwrap().contains("999"));
    }

    #[test]
    fn test_save_and_load_with_links() {
        let dir = TempDir::new().unwrap();
        let index = ConcurrentIndex::new();
        let source = std::path::PathBuf::from("a.md");
        let target = std::path::PathBuf::from("b.md");
        let entry = LinkEntry {
            source: source.clone(),
            target: target.clone(),
            label: "B".to_string(),
            span: ByteSpan { offset: 0, length: 10 },
        };
        index.links.add(source.clone(), entry);

        index.save(dir.path()).unwrap();
        let loaded = ConcurrentIndex::load(dir.path()).unwrap();
        let backlinks = loaded.links.backlinks(&target);
        assert_eq!(backlinks.len(), 1);
        assert_eq!(backlinks[0].source, source);
    }
}
